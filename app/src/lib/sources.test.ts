import { describe, expect, it } from "vitest";
import { buildArgs, builtFrom, edited, fromDisk, mergeSources, sourcesOf, type DiskSources } from "./sources";
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
      {
        key: "plc/Main.st",
        label: "plc/Main.st",
        kind: "program",
        text: "PROGRAM Main END_PROGRAM",
        base: "PROGRAM Main END_PROGRAM",
      },
      {
        key: "plc/Lib.st",
        label: "plc/Lib.st",
        kind: "program",
        text: "FUNCTION F : INT END_FUNCTION",
        base: "FUNCTION F : INT END_FUNCTION",
      },
      {
        key: "twinova.belt@1",
        label: "twinova.belt@1",
        kind: "behaviour",
        text: "FUNCTION_BLOCK BeltBehaviour END_FUNCTION_BLOCK",
        base: "FUNCTION_BLOCK BeltBehaviour END_FUNCTION_BLOCK",
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

const disk = (main: string, belt = project.behaviours["twinova.belt@1"]): DiskSources => ({
  kind: "program",
  program: [
    ["plc/Main.st", main],
    ["plc/Lib.st", "FUNCTION F : INT END_FUNCTION"],
  ],
  behaviours: { "twinova.belt@1": belt },
});

describe("sources saved outside Twinova", () => {
  it("marks a tab edited while its text differs from the project's", () => {
    const sources = sourcesOf(project);
    expect(sources.map(edited)).toEqual([false, false, false]);
    sources[1].text = "typed";
    expect(sources.map(edited)).toEqual([false, true, false]);
    sources[1].text = sources[1].base;
    expect(edited(sources[1])).toBe(false);
  });

  it("takes each file that changed on disk, as its tab's text and base", () => {
    const next = fromDisk(sourcesOf(project), disk("PROGRAM Main (* saved *) END_PROGRAM", "FB saved"));
    expect(next.held).toEqual([]);
    expect(next.sources.map((s) => [s.text, s.base])).toEqual([
      ["PROGRAM Main (* saved *) END_PROGRAM", "PROGRAM Main (* saved *) END_PROGRAM"],
      ["FUNCTION F : INT END_FUNCTION", "FUNCTION F : INT END_FUNCTION"],
      ["FB saved", "FB saved"],
    ]);
  });

  it("keeps a tab's own edits when its file did not change on disk", () => {
    const sources = sourcesOf(project);
    sources[1].text = "FUNCTION F : INT (* typed *) END_FUNCTION";
    const next = fromDisk(sources, disk("PROGRAM Main (* saved *) END_PROGRAM"));
    expect(next.held).toEqual([]);
    expect(next.sources[1]).toEqual(sources[1]);
    expect(next.sources[0].text).toBe("PROGRAM Main (* saved *) END_PROGRAM");
  });

  it("holds everything when a file changed on disk and in its tab, naming it", () => {
    const sources = sourcesOf(project);
    sources[0].text = "PROGRAM Main (* typed *) END_PROGRAM";
    const next = fromDisk(sources, disk("PROGRAM Main (* saved *) END_PROGRAM", "FB saved"));
    expect(next).toEqual({ sources, held: ["plc/Main.st"] });
  });

  it("takes a file whose disk text is what the tab already holds", () => {
    const sources = sourcesOf(project);
    sources[0].text = "PROGRAM Main (* both *) END_PROGRAM";
    const next = fromDisk(sources, disk("PROGRAM Main (* both *) END_PROGRAM"));
    expect(next.held).toEqual([]);
    expect(edited(next.sources[0])).toBe(false);
  });

  it("leaves no tab edited once Build took the texts", () => {
    const sources = sourcesOf(project);
    sources[2].text = "typed";
    const built = builtFrom(sources);
    expect(built.map(edited)).toEqual([false, false, false]);
    expect(built[2].text).toBe("typed");
  });
});
