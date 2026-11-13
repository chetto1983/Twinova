// Breakpoints and HOLD in the ST editor (spec 2026-11-13-breakpoints-hold-design.md): the keys, VS Code's (decision
// 5), and the editor's marks. The commands are in backend.ts (`debug`), on debug_commands.rs.

/** Where the program is held (debugger.rs `Hold`): the call stack's frames innermost first, as [file, line]. */
export interface Hold {
  file: string;
  line: number;
  frames: [string, number][];
}
export type Resume = "continue" | "over" | "into" | "out";
export type DebugAction = Resume | "toggle" | "cursor";

/** `lines` with `line` added, or taken out when it was there, in order. */
export function toggleLine(lines: number[], line: number): number[] {
  const next = lines.includes(line) ? lines.filter((l) => l !== line) : [...lines, line];
  return next.toSorted((a, b) => a - b);
}

/** VS Code's debug keys: F9 toggles, F5 continues, F10 steps over (Ctrl+F10 runs to the cursor), F11 steps into,
 *  Shift+F11 out. */
export function keyAction(e: {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
}): DebugAction | null {
  const ctrl = e.ctrlKey || e.metaKey;
  if (e.key === "F9") return "toggle";
  if (e.key === "F5" && !e.shiftKey) return "continue";
  if (e.key === "F10") return ctrl ? "cursor" : "over";
  if (e.key === "F11") return e.shiftKey ? "out" : "into";
  return null;
}

/** A Monaco range at the start of `line`. */
const at = (line: number) => ({ startLineNumber: line, startColumn: 1, endLineNumber: line, endColumn: 1 });

/** Monaco decorations: a dot in the glyph margin per breakpoint, and the held line across the editor. */
export function breakpointDecorations(lines: number[], held: number | null) {
  const marks = lines.map((line) => ({
    range: at(line),
    options: {
      glyphMarginClassName: "debug-breakpoint",
      className: undefined as string | undefined,
      isWholeLine: false,
    },
  }));
  if (held === null) return marks;
  return [
    ...marks,
    {
      range: at(held),
      options: { glyphMarginClassName: "debug-held", className: "debug-held-line", isWholeLine: true },
    },
  ];
}

export const holdLabel = (hold: Hold): string => `${hold.file}:${hold.line}`;
