import { describe, expect, it } from "vitest";
import { buildArgs, mergeSources, sourcesOf } from "./sources";
import type { ProjectView } from "./project";

const project = {
  program: [
    ["plc/Main.st", "PROGRAM Main END_PROGRAM"],
    ["plc/Lib.st", "FUNCTION F : INT END_FUNCTION"],
  ],
  behaviours: { "twinova.belt@1": "FUNCTION_BLOCK BeltBehaviour END_FUNCTION_BLOCK" },
} as unknown as ProjectView;

describe("editor sources", () => {
  it("lists the program's files first, then each behaviour", () => {
    expect(sourcesOf(project)).toEqual([
      { key: "plc/Main.st", label: "plc/Main.st", kind: "program", text: "PROGRAM Main END_PROGRAM" },
      { key: "plc/Lib.st", label: "plc/Lib.st", kind: "program", text: "FUNCTION F : INT END_FUNCTION" },
      {
        key: "twinova.belt@1",
        label: "twinova.belt@1",
        kind: "behaviour",
        text: "FUNCTION_BLOCK BeltBehaviour END_FUNCTION_BLOCK",
      },
    ]);
  });

  it("gives Build back what it lists, edits included", () => {
    const sources = sourcesOf(project);
    sources[2].text = "edited";
    expect(buildArgs(sources)).toEqual({
      program: [
        ["plc/Main.st", "PROGRAM Main END_PROGRAM"],
        ["plc/Lib.st", "FUNCTION F : INT END_FUNCTION"],
      ],
      behaviours: { "twinova.belt@1": "edited" },
    });
  });

  it("keeps the texts being edited when the scene changes, adds a new block and drops a gone one", () => {
    const sources = sourcesOf(project);
    sources[0].text = "PROGRAM Main (* typed *) END_PROGRAM";
    const next = {
      program: project.program,
      behaviours: { "twinova.pusher@1": "FUNCTION_BLOCK PusherBehaviour END_FUNCTION_BLOCK" },
    } as unknown as ProjectView;
    expect(mergeSources(sources, next).map((s) => [s.key, s.text])).toEqual([
      ["plc/Main.st", "PROGRAM Main (* typed *) END_PROGRAM"],
      ["plc/Lib.st", "FUNCTION F : INT END_FUNCTION"],
      ["twinova.pusher@1", "FUNCTION_BLOCK PusherBehaviour END_FUNCTION_BLOCK"],
    ]);
  });
});
