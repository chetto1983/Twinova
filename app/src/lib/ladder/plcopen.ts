// PLCopen TC6 XML in and out of the PLC pane (spec 2026-11-13-ladder-editor-design.md, decision 7): what the
// editor's tabs become once a file's Ladder programs are imported, and the name an export is offered under.
import type { Source } from "../sources";

/** What `ladder_import` gives (twinova_engine::plcopen::Imported): the programs as files of the PLC, and why the rest
 *  of the file was left out. */
export interface Imported {
  files: [string, string][];
  notes: string[];
}

/** The tabs with `files` added as program tabs not built yet, after the program's own, so Build runs them and Save
 *  writes them; or, when a file of the program has one of their paths, those paths, and nothing is added. */
export function withImported(
  sources: Source[],
  files: [string, string][],
): { sources: Source[] } | { taken: string[] } {
  const taken = files.map(([path]) => path).filter((path) => sources.some((s) => s.key === path));
  if (taken.length > 0) return { taken };
  const added: Source[] = files.map(([path, text]) => ({ key: path, label: path, kind: "program", text, base: "" }));
  const programs = sources.filter((s) => s.kind === "program");
  return { sources: [...programs, ...added, ...sources.filter((s) => s.kind !== "program")] };
}

/** The name the save dialog offers for Ladder file `file`: `plc/Pump.ld` as `Pump.xml`. */
export const exportName = (file: string): string => `${(file.split("/").pop() ?? file).replace(/\.ld$/, "")}.xml`;
