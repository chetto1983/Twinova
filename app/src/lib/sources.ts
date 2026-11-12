// The editor's tabs (Editor.svelte): a project's ST sources, and back to what Build sends (commands.rs `build`); the
// sources saved outside Twinova come in through watch.rs (spec 2026-11-12-st-vscode-design.md).
import type { ProjectView } from "./project";

export interface Source {
  /** The program file's path, or the definition's `id@version`. */
  key: string;
  label: string;
  kind: "program" | "behaviour";
  text: string;
  /** The text as the project or its folder last had it: the tab has edits of its own while its text differs. */
  base: string;
}

/** The sources in the project's folder (watch.rs `DiskSources`): `behaviours` when a behaviour changed, a rebuild. */
export interface DiskSources {
  kind: "program" | "behaviours";
  program: [string, string][];
  behaviours: Record<string, string>;
}

/** The line program's files first, then each behaviour by definition key. */
export function sourcesOf(project: ProjectView): Source[] {
  const source = (key: string, kind: Source["kind"], text: string): Source => ({
    key,
    label: key,
    kind,
    text,
    base: text,
  });
  return [
    ...project.program.map(([path, text]) => source(path, "program", text)),
    ...Object.entries(project.behaviours).map(([key, text]) => source(key, "behaviour", text)),
  ];
}

export const edited = (source: Source): boolean => source.text !== source.base;

/** What Build takes: the program as [path, text] pairs in order, and the behaviours by definition key. */
export function buildArgs(sources: Source[]): { program: [string, string][]; behaviours: Record<string, string> } {
  return {
    program: sources.filter((s) => s.kind === "program").map((s) => [s.key, s.text]),
    behaviours: Object.fromEntries(sources.filter((s) => s.kind === "behaviour").map((s) => [s.key, s.text])),
  };
}

/** The editor's tabs after an edit of the scene: the texts being edited stay, a block a newly placed family brought is
 *  added, and a block whose definition left the project goes with it. */
export function mergeSources(current: Source[], project: ProjectView): Source[] {
  const texts = new Map(current.map((s) => [s.key, s.text]));
  return sourcesOf(project).map((s) => Object.assign(s, { text: texts.get(s.key) ?? s.text }));
}

/** The tabs after the folder's sources (decision 6): each file that changed there replaces its tab's text, and a tab
 *  with edits of its own keeps them while its file did not change. When a file changed there and in its tab, nothing
 *  is taken and `held` names those files, so nothing typed is lost. */
export function fromDisk(sources: Source[], disk: DiskSources): { sources: Source[]; held: string[] } {
  const texts = new Map([...disk.program, ...Object.entries(disk.behaviours)]);
  const changed = (s: Source): string | undefined => {
    const text = texts.get(s.key);
    return text === s.base ? undefined : text;
  };
  const held = sources.filter((s) => edited(s) && changed(s) !== undefined && changed(s) !== s.text);
  if (held.length > 0) return { sources, held: held.map((s) => s.label) };
  return {
    sources: sources.map((s) => {
      const text = changed(s);
      return text === undefined ? s : { ...s, text, base: text };
    }),
    held: [],
  };
}

/** The tabs after a Build or an online change that took their texts: none has edits of its own any more. */
export const builtFrom = (sources: Source[]): Source[] => sources.map((s) => ({ ...s, base: s.text }));
