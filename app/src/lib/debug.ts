// Breakpoints and HOLD in the ST editor (specs 2026-11-13-breakpoints-hold-design.md and
// -breakpoints-complete-design.md): the keys, VS Code's, the breakpoints' kinds and the editor's marks. The commands
// are in backend.ts (`debug`), on debug_commands.rs.

/** Where the program is held (debugger.rs `Hold`): the call stack's frames innermost first, as [file, line], and in a
 *  block, the device whose instance it is. */
export interface Hold {
  file: string;
  line: number;
  frames: [string, number][];
  device?: string;
}
export type Resume = "continue" | "over" | "into" | "out";
export type DebugAction = Resume | "toggle" | "cursor" | "edit";

/** Stops when the breakpoint's hits compare so with `count` (truST's three). */
export interface Hit {
  op: "equal" | "at_least" | "greater_than";
  count: number;
}
/** A breakpoint as debugger.rs takes it: a line, and optionally a condition, a hit count and a log message. */
export interface Breakpoint {
  line: number;
  condition?: string;
  hit?: Hit;
  log?: string;
}
/** A line a logpoint wrote. */
export interface Log {
  file: string;
  line: number;
  message: string;
}

/** `list` with a plain breakpoint added at `line`, or the one there taken out, in order. */
export function toggle(list: Breakpoint[], line: number): Breakpoint[] {
  const next = list.some((b) => b.line === line) ? list.filter((b) => b.line !== line) : [...list, { line }];
  return next.toSorted((a, b) => a.line - b.line);
}

/** `list` with `breakpoint` in place of the one on its line. */
export function replaced(list: Breakpoint[], breakpoint: Breakpoint): Breakpoint[] {
  return [...list.filter((b) => b.line !== breakpoint.line), breakpoint].toSorted((a, b) => a.line - b.line);
}

/** A breakpoint's own fields only: what the bar left empty is not sent. */
export function trimmed(b: Breakpoint): Breakpoint {
  const out: Breakpoint = { line: b.line };
  if (b.condition?.trim()) out.condition = b.condition.trim();
  if (b.hit && b.hit.count > 0) out.hit = b.hit;
  if (b.log?.trim()) out.log = b.log;
  return out;
}

/** The dot's class: a logpoint, one that stops on a condition or a hit count, or a plain one. */
export function kindOf(b: Breakpoint): string {
  if (b.log) return "debug-logpoint";
  return b.condition || b.hit ? "debug-breakpoint-conditional" : "debug-breakpoint";
}

/** VS Code's debug keys: F9 toggles and Shift+F9 edits, F5 continues, F10 steps over (Ctrl+F10 runs to the cursor),
 *  F11 steps into, Shift+F11 out. */
export function keyAction(e: {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
}): DebugAction | null {
  const ctrl = e.ctrlKey || e.metaKey;
  if (e.key === "F9") return e.shiftKey ? "edit" : "toggle";
  if (e.key === "F5" && !e.shiftKey) return "continue";
  if (e.key === "F10") return ctrl ? "cursor" : "over";
  if (e.key === "F11") return e.shiftKey ? "out" : "into";
  return null;
}

/** A Monaco range at the start of `line`. */
const at = (line: number) => ({ startLineNumber: line, startColumn: 1, endLineNumber: line, endColumn: 1 });

/** Monaco decorations: a dot in the glyph margin per breakpoint, its class by kind. `stickiness` keeps each on its
 *  line's start, so a line inserted above moves it down with its line. */
export function breakpointMarks(list: Breakpoint[], stickiness: number) {
  return list.map((b) => ({ range: at(b.line), options: { glyphMarginClassName: kindOf(b), stickiness } }));
}

/** The held line, marked across the editor. */
export function heldMark(held: number | null) {
  if (held === null) return [];
  return [
    {
      range: at(held),
      options: { glyphMarginClassName: "debug-held", className: "debug-held-line", isWholeLine: true },
    },
  ];
}

/** The breakpoints after an edit, from their marks' ranges, which come in the order the marks were set: each keeps its
 *  kind on its new line, and two brought onto one line keep the first. */
export function moved(list: Breakpoint[], ranges: { startLineNumber: number }[]): Breakpoint[] {
  const placed = list.map((b, i) => ({ ...b, line: ranges[i]?.startLineNumber ?? b.line }));
  return placed.filter((b, i) => placed.findIndex((o) => o.line === b.line) === i).toSorted((a, b) => a.line - b.line);
}

/** Whether two lists stand on the same lines. */
export const sameLines = (a: Breakpoint[], b: Breakpoint[]): boolean =>
  a.map((x) => x.line).join() === b.map((x) => x.line).join();

export const holdLabel = (hold: Hold): string => `${hold.file}:${hold.line}${hold.device ? ` (${hold.device})` : ""}`;
