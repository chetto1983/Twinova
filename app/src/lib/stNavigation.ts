// Navigation, call templates and Auto Declare in the ST editor (spec 2026-11-13-completion-navigation-design.md):
// Monaco's providers over truST's answers (navigation.rs, declarations.rs, call_templates.rs). Monaco's ESM build adds
// each feature by importing its contribution, as StEditor.svelte does for hover and suggest.
import * as monaco from "monaco-editor/editor/editor.api.js";
import "monaco-editor/editor/contrib/gotoSymbol/browser/goToCommands.js";
import "monaco-editor/editor/contrib/gotoSymbol/browser/link/goToDefinitionAtPosition.js";
// The peek list's controller is abstract in contrib; the standalone build registers its own (measured: no
// `editor.contrib.referencesController` on the editor, and Shift+F12 did nothing, with contrib's alone).
import "monaco-editor/editor/standalone/browser/referenceSearch/standaloneReferenceSearch.js";
import "monaco-editor/editor/contrib/rename/browser/rename.js";
import "monaco-editor/editor/contrib/codeAction/browser/codeActionContributions.js";
import "monaco-editor/editor/contrib/snippet/browser/snippetController2.js";
import { m } from "../paraglide/messages.js";
import { navigation } from "./backend";
import { templateText, type Span } from "./navigation";

export const ST = "st";
/** The command a quick fix runs to open Auto Declare on a name. */
const DECLARE = "twinova.declare";

/** A model's file: its URI's path, `inmemory://twinova/<file>`. */
export const fileOfModel = (model: monaco.editor.ITextModel): string => model.uri.path.slice(1);
export const uriOf = (file: string): monaco.Uri => monaco.Uri.parse(`inmemory://twinova/${file}`);

const rangeOf = (s: Span): monaco.IRange => ({
  startLineNumber: s.line,
  startColumn: s.column,
  endLineNumber: s.end_line,
  endColumn: s.end_column,
});

/** A place of a model, as the navigation commands take it. */
const ask = (model: monaco.editor.ITextModel, at: monaco.Position) =>
  [fileOfModel(model), model.getValue(), at.lineNumber, at.column] as const;

/** What the providers need of the editor showing the tabs. */
export interface NavigationHost {
  /** Every tab's file and text: a rename, a template and a proposal read the whole project. */
  files(): [string, string][];
  /** The texts a rename changed, by file. */
  renamed(changed: [string, string][]): void;
  /** Shows `file` at a place: a definition or a reference in another tab. */
  open(file: string, line: number, column: number): void;
  /** Opens Auto Declare on the name at a place. */
  declare(line: number, column: number): void;
}

/** Registers the providers for `host`; dispose them when its editor goes. */
export function registerNavigation(host: NavigationHost): monaco.IDisposable[] {
  return [
    monaco.languages.registerDefinitionProvider(ST, {
      provideDefinition: async (model, at) => {
        const found = await navigation.definition(...ask(model, at)).catch(() => null);
        return found ? { uri: uriOf(found.file), range: rangeOf(found) } : null;
      },
    }),
    monaco.languages.registerReferenceProvider(ST, {
      provideReferences: async (model, at) =>
        (await navigation.references(...ask(model, at)).catch(() => [])).map((r) => ({
          uri: uriOf(r.span.file),
          range: rangeOf(r.span),
        })),
    }),
    monaco.languages.registerDocumentHighlightProvider(ST, {
      provideDocumentHighlights: async (model, at) =>
        (await navigation.references(...ask(model, at)).catch(() => []))
          .filter((r) => r.span.file === fileOfModel(model))
          .map((r) => ({
            range: rangeOf(r.span),
            kind: r.write ? monaco.languages.DocumentHighlightKind.Write : monaco.languages.DocumentHighlightKind.Read,
          })),
    }),
    // The rename goes through the variables table's path, every tab at once, so Monaco is handed no edits of its own.
    monaco.languages.registerRenameProvider(ST, {
      provideRenameEdits: async (model, at, newName) => {
        try {
          host.renamed(await navigation.rename(host.files(), fileOfModel(model), at.lineNumber, at.column, newName));
          return { edits: [] };
        } catch (e) {
          return { edits: [], rejectReason: String(e) };
        }
      },
    }),
    // A call's parameters, offered when its `(` is typed (spec decision 3).
    monaco.languages.registerCompletionItemProvider(ST, {
      triggerCharacters: ["("],
      provideCompletionItems: async (model, at) => {
        const before = model.getValueInRange(new monaco.Range(at.lineNumber, at.column - 1, at.lineNumber, at.column));
        if (before !== "(") return { suggestions: [] };
        const found = await navigation
          .callTemplate(host.files(), fileOfModel(model), at.lineNumber, at.column)
          .catch(() => null);
        if (!found) return { suggestions: [] };
        const next = model.getValueInRange(new monaco.Range(at.lineNumber, at.column, at.lineNumber, at.column + 1));
        const range = new monaco.Range(at.lineNumber, at.column, at.lineNumber, at.column);
        const item = (label: string, params: string, sortText: string) => ({
          label,
          detail: params.replace(/\$\{\d+\}/g, "").trim() || "()",
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertText: templateText(params, next),
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          sortText,
          range,
        });
        return { suggestions: [item(m.call_all(), found.all, "0"), item(m.call_required(), found.required, "1")] };
      },
    }),
    monaco.editor.registerCommand(DECLARE, (_accessor, line: number, column: number) => host.declare(line, column)),
    // Auto Declare on an undeclared name (E101), as CODESYS's (spec decision 4).
    monaco.languages.registerCodeActionProvider(ST, {
      provideCodeActions: (model, _range, context) => ({
        actions: context.markers
          .filter((marker) => marker.code === "E101")
          .map((marker) => {
            const name = model.getWordAtPosition({ lineNumber: marker.startLineNumber, column: marker.startColumn });
            const title = m.declare_action({ name: name?.word ?? "" });
            return {
              title,
              kind: "quickfix",
              diagnostics: [marker],
              isPreferred: true,
              command: { id: DECLARE, title, arguments: [marker.startLineNumber, marker.startColumn] },
            };
          }),
        dispose: () => {},
      }),
    }),
    // A definition or a reference in another tab opens that tab.
    monaco.editor.registerEditorOpener({
      openCodeEditor: (_source, resource, selection) => {
        const at =
          selection && "startLineNumber" in selection
            ? { line: selection.startLineNumber, column: selection.startColumn }
            : { line: selection?.lineNumber ?? 1, column: selection?.column ?? 1 };
        host.open(resource.path.slice(1), at.line, at.column);
        return true;
      },
    }),
  ];
}
