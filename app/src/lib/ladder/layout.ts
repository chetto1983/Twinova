// A rung's automatic layout: a pure function from the rung's tree to SVG geometry, so the view draws and never measures.
// Ported from nautilus (https://github.com/joyautomation/nautilus, `tools/vscode-iec/webview-ui/src/ladderLayout.ts` at
// 2db093e, Apache-2.0, Copyright 2026 Joy Automation; THIRD_PARTY_NOTICES.md), itself from the tentacle-plc editor,
// changed for Twinova's model: coils may stand anywhere on a rung, a box has pins on both sides, a rung may end in a
// jump or a RETURN.
//
// Layout is bottom-up: each element reports its width and its ascent and descent around the rung's wire. A series flows
// left to right, a branch stacks its paths downward from the wire between two vertical rails, and a short path is
// padded with wire to the branch's right rail. The coils that end a rung stand against the right rail, as CODESYS and
// RSLogix draw them.
import type { BoxPins, CoilMode, ContactMode, Element, Pin, Rung } from "./model";

export const L = {
  RAIL_X: 10,
  RAIL_LEFT: 30,
  RAIL_RIGHT_MARGIN: 30,
  SYMBOL_W: 64,
  SYMBOL_H: 24,
  WIRE_GAP: 10,
  BRANCH_OUTER_GAP: 18,
  BRANCH_GAP: 12,
  EMPTY_LEG_W: 36,
  LABEL_TOP: 18,
  LABEL_BOT: 10,
  RUNG_PAD_Y: 10,
  MIN_WIDTH: 560,
  BOX_MIN_W: 72,
  BOX_HEAD: 18,
  PIN_STEP: 16,
} as const;

/** A pin drawn on a box: its name, the value or place written beside it, and whether the rung's wire meets it. */
export interface PinRow {
  name: string;
  value: string | null;
  rung: boolean;
}

export type LNode =
  | {
      kind: "contact";
      x: number;
      y: number;
      w: number;
      h: number;
      path: number[];
      mode: ContactMode;
      operand: string | null;
    }
  | { kind: "coil"; x: number; y: number; w: number; h: number; path: number[]; mode: CoilMode; operand: string | null }
  | {
      kind: "box";
      x: number;
      y: number;
      w: number;
      h: number;
      path: number[];
      title: string;
      instance: string | null;
      left: PinRow[];
      right: PinRow[];
    }
  | { kind: "exit"; x: number; y: number; w: number; h: number; text: string };

export interface LWire {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}
export interface LVLine {
  x: number;
  y1: number;
  y2: number;
}
export interface LBranch {
  x: number;
  y: number;
  w: number;
  h: number;
  path: number[];
}

export interface RungLayout {
  nodes: LNode[];
  branches: LBranch[];
  wires: LWire[];
  vlines: LVLine[];
  width: number;
  height: number;
  wireY: number;
}

/** How a box at a path sits on its rung, when it lowered. */
export type Resolve = (path: readonly number[]) => BoxPins | undefined;

interface Res {
  nodes: LNode[];
  branches: LBranch[];
  wires: LWire[];
  vlines: LVLine[];
  width: number;
  ascent: number;
  descent: number;
}

/** A monospace text's width at `px`. */
export const textW = (s: string, px: number): number => s.length * px * 0.62;

/** The longest operand a contact or a coil grows to fit; a longer one is cut in the view. */
export const OPERAND_MAX = 20;

const symbolWidth = (operand: string | null): number =>
  Math.max(L.SYMBOL_W, Math.ceil(textW((operand ?? "???").slice(0, OPERAND_MAX), 10)) + 8);

const halfH = L.SYMBOL_H / 2;

const onRung = (name: string): PinRow => ({ name, value: null, rung: true });

/** A box's pins as drawn: on the left the pin the rung feeds, then the inputs with a value; on the right the pin that
 *  carries the rung on, then the outputs bound to a place. A function that starts a path (`EQ(..., OUT)`) has nothing
 *  fed: its wire row stays blank, so no input's value is drawn on the wire. A box that did not lower shows its pins as
 *  written. */
export function boxPins(pins: readonly Pin[], resolved: BoxPins | undefined): { left: PinRow[]; right: PinRow[] } {
  const valued = (out: boolean): PinRow[] =>
    pins.filter((p) => p.out === out && p.value !== null).map((p) => ({ name: p.name, value: p.value, rung: false }));
  if (!resolved) {
    const written = (out: boolean): PinRow[] =>
      pins.filter((p) => p.out === out).map((p) => ({ name: p.name, value: p.value, rung: p.value === null }));
    return { left: written(false), right: written(true) };
  }
  return {
    left: [resolved.input === null ? { name: "", value: null, rung: false } : onRung(resolved.input), ...valued(false)],
    right: [onRung(resolved.output), ...valued(true)],
  };
}

