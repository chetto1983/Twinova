import { describe, expect, it } from "vitest";
import type { Completion } from "../backend";
import type { Element } from "./model";
import { boolRow, candidates, elementType, declarable, operandOf, roleOf } from "./operand";

const c = (label: string, kind: string, detail: string | null = null): Completion => ({
  label,
  kind,
  detail,
  insert_text: label,
});

// what truST offers for "St" in a Ladder file (ladder_operand_tests.rs), keywords and snippets among them
const OFFERED = [
  c("STRING#'text'", "snippet"),
  c("STRUCT", "keyword"),
  c("Stage", "variable", "INT"),
  c("Start", "variable", "BOOL"),
  c("Stop", "variable", "BOOL"),
  c("Start", "variable", "BOOL"),
  c("STRING_TO_CHAR", "function", "standard function"),
  c("Stamp", "function_block"),
  c("Steps", "variable", "ARRAY [0..9] OF BOOL"),
];

describe("an element's operand", () => {
  it("offers a bit its variables, the BOOLs first, each once", () => {
    expect(candidates(OFFERED, "St", "bit")).toEqual(["Start", "Stop", "Steps", "Stage"]);
    expect(candidates(OFFERED, "sto", "bit")).toEqual(["Stop"]);
    expect(candidates(OFFERED, "X", "bit")).toEqual([]);
  });

  it("offers a box its instances, functions and blocks, in truST's order", () => {
    expect(candidates(OFFERED, "st", "box")).toEqual(["Stage", "Start", "Stop", "STRING_TO_CHAR", "Stamp", "Steps"]);
  });

  it("reads a declared type's element", () => {
    expect(elementType("ARRAY [0..9] OF BOOL")).toBe("BOOL");
    expect(elementType("string[20]")).toBe("STRING");
    expect(elementType(" dint ")).toBe("DINT");
  });

  it("knows the role and the operand of an element", () => {
    const contact: Element = { kind: "contact", mode: "open", operand: "A" };
    const box: Element = { kind: "box", head: "T1", pins: [] };
    expect([roleOf(contact), roleOf({ kind: "coil", mode: "set", operand: null }), roleOf(box)]).toEqual([
      "bit",
      "bit",
      "box",
    ]);
    expect([roleOf({ kind: "branch", paths: [] }), roleOf(null)]).toEqual([null, null]);
    expect([operandOf(contact), operandOf(box), operandOf({ kind: "coil", mode: "set", operand: null })]).toEqual([
      "A",
      "T1",
      "",
    ]);
    expect([operandOf({ kind: "branch", paths: [] }), operandOf(null)]).toEqual(["", ""]);
  });

  it("declares an undeclared bit as a BOOL of the program, when it is one identifier", () => {
    expect(["Pump_1", "_x", "a__b", "1st", "T1.Q", "A B", "", "TRUE", "false"].map(declarable)).toEqual([
      true,
      true,
      false,
      false,
      false,
      false,
      false,
      false,
      false,
    ]);
    expect(boolRow("Pump", "Run")).toEqual({
      pou: "Pump",
      name: "Run",
      class: "VAR",
      qualifier: null,
      type_name: "BOOL",
      address: null,
      initial: null,
      comment: null,
    });
  });
});
