<script lang="ts" module>
  // Monaco 0.57.0, the editor core only, with its own Structured Text language (spec 2026-11-12-st-editor-design.md,
  // decisions 1 and 2, `languages/definitions/st`, registered here as its `register.js` would), its worker through
  // Vite, and hover and completion from truST's analysis (analysis.rs).
  import * as monaco from "monaco-editor/editor/editor.api.js";
  import { conf, language } from "monaco-editor/languages/definitions/st/st.js";
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
      const shown = await analysis.hover(fileOfModel(model), position.lineNumber, position.column).catch(() => null);
      return shown ? { contents: [{ value: shown }] } : null;
    },
  });
  monaco.languages.registerCompletionItemProvider(ST, {
    provideCompletionItems: async (model, position) => {
      const word = model.getWordUntilPosition(position);
      const range = new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn);
      const items = await analysis.completion(fileOfModel(model), position.lineNumber, position.column).catch(() => []);
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
  import type { Source } from "./sources";

  let {
    sources = $bindable(),
    active,
    problems,
    reveal,
  }: {
    sources: Source[];
    active: number;
    problems: Problems;
    /** A place to show: a Problems row the user clicked. */
    reveal: { key: string; line: number; column: number } | null;
  } = $props();

  let host = $state<HTMLDivElement>();
  let editor = $state<monaco.editor.IStandaloneCodeEditor>();
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
    });
    // The E2E reaches the text through the editor's host (plan 2026-11-12-st-editor.md, decision 1): WebKitWebDriver
    // drops typed spaces.
    (host as HTMLDivElement & { editor?: unknown }).editor = made;
    editor = made;
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
    if (model.getValue() !== current.text) model.setValue(current.text);
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

  $effect(() => {
    if (!editor || !reveal) return;
    editor.revealLineInCenter(reveal.line);
    editor.setPosition({ lineNumber: reveal.line, column: reveal.column });
    editor.focus();
  });
</script>

<div class="monaco-host" bind:this={host}></div>

<style>
  .monaco-host {
    width: 100%;
    height: 100%;
    min-height: 0;
  }
</style>
