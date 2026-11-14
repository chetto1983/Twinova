// The editor's gestures (edit.ts), ported with Open Industry Project's tests (`ladderEdit.test.ts` at 1e7488c, MIT).
import { describe, expect, it } from "vitest";
import {
  addBranch,
  addRung,
  elementAt,
  hasModifier,
  insertElement,
  modify,
  place,
  removeSelected,
  step,
  takesModifier,
  type Edit,
  type Selection,
} from "./edit";
import type { Element, Rung } from "./model";

// the `.ld` text a rung prints as (`print.rs`), enough to read a test
const show = (el: Element): string => {
  const mark = { open: "", closed: "/", rising: "P ", falling: "N ", normal: "", negated: "/", set: "S ", reset: "R " };
  if (el.kind === "contact") return `|${mark[el.mode]}${el.operand ?? "?"}|`;
  if (el.kind === "coil") return `(${mark[el.mode]}${el.operand ?? "?"})`;
  if (el.kind === "box") return `${el.head ?? "?"}()`;
  return `[${el.paths.map((p) => p.map(show).join(" ")).join(", ")}]`;
};
const text = (e: Edit | Rung[]): string[] =>
  (Array.isArray(e) ? e : e.rungs).map((r) => r.elements.map(show).join(" "));

const NO: Element = { kind: "contact", mode: "open", operand: null };
const NC: Element = { kind: "contact", mode: "closed", operand: null };
const COIL: Element = { kind: "coil", mode: "normal", operand: null };
const named = (el: Element, operand: string): Element => ({ ...el, operand }) as Element;
const rung = (...elements: Element[]): Rung => ({ comment: null, label: null, elements, exit: null });
const at = (r: number, ...path: number[]): Selection => ({ rung: r, path });

/** `|Start| |/Stop| (Motor)` */
const series = (): Rung[] => [rung(named(NO, "Start"), named(NC, "Stop"), named(COIL, "Motor"))];

describe("the Ladder editor's gestures (OIP's, ported)", () => {
  it("builds a rung left to right: each element follows the one the last step selected", () => {
    let e = addRung([], -1);
    expect(e.selection).toEqual(at(0));
    e = insertElement(e.rungs, e.selection as Selection, named(NO, "Start"));
    e = insertElement(e.rungs, e.selection as Selection, named(NC, "Stop"));
    e = insertElement(e.rungs, e.selection as Selection, COIL);
    expect(text(e)).toEqual(["|Start| |/Stop| (?)"]);
    expect(e.selection).toEqual(at(0, 2));
  });

  it("puts an element picked with the rung selected at its start, a coil at its end", () => {
    const rungs = series();
    expect(text(insertElement(rungs, at(0), NO))).toEqual(["|?| |Start| |/Stop| (Motor)"]);
    const coil = insertElement(rungs, at(0), COIL);
    expect(text(coil)).toEqual(["|Start| |/Stop| (Motor) (?)"]);
    expect(coil.selection).toEqual(at(0, 3));
    // off the rungs: nothing changes
    expect(insertElement(rungs, at(4), NO).rungs).toEqual(rungs);
    expect(insertElement(rungs, at(0, 0, 1), NO).rungs).toEqual(rungs);
  });

  it("inserts an empty branch after the selected element and selects its lower path, for what runs in parallel", () => {
    const after = addBranch(series(), at(0, 0));
    expect(text(after)).toEqual(["|Start| [, ] |/Stop| (Motor)"]);
    expect(after.selection).toEqual(at(0, 1, 1));
    const parallel = insertElement(after.rungs, after.selection as Selection, named(NO, "Seal"));
    expect(text(parallel)).toEqual(["|Start| [, |Seal|] |/Stop| (Motor)"]);
    expect(parallel.selection).toEqual(at(0, 1, 1, 0));
    const nested = addBranch(after.rungs, at(0, 1, 1));
    expect(text(nested)).toEqual(["|Start| [, [, ]] |/Stop| (Motor)"]);
    expect(addBranch(series(), at(3)).selection).toEqual(at(3));
  });

  it("deletes an element, a path (the last one left rejoins the series), or the rung", () => {
    const rungs = [
      rung(named(NO, "A"), { kind: "branch", paths: [[named(NO, "B")], [named(NO, "C"), named(NO, "D")]] }),
    ];
    const element = removeSelected(rungs, at(0, 0));
    expect(text(element)).toEqual(["[|B|, |C| |D|]"]);
    expect(element.selection).toEqual(at(0));
    const path = removeSelected(rungs, at(0, 1, 0));
    expect(text(path)).toEqual(["|A| |C| |D|"]);
    expect(path.selection).toEqual(at(0));
    const two = addRung(rungs, 0).rungs;
    expect(removeSelected(two, at(1))).toEqual({ rungs, selection: at(0) });
    expect(removeSelected(rungs, at(0)).selection).toBeNull();
    for (const off of [at(2), at(0, 7), at(0, 1, 5), at(0, 0, 0)])
      expect(removeSelected(rungs, off).rungs).toEqual(rungs);
  });

  it("adds a rung below another, or first", () => {
    const rungs = series();
    expect(text(addRung(rungs, 0))).toEqual(["|Start| |/Stop| (Motor)", ""]);
    expect(text(addRung(rungs, -1))).toEqual(["", "|Start| |/Stop| (Motor)"]);
    expect(addRung(rungs, 9).selection).toEqual(at(1));
  });

  it("walks the elements with Left and Right, through branches and across rungs", () => {
    const rungs = [rung(named(NO, "A"), { kind: "branch", paths: [[named(NO, "B")], [named(NO, "C")]] }), rung()];
    expect(step(rungs, at(0, 0), 1)).toEqual(at(0, 1, 0, 0));
    expect(step(rungs, at(0, 1, 0, 0), 1)).toEqual(at(0, 1, 1, 0));
    expect(step(rungs, at(0, 1, 1, 0), 1)).toEqual(at(1));
    expect(step(rungs, at(1), 1)).toBeNull();
    expect(step(rungs, at(0, 0), -1)).toBeNull();
    // from a rung or a branch, into it from its near edge
    expect(step(rungs, at(0), 1)).toEqual(at(0, 0));
    expect(step(rungs, at(0, 1), -1)).toEqual(at(0, 0));
    expect(step(rungs, at(0, 1), 1)).toEqual(at(0, 1, 0, 0));
    expect(step(rungs, at(5), 1)).toBeNull();
  });

  it("edits rungs held behind a proxy, as Svelte's state holds them", () => {
    const held = new Proxy(series(), {});
    expect(text(insertElement(held, at(0, 0), NO))).toEqual(["|Start| |?| |/Stop| (Motor)"]);
  });

  it("finds the selected element", () => {
    const rungs = series();
    expect(elementAt(rungs, at(0, 2))).toEqual(named(COIL, "Motor"));
    expect(elementAt(rungs, at(0))).toBeNull();
    expect(elementAt(rungs, at(1, 0))).toBeNull();
  });
});