const longestName = (rows: readonly PinRow[]): number => Math.max(0, ...rows.map((r) => r.name.length));

const labelWidth = (rows: readonly PinRow[]): number =>
  Math.max(0, ...rows.map((r) => (r.value === null ? 0 : Math.ceil(textW(r.value, 10)) + 8)));

/** Lay out a series of elements on the wire at `cy` from `x`; `path` addresses the series in the rung's tree. */
export function layoutSeries(
  elements: readonly Element[],
  x: number,
  cy: number,
  path: number[],
  resolve: Resolve,
): Res {
  const res: Res = {
    nodes: [],
    branches: [],
    wires: [],
    vlines: [],
    width: 0,
    ascent: halfH + L.LABEL_TOP,
    descent: halfH + L.LABEL_BOT,
  };
  let cursor = x;
  if (elements.length === 0) {
    // an empty path is a wire, always on
    res.wires.push({ x1: cursor, y1: cy, x2: cursor + L.EMPTY_LEG_W, y2: cy });
    res.width = L.EMPTY_LEG_W;
    return res;
  }
  elements.forEach((element, i) => {
    if (i > 0) {
      const nearBranch = element.kind === "branch" || elements[i - 1]?.kind === "branch";
      const gap = nearBranch ? L.BRANCH_OUTER_GAP : L.WIRE_GAP;
      res.wires.push({ x1: cursor, y1: cy, x2: cursor + gap, y2: cy });
      cursor += gap;
    }
    const at = [...path, i];
    switch (element.kind) {
      case "contact":
      case "coil": {
        const w = symbolWidth(element.operand);
        const common = { x: cursor, y: cy - halfH, w, h: L.SYMBOL_H, path: at, operand: element.operand };
        res.nodes.push(
          element.kind === "contact"
            ? { kind: "contact", mode: element.mode, ...common }
            : { kind: "coil", mode: element.mode, ...common },
        );
        cursor += w;
        break;
      }
      case "box":
        cursor = placeBox(res, element.head, element.pins, resolve(at), at, cursor, cy);
        break;
      case "branch":
        cursor = placeBranch(res, element.paths, at, cursor, cy, resolve);
        break;
    }
  });
  res.width = cursor - x;
  return res;
}

function placeBox(
  res: Res,
  head: string | null,
  pins: readonly Pin[],
  resolved: BoxPins | undefined,
  at: number[],
  x: number,
  cy: number,
): number {
  const { left, right } = boxPins(pins, resolved);
  const instance = resolved && !resolved.function ? head : null;
  const title = resolved?.type_name || (head ?? "???");
  const rows = Math.max(left.length, right.length, 1);
  const w = Math.max(
    L.BOX_MIN_W,
    Math.ceil(textW(title, 11)) + 16,
    Math.ceil(textW("x".repeat(longestName(left) + longestName(right)), 9)) + 24,
  );
  const h = L.BOX_HEAD + rows * L.PIN_STEP + 4;
  // the first pins are on the rung's wire
  const y = cy - L.BOX_HEAD - L.PIN_STEP / 2;
  const before = labelWidth(left);
  const after = labelWidth(right);
  if (before > 0) res.wires.push({ x1: x, y1: cy, x2: x + before, y2: cy });
  res.nodes.push({ kind: "box", x: x + before, y, w, h, path: at, title, instance, left, right });
  if (after > 0) res.wires.push({ x1: x + before + w, y1: cy, x2: x + before + w + after, y2: cy });
  res.ascent = Math.max(res.ascent, cy - y + (instance === null ? 6 : L.LABEL_TOP));
  res.descent = Math.max(res.descent, y + h - cy + L.LABEL_BOT);
  return x + before + w + after;
}

function placeBranch(
  res: Res,
  paths: readonly Element[][],
  at: number[],
  x: number,
  cy: number,
  resolve: Resolve,
): number {
  const probes = paths.map((p, j) => layoutSeries(p, 0, 0, [...at, j], resolve));
  const maxW = Math.max(L.EMPTY_LEG_W, ...probes.map((p) => p.width));
  // the first path rides the wire; the others stack below it
  let pen = cy;
  const ys: number[] = [];
  paths.forEach((p, j) => {
    const probe = probes[j] as Res;
    if (j > 0) pen += (probes[j - 1] as Res).descent + L.BRANCH_GAP + probe.ascent;
    ys.push(pen);
    const r = layoutSeries(p, x, pen, [...at, j], resolve);
    res.nodes.push(...r.nodes);
    res.branches.push(...r.branches);
    res.wires.push(...r.wires);
    res.vlines.push(...r.vlines);
    if (r.width < maxW) res.wires.push({ x1: x + r.width, y1: pen, x2: x + maxW, y2: pen });
  });
  const top = ys[0] as number;
  const bottom = ys[ys.length - 1] as number;
  res.vlines.push({ x, y1: top, y2: bottom }, { x: x + maxW, y1: top, y2: bottom });
  const first = probes[0] as Res;
  const last = probes[probes.length - 1] as Res;
  res.branches.push({
    x: x - 6,
    y: top - first.ascent,
    w: maxW + 12,
    h: bottom + last.descent - (top - first.ascent),
    path: at,
  });
  res.ascent = Math.max(res.ascent, first.ascent);
  res.descent = Math.max(res.descent, bottom - cy + last.descent);
  return x + maxW;
}

