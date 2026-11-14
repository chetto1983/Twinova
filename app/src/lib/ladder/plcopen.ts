// PLCopen TC6 XML in and out of the PLC pane (spec 2026-11-13-ladder-editor-design.md, decision 7): what the
// editor's tabs become once a file's Ladder programs are imported, and the name an export is offered under.
import type { Source } from "../sources";

/** What `ladder_import` gives (twinova_engine::plcopen::Imported): the programs as files of the PLC, and why the rest
 *  of the file was left out. */
export interface Imported {
  files: [string, string][];
  notes: string[];
}

/** The tabs after an import: a file the program has takes the imported text, a new one is added after the program's
 *  own tabs; either is a tab Build has not run yet, which Build runs and Save writes. The paths replaced and added. */
export function withImported(
  sources: Source[],
  files: [string, string][],
): { sources: Source[]; replaced: string[]; added: string[] } {
  const texts = new Map(files);
  const replaced = files.map(([path]) => path).filter((path) => sources.some((s) => s.key === path));
  const added = files.map(([path]) => path).filter((path) => !replaced.includes(path));
  const kept = sources.map((s) => (texts.has(s.key) ? { ...s, text: texts.get(s.key) ?? s.text } : s));
  const tabs: Source[] = added.map((path) => ({
    key: path,
    label: path,
    kind: "program",
    text: texts.get(path) ?? "",
    base: "",
  }));
  const programs = kept.filter((s) => s.kind === "program");
  return { sources: [...programs, ...tabs, ...kept.filter((s) => s.kind !== "program")], replaced, added };
}

/** The name the save dialog offers for Ladder file `file`: `plc/Pump.ld` as `Pump.xml`. */
export const exportName = (file: string): string => `${(file.split("/").pop() ?? file).replace(/\.ld$/, "")}.xml`;
