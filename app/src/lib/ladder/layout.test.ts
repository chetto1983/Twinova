import { describe, expect, it } from "vitest";
import {
  L,
  READING_MIN,
  boxPins,
  isReference,
  layoutRung,
  layoutRungs,
  pinLabels,
  rungMinWidth,
  textW,
  type LNode,
  type Resolve,
} from "./layout";
import type { BoxPins, Element, Rung } from "./model";

const contact = (operand: string | null, mode: "open" | "closed" | "rising" | "falling" = "open"): Element => ({
  kind: "contact",
  mode,
  operand,
});
const coil = (operand: string, mode: "normal" | "set" = "normal"): Element => ({ kind: "coil", mode, operand });
const rung = (elements: Element[], exit: Rung["exit"] = null): Rung => ({ comment: null, label: null, elements, exit });
const none: Resolve = () => undefined;
const feeds = (list: { feed: number[] | null }[]) => list.map((w) => (w.feed === null ? "rail" : w.feed.join(".")));
const at = (nodes: LNode[], path: number[]) => nodes.find((n) => "path" in n && n.path.join() === path.join());

/** An ADD that adds 1 to `target`, as ladder_line's rung 3 writes it. */
const increment = (target: string): Element => ({
  kind: "box",
  head: "ADD",
  pins: [
    { name: "IN1", value: target, out: false },
    { name: "IN2", value: "1", out: false },
    { name: "OUT", value: target, out: true },
  ],
});

const TON: BoxPins = { function: false, type_name: "TON", input: "IN", output: "Q" };
const timer: Element = {
  kind: "box",
  head: "T1",
  pins: [
    { name: "IN", value: null, out: false },
    { name: "PT", value: "T#2s", out: false },
    { name: "ET", value: "W", out: true },
  ],
};

