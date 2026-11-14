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
    });
  });

  it("refuses files the program has already, adding none of them", () => {
    expect(
      withImported(SOURCES, [
        ["plc/Pump.ld", ""],
        ["plc/Main.st", ""],
      ]),
    ).toEqual({ taken: ["plc/Main.st"] });
  });

  it("offers an export under the file's own name", () => {
    expect([exportName("plc/Pump.ld"), exportName("Main.ld"), exportName("plc/x.st")]).toEqual([
      "Pump.xml",
      "Main.xml",
      "x.st.xml",
    ]);
  });
});
