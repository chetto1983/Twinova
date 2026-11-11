// The Tauri commands of src-tauri/src/commands.rs, project_commands.rs and edit_commands.rs, and the events of the Run
// loop.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { Edit, ProjectView } from "./project";
import type { Built, Snapshot } from "./snapshot";

export const backend = {
  /** Builds the current project with the editor's sources (commands.rs `build`). */
  build: (program: [string, string][], behaviours: Record<string, string>) =>
    invoke<Built>("build", { program, behaviours }),
  reset: () => invoke<Snapshot>("reset"),
  step: () => invoke<Snapshot>("step"),
  /** `speed` times the wall clock, or as fast as possible when null. */
  run: (speed: number | null) => invoke<void>("run", { speed }),
  saveState: () => invoke<void>("save_state"),
  restoreState: () => invoke<Snapshot>("restore_state"),
  pause: () => invoke<void>("pause"),
  /** A panel button, by its input address: held for the next scan. */
  press: (address: string) => invoke<void>("press", { address }),
  setSinkBlocked: (sink: number, blocked: boolean) => invoke<Snapshot>("set_sink_blocked", { sink, blocked }),
};

export const onSnapshot = (f: (s: Snapshot) => void): Promise<UnlistenFn> =>
  listen<Snapshot>("snapshot", (e) => f(e.payload));
export const onEngineError = (f: (error: string) => void): Promise<UnlistenFn> =>
  listen<string>("engine-error", (e) => f(e.payload));
export const onStopped = (f: () => void): Promise<UnlistenFn> => listen("stopped", () => f());

/** An archive is the project folder zipped (spec 2026-11-08-project-format-design.md, decision 1). */
const ARCHIVE = [{ name: "Twinova project", extensions: ["zip"] }];

/** Project commands behind the system dialogs; each resolves to null when the user cancels the dialog. */
export const projects = {
  open: async (): Promise<ProjectView | null> => {
    const folder = await open({ directory: true });
    return folder === null ? null : invoke<ProjectView>("open_project", { folder });
  },
  /** The project the app shows: the bundled demo at start. */
  current: () => invoke<ProjectView>("current_project"),
  demo: () => invoke<ProjectView>("open_demo"),
  /** The folder named on the command line (spec 2026-11-08-project-format-design.md, decision 8), or null. */
  launch: () => invoke<ProjectView | null>("open_launch_project"),
  save: (folder: string) => invoke<ProjectView>("save_project", { folder }),
  saveAs: async (): Promise<ProjectView | null> => {
    const folder = await open({ directory: true });
    return folder === null ? null : projects.save(folder);
  },
  exportArchive: async (): Promise<string | null> => {
    const file = await save({ filters: ARCHIVE });
    return file === null ? null : invoke<void>("export_project", { file }).then(() => file);
  },
  /** An empty scene with an empty program, not yet saved to a folder. */
  create: () => invoke<ProjectView>("new_project"),
  /** One edit of the open project, one undo step (edit_commands.rs); rejects with the edit's error. */
  edit: (op: Edit) => invoke<ProjectView>("edit", { op }),
  undo: () => invoke<ProjectView>("undo"),
  redo: () => invoke<ProjectView>("redo"),
  importArchive: async (): Promise<ProjectView | null> => {
    const file = await open({ filters: ARCHIVE });
    return file === null ? null : invoke<ProjectView>("import_project", { file });
  },
};