describe("modifiers on the selected contact or coil", () => {
  it("toggles a contact's: negated, then rising, and back to plain", () => {
    let e: Edit = { rungs: series(), selection: at(0, 0) };
    e = modify(e.rungs, at(0, 0), "negated");
    expect(text(e)).toEqual(["|/Start| |/Stop| (Motor)"]);
    expect(e.selection).toEqual(at(0, 0));
    e = modify(e.rungs, at(0, 0), "rising");
    expect(text(e)).toEqual(["|P Start| |/Stop| (Motor)"]);
    e = modify(e.rungs, at(0, 0), "rising");
    expect(text(e)).toEqual(["|Start| |/Stop| (Motor)"]);
    expect(text(modify(series(), at(0, 1), "falling"))).toEqual(["|Start| |N Stop| (Motor)"]);
  });

  it("toggles a coil's: set, reset, negated, P and N", () => {
    const rungs = series();
    expect(text(modify(rungs, at(0, 2), "set"))).toEqual(["|Start| |/Stop| (S Motor)"]);
    expect(text(modify(rungs, at(0, 2), "reset"))).toEqual(["|Start| |/Stop| (R Motor)"]);
    const negated = modify(rungs, at(0, 2), "negated");
    expect(text(negated)).toEqual(["|Start| |/Stop| (/Motor)"]);
    expect(text(modify(negated.rungs, at(0, 2), "negated"))).toEqual(["|Start| |/Stop| (Motor)"]);
    expect(text(modify(rungs, at(0, 2), "falling"))).toEqual(["|Start| |/Stop| (N Motor)"]);
  });

  it("leaves what a modifier does not apply to", () => {
    const rungs = [rung(named(NO, "A"), { kind: "box", head: "T1", pins: [] })];
    expect(modify(rungs, at(0, 0), "set").rungs).toEqual(rungs);
    expect(modify(rungs, at(0, 1), "negated").rungs).toEqual(rungs);
    expect(modify(rungs, at(0), "negated").rungs).toEqual(rungs);
    expect(takesModifier(named(NO, "A"), "set")).toBe(false);
    expect(takesModifier(COIL, "set")).toBe(true);
    expect(takesModifier(null, "negated")).toBe(false);
    expect(hasModifier(NC, "negated")).toBe(true);
    expect(hasModifier(NO, "negated")).toBe(false);
    expect(hasModifier(rungs[0]?.elements[1] ?? null, "negated")).toBe(false);
    expect(hasModifier(null, "negated")).toBe(false);
  });
});

describe("the toolbox", () => {
  it("places a blank contact, coil, box or branch by the selection", () => {
    const rungs = series();
    expect(text(place(rungs, at(0, 0), "contact"))).toEqual(["|Start| |?| |/Stop| (Motor)"]);
    expect(text(place(rungs, at(0), "coil"))).toEqual(["|Start| |/Stop| (Motor) (?)"]);
    expect(text(place(rungs, at(0, 1), "box"))).toEqual(["|Start| |/Stop| ?() (Motor)"]);
    expect(place(rungs, at(0, 0), "branch").selection).toEqual(at(0, 1, 1));
  });
});
