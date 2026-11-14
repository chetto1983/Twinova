// The Ladder editor's gestures as transforms on rungs: each takes the rungs and a selection and hands back new rungs and
// where the selection went; the view keeps nothing, and Rust prints the rungs back into the file (`ladder_print`).
// Ported from Open Industry Project (`src/plc/ladder/ladderEdit.ts` at 1e7488c, MIT; THIRD_PARTY_NOTICES.md), changed
// for Twinova's model: a branch is `{ kind: "branch" }`, a rung's comment, label and exit are null when absent, and a
// coil picked with a rung or a path selected goes at its end, where coils stand, not at its start.
import type { CoilMode, ContactMode, Element, Rung } from "./model";

/** What is selected on rung `rung`: `path` walks down from its series, an element's index then a path's in turn, so
 *  `[2]` is its third element, `[2, 1]` the second path of the branch there, `[2, 1, 0]` that path's first element.
 *  Empty selects the rung itself. */
export interface Selection {
  rung: number;
  path: number[];
}

export interface Edit {
  rungs: Rung[];
  selection: Selection | null;
}

export const selectionKind = (s: Selection): "rung" | "element" | "path" =>
  s.path.length === 0 ? "rung" : s.path.length % 2 === 1 ? "element" : "path";

// JSON, not structuredClone: the view's rungs are Svelte state proxies, which structuredClone refuses
const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;
const copy = (rungs: readonly Rung[]): Rung[] => clone(rungs) as Rung[];
const unchanged = (rungs: readonly Rung[], selection: Selection | null): Edit => ({ rungs: [...rungs], selection });

/** The series a path of even length ends in: `[]` is the rung's own, `[e, p]` a branch's path. */
function seriesAt(rung: Rung, path: readonly number[]): Element[] | null {
  let series = rung.elements;
  for (let i = 0; i + 1 < path.length; i += 2) {
    const el = series[path[i] as number];
    if (el?.kind !== "branch") return null;
    const next = el.paths[path[i + 1] as number];
    if (!next) return null;
    series = next;
  }
  return series;
}

export function elementAt(rungs: readonly Rung[], s: Selection): Element | null {
  const rung = rungs[s.rung];
  if (!rung || selectionKind(s) !== "element") return null;
  return seriesAt(rung, s.path.slice(0, -1))?.[s.path[s.path.length - 1] as number] ?? null;
}

/** Where a new element goes: right after a selected element; into a selected rung or path, at its start, or at its end
 *  for a coil. Its home series and index, or null when the selection is not on the rungs. */
function slotFor(rungs: Rung[], s: Selection, el: Element): { series: Element[]; home: number[]; at: number } | null {
  const rung = rungs[s.rung];
  const onElement = selectionKind(s) === "element";
  const home = onElement ? s.path.slice(0, -1) : s.path;
  const series = rung ? seriesAt(rung, home) : null;
  if (!series) return null;
  if (onElement) return { series, home, at: (s.path[s.path.length - 1] as number) + 1 };
  return { series, home, at: el.kind === "coil" ? series.length : 0 };
}

/** `el` placed where the selection says, and selected, so the next one follows it. */
export function insertElement(rungs: readonly Rung[], s: Selection, el: Element): Edit {
  const next = copy(rungs);
  const slot = slotFor(next, s, el);
  if (!slot) return unchanged(rungs, s);
  slot.series.splice(slot.at, 0, clone(el));
  return { rungs: next, selection: { rung: s.rung, path: [...slot.home, slot.at] } };
}

/** An empty branch, two bare paths, placed as an element is; its lower path is selected, so the next element picked
 *  goes in parallel. */
export function addBranch(rungs: readonly Rung[], s: Selection): Edit {
  const inserted = insertElement(rungs, s, { kind: "branch", paths: [[], []] });
  if (inserted.selection === s) return inserted;
  const at = inserted.selection as Selection;
  return { rungs: inserted.rungs, selection: { rung: at.rung, path: [...at.path, 1] } };
}

/** A new empty rung below rung `after` (first, for -1), selected. */
export function addRung(rungs: readonly Rung[], after: number): Edit {
  const next = copy(rungs);
  const at = Math.max(0, Math.min(after + 1, next.length));
  next.splice(at, 0, { comment: null, label: null, elements: [], exit: null });
  return { rungs: next, selection: { rung: at, path: [] } };
}

/** What is selected deleted: an element, a path, or the rung. A branch left with one path is no branch: that path's
 *  elements rejoin the series in its place. */
export function removeSelected(rungs: readonly Rung[], s: Selection): Edit {
  const next = copy(rungs);
  const kind = selectionKind(s);
  if (!next[s.rung]) return unchanged(rungs, s);
  if (kind === "rung") {
    next.splice(s.rung, 1);
    return { rungs: next, selection: next.length === 0 ? null : { rung: Math.min(s.rung, next.length - 1), path: [] } };
  }
  const rung = next[s.rung] as Rung;
  const last = s.path[s.path.length - 1] as number;
  if (kind === "element") {
    const home = s.path.slice(0, -1);
    const series = seriesAt(rung, home);
    if (!series?.[last]) return unchanged(rungs, s);
    series.splice(last, 1);
    return { rungs: next, selection: { rung: s.rung, path: home } };
  }
  const branchPath = s.path.slice(0, -1);
  const branch = elementAt(next, { rung: s.rung, path: branchPath });
  const series = seriesAt(rung, branchPath.slice(0, -1));
  if (!series || branch?.kind !== "branch" || !branch.paths[last]) return unchanged(rungs, s);
  branch.paths.splice(last, 1);
  if (branch.paths.length < 2)
    series.splice(branchPath[branchPath.length - 1] as number, 1, ...(branch.paths[0] ?? []));
  return { rungs: next, selection: { rung: s.rung, path: branchPath.slice(0, -1) } };
}

