<script lang="ts" module>
  // Monaco 0.57.0, the editor core only, with its own Structured Text language (spec 2026-11-12-st-editor-design.md,
  // decisions 1 and 2, `languages/definitions/st`, registered here as its `register.js` would), its worker through
  // Vite, and hover and completion from truST's analysis (analysis.rs), on the text the model holds now: the analysis
  // that feeds the markers waits for a pause in typing.
  import * as monaco from "monaco-editor/editor/editor.api.js";
  import { conf, language } from "monaco-editor/languages/definitions/st/st.js";
  // The editor API alone has no hover nor suggest: Monaco's ESM build adds each feature by importing its contribution
  // (measured: `getAction("editor.action.showHover")` was undefined without it). These are what a programmer expects.
  import "monaco-editor/editor/contrib/hover/browser/hoverContribution.js";
  import "monaco-editor/editor/contrib/suggest/browser/suggestController.js";
  import "monaco-editor/editor/contrib/gotoError/browser/gotoError.js";
  import "monaco-editor/editor/contrib/bracketMatching/browser/bracketMatching.js";
  import "monaco-editor/editor/contrib/comment/browser/comment.js";
  import "monaco-editor/editor/contrib/find/browser/findController.js";
  import "monaco-editor/editor/contrib/folding/browser/folding.js";
  import "monaco-editor/editor/contrib/clipboard/browser/clipboard.js";
  import "monaco-editor/editor/contrib/contextmenu/browser/contextmenu.js";
  import "monaco-editor/editor/contrib/linesOperations/browser/linesOperations.js";
  import "monaco-editor/editor/contrib/wordHighlighter/browser/wordHighlighter.js";
  // oxlint-disable-next-line import/default -- Vite's `?worker` import has a default export the linter cannot see.
  import EditorWorker from "monaco-editor/editor/editor.worker.js?worker";
  import { analysis } from "./backend";
  import { completionKind } from "./analysis";

  const ST = "st";
  monaco.languages.register({ id: ST, extensions: [".st"] });
  monaco.languages.setLanguageConfiguration(ST, conf);
  monaco.languages.setMonarchTokensProvider(ST, language);
  (self as unknown as { MonacoEnvironment: monaco.Environment }).MonacoEnvironment = {
    getWorker: () => new EditorWorker(),
  };

  /** A model's file: its URI's path, `inmemory://twinova/<file>`. */
  const fileOfModel = (model: monaco.editor.ITextModel): string => model.uri.path.slice(1);
  const uriOf = (file: string): monaco.Uri => monaco.Uri.parse(`inmemory://twinova/${file}`);

  monaco.languages.registerHoverProvider(ST, {
    provideHover: async (model, position) => {
      const { lineNumber, column } = position;
      const shown = await analysis.hover(fileOfModel(model), model.getValue(), lineNumber, column).catch(() => null);
      return shown ? { contents: [{ value: shown }] } : null;
    },
  });
  monaco.languages.registerCompletionItemProvider(ST, {
    provideCompletionItems: async (model, position) => {
      const word = model.getWordUntilPosition(position);
      const range = new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn);
      const items = await analysis
        .completion(fileOfModel(model), model.getValue(), position.lineNumber, position.column)
        .catch(() => []);
      const kinds = monaco.languages.CompletionItemKind as unknown as Record<string, number>;
      return {
        suggestions: items.map((c) => ({
          label: c.label,
          kind: completionKind(c.kind, kinds),
          detail: c.detail ?? undefined,
          insertText: c.insert_text,
          range,
        })),
      };
    },
  });

  const token = (name: string): string => getComputedStyle(document.documentElement).getPropertyValue(name).trim();

  /** A Monaco theme from the app's Aura tokens, as the page computes them now. */
  function themeFromTokens(dark: boolean): monaco.editor.IStandaloneThemeData {
    return {
      base: dark ? "vs-dark" : "vs",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": token("--color-surface"),
        "editor.foreground": token("--color-text"),
        "editorLineNumber.foreground": token("--color-text-faint") || token("--color-text-muted"),
        "editor.lineHighlightBackground": token("--color-surface-2"),
        "editor.selectionBackground": token("--color-accent-muted") || token("--color-accent"),
      },
    };
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { fileOf, marker, type Problems } from "./analysis";
  import { decorations, spotAt, type Spot } from "./monitor";
  import { breakpointMarks, heldMark, markedLines } from "./debug";
  import type { Source } from "./sources";

  let {
    sources = $bindable(),
    active,
    problems,
    reveal,
    spots = [],
    spotValues = [],
    forced = new Set<string>(),
    onVisible,
    onSpot,
    onToggleMonitor,
    breakpoints = [],
    heldLine = null,
    onBreakpoint,
    onBreakpointsMoved,
    onCursor,
  }: {
    sources: Source[];
    active: number;
    problems: Problems;
    /** A place to show: a Problems row the user clicked. */
    reveal: { key: string; line: number; column: number } | null;
    /** While monitoring: where the values go in the shown tab, and the values (monitor.rs). */
    spots?: Spot[];
    spotValues?: (string | null)[];
    forced?: Set<string>;
    /** The first and last lines in view, for the monitoring to read only those. */
    onVisible?: (first: number, last: number) => void;
    /** A drawn value was clicked: its spot's index. */
    onSpot?: (index: number) => void;
    onToggleMonitor?: () => void;
    /** The shown tab's breakpoints, and the line the program is held at in it (spec 2026-11-13-breakpoints-hold-design.md). */
    breakpoints?: number[];
    heldLine?: number | null;
    /** A breakpoint toggled on `line`, by a click in the glyph margin or F9. */
    onBreakpoint?: (line: number) => void;
    /** An edit moved the shown tab's breakpoints with their lines: where they are now. */
    onBreakpointsMoved?: (lines: number[]) => void;
    /** The cursor's line, for Run to cursor. */
    onCursor?: (line: number) => void;
  } = $props();

  let host = $state<HTMLDivElement>();
  let editor = $state<monaco.editor.IStandaloneCodeEditor>();
  let values: monaco.editor.IEditorDecorationsCollection | undefined;
  let marks: monaco.editor.IEditorDecorationsCollection | undefined;
  let held: monaco.editor.IEditorDecorationsCollection | undefined;
  /** A text set from outside (disk, Revert) replaces the model's content and drops its decorations: no move then. */
  let replacing = false;
  function drawMarks(): void {
    marks?.set(breakpointMarks(breakpoints, monaco.editor.TrackedRangeStickiness.GrowsOnlyWhenTypingAfter));
  }
  const models = new Map<string, monaco.editor.ITextModel>();

  /** A tab's model, made on first use; what is typed in it goes back to the tab's source. */
  function modelOf(source: Source): monaco.editor.ITextModel {
    const file = fileOf(source);
    const known = models.get(file);
    if (known) return known;
    const model = monaco.editor.createModel(source.text, ST, uriOf(file));
    model.onDidChangeContent(() => {
      const tab = sources.find((s) => fileOf(s) === file);
      if (tab && tab.text !== model.getValue()) tab.text = model.getValue();
    });
    models.set(file, model);
    return model;
  }

  function applyTheme(): void {
    const dark = document.documentElement.dataset.theme !== "light";
    monaco.editor.defineTheme("twinova", themeFromTokens(dark));
    monaco.editor.setTheme("twinova");
  }

  onMount(() => {
    applyTheme();
    const made = monaco.editor.create(host!, {
      automaticLayout: true,
      fontSize: 12,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      tabSize: 4,
      theme: "twinova",
      glyphMargin: true,
    });
    // The E2E reaches the text through the editor's host (plan 2026-11-12-st-editor.md, decision 1): WebKitWebDriver
    // drops typed spaces.
    (host as HTMLDivElement & { editor?: unknown }).editor = made;
    editor = made;
    values = made.createDecorationsCollection();
    marks = made.createDecorationsCollection();
    held = made.createDecorationsCollection();
    made.onDidChangeModelContent(() => {
      if (replacing || !marks) return;
      const lines = markedLines(marks.getRanges());
      if (lines.join() !== breakpoints.join()) onBreakpointsMoved?.(lines);
    });
    const visible = (): void => {
      const ranges = made.getVisibleRanges();
      if (ranges.length > 0) onVisible?.(ranges[0].startLineNumber, ranges[ranges.length - 1].endLineNumber);
    };
    made.onDidScrollChange(visible);
    made.onDidLayoutChange(visible);
    made.onDidChangeModel(visible);
    // Monitoring on and off, as TIA Portal's Ctrl+T.
    made.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyT, () => onToggleMonitor?.());
    // A breakpoint on the cursor's line, as VS Code's F9.
    made.addCommand(monaco.KeyCode.F9, () => {
      const at = made.getPosition();
      if (at) onBreakpoint?.(at.lineNumber);
    });
    made.onDidChangeCursorPosition((e) => onCursor?.(e.position.lineNumber));
    // A click on a drawn value lands on its spot's column (Monaco gives injected text no column of its own).
    made.onMouseDown((e) => {
      const at = e.target.position;
      if (at && e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
        onBreakpoint?.(at.lineNumber);
        return;
      }
      if (!at || !e.target.element?.classList.contains("monitor-value")) return;
      const index = spotAt(spots, at.lineNumber, at.column);
      if (index >= 0) onSpot?.(index);
    });
    const themes = new MutationObserver(applyTheme);
    themes.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    return () => {
      themes.disconnect();
      made.dispose();
      models.forEach((m) => m.dispose());
      models.clear();
    };
  });

  // The tab shown, and its text when it changes outside the editor: a disk change, Revert, a new project.
  $effect(() => {
    const current = sources[Math.min(active, sources.length - 1)];
    if (!editor || !current) return;
    const model = modelOf(current);
    if (model.getValue() !== current.text) {
      replacing = true;
      model.setValue(current.text);
      replacing = false;
      drawMarks();
    }
    if (editor.getModel() !== model) editor.setModel(model);
  });

  // A tab that left the project takes its model along.
  $effect(() => {
    const files = new Set(sources.map(fileOf));
    for (const [file, model] of models) {
      if (!files.has(file)) {
        model.dispose();
        models.delete(file);
      }
    }
  });

  $effect(() => {
    if (!editor) return;
    for (const source of sources) {
      monaco.editor.setModelMarkers(modelOf(source), "truST", (problems[fileOf(source)] ?? []).map(marker));
    }
  });

  // The monitored values after their references and conditions (spec 2026-11-13-online-monitoring-design.md, decision 5).
  $effect(() => {
    const drawn = decorations(spots, spotValues, forced);
    if (editor) values?.set(drawn);
  });

  $effect(() => {
    void breakpoints;
    if (editor) drawMarks();
  });
  $effect(() => {
    const drawn = heldMark(heldLine);
    if (editor) held?.set(drawn);
  });

  // The held line comes into view, as a debugger's does.
  $effect(() => {
    if (editor && heldLine !== null) editor.revealLineInCenterIfOutsideViewport(heldLine);
  });

  $effect(() => {
    if (!editor || !reveal) return;
    editor.revealLineInCenter(reveal.line);
    editor.setPosition({ lineNumber: reveal.line, column: reveal.column });
    editor.focus();
  });
</script>

<div class="monaco-host" bind:this={host}></div>

<style>
  :global(.monitor-value) {
    color: var(--color-text-muted);
    background: var(--color-surface-2);
    border-radius: 3px;
    margin-left: 4px;
    padding: 0 3px;
    cursor: pointer;
  }
  :global(.monitor-true) {
    color: var(--color-success);
  }
  :global(.monitor-false) {
    color: var(--color-text-faint);
  }
  :global(.monitor-forced) {
    outline: 1px solid var(--color-warning);
  }
  :global(.debug-breakpoint) {
    background: var(--color-danger);
    border-radius: 50%;
    width: 10px !important;
    height: 10px !important;
    margin: 4px 0 0 5px;
  }
  :global(.debug-held) {
    border-left: 8px solid var(--color-warning);
    border-top: 6px solid transparent;
    border-bottom: 6px solid transparent;
    width: 0 !important;
    height: 0 !important;
    margin: 3px 0 0 6px;
  }
  :global(.debug-held-line) {
    background: color-mix(in oklab, var(--color-warning) 22%, transparent);
  }
  .monaco-host {
    width: 100%;
    height: 100%;
    min-height: 0;
  }
</style>
