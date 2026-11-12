<script lang="ts">
  // The ST sources of the project, one tab each: the line program's files, then the behaviour block of each
  // definition that has one (spec 2026-11-09-component-behaviour-design.md, "The app"). A definition's block serves
  // every instance of it. Build sends them all. A tab edited here since its last Build or save outside Twinova is
  // marked, and Revert takes its text back (spec 2026-11-12-st-vscode-design.md, decision 6).
  import { m } from "../paraglide/messages.js";
  import { edited, type Source } from "./sources";

  let {
    sources = $bindable(),
    status,
    error,
  }: { sources: Source[]; status: "none" | "ok" | "failed"; error: string } = $props();

  let active = $state(0);
  const current = $derived(sources[Math.min(active, sources.length - 1)]);
</script>

<section class="panel editor">
  <header class="panel-head">
    <h2 class="panel-title">{m.editor_title()}</h2>
    <span class="spacer"></span>
    <span class="led" class:on={status === "ok"} class:err={status === "failed"}></span>
    <span class="muted">
      {status === "ok" ? m.built_ok() : status === "failed" ? m.build_failed() : m.not_built()}
    </span>
  </header>
  <div class="panel-body body">
    <div class="tabs" role="tablist">
      {#each sources as source, i (source.key)}
        <button
          role="tab"
          class="tab"
          class:edited={edited(source)}
          title={edited(source) ? m.editor_edited() : undefined}
          aria-selected={source === current}
          onclick={() => (active = i)}
        >
          {source.label}
        </button>
      {/each}
    </div>
    <div class="row">
      <p class="muted hint">{current?.kind === "behaviour" ? m.editor_hint_behaviour() : m.editor_hint()}</p>
      {#if current && edited(current)}
        <button class="btn ghost" onclick={() => (current.text = current.base)}>{m.editor_revert()}</button>
      {/if}
    </div>
    {#if current}
      <textarea class="data" spellcheck="false" bind:value={current.text} aria-label={current.label}></textarea>
    {/if}
    {#if error}<pre class="error-line">{error}</pre>{/if}
  </div>
</section>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .body {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 10px;
    min-height: 0;
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tab {
    padding: 4px 10px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12.5px;
    cursor: pointer;
  }
  /* Outside the label's text, so a tab keeps its name for assistive technology and the E2E. */
  .tab.edited::after {
    content: " •";
  }
  .row {
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: space-between;
  }
  .tab[aria-selected="true"] {
    background: var(--color-surface-3);
    color: var(--color-text);
  }
  .hint {
    margin: 0;
    font-size: 12.5px;
  }
  textarea {
    flex: 1;
    min-height: 200px;
    padding: 10px 12px;
    resize: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-3);
    color: var(--color-text);
    line-height: 1.5;
    tab-size: 4;
    outline: none;
  }
  textarea:focus-visible {
    box-shadow: 0 0 0 2px var(--color-ring);
  }
  pre {
    margin: 0;
    white-space: pre-wrap;
  }
</style>
