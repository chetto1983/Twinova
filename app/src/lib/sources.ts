// The editor's tabs (Editor.svelte): a project's ST sources, and back to what Build sends (commands.rs `build`).
import type { ProjectView } from "./project";

export interface Source {
  /** The program file's path, or the definition's `id@version`. */
  key: string;
  label: string;
  kind: "program" | "behaviour";
  text: string;
}

/** The line program's files first, then each behaviour by definition key. */
export function sourcesOf(project: ProjectView): Source[] {
  return [
    ...project.program.map(([path, text]) => ({ key: path, label: path, kind: "program" as const, text })),
    ...Object.entries(project.behaviours).map(([key, text]) => ({ key, label: key, kind: "behaviour" as const, text })),
  ];
}

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
  const edited = new Map(current.map((s) => [s.key, s.text]));
  return sourcesOf(project).map((s) => Object.assign(s, { text: edited.get(s.key) ?? s.text }));
}