/** Every element that is not a branch, in reading order: left to right, and through a branch path by path. */
function leaves(series: readonly Element[], home: readonly number[] = []): number[][] {
  return series.flatMap((el, i) =>
    el.kind !== "branch" ? [[...home, i]] : el.paths.flatMap((p, j) => leaves(p, [...home, i, j])),
  );
}

/** Where Left (-1) or Right (+1) goes from the selection: the next element in reading order, through branches and on to
 *  the next rung; a rung with no element is a stop of its own. Null past either end. */
export function step(rungs: readonly Rung[], s: Selection, by: -1 | 1): Selection | null {
  const stops: Selection[] = rungs.flatMap((rung, r) => {
    const found = leaves(rung.elements);
    return found.length > 0 ? found.map((path) => ({ rung: r, path })) : [{ rung: r, path: [] }];
  });
  const same = (a: Selection) => a.rung === s.rung && a.path.join() === s.path.join();
  const at = stops.findIndex(same);
  if (at >= 0) return stops[at + by] ?? null;
  // a rung, a path or a branch selected: enter it from its near edge
  const under = stops.flatMap((t, i) => (t.rung === s.rung && s.path.every((n, k) => n === t.path[k]) ? [i] : []));
  const first = under[0];
  if (first === undefined) return null;
  return by === 1 ? (stops[first] as Selection) : (stops[first - 1] ?? null);
}

/** The variants a selected contact or coil takes (spec 2026-11-13-ladder-editor-design.md: variants are modifiers). */
export type Modifier = "negated" | "rising" | "falling" | "set" | "reset";

const CONTACT: Partial<Record<Modifier, ContactMode>> = { negated: "closed", rising: "rising", falling: "falling" };
const COIL: Record<Modifier, CoilMode> = {
  negated: "negated",
  rising: "rising",
  falling: "falling",
  set: "set",
  reset: "reset",
};

/** The mode modifier `mod` gives element `el`, or null when it does not apply. */
const modeOf = (el: Element, mod: Modifier): string | null =>
  el.kind === "contact" ? (CONTACT[mod] ?? null) : el.kind === "coil" ? COIL[mod] : null;

/** Whether modifier `mod` is what element `el` is now. */
export const hasModifier = (el: Element | null, mod: Modifier): boolean =>
  !!el && (el.kind === "contact" || el.kind === "coil") && modeOf(el, mod) === el.mode;

/** Whether modifier `mod` applies to element `el`. */
export const takesModifier = (el: Element | null, mod: Modifier): boolean => !!el && modeOf(el, mod) !== null;

/** The selected contact or coil with modifier `mod` toggled: on, or back to plain when it is on already. */
export function modify(rungs: readonly Rung[], s: Selection, mod: Modifier): Edit {
  const next = copy(rungs);
  const el = elementAt(next, s);
  if (!takesModifier(el, mod)) return unchanged(rungs, s);
  if (el?.kind === "contact") el.mode = hasModifier(el, mod) ? "open" : (CONTACT[mod] as ContactMode);
  if (el?.kind === "coil") el.mode = hasModifier(el, mod) ? "normal" : COIL[mod];
  return { rungs: next, selection: s };
}

/** What a toolbox button places; a box comes from the picker, whole (`LadderPicker.svelte`). */
export type Placed = "contact" | "coil" | "branch";

const NEW: Record<Exclude<Placed, "branch">, Element> = {
  contact: { kind: "contact", mode: "open", operand: null },
  coil: { kind: "coil", mode: "normal", operand: null },
};

/** The rungs with a new, blank element of the toolbox placed by the selection. */
export const place = (rungs: readonly Rung[], s: Selection, what: Placed): Edit =>
  what === "branch" ? addBranch(rungs, s) : insertElement(rungs, s, NEW[what]);

/** The selected contact's or coil's operand set to `text`: blank leaves it to fill (`???`). A box is retyped whole
 *  (`replaceElement`). */
export function setOperand(rungs: readonly Rung[], s: Selection, text: string): Edit {
  const next = copy(rungs);
  const el = elementAt(next, s);
  const value = text.trim() === "" ? null : text.trim();
  if (el?.kind !== "contact" && el?.kind !== "coil") return unchanged(rungs, s);
  el.operand = value;
  return { rungs: next, selection: s };
}

/** The selected element replaced by `el`, selected: a box whose call was retyped. */
export function replaceElement(rungs: readonly Rung[], s: Selection, el: Element): Edit {
  const next = copy(rungs);
  const rung = next[s.rung];
  const series = rung && selectionKind(s) === "element" ? seriesAt(rung, s.path.slice(0, -1)) : null;
  const at = s.path[s.path.length - 1] as number;
  if (!series?.[at]) return unchanged(rungs, s);
  series[at] = clone(el);
  return { rungs: next, selection: s };
}
