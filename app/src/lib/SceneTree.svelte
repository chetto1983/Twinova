<script lang="ts">
  import { Box, Eye, EyeOff, LockKeyhole, UnlockKeyhole, Search, History, Layers, Circle } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { familyName } from "./familyNames";
  import type { ProjectView } from "./project";
  let {
    project,
    selected = $bindable(),
    hidden,
    locked,
    history,
    onVisibility,
    onLock,
  }: {
    project: ProjectView;
    selected: string | null;
    hidden: string[];
    locked: string[];
    history: { label: string; time: string }[];
    onVisibility: (id: string) => void;
    onLock: (id: string) => void;
  } = $props();
  let tab = $state("scene");
  let search = $state("");
  const filtered = $derived(
    project.instances.filter((i) =>
      `${i.name} ${familyName(i.definition)}`.toLowerCase().includes(search.toLowerCase()),
    ),
  );
</script>

<section class="tree">
  <div class="panel-tabs" role="tablist" aria-label={m.tree_components()}>
    <button role="tab" aria-selected={tab === "scene"} onclick={() => (tab = "scene")}
      ><Layers size={13} />{m.tree_components()}<span class="count">{project.instances.length}</span></button
    >
    <button role="tab" aria-selected={tab === "history"} onclick={() => (tab = "history")}
      ><History size={13} />{m.ui_history()}</button
    >
  </div>
  {#if tab === "scene"}
    <label class="search-field"
      ><Search size={13} /><input
        aria-label={m.ui_filter_scene()}
        placeholder={m.ui_filter_scene()}
        bind:value={search}
      /></label
    >
    <ul>
      {#each filtered as instance (instance.id)}
        <li class:on={instance.id === selected} class:hidden-object={hidden.includes(instance.id)}>
          <button
            class="item"
            aria-pressed={instance.id === selected}
            onclick={() => (selected = instance.id)}
            title={`${instance.name} · ${familyName(instance.definition)}`}
          >
            {#if instance.definition.includes("sensor")}<Circle size={13} />{:else}<Box size={14} />{/if}
            <span class="item-name">{instance.name}</span><small>{familyName(instance.definition)}</small>
          </button>
          <button
            class="row-action"
            title={locked.includes(instance.id) ? m.ui_unlock() : m.ui_lock()}
            aria-label={`${locked.includes(instance.id) ? m.ui_unlock() : m.ui_lock()} ${instance.name}`}
            aria-pressed={locked.includes(instance.id)}
            onclick={() => onLock(instance.id)}
          >
            {#if locked.includes(instance.id)}<LockKeyhole size={12} />{:else}<UnlockKeyhole size={12} />{/if}
          </button>
          <button
            class="row-action"
            title={hidden.includes(instance.id) ? m.ui_show() : m.ui_hide()}
            aria-label={`${hidden.includes(instance.id) ? m.ui_show() : m.ui_hide()} ${instance.name}`}
            aria-pressed={hidden.includes(instance.id)}
            onclick={() => onVisibility(instance.id)}
          >
            {#if hidden.includes(instance.id)}<EyeOff size={13} />{:else}<Eye size={13} />{/if}
          </button>
        </li>
      {/each}
    </ul>
    {#if !filtered.length}<p class="empty">{m.ui_no_results()}</p>{/if}
  {:else}
    <ol class="history">
      {#each history as item, i (i)}<li>
          <History size={13} />
          <div>{item.label}<small>{item.time}</small></div>
        </li>{/each}
    </ol>
    {#if !history.length}<p class="empty">{m.ui_no_history()}</p>{/if}
  {/if}
</section>

<style>
  .tree {
    min-height: 100%;
    display: flex;
    flex-direction: column;
  }
  .search-field {
    margin: 10px 9px;
  }
  ul {
    padding: 0 5px;
    margin: 0;
    list-style: none;
  }
  li {
    display: flex;
    align-items: center;
    border-radius: 4px;
    min-width: 0;
  }
  li.on {
    background: var(--color-accent-soft);
    color: var(--color-accent-text);
  }
  li:hover {
    background: var(--color-surface-2);
  }
  .item {
    border: 0;
    background: transparent;
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    flex: 1;
    padding: 7px 2px 7px 14px;
    text-align: left;
    cursor: pointer;
    font-size: var(--text-sm);
  }
  .item :global(svg) {
    flex: none;
    color: var(--color-text-faint);
  }
  .item-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  small {
    color: var(--color-text-faint);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 63px;
  }
  .row-action {
    padding: 4px;
    border: 0;
    display: flex;
    align-items: center;
    background: transparent;
    color: var(--color-text-faint);
    cursor: pointer;
    flex: none;
  }
  .row-action[aria-pressed="true"] {
    color: var(--color-ring);
  }
  .hidden-object .item {
    opacity: 0.45;
  }
  .count {
    margin-left: 4px;
    color: var(--color-text-faint);
  }
  .empty {
    color: var(--color-text-faint);
    padding: 14px;
    font-size: var(--text-sm);
  }
  .history {
    padding: 10px;
    margin: 0;
    list-style: none;
  }
  .history li {
    gap: 8px;
    align-items: start;
    padding: 9px 0;
    border-bottom: 1px solid var(--color-border);
    font-size: var(--text-sm);
  }
  .history small {
    display: block;
    max-width: none;
    margin-top: 3px;
  }
</style>
