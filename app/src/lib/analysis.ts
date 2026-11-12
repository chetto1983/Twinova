// The in-app ST editor's analysis on the webview's side (spec 2026-11-12-st-editor-design.md): which file a tab is,
// truST's diagnostics as Monaco markers and as the Problems pane's list, its completion kinds as Monaco's, and the
// debounce of decision 6. The analysis itself is Rust's (src-tauri/src/analysis.rs).
import type { Source } from "./sources";

/** A diagnostic as `analysis.rs` sends it: lines from 1, columns in UTF-16 units from 1, as Monaco counts them. */
export interface Problem {
  severity: string;
  code: string;
  message: string;
  line: number;
  column: number;
  end_line: number;
  end_column: number;
}

/** Each file's problems, by its path in the project folder. */
export type Problems = Record<string, Problem[]>;

/** A Problems pane row. */
export interface Listed extends Problem {
  file: string;
  /** The tab that shows the file. */
  key: string;
}

/** The file a tab shows, as the project folder holds it. */
export const fileOf = (source: Source): string =>
  source.kind === "program" ? source.key : `components/${source.key}.st`;

/** Monaco's `MarkerSeverity` values (monaco-editor 0.57.0, `editor.api.d.ts`). */
const SEVERITY: Record<string, number> = { error: 8, warning: 4, info: 2, hint: 1 };

/** A problem as a Monaco marker. */
export function marker(problem: Problem) {
  return {
    severity: SEVERITY[problem.severity] ?? SEVERITY.info,
    code: problem.code,
    message: problem.message,
    source: "truST",
    startLineNumber: problem.line,
    startColumn: problem.column,
    endLineNumber: problem.end_line,
    endColumn: problem.end_column,
  };
}

/** Every problem, the tabs' order first, then by line and column within a file. */
export function listed(problems: Problems, sources: Source[]): Listed[] {
  return sources.flatMap((source) => {
    const file = fileOf(source);
    const rows = (problems[file] ?? []).map((p): Listed => Object.assign({}, p, { file, key: source.key }));
    return rows.toSorted((a, b) => a.line - b.line || a.column - b.column);
  });
}

/** The completion kinds of `trust-wasm-analysis` (`completion_kind_label`), by the name of Monaco's
 *  `CompletionItemKind` that suits each. */
const KIND: Record<string, string> = {
  keyword: "Keyword",
  function: "Function",
  function_block: "Class",
  method: "Method",
  property: "Property",
  variable: "Variable",
  constant: "Constant",
  type: "Struct",
  enum_value: "EnumMember",
  namespace: "Module",
  snippet: "Snippet",
};

/** Monaco's kind for an engine's kind, from Monaco's own `CompletionItemKind` enum; text when unknown. */
export const completionKind = (kind: string, kinds: Record<string, number>): number => kinds[KIND[kind] ?? "Text"];

/** Calls `run` once, `ms` after the last of a burst of calls; `cancel` drops a pending call. */
export function debounce<T extends unknown[]>(run: (...args: T) => void, ms: number) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return {
    call: (...args: T) => {
      clearTimeout(timer);
      timer = setTimeout(() => run(...args), ms);
    },
    cancel: () => clearTimeout(timer),
  };
}
