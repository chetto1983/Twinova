// The main window's commands (spec 2026-11-23-app-mcp-design.md, decision 2): projects, the scene's edits, Build and
// the run controls, the sources, the panel, the theme. Each runs the function App.svelte's own buttons run, through the
// context App.svelte hands; an action the window reports as an error fails the command with that error.
import { projects } from "../backend";
import type { Edit, ProjectView } from "../project";
import type { Snapshot } from "../snapshot";
import { edited, type Source } from "../sources";
import { boolean, none, number, object, oneOf, string, stringOf, type Command } from "./registry";

export interface AppContext {
  project: () => ProjectView | null;
  snapshot: () => Snapshot | null;
  sources: () => Source[];
  setSources: (next: Source[]) => void;
  status: () => "none" | "ok" | "failed";
  running: () => boolean;
  warnings: () => string[];
  notice: () => string;
  /** The error the window shows now, "" for none. */
  error: () => string;
  /** The project shown, built (App.svelte `showProject`). */
  show: (next: ProjectView) => Promise<void>;
  build: () => Promise<void>;
  run: (speed: number | null) => Promise<void>;
  pause: () => Promise<void>;
  step: () => Promise<void>;
  reset: () => Promise<void>;
  saveState: () => Promise<void>;
  restoreState: () => Promise<void>;
  edit: (op: Edit) => Promise<boolean>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  select: (id: string | null) => void;
  press: (address: string) => Promise<void>;
  blockSink: (sink: number, blocked: boolean) => Promise<void>;
  theme: (theme: "dark" | "light") => void;
}

/** The window's state as an agent reads it: the project, the line, the sources and what the window reports. */
export function stateOf(ctx: AppContext): Record<string, unknown> {
  const project = ctx.project();
  return {
    project: project && {
      name: project.name,
      folder: project.folder,
      instances: project.instances.map((i) => ({ id: i.id, definition: i.definition })),
      can_undo: project.can_undo,
      can_redo: project.can_redo,
    },
    build: ctx.status(),
    running: ctx.running(),
    snapshot: ctx.snapshot(),
    sources: ctx.sources().map((s) => ({ file: s.key, kind: s.kind, edited: edited(s) })),
    warnings: ctx.warnings(),
    notice: ctx.notice(),
    error: ctx.error(),
  };
}

/** The line's virtual time after `action`, as a command that moves the line answers it. */
const time = (ctx: AppContext) => ({ time_ms: ctx.snapshot()?.time_ms ?? null });

/** `action`, then the window's error as the command's failure. */
async function checked(ctx: AppContext, action: () => Promise<unknown>): Promise<unknown> {
  const result = await action();
  const error = ctx.error();
  if (error) throw new Error(error);
  return result;
}

function source(ctx: AppContext, file: string): Source {
  const found = ctx.sources().find((s) => s.key === file);
  if (!found) throw new Error(`${file} is not a source of the project: state lists them`);
  return found;
}