describe("a rung laid out (nautilus's layout, ported)", () => {
  it("draws a seal-in: a branch of two contacts on the wire and below it, a contact, the coil against the right rail", () => {
    const seal = rung([
      { kind: "branch", paths: [[contact("Start")], [contact("Motor")]] },
      contact("Stop", "closed"),
      coil("Motor"),
    ]);
    const r = layoutRung(seal, 600, none);
    const start = at(r.nodes, [0, 0, 0]);
    const motor = at(r.nodes, [0, 1, 0]);
    const stop = at(r.nodes, [1]);
    const out = at(r.nodes, [2]);
    expect(start?.y).toBe(r.wireY - L.SYMBOL_H / 2);
    expect(motor && start && motor.y > start.y).toBe(true);
    expect(stop?.kind === "contact" && stop.mode).toBe("closed");
    expect(out && out.x + out.w).toBe(600 - L.RAIL_RIGHT_MARGIN);
    // the branch's rails join the paths, and the rung is tall enough for its lower path
    expect(r.vlines).toHaveLength(2);
    expect(r.branches[0]?.path).toEqual([0]);
    expect(r.branches[0]?.legs).toEqual([r.wireY, (motor?.y ?? 0) + L.SYMBOL_H / 2]);
    expect(motor && r.height > motor.y + motor.h).toBe(true);
    // the wire runs from the left rail to the right one
    expect(r.wires.some((w) => w.x1 === L.RAIL_X && w.y1 === r.wireY)).toBe(true);
    // each wire and rail carries one element's power out, or the left rail's
    expect(feeds(r.vlines)).toEqual(["rail", "0"]);
    expect(feeds(r.wires.filter((w) => w.y1 === r.wireY)).toSorted()).toEqual(["0", "1", "2", "rail"]);
    expect(r.wires.some((w) => w.x2 === 600 - L.RAIL_X && w.y2 === r.wireY)).toBe(true);
  });

  it("pads a shorter path, and an empty path, with wire to the branch's right rail", () => {
    const r = layoutRung(
      rung([contact("A"), { kind: "branch", paths: [[contact("B"), contact("C")], []] }, coil("Q")]),
      600,
      none,
    );
    const [left, right] = r.vlines;
    const lowY = left?.y2 ?? 0;
    const pad = r.wires.find((w) => w.y1 === lowY && w.x2 === right?.x);
    expect(pad).toBeDefined();
    expect(left && right && right.x - left.x).toBeGreaterThan(L.EMPTY_LEG_W);
  });

  it("draws a box with the rung's pins on the wire, values and places outside", () => {
    const r = layoutRung(rung([contact("A"), timer, coil("X")]), 600, (path) =>
      path.join() === "1" ? TON : undefined,
    );
    const box = at(r.nodes, [1]);
    expect(box?.kind).toBe("box");
    if (box?.kind !== "box") return;
    expect(box.title).toBe("TON");
    expect(box.instance).toBe("T1");
    expect(box.left).toEqual([
      { name: "IN", value: null, rung: true },
      { name: "PT", value: "T#2s", rung: false },
    ]);
    expect(box.right).toEqual([
      { name: "Q", value: null, rung: true },
      { name: "ET", value: "W", rung: false },
    ]);
    // the first pins sit on the rung's wire
    expect(box.y + L.BOX_HEAD + L.PIN_STEP / 2).toBe(r.wireY);
    // room after it for W, its value online beside it, and the wire to ET
    const room = L.PIN_STUB + 3 * L.LABEL_GAP + Math.ceil(textW("W", 10)) + READING_MIN;
    expect(r.wires.some((w) => w.x2 - w.x1 === room)).toBe(true);
    expect([
      isReference("W"),
      isReference("T1.Q"),
      isReference("T#2s"),
      isReference("count MOD 3 = 0"),
      isReference("TRUE"),
      isReference(null),
    ]).toEqual([true, true, false, false, false, false]);
    const a = at(r.nodes, [0]);
    expect(a && box.x - (a.x + a.w)).toBeGreaterThanOrEqual(Math.ceil(textW("T#2s", 10)));
  });

  it("names a function's box by the function and no instance, and one that did not lower by its pins as written", () => {
    const add: Element = {
      kind: "box",
      head: "ADD",
      pins: [
        { name: "IN1", value: "N", out: false },
        { name: "OUT", value: "N", out: true },
      ],
    };
    const lowered: BoxPins = { function: true, type_name: "ADD", input: "EN", output: "ENO" };
    const r = layoutRung(rung([contact("A"), add]), 600, () => lowered);
    const box = at(r.nodes, [1]);
    expect(box?.kind === "box" && [box.title, box.instance]).toEqual(["ADD", null]);
    expect(boxPins(timer.kind === "box" ? timer.pins : [], undefined)).toEqual({
      left: [
        { name: "IN", value: null, rung: true },
        { name: "PT", value: "T#2s", rung: false },
      ],
      right: [{ name: "ET", value: "W", rung: false }],
    });
    const compare: BoxPins = { function: true, type_name: "EQ", input: null, output: "OUT" };
    const pins = [
      { name: "IN1", value: "N", out: false },
      { name: "IN2", value: "1", out: false },
      { name: "OUT", value: null, out: true },
    ];
    expect(boxPins(pins, compare)).toEqual({
      left: [
        { name: "", value: null, rung: false },
        { name: "IN1", value: "N", rung: false },
        { name: "IN2", value: "1", rung: false },
      ],
      right: [{ name: "OUT", value: null, rung: true }],
    });
    const blank = layoutRung(rung([{ kind: "box", head: null, pins: [] }]), 600, none);
    expect(blank.nodes[0]?.kind === "box" && blank.nodes[0].title).toBe("???");
  });

  it("ends a rung with its jump or RETURN after its coils", () => {
    const r = layoutRung(rung([contact("A"), coil("X", "set")], { kind: "jump", label: "Done" }), 600, none);
    const exit = r.nodes.find((n) => n.kind === "exit");
    const x = at(r.nodes, [1]);
    expect(exit?.kind === "exit" && exit.text).toBe(">> Done");
    expect(exit && x && exit.x).toBeGreaterThan((x?.x ?? 0) + (x?.w ?? 0));
    const ret = layoutRung(rung([], { kind: "return" }), 600, none);
    expect(ret.nodes.find((n) => n.kind === "exit")).toMatchObject({ text: "RETURN" });
  });

  it("writes each pin's place on its own row, joined to the pin, with room for its reading (ladder_line's rung 3)", () => {
    const move: Element = {
      kind: "box",
      head: "MOVE",
      pins: [
        { name: "IN", value: "count MOD 3 = 0", out: false },
        { name: "OUT", value: "reject_next", out: true },
      ],
    };
    const fn: BoxPins = { function: true, type_name: "ADD", input: "EN", output: "ENO" };
    const r = layoutRung(rung([contact("entered.Q"), increment("count"), increment("in_zone"), move]), 600, () => fn);
    const boxes = r.nodes.filter((n) => n.kind === "box");
    const extents = boxes.flatMap((box) =>
      pinLabels(box).map((label) => {
        const reading = isReference(label.text) ? READING_MIN + L.LABEL_GAP : 0;
        const text = textW(label.text, 10);
        return label.side === "left"
          ? { y: label.y, from: label.textX - text - reading, to: label.stub?.x2 ?? label.textX, box }
          : { y: label.y, from: label.stub?.x1 ?? label.textX, to: label.textX + text + reading, box };
      }),
    );
    // IN1's place and reading, then IN2's constant, each on its own row of the box
    const first = pinLabels(boxes[0]!);
    expect(first.map((l) => [l.side, l.text])).toEqual([
      ["left", "count"],
      ["left", "1"],
      ["right", "count"],
    ]);
    expect(new Set(first.filter((l) => l.side === "left").map((l) => l.y)).size).toBe(2);
    // every valued pin has a wire from its place to the box's edge
    for (const box of boxes) {
      for (const label of pinLabels(box).filter((l) => l.stub !== null)) {
        expect(label.side === "left" ? label.stub?.x2 : label.stub?.x1).toBe(
          label.side === "left" ? box.x : box.x + box.w,
        );
        expect(label.stub?.y).toBe(label.y);
      }
    }
    // no place or reading reaches into the element before or after it, or into another place on its row
    const placed = r.nodes.map((n) => [n.x, n.x + n.w] as const);
    for (const e of extents) {
      for (const [from, to] of placed) expect(e.to <= from || e.from >= to || e.box.x === from).toBe(true);
      for (const other of extents)
        if (other !== e && other.y === e.y) expect(e.to <= other.from || e.from >= other.to).toBe(true);
    }
    // MOVE's expression is joined to IN, and the room left of MOVE holds it
    const expression = pinLabels(boxes[2]!).find((l) => l.text === "count MOD 3 = 0");
    expect(expression?.stub).not.toBeNull();
  });

  it("lays the rungs out as wide as the pane, or as the widest rung when the pane is narrower", () => {
    const short = rung([contact("A"), coil("X")]);
    expect(layoutRungs([short], () => none, 1200)[0]?.width).toBe(1200);
    expect(layoutRungs([short], () => none, 300)[0]?.width).toBe(L.MIN_WIDTH);
    const coilAt = (width: number) => at(layoutRungs([short], () => none, width)[0]!.nodes, [1]);
    expect((coilAt(1200)?.x ?? 0) - (coilAt(600)?.x ?? 0)).toBe(600);
  });

  it("lays every rung as wide as the widest, its right rail with the others'", () => {
    const short = rung([contact("A"), coil("X")]);
    const long = rung(Array.from({ length: 9 }, (_, i) => contact(`Input_${i}`)).concat(coil("Y")));
    expect(rungMinWidth(long, none)).toBeGreaterThan(L.MIN_WIDTH);
    const [a, b] = layoutRungs([short, long], () => none);
    expect(a?.width).toBe(b?.width);
    expect(a?.width).toBe(rungMinWidth(long, none));
    expect(layoutRungs([short], () => none)[0]?.width).toBe(L.MIN_WIDTH);
  });

  it("draws an empty rung as a wire, and a blank operand as ???", () => {
    const empty = layoutRung(rung([]), 600, none);
    expect(empty.nodes).toEqual([]);
    expect(empty.wires.some((w) => w.x2 - w.x1 === L.EMPTY_LEG_W)).toBe(true);
    const blank = layoutRung(rung([contact(null)]), 600, none);
    expect(blank.nodes[0]?.w).toBe(L.SYMBOL_W);
  });
});
