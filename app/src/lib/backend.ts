// The Tauri commands of src-tauri/src/commands.rs, project_commands.rs and edit_commands.rs, and the events of the Run
// loop and of the project folder's watch.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { Edit, ProjectView } from "./project";
import type { Built, Snapshot, ScanTiming } from "./snapshot";
import type { DiskSources } from "./sources";
import type { Problems } from "./analysis";
import type { Change, Declaration, Force, Target, Watched } from "./monitor";
import type { Breakpoint, Hold, Log, Resume } from "./debug";
import type { Proposal, Reference, Span, Template } from "./navigation";

/** Each invoke is a fetch of its own (Tauri 2.12 `scripts/ipc-protocol.js`, `sendIpcMessage`), so two not awaited
 * may reach the app in either order; WebView2 was measured doing so (CI, 2026-11-13). A command that moves the line
 * waits for the breakpoints set before it, so it never runs with the ones the user just changed. */
let breakpointsSent: Promise<unknown> = Promise.resolve();
const afterBreakpoints = <T>(command: () => Promise<T>): Promise<T> => breakpointsSent.then(command);

export const backend = {
  scanTiming: () => invoke<ScanTiming>("scan_timing"),
  /** Builds the current project with the editor's sources (commands.rs `build`). */
  build: (program: [string, string][], behaviours: Record<string, string>) =>
    invoke<Built>("build", { program, behaviours }),
  /** An online change of the line program while the plant goes on (commands.rs `change_program`). */
  changeProgram: (program: [string, string][]) => invoke<Built>("change_program", { program }),
  reset: () => invoke<Snapshot>("reset"),
  step: () => afterBreakpoints(() => invoke<Snapshot>("step")),
  /** `speed` times the wall clock, or as fast as possible when null. */
  run: (speed: number | null) => afterBreakpoints(() => invoke<void>("run", { speed })),
  saveState: () => invoke<void>("save_state"),
  restoreState: () => invoke<Snapshot>("restore_state"),
  pause: () => invoke<void>("pause"),
  /** A panel button, by its input address: held for the next scan. */
  press: (address: string) => invoke<void>("press", { address }),
  setSinkBlocked: (sink: number, blocked: boolean) => invoke<Snapshot>("set_sink_blocked", { sink, blocked }),
};

/** A family the catalogue offers (edit_commands.rs `CatalogueEntry`). */
export interface CatalogueEntry {
  key: string;
  group: "conveyors" | "devices" | "sensors" | "flow";
}

export const catalogue = () => invoke<CatalogueEntry[]>("catalogue");

/** A completion as `analysis.rs` sends it. */
export interface Completion {
  label: string;
  kind: string;
  detail: string | null;
  insert_text: string;
}

/** truST's analysis of the editor's texts (src-tauri/src/analysis.rs): positions are 1-based, in UTF-16 units. */
export const analysis = {
  analyze: (program: [string, string][], behaviours: Record<string, string>) =>
    invoke<Problems>("analyze", { program, behaviours }),
  hover: (file: string, text: string, line: number, column: number) =>
    invoke<string | null>("hover", { file, text, line, column }),
  completion: (file: string, text: string, line: number, column: number) =>
    invoke<Completion[]>("completion", { file, text, line, column }),
};

/** Navigation, call templates and Auto Declare (navigation.rs, declarations.rs, call_templates.rs). */
export const navigation = {
  definition: (file: string, text: string, line: number, column: number) =>
    invoke<Span | null>("definition", { file, text, line, column }),
  references: (file: string, text: string, line: number, column: number) =>
    invoke<Reference[]>("references", { file, text, line, column }),
  rename: (files: [string, string][], file: string, line: number, column: number, newName: string) =>
    invoke<[string, string][]>("rename_symbol", { files, file, line, column, newName }),
  callTemplate: (files: [string, string][], file: string, line: number, column: number) =>
    invoke<Template | null>("call_template", { files, file, line, column }),
  declareProposal: (files: [string, string][], file: string, line: number, column: number) =>
    invoke<Proposal | null>("declare_proposal", { files, file, line, column }),
};

/** Online monitoring and forcing (src-tauri/src/monitor.rs). */
/** Breakpoints and HOLD (debug_commands.rs): the lines a file's breakpoints landed on, and where the program is held. */
export const debug = {
  setBreakpoints: (file: string, breakpoints: Breakpoint[]) => {
    const set = invoke<Breakpoint[]>("set_breakpoints", { file, breakpoints });
    const settled = set.catch(() => undefined);
    breakpointsSent = Promise.all([breakpointsSent, settled]);
    return set;
  },
  runToCursor: (file: string, line: number) => afterBreakpoints(() => invoke<void>("run_to_cursor", { file, line })),
  resume: (how: Resume) => afterBreakpoints(() => invoke<void>("resume", { how })),
  hold: () => invoke<Hold | null>("hold"),
  /** The logpoints' lines since the last call. */
  logs: () => invoke<Log[]>("debug_logs"),
};

export const monitor = {
  watchRange: (file: string, first: number, last: number, device: string | null, unfolded: string[]) =>
    invoke<Watched>("watch_range", { file, first, last, device, unfolded }),
  unwatch: () => invoke<void>("unwatch"),
  watched: () => invoke<(string | null)[]>("watched"),
  forces: () => invoke<Force[]>("forces"),
  force: (target: Target, text: string) => invoke<Force[]>("force", { target, text }),
  release: (target: Target) => invoke<Force[]>("release", { target }),
  writeValue: (target: Target, text: string) => invoke<void>("write_value", { target, text }),
};

/** The variables table's declarations (src-tauri/src/declarations.rs). */
export const declarations = {
  of: (text: string) => invoke<Declaration[]>("declarations_of", { text }),
  edit: (text: string, change: Change) => invoke<string>("edit_declaration", { text, change }),
  rename: (files: [string, string][], file: string, index: number, newName: string) =>
    invoke<[string, string][]>("rename_declaration", { files, file, index, newName }),
};

/** The monitored values after each tick of Run, in `watch_range`'s order. */
export const onMonitor = (f: (values: (string | null)[]) => void): Promise<UnlistenFn> =>
  listen<(string | null)[]>("monitor", (e) => f(e.payload));
export const onSnapshot = (f: (s: Snapshot) => void): Promise<UnlistenFn> =>
  listen<Snapshot>("snapshot", (e) => f(e.payload));
export const onEngineError = (f: (error: string) => void): Promise<UnlistenFn> =>
  listen<string>("engine-error", (e) => f(e.payload));
export const onStopped = (f: () => void): Promise<UnlistenFn> => listen("stopped", () => f());
/** The project's sources, saved in its folder outside Twinova (watch.rs). */
export const onDiskSources = (f: (disk: DiskSources) => void): Promise<UnlistenFn> =>
  listen<DiskSources>("disk-sources", (e) => f(e.payload));

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
