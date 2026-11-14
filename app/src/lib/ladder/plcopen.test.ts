import { describe, expect, it } from "vitest";
import { exportName, withImported } from "./plcopen";
import type { Source } from "../sources";

const tab = (key: string, kind: Source["kind"], text = ""): Source => ({ key, label: key, kind, text, base: text });
const SOURCES = [tab("plc/Main.st", "program", "PROGRAM Main END_PROGRAM"), tab("twinova.belt@1", "behaviour")];

describe("PLCopen in the PLC pane", () => {
  it("adds imported programs after the program's own, as tabs Build has not run yet", () => {
    const next = withImported(SOURCES, [["plc/Pump.ld", "|A| (X);\n"]]);
    expect(next).toEqual({
      sources: [
        SOURCES[0],
        { key: "plc/Pump.ld", label: "plc/Pump.ld", kind: "program", text: "|A| (X);\n", base: "" },
        SOURCES[1],
      ],
      replaced: [],
      added: ["plc/Pump.ld"],
    });
  });

  it("replaces the text of a file the program has, which stays to build until Build", () => {
    const pump = tab("plc/Pump.ld", "program", "|A| (X);\n");
    const next = withImported(
      [SOURCES[0] as Source, pump, SOURCES[1] as Source],
      [
        ["plc/Pump.ld", "|B| (X);\n"],
        ["plc/Valve.ld", "|C| (Y);\n"],
      ],
    );
    expect(next.replaced).toEqual(["plc/Pump.ld"]);
    expect(next.added).toEqual(["plc/Valve.ld"]);
    expect(next.sources.map((s) => [s.key, s.text, s.base])).toEqual([
      ["plc/Main.st", "PROGRAM Main END_PROGRAM", "PROGRAM Main END_PROGRAM"],
      ["plc/Pump.ld", "|B| (X);\n", "|A| (X);\n"],
      ["plc/Valve.ld", "|C| (Y);\n", ""],
      ["twinova.belt@1", "", ""],
    ]);
  });

  it("offers an export under the file's own name", () => {
    expect([exportName("plc/Pump.ld"), exportName("Main.ld"), exportName("plc/x.st")]).toEqual([
      "Pump.xml",
      "Main.xml",
      "x.st.xml",
    ]);
  });
});
