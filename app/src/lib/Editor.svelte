<script lang="ts">
  import { m } from "../paraglide/messages.js";

  let {
    source = $bindable(),
    status,
    error,
  }: { source: string; status: "none" | "ok" | "failed"; error: string } = $props();
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
    <p class="muted hint">{m.editor_hint()}</p>
    <textarea class="data" spellcheck="false" bind:value={source} aria-label={m.editor_title()}></textarea>
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
