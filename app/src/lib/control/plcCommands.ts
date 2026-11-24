// The PLC pane's and the dock's commands (spec 2026-11-23-app-mcp-design.md, decision 2): which file and view the pane
// shows, online monitoring and the variables table's values, writes and forces, breakpoints and the steps, PLCopen; and
// which tab of the dock shows. Each runs what the pane's own controls run (the value bar's `applyValue`, the
// breakpoints' `set`, the debug bar's `resume`), through the context Editor.svelte and WorkspaceDock.svelte hand.
import { debug, ladder } from "../backend";
import type { Breakpoint, Hold, Resume } from "../debug";
import { withImported } from "../ladder/plcopen";
import { applyValue, type Declaration, type Force, type Row, type Target } from "../monitor";
import { buildArgs, type Source } from "../sources";
import { boolean, none, number, object, oneOf, string, stringOf, type Command, type Schema } from "./registry";

export interface PlcContext {
  sources: () => Source[];
  setSources: (next: Source[]) => void;
  /** The file the pane shows, or null. */
  current: () => string | null;
  open: (file: string) => void;
  setView: (ladder: boolean) => void;
  setMonitoring: (on: boolean) => void;
  setTable: (on: boolean) => void;
  /** The table's declarations, the rows the engine reads while monitoring, and their values, as they are now: read again
   *  rather than when the pane's own debounced reads come round. */
  table: () => Promise<{ declarations: Declaration[]; rows: Row[] | null; values: (string | null)[] }>;
  setForces: (forces: Force[]) => void;
  /** The file a tab's key names: a program file's own path, a block's `components/<key>.st`. */
  pathOf: (key: string) => string;
  /** Answers the debugger's refusal, or null. */
  setBreakpoints: (file: string, list: Breakpoint[]) => Promise<string | null>;
  breakpointsOf: (file: string) => Breakpoint[];
  hold: () => Hold | null;
}

/** A variable of a program (`program` and `path`, as `Main` and `carry.PT`) or an I/O address. */
const target: Schema = {
  type: "object",
  description: "{ program, path } for a variable (path as `count` or `carry.PT`), or { address } as `%QX0.0`",
  properties: { program: { type: "string" }, path: { type: "string" }, address: { type: "string" } },
};

function targetOf(params: Record<string, unknown>): Target {
  const t = (params.target ?? {}) as Record<string, unknown>;
  if (typeof t.address === "string") return { kind: "address", address: t.address };
  if (typeof t.program === "string" && typeof t.path === "string")
    return { kind: "variable", program: t.program, path: t.path };
  throw new Error("target must be { program, path } or { address }");
}

function file(ctx: PlcContext, params: Record<string, unknown>): string {
  const named = (params.file as string | undefined) ?? ctx.current();
  if (!named || !ctx.sources().some((s) => s.key === named)) throw new Error(`no file ${named ?? ""} in the PLC`);
  return named;
}

/** The variables table as the pane shows it: each row's name, type and address, and its value while monitoring. */
export async function valuesOf(ctx: PlcContext): Promise<Record<string, unknown>[]> {
  const { declarations, rows, values } = await ctx.table();
  if (!rows) return declarations.map((d) => ({ name: d.name, type: d.type_name, address: d.address, value: null }));
  return rows.map((row, i) => {
    const d = row.declaration === null ? null : declarations[row.declaration];
    return { name: row.name, type: d?.type_name ?? null, address: d?.address ?? null, value: values[i] ?? null };
  });
}

