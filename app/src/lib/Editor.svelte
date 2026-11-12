<script lang="ts">
  import { Cpu, ChevronDown, FileCode2, Braces, Clock3, Folder } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { edited, type Source } from "./sources";
  import type { Problems } from "./analysis";
  import StEditor from "./StEditor.svelte";
  let {
    sources = $bindable(),
    status,
    problems,
    reveal,
  }: {
    sources: Source[];
    status: "none" | "ok" | "failed";
    problems: Problems;
    reveal: { key: string; line: number; column: number } | null;
  } = $props();
  let active = $state(0);
  // A Problems row opens its tab.
  $effect(() => {
    const at = reveal ? sources.findIndex((s) => s.key === reveal.key) : -1;
    if (at >= 0) active = at;
  });
  const current = $derived(sources[Math.min(active, sources.length - 1)]);
  const task = $derived(
    sources
      .filter((s) => s.kind !== "behaviour")
      .map((s) => s.text)
      .join("\n")
      .match(/\bTASK\s+(\w+)\s*\([^)]*INTERVAL\s*:=\s*(T#[\w.]+)/i),
  );
</script>

<section class="editor">
  <aside class="organizer">
    <h3>{m.ui_organizer()}</h3>
    <div class="tree-line">
      <ChevronDown size={12} /><Cpu size={14} /><strong>PLC</strong><span
        class="led"
        class:on={status === "ok"}
        class:err={status === "failed"}
      ></span>
    </div>
    <div class="tree-line task">
      <Clock3 size={13} /><span>{task?.[1] ?? "MainTask"}</span><small>{task?.[2].replace(/^T#/i, "") ?? "10 ms"}</small
      >
    </div>
    {#each ["program", "behaviour"] as kind (kind)}
      <details open>
        <summary><Folder size={12} />{kind === "program" ? m.ui_programs() : m.ui_function_blocks()}</summary>
        {#each sources as source, i (source.key)}
          {#if (source.kind === "behaviour") === (kind === "behaviour")}
            <button class="source" class:active={source === current} title={source.label} onclick={() => (active = i)}>
              {#if kind === "behaviour"}<Braces size={12} />{:else}<FileCode2 size={12} />{/if}<span
                >{source.label}</span
              >{#if edited(source)}<b>•</b>{/if}
            </button>
          {/if}
        {/each}
      </details>
    {/each}
  </aside>
  <div class="code-pane">
    <header class="code-head">
      <div class="tabs" role="tablist" aria-label={m.editor_title()}>
        {#each sources as source, i (source.key)}
          <button
            role="tab"
            class="tab"
            class:edited={edited(source)}
            title={edited(source) ? `${source.label} · ${m.editor_edited()}` : source.label}
            aria-selected={source === current}
            onclick={() => (active = i)}><FileCode2 size={12} />{source.label}</button
          >
        {/each}
      </div>
      <span class="language">ST</span><button class="ladder" disabled title={m.ui_ladder_unavailable()}>LD</button>
    </header>
    {#if current}
      <div class="code-area" aria-label={current.label}>
        <StEditor bind:sources {active} {problems} {reveal} />
      </div>
    {/if}
    <footer>
      <span class="led" class:on={status === "ok"} class:err={status === "failed"}></span><span
        >{status === "ok" ? m.built_ok() : status === "failed" ? m.build_failed() : m.not_built()}</span
      ><span class="spacer"></span>{#if current && edited(current)}<button
          class="revert"
          onclick={() => (current.text = current.base)}>{m.editor_revert()}</button
        >{/if}<span title={m.editor_hint()}>Structured Text · UTF-8</span>
    </footer>
  </div>
</section>

<style>
  .editor {
    display: grid;
    grid-template-columns: 224px minmax(0, 1fr);
    height: 100%;
    min-height: 0;
  }
  .organizer {
    overflow: auto;
    border-right: 1px solid var(--color-border);
    padding: 10px 8px;
  }
  h3 {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--color-text-faint);
    margin: 0 0 10px 6px;
  }
  .tree-line {
    display: flex;
    gap: 7px;
    align-items: center;
    font-size: 11px;
    padding: 4px 5px;
  }
  .tree-line .led {
    width: 5px;
    height: 5px;
    margin-left: auto;
  }
  .task {
    padding-left: 25px;
    color: var(--color-text-muted);
  }
  small {
    margin-left: auto;
    font-size: 9px;
    color: var(--color-text-faint);
  }
  summary {
    font-size: 10px;
    padding: 6px 5px 5px 18px;
    color: var(--color-text-faint);
  }
  summary :global(svg) {
    vertical-align: middle;
    margin-right: 5px;
  }
  .source {
    width: 100%;
    display: flex;
    gap: 6px;
    align-items: center;
    border: 0;
    background: transparent;
    padding: 5px 8px 5px 32px;
    font-size: 10px;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
  }
  .source span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .source :global(svg) {
    flex: none;
  }
  .source.active {
    background: var(--color-accent-soft);
    color: var(--color-ring);
  }
  .code-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
  }
  .code-head {
    display: flex;
    height: 32px;
    flex: none;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    padding-right: 8px;
  }
  .tabs {
    display: flex;
    flex: 1;
    overflow: auto;
    height: 100%;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    border: 0;
    border-right: 1px solid var(--color-border);
    background: transparent;
    padding: 0 12px;
    font-size: 10px;
    white-space: nowrap;
    cursor: pointer;
    color: var(--color-text-faint);
  }
  .tab[aria-selected="true"] {
    background: var(--color-bg);
    color: var(--color-text);
    border-top: 2px solid var(--color-ring);
  }
  .tab.edited::after {
    content: " •";
    color: var(--color-ring);
  }
  .language,
  .ladder {
    font-size: 9px;
    padding: 3px 6px;
  }
  .language {
    background: var(--color-accent-soft);
    color: var(--color-ring);
    border-radius: 3px;
  }
  .ladder {
    border: 0;
    background: transparent;
    color: var(--color-text-disabled);
  }
  .code-area {
    display: flex;
    flex: 1;
    min-height: 0;
    background: var(--color-bg);
  }
  footer {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-size: 9px;
    color: var(--color-text-faint);
    border-top: 1px solid var(--color-border);
  }
  footer .led {
    width: 5px;
    height: 5px;
  }
  .revert {
    background: transparent;
    border: 0;
    font-size: 9px;
    color: var(--color-ring);
    cursor: pointer;
  }
  @media (max-width: 1200px) {
    .editor {
      grid-template-columns: 190px minmax(0, 1fr);
    }
  }
</style>