/** The coils that end a rung's root series, which stand against the right rail. */
function trailingCoils(rung: Rung): number {
  let n = 0;
  while (n < rung.elements.length && rung.elements[rung.elements.length - 1 - n]?.kind === "coil") n++;
  return n;
}

const exitText = (rung: Rung): string | null =>
  rung.exit === null ? null : rung.exit.kind === "jump" ? `>> ${rung.exit.label}` : "RETURN";

function coilZone(rung: Rung): { coils: Element[]; widths: number[]; exitW: number; width: number } {
  const coils = rung.elements.slice(rung.elements.length - trailingCoils(rung));
  const widths = coils.map((c) => symbolWidth(c.kind === "coil" ? c.operand : null));
  const exit = exitText(rung);
  const exitW = exit === null ? 0 : Math.ceil(textW(exit, 11)) + 16;
  const width = widths.reduce((a, b) => a + b, 0) + Math.max(0, coils.length - 1) * L.WIRE_GAP + exitW;
  return { coils, widths, exitW, width };
}

/** A rung's natural width, its rails included. */
export function rungMinWidth(rung: Rung, resolve: Resolve): number {
  const zone = coilZone(rung);
  const series = rung.elements.slice(0, rung.elements.length - zone.coils.length);
  return L.RAIL_LEFT + layoutSeries(series, 0, 0, [], resolve).width + L.WIRE_GAP + zone.width + L.RAIL_RIGHT_MARGIN;
}

/** One rung laid out `width` wide: its logic from the left rail, its last coils and its exit against the right one. */
export function layoutRung(rung: Rung, width: number, resolve: Resolve): RungLayout {
  const zone = coilZone(rung);
  const series = rung.elements.slice(0, rung.elements.length - zone.coils.length);
  const probe = layoutSeries(series, 0, 0, [], resolve);
  const wireY = L.RUNG_PAD_Y + probe.ascent;
  const r = layoutSeries(series, L.RAIL_LEFT, wireY, [], resolve);
  const nodes = [...r.nodes];
  const wires = [...r.wires, { x1: L.RAIL_X, y1: wireY, x2: L.RAIL_LEFT, y2: wireY }];
  const zoneX = Math.max(width - L.RAIL_RIGHT_MARGIN - zone.width, L.RAIL_LEFT + r.width + L.WIRE_GAP);
  wires.push({ x1: L.RAIL_LEFT + r.width, y1: wireY, x2: zoneX, y2: wireY });
  let cx = zoneX;
  zone.coils.forEach((coil, i) => {
    if (i > 0) {
      wires.push({ x1: cx, y1: wireY, x2: cx + L.WIRE_GAP, y2: wireY });
      cx += L.WIRE_GAP;
    }
    const w = zone.widths[i] as number;
    if (coil.kind === "coil") {
      const path = [series.length + i];
      nodes.push({
        kind: "coil",
        x: cx,
        y: wireY - halfH,
        w,
        h: L.SYMBOL_H,
        path,
        mode: coil.mode,
        operand: coil.operand,
      });
    }
    cx += w;
  });
  const exit = exitText(rung);
  if (exit === null) {
    wires.push({ x1: cx, y1: wireY, x2: width - L.RAIL_X, y2: wireY });
  } else {
    nodes.push({ kind: "exit", x: cx + 8, y: wireY - halfH, w: zone.exitW - 8, h: L.SYMBOL_H, text: exit });
    wires.push({ x1: cx, y1: wireY, x2: cx + 8, y2: wireY });
  }
  return {
    nodes,
    branches: r.branches,
    wires,
    vlines: r.vlines,
    width,
    height: wireY + r.descent + L.RUNG_PAD_Y,
    wireY,
  };
}

/** Every rung of a program laid out as wide as the widest, so their right rails line up. */
export function layoutRungs(rungs: readonly Rung[], resolve: (rung: number) => Resolve): RungLayout[] {
  const width = Math.max(L.MIN_WIDTH, ...rungs.map((r, i) => rungMinWidth(r, resolve(i))));
  return rungs.map((r, i) => layoutRung(r, width, resolve(i)));
}
