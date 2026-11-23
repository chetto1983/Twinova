<script lang="ts">
  // The PLC organizer, out of Editor.svelte: the PLC, its task and interval (from the program's TASK, 10 ms without
  // one), and the program files and the blocks, each opening its tab.
  import { Cpu, ChevronDown, FileCode2, Braces, Clock3, Folder } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { edited, type Source } from "./sources";

  let {
    sources,
    current,
    status,
    onOpen,
  }: {
    sources: Source[];
    current: Source | undefined;
    status: "none" | "ok" | "failed";
    onOpen: (index: number) => void;
  } = $props();

  const task = $derived(
    sources
      .filter((s) => s.kind !== "behaviour")
      .map((s) => s.text)
      .join("\n")
      .match(/\bTASK\s+(\w+)\s*\([^)]*INTERVAL\s*:=\s*(T#[\w.]+)/i),
  );
</script>

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
    <Clock3 size={13} /><span>{task?.[1] ?? "MainTask"}</span><small>{task?.[2].replace(/^T#/i, "") ?? "10 ms"}</small>
  </div>
  {#each ["program", "behaviour"] as kind (kind)}
    <details open>
      <summary><Folder size={12} />{kind === "program" ? m.ui_programs() : m.ui_function_blocks()}</summary>
      {#each sources as source, i (source.key)}
        {#if (source.kind === "behaviour") === (kind === "behaviour")}
          <button class="source" class:active={source === current} title={source.label} onclick={() => onOpen(i)}>
            {#if kind === "behaviour"}<Braces size={12} />{:else}<FileCode2 size={12} />{/if}<span>{source.label}</span
            >{#if edited(source)}<b>•</b>{/if}
          </button>
        {/if}
      {/each}
    </details>
  {/each}
</aside>

<style>
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
</style>