export function plcCommands(ctx: PlcContext): Command[] {
  const value = (action: "write" | "force" | "release") => async (p: Record<string, unknown>) => {
    const forces = await applyValue(action, targetOf(p), action === "release" ? "" : stringOf(p, "value"));
    if (forces) ctx.setForces(forces);
    return forces;
  };
  return [
    {
      name: "open_file",
      description: "Show a file of the PLC in the pane: a program file (plc/…) or a block (its key).",
      params: object({ file: string("as state's sources name it") }),
      run: (p) => void ctx.open(file(ctx, { file: stringOf(p, "file") })),
    },
    {
      name: "view",
      description: "Show a Ladder file drawn or as its text.",
      params: object({ view: oneOf(["ladder", "text"], "the view") }),
      run: (p) => void ctx.setView(p.view === "ladder"),
    },
    {
      name: "monitoring",
      description: "Online monitoring on or off (Ctrl+T): the values beside the code and in the table.",
      params: object({ on: boolean("on") }),
      run: (p) => void ctx.setMonitoring(Boolean(p.on)),
    },
    {
      name: "variables_table",
      description: "Show or hide the variables table above the code.",
      params: object({ on: boolean("shown") }),
      run: (p) => void ctx.setTable(Boolean(p.on)),
    },
    {
      name: "values",
      description:
        "The variables table of the file shown: each name, type and address, and its value while monitoring.",
      params: none,
      run: () => valuesOf(ctx),
    },
    {
      name: "write",
      description: "Write a value once to a variable, as the value bar's Write does.",
      params: object({ target, value: string("the value, as ST writes it: TRUE, 3, T#2s") }),
      run: value("write"),
    },
    {
      name: "force",
      description: "Force a variable or an address until released; answers the forces.",
      params: object({ target, value: string("the value") }),
      run: value("force"),
    },
    {
      name: "release",
      description: "Release a force; answers the forces left.",
      params: object({ target }),
      run: value("release"),
    },
    {
      name: "breakpoints",
      description:
        "Set a file's breakpoints (each a line, with an optional condition, hit count and log message), replacing its " +
        "own; answers them as placed, each on its statement's line.",
      params: object(
        { file: string("the file; the one shown"), lines: { type: "array", items: { type: ["number", "object"] } } },
        ["file"],
      ),
      run: async (p) => {
        const path = ctx.pathOf(file(ctx, p));
        const list = ((p.lines as unknown[]) ?? []).map((l) =>
          typeof l === "number" ? { line: l } : l,
        ) as Breakpoint[];
        const refused = await ctx.setBreakpoints(path, list);
        if (refused) throw new Error(refused);
        return ctx.breakpointsOf(path);
      },
    },
    {
      name: "hold",
      description: "Where a breakpoint holds the line, with its call stack, or null while it runs.",
      params: none,
      run: () => ctx.hold(),
    },
    {
      name: "resume",
      description: "Go on from a HOLD: continue (F5), over (F10), into (F11) or out (Shift+F11).",
      params: object({ how: oneOf(["continue", "over", "into", "out"], "how") }),
      run: (p) => debug.resume(stringOf(p, "how") as Resume),
    },
    {
      name: "run_to_cursor",
      description: "Run until a line of a file is reached (Ctrl+F10).",
      params: object({ file: string("the file; the one shown"), line: number("its line, from 1") }, ["file"]),
      run: (p) => debug.runToCursor(ctx.pathOf(file(ctx, p)), Number(p.line)),
    },
    {
      name: "plcopen_export",
      description: "Write a Ladder file of the PLC as PLCopen TC6 XML, as the pane's Export does without its dialog.",
      params: object({ target: string("the .xml to write"), file: string("the Ladder file; the one shown") }, ["file"]),
      run: (p) => ladder.exportPlcopenTo(buildArgs(ctx.sources()).program, file(ctx, p), stringOf(p, "target")),
    },
    {
      name: "plcopen_import",
      description:
        "Bring a PLCopen TC6 XML file's Ladder programs into the PLC, as the pane's Import does: a file the PLC has " +
        "takes the imported text, a new one is added; both run at the next build. Answers what changed.",
      params: object({ source: string("the .xml to read") }),
      run: async (p) => {
        const imported = await ladder.importPlcopenFrom(stringOf(p, "source"));
        const next = withImported(ctx.sources(), imported.files);
        ctx.setSources(next.sources);
        return { replaced: next.replaced, added: next.added, notes: imported.notes };
      },
    },
  ];
}

export interface DockContext {
  show: (panel: string) => void;
  maximize: (on: boolean) => void;
  /** The PLC pane in a window of its own, or back in the dock, as the dock's buttons do. */
  plcWindow: (open: boolean) => void;
}

export const dockCommands = (ctx: DockContext): Command[] => [
  {
    name: "show",
    description: "Show a tab of the dock below the scene, and lay it over the workspace when maximized is true.",
    params: object(
      { panel: oneOf(["parts", "plc", "problems", "watch"], "the tab"), maximized: boolean("over the workspace") },
      ["maximized"],
    ),
    run: (p) => {
      ctx.show(stringOf(p, "panel"));
      if (p.maximized !== undefined) ctx.maximize(Boolean(p.maximized));
    },
  },
  {
    name: "plc_window",
    description:
      "Move the PLC pane to a window of its own, or back into the dock, as the dock's buttons do. The pane's commands " +
      "go to its window while it is open; screenshot it with window `plc`.",
    params: object({ open: boolean("in its own window") }),
    run: (p) => void ctx.plcWindow(Boolean(p.open)),
  },
];
