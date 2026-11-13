// Completion as in PLC IDEs, and navigation, in the ST editor (spec 2026-11-13-completion-navigation-design.md): what
// the providers in stNavigation.ts compute without Monaco.
import { fileOf } from "./analysis";
import type { Declaration } from "./monitor";
import type { Source } from "./sources";

/** A stretch of a file: Monaco's lines and columns, from 1 (navigation.rs). */
export interface Span {
  file: string;
  line: number;
  column: number;
  end_line: number;
  end_column: number;
}
export interface Reference {
  span: Span;
  write: boolean;
}
/** A call's parameters as snippets: every one, and the mandatory ones only (call_templates.rs). */
export interface Template {
  all: string;
  required: string;
}
/** Auto Declare's proposal for an undeclared name. */
export interface Proposal {
  name: string;
  pou: string;
  type_name: string | null;
}

/** A call's parameters as typed after its `(`, closed unless the editor already put the `)` after the cursor. */
export const templateText = (params: string, next: string): string => (next === ")" ? params : `${params})`);

/** truST's completions carry Monaco snippet placeholders, `$0` and `${1:TRUE}`, which go in as snippets. */
export const isSnippet = (text: string): boolean => /\$(\d|\{\d)/.test(text);

const valueOf = (text: string): string | null => (text.trim() === "" ? null : text.trim());

/** The variables table's row Auto Declare adds: an empty field is no value. */
export function declaration(
  proposal: Proposal,
  cls: string,
  typeName: string,
  initial: string,
  comment: string,
): Declaration {
  return {
    pou: proposal.pou,
    name: proposal.name,
    class: cls,
    qualifier: null,
    type_name: typeName.trim(),
    address: null,
    initial: valueOf(initial),
    comment: valueOf(comment),
  };
}

/** Puts each changed text of a rename in its tab; a file with no tab has nothing to show. */
export function applyChanged(sources: Source[], changed: [string, string][]): void {
  for (const [file, text] of changed) {
    const tab = sources.find((s) => fileOf(s) === file);
    if (tab) tab.text = text;
  }
}
