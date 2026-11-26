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
  PIN_STEP: 24,
  /** The wire from a pin's place to the box, and the space between a place, its reading and that wire. */
  PIN_STUB: 10,
  LABEL_GAP: 4,
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
      feed: Feed;
      mode: ContactMode;
      operand: string | null;
    }
  | {
      kind: "coil";
      x: number;
      y: number;
      w: number;
      h: number;
      path: number[];
      feed: Feed;
      mode: CoilMode;
      operand: string | null;
    }
  | {
      kind: "box";
      x: number;
      y: number;
      w: number;
      h: number;
      path: number[];
      feed: Feed;
      title: string;
      instance: string | null;
      left: PinRow[];
      right: PinRow[];
    }
  | { kind: "exit"; x: number; y: number; w: number; h: number; text: string };

/** Whose power a wire carries, online: the element at that path's power out, or the left rail's (null). */
export type Feed = number[] | null;

export interface LWire {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  feed: Feed;
}
export interface LVLine {
  x: number;
  y1: number;
  y2: number;
  feed: Feed;
}
export interface LBranch {
  x: number;
  y: number;
  w: number;
  h: number;
  path: number[];
  /** each path's wire, top to bottom */
  legs: number[];
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

/** The drawing's text sizes, as LadderRung.svelte's styles take them from app.css's type tokens: `--text-xs` for
 *  operands, values, readings and pins, `--text-sm` for a box's title and a jump's label. A change of either token
 *  changes these. */
export const TEXT = { small: 11, title: 12 } as const;

/** The longest operand a contact or a coil grows to fit; a longer one is cut in the view. */
export const OPERAND_MAX = 20;

const symbolWidth = (operand: string | null): number =>
  Math.max(L.SYMBOL_W, Math.ceil(textW((operand ?? "???").slice(0, OPERAND_MAX), TEXT.small)) + 8);

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

/** Whether a pin's value names a variable, which shows its value online (monitor_ladder.rs `reference`). */
export const isReference = (value: string | null): boolean =>
  value !== null && /^[A-Za-z_]\w*(\.[A-Za-z_]\w*)*$/.test(value) && !/^(TRUE|FALSE)$/i.test(value);

/** The least room beside a box for a pin's reference: its value online is drawn under it, and may be as wide as
 *  FALSE. Kept offline too, so the drawing does not move when monitoring starts. */
export const READING_MIN = Math.ceil(textW("FALSE", TEXT.small)) + 8;

/** The room a pin's place takes beside its box, on its row: the place, its reading when it names a variable, and the
 *  wire to the pin. */
const placeRoom = (value: string): number =>
  L.PIN_STUB +
  L.LABEL_GAP +
  Math.ceil(textW(value, TEXT.small)) +
  (isReference(value) ? L.LABEL_GAP + READING_MIN : 0) +
  L.LABEL_GAP;

const labelWidth = (rows: readonly PinRow[]): number =>
  Math.max(0, ...rows.map((r) => (r.value === null ? 0 : placeRoom(r.value))));

/** A pin's place drawn beside its box, on the pin's row `y`: its text ends (left) or starts (right) at `textX`, its
 *  reading online ends or starts at `readingX`, and a wire joins it to the pin. On the rung's own wire, the first row of a
 *  box that did not lower, the place stands above the wire, which is already there. */
export interface PinLabel {
  side: "left" | "right";
  pin: string;
  y: number;
  text: string;
  textX: number;
  readingX: number;
  stub: { x1: number; x2: number; y: number } | null;
}

/** The places of a box's valued pins (decision 1 of spec 2026-11-24-open-points-design.md): each on its own row, so a
 *  place, its reading and the next row's place never meet. */
export function pinLabels(box: Extract<LNode, { kind: "box" }>): PinLabel[] {
  const rowY = (k: number) => box.y + L.BOX_HEAD + L.PIN_STEP * k + L.PIN_STEP / 2;
  const side = (rows: readonly PinRow[], left: boolean): PinLabel[] =>
    rows.flatMap((pin, k) => {
      if (pin.value === null) return [];
      const y = rowY(k);
      const edge = left ? box.x : box.x + box.w;
      const out = left ? -1 : 1;
      const onWire = k === 0;
      const textX = edge + out * (onWire ? L.LABEL_GAP : L.PIN_STUB + L.LABEL_GAP);
      return [
        {
          side: left ? "left" : "right",
          pin: pin.name,
          y,
          text: pin.value,
          textX,
          readingX: textX + out * (Math.ceil(textW(pin.value, TEXT.small)) + L.LABEL_GAP),
          stub: onWire
            ? null
            : { x1: Math.min(edge, edge + out * L.PIN_STUB), x2: Math.max(edge, edge + out * L.PIN_STUB), y },
        },
      ];
    });
  return [...side(box.left, true), ...side(box.right, false)];
}

/** Lay out a series of elements on the wire at `cy` from `x`; `path` addresses the series in the rung's tree, `feed`
 *  the power it takes. */
export function layoutSeries(
  elements: readonly Element[],
  x: number,
  cy: number,
  path: number[],
  resolve: Resolve,
  feed: Feed = null,
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
    // an empty path is a wire: it carries what feeds it
    res.wires.push({ x1: cursor, y1: cy, x2: cursor + L.EMPTY_LEG_W, y2: cy, feed });
    res.width = L.EMPTY_LEG_W;
    return res;
  }
  let into = feed;
  elements.forEach((element, i) => {
    if (i > 0) {
      const nearBranch = element.kind === "branch" || elements[i - 1]?.kind === "branch";
      const gap = nearBranch ? L.BRANCH_OUTER_GAP : L.WIRE_GAP;
      res.wires.push({ x1: cursor, y1: cy, x2: cursor + gap, y2: cy, feed: into });
      cursor += gap;
    }
    const at = [...path, i];
    switch (element.kind) {
      case "contact":
      case "coil": {
        const w = symbolWidth(element.operand);
        const common = { x: cursor, y: cy - halfH, w, h: L.SYMBOL_H, path: at, feed: into, operand: element.operand };
        res.nodes.push(
          element.kind === "contact"
            ? { kind: "contact", mode: element.mode, ...common }
            : { kind: "coil", mode: element.mode, ...common },
        );
        cursor += w;
        break;
      }
      case "box":
        cursor = placeBox(res, element.head, element.pins, resolve(at), at, cursor, cy, into);
        break;
      case "branch":
        cursor = placeBranch(res, element.paths, at, cursor, cy, resolve, into);
        break;
    }
    into = at;
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
  feed: Feed,
): number {
  const { left, right } = boxPins(pins, resolved);
  const instance = resolved && !resolved.function ? head : null;
  const title = resolved?.type_name || (head ?? "???");
  const rows = Math.max(left.length, right.length, 1);
  const w = Math.max(
    L.BOX_MIN_W,
    Math.ceil(textW(title, TEXT.title)) + 16,
    Math.ceil(textW("x".repeat(longestName(left) + longestName(right)), TEXT.small)) + 24,
  );
  const h = L.BOX_HEAD + rows * L.PIN_STEP + 4;
  // the first pins are on the rung's wire
  const y = cy - L.BOX_HEAD - L.PIN_STEP / 2;
  const before = labelWidth(left);
  const after = labelWidth(right);
  if (before > 0) res.wires.push({ x1: x, y1: cy, x2: x + before, y2: cy, feed });
  res.nodes.push({ kind: "box", x: x + before, y, w, h, path: at, feed, title, instance, left, right });
  if (after > 0) res.wires.push({ x1: x + before + w, y1: cy, x2: x + before + w + after, y2: cy, feed: at });
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
  feed: Feed,
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
    const r = layoutSeries(p, x, pen, [...at, j], resolve, feed);
    res.nodes.push(...r.nodes);
    res.branches.push(...r.branches);
    res.wires.push(...r.wires);
    res.vlines.push(...r.vlines);
    // a shorter path's end is padded with the wire its last element feeds
    const end = p.length > 0 ? [...at, j, p.length - 1] : feed;
    if (r.width < maxW) res.wires.push({ x1: x + r.width, y1: pen, x2: x + maxW, y2: pen, feed: end });
  });
  const top = ys[0] as number;
  const bottom = ys[ys.length - 1] as number;
  res.vlines.push({ x, y1: top, y2: bottom, feed }, { x: x + maxW, y1: top, y2: bottom, feed: at });
  const first = probes[0] as Res;
  const last = probes[probes.length - 1] as Res;
  res.branches.push({
    x: x - 6,
    y: top - first.ascent,
    w: maxW + 12,
    h: bottom + last.descent - (top - first.ascent),
    path: at,
    legs: ys,
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
  const exitW = exit === null ? 0 : Math.ceil(textW(exit, TEXT.title)) + 16;
  const width = widths.reduce((a, b) => a + b, 0) + Math.max(0, coils.length - 1) * L.WIRE_GAP + exitW;
  return { coils, widths, exitW, width };
}

/** A rung's natural width, its rails included. */
export function rungMinWidth(rung: Rung, resolve: Resolve): number {
  const zone = coilZone(rung);
  const series = rung.elements.slice(0, rung.elements.length - zone.coils.length);
  return L.RAIL_LEFT + layoutSeries(series, 0, 0, [], resolve).width + L.WIRE_GAP + zone.width + L.RAIL_RIGHT_MARGIN;
}

/** The power out of element `n` of a rung's root series, or the rail's before the first. */
const outOf = (n: number): Feed => (n >= 0 ? [n] : null);

/** One rung laid out `width` wide: its logic from the left rail, its last coils and its exit against the right one. */
export function layoutRung(rung: Rung, width: number, resolve: Resolve): RungLayout {
  const zone = coilZone(rung);
  const series = rung.elements.slice(0, rung.elements.length - zone.coils.length);
  const probe = layoutSeries(series, 0, 0, [], resolve);
  const wireY = L.RUNG_PAD_Y + probe.ascent;
  const r = layoutSeries(series, L.RAIL_LEFT, wireY, [], resolve);
  const nodes = [...r.nodes];
  const wires: LWire[] = [...r.wires, { x1: L.RAIL_X, y1: wireY, x2: L.RAIL_LEFT, y2: wireY, feed: null }];
  const zoneX = Math.max(width - L.RAIL_RIGHT_MARGIN - zone.width, L.RAIL_LEFT + r.width + L.WIRE_GAP);
  wires.push({ x1: L.RAIL_LEFT + r.width, y1: wireY, x2: zoneX, y2: wireY, feed: outOf(series.length - 1) });
  let cx = zoneX;
  zone.coils.forEach((coil, i) => {
    if (i > 0) {
      wires.push({ x1: cx, y1: wireY, x2: cx + L.WIRE_GAP, y2: wireY, feed: outOf(series.length + i - 1) });
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
        feed: outOf(series.length + i - 1),
        mode: coil.mode,
        operand: coil.operand,
      });
    }
    cx += w;
  });
  const exit = exitText(rung);
  const last = outOf(rung.elements.length - 1);
  if (exit === null) {
    wires.push({ x1: cx, y1: wireY, x2: width - L.RAIL_X, y2: wireY, feed: last });
  } else {
    nodes.push({ kind: "exit", x: cx + 8, y: wireY - halfH, w: zone.exitW - 8, h: L.SYMBOL_H, text: exit });
    wires.push({ x1: cx, y1: wireY, x2: cx + 8, y2: wireY, feed: last });
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

/** Every rung of a program laid out as wide as the pane (`pane`, 0 when unknown), or as the widest rung when the pane is
 *  narrower, so their right rails line up (decision 1 of spec 2026-11-24-open-points-design.md). */
export function layoutRungs(rungs: readonly Rung[], resolve: (rung: number) => Resolve, pane = 0): RungLayout[] {
  const width = Math.max(pane, L.MIN_WIDTH, ...rungs.map((r, i) => rungMinWidth(r, resolve(i))));
  return rungs.map((r, i) => layoutRung(r, width, resolve(i)));
}
