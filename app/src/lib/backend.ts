// The Tauri commands of src-tauri/src/commands.rs and project_commands.rs, and the events of the Run loop.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { ProjectView } from "./project";
import type { Snapshot } from "./snapshot";

export const backend = {
  defaultProgram: () => invoke<string>("default_program"),
  build: (source: string) => invoke<Snapshot>("build", { source }),
  reset: () => invoke<Snapshot>("reset"),
  step: () => invoke<Snapshot>("step"),
  /** `speed` times the wall clock, or as fast as possible when null. */
  run: (speed: number | null) => invoke<void>("run", { speed }),
  saveState: () => invoke<void>("save_state"),
  restoreState: () => invoke<Snapshot>("restore_state"),
  pause: () => invoke<void>("pause"),
  press: (start: boolean, stop: boolean) => invoke<void>("press", { start, stop }),
  setOutletBlocked: (blocked: boolean) => invoke<Snapshot>("set_outlet_blocked", { blocked }),
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
  save: (folder: string) => invoke<ProjectView>("save_project", { folder }),
  saveAs: async (): Promise<ProjectView | null> => {
    const folder = await open({ directory: true });
    return folder === null ? null : projects.save(folder);
  },
  exportArchive: async (): Promise<string | null> => {
    const file = await save({ filters: ARCHIVE });
    return file === null ? null : invoke<void>("export_project", { file }).then(() => file);
  },
  importArchive: async (): Promise<ProjectView | null> => {
    const file = await open({ filters: ARCHIVE });
    return file === null ? null : invoke<ProjectView>("import_project", { file });
  },
};