export function appCommands(ctx: AppContext): Command[] {
  const open = (view: () => Promise<ProjectView>) => () => checked(ctx, async () => ctx.show(await view()));
  return [
    {
      name: "state",
      description: "The window's state: the project, the build, the line's snapshot, the sources, notices and errors.",
      params: none,
      run: () => stateOf(ctx),
    },
    {
      name: "build",
      description: "Build the project with the sources as the editor holds them, as the Build button does.",
      params: none,
      run: () => checked(ctx, ctx.build).then(() => ({ build: ctx.status(), warnings: ctx.warnings() })),
    },
    {
      name: "run",
      description:
        "Run the line, at `speed` times the wall clock or as fast as possible when null (the toolbar's Run).",
      params: object({ speed: { type: ["number", "null"], description: "×0.25 to ×10, or null" } }, ["speed"]),
      run: (p) => checked(ctx, () => ctx.run(p.speed === undefined ? 1 : (p.speed as number | null))),
    },
    { name: "pause", description: "Pause the line.", params: none, run: () => checked(ctx, ctx.pause) },
    {
      name: "step",
      description: "One scan of the line (the toolbar's Step).",
      params: none,
      run: () => checked(ctx, ctx.step).then(() => time(ctx)),
    },
    {
      name: "reset",
      description: "The line back to time zero (the toolbar's Stop).",
      params: none,
      run: () => checked(ctx, ctx.reset).then(() => time(ctx)),
    },
    {
      name: "save_state",
      description: "Keep the line's state (the toolbar's Save state).",
      params: none,
      run: () => checked(ctx, ctx.saveState),
    },
    {
      name: "restore_state",
      description: "The line back at the state kept.",
      params: none,
      run: () => checked(ctx, ctx.restoreState).then(() => time(ctx)),
    },
    {
      name: "edit",
      description:
        "One edit of the scene, one undo step, as the catalogue, the view and the inspector make them: `op` is an Edit " +
        "(place, move, remove, set_parameter, bind_signal, connect, …), as project.ts declares it.",
      params: object({ op: { type: "object", description: "the Edit, with its `op` field" } }),
      run: (p) =>
        checked(ctx, async () => {
          if (!(await ctx.edit(p.op as Edit)))
            throw new Error(ctx.error() || "the edit was refused (a locked instance?)");
        }),
    },
    { name: "undo", description: "Undo the scene's last edit.", params: none, run: () => checked(ctx, ctx.undo) },
    { name: "redo", description: "Redo the edit undone.", params: none, run: () => checked(ctx, ctx.redo) },
    {
      name: "select",
      description: "Select an instance in the scene and the inspector, or nothing with null.",
      params: object({ id: { type: ["string", "null"] } }),
      run: (p) => ctx.select((p.id as string | null) ?? null),
    },
    { name: "new_project", description: "An empty project.", params: none, run: open(projects.create) },
    { name: "open_demo", description: "The bundled demo, one belt.", params: none, run: open(projects.demo) },
    {
      name: "open_project",
      description: "Open the project in a folder, as the project menu's Open does, without its dialog.",
      params: object({ folder: string("the project's folder") }),
      run: (p) => open(() => projects.openFolder(stringOf(p, "folder")))(),
    },
    {
      name: "save_project",
      description: "Save the project to `folder`, or to the folder it was opened from.",
      params: object({ folder: string("a folder; the project's own when absent") }, ["folder"]),
      run: (p) => {
        const folder = (p.folder as string | undefined) ?? ctx.project()?.folder;
        if (!folder) throw new Error("the project has no folder yet: give one");
        return open(() => projects.save(folder))();
      },
    },
    {
      name: "export_archive",
      description: "Write the project as one archive (.zip).",
      params: object({ file: string("the .zip to write") }),
      run: (p) => projects.exportTo(stringOf(p, "file")),
    },
    {
      name: "import_archive",
      description: "Open a project from an archive (.zip).",
      params: object({ file: string("the .zip to read") }),
      run: (p) => open(() => projects.importFrom(stringOf(p, "file")))(),
    },
    {
      name: "read_source",
      description: "The text of a source as the editor holds it: a program file (plc/…) or a behaviour (its key).",
      params: object({ file: string("as state's sources name it") }),
      run: (p) => source(ctx, stringOf(p, "file")).text,
    },
    {
      name: "write_source",
      description: "Replace a source's text in the editor; it runs at the next build, as typed text does.",
      params: object({ file: string("as state's sources name it"), text: string("the new text") }),
      run: (p) => {
        const file = stringOf(p, "file");
        const text = stringOf(p, "text");
        source(ctx, file);
        ctx.setSources(ctx.sources().map((s) => (s.key === file ? Object.assign({}, s, { text }) : s)));
      },
    },
    {
      name: "press",
      description: "Press a panel button by its input address: it reads true for the next scan.",
      params: object({ address: string("the button's address, %IX…") }),
      run: (p) => checked(ctx, () => ctx.press(stringOf(p, "address"))),
    },
    {
      name: "block_sink",
      description: "Block or free a sink, as its fault does.",
      params: object({ sink: number("the sink's index in the snapshot"), blocked: boolean("blocked") }),
      run: (p) => checked(ctx, () => ctx.blockSink(Number(p.sink), Boolean(p.blocked))),
    },
    {
      name: "theme",
      description: "The window's theme.",
      params: object({ theme: oneOf(["dark", "light"], "the theme") }),
      run: (p) => ctx.theme(p.theme === "light" ? "light" : "dark"),
    },
  ];
}
