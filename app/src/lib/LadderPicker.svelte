<script lang="ts">
  // The box picker (Ctrl+B, the toolbox's Box), ported from nautilus's (`LdBlockPicker.svelte` at 2db093e, Apache-2.0,
  // Copyright 2026 Joy Automation; THIRD_PARTY_NOTICES.md): every block and function truST knows for this program
  // (`ladder_catalog`), grouped by IEC table, a block's new instance named, and the call's pins editable before the
  // insert. A name the catalogue does not hold inserts too: the rung's errors then say what it is not.
  import { m } from "../paraglide/messages.js";
  import { filtered, freeInstance, grouped, template, type CatalogEntry, type Group } from "./ladder/catalog";

  let {
    entries,
    taken,
    problem,
    onInsert,
    onClose,
  }: {
    entries: CatalogEntry[];
    taken: string[];
    problem: string;
    onInsert: (choice: { entry: CatalogEntry | null; head: string; pins: string }) => void;
    onClose: () => void;
  } = $props();

  const GROUP_NAMES: Record<Group, () => string> = {
    project: m.ladder_group_project,
    timer: m.ladder_group_timer,
    counter: m.ladder_group_counter,
    edge: m.ladder_group_edge,
    bistable: m.ladder_group_bistable,
    comparison: m.ladder_group_comparison,
    math: m.ladder_group_math,
    bitwise: m.ladder_group_bitwise,
    selection: m.ladder_group_selection,
    string: m.ladder_group_string,
    time: m.ladder_group_time,
    conversion: m.ladder_group_conversion,
    other: m.ladder_group_other,
  };

  let filter = $state("");
  let chosen = $state<CatalogEntry | null>(null);
  let instance = $state("");
  let pins = $state("");
  let active = $state(-1);
  let filterField: HTMLInputElement | undefined = $state();
  let instanceField: HTMLInputElement | undefined = $state();

  const groups = $derived(grouped(filtered(entries, chosen?.name === filter ? "" : filter)));
  const ordered = $derived(groups.flatMap(([, list]) => list));
  const identifier = /^[A-Za-z_][A-Za-z0-9_]*$/;
  const block = $derived(chosen?.block ?? false);
  const head = $derived(block ? instance.trim() : (chosen?.name ?? filter.trim()));
  const ready = $derived(identifier.test(head));
  const signature = (e: CatalogEntry): string =>
    `(${e.inputs.map((p) => p.name).join(", ")}${e.variadic ? ", …" : ""}) → ${e.outputs.map((p) => p.name).join(", ")}`;

  $effect(() => filterField?.focus());

  function choose(e: CatalogEntry, then: "instance" | "insert" | "stay" = "instance"): void {
    chosen = e;
    filter = e.name;
    pins = template(e);
    instance = e.block ? freeInstance(e.name, taken) : "";
    active = -1;
    if (then === "insert") insert();
    else if (then === "instance" && e.block) queueMicrotask(() => instanceField?.select());
  }

  function insert(): void {
    if (ready) onInsert({ entry: chosen, head, pins: pins.trim() });
  }

  function onFilterKey(e: KeyboardEvent): void {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const n = ordered.length;
      if (n > 0) active = e.key === "ArrowDown" ? (active + 1) % n : active <= 0 ? n - 1 : active - 1;
    } else if (e.key === "Enter") {
      e.preventDefault();
      const exact = entries.find((x) => x.name.toLowerCase() === filter.trim().toLowerCase());
      const pick = ordered[active] ?? exact ?? (ordered.length === 1 ? ordered[0] : undefined);
      if (pick && pick !== chosen) choose(pick);
      else insert();
    }
  }

  function onkeydown(e: KeyboardEvent): void {
    // nothing typed here may reach the view's keys (Delete, the arrows, Enter)
    e.stopPropagation();
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "Enter" && e.target !== filterField && (e.target as HTMLElement).tagName === "INPUT") {
      e.preventDefault();
      insert();
    }
  }
</script>

<div class="picker" role="dialog" aria-label={m.ladder_picker()} tabindex="-1" {onkeydown}>
  <input
    bind:this={filterField}
    bind:value={filter}
    aria-label={m.ladder_picker()}
    placeholder={m.ladder_picker_filter()}
    spellcheck="false"
    autocomplete="off"
    oninput={() => {
      chosen = null;
      active = -1;
    }}
    onkeydown={onFilterKey}
  />
  <div class="list" role="listbox" aria-label={m.ladder_candidates()}>
    {#each groups as [group, list] (group)}
      <div class="group">{GROUP_NAMES[group]()}</div>
      {#each list as e (e.name)}
        <button
          role="option"
          aria-selected={e === chosen}
          class:on={e === chosen}
          class:active={ordered[active] === e}
          title={`${e.name}${signature(e)}`}
          onclick={() => choose(e)}
          ondblclick={() => choose(e, "insert")}
          ><span class="name">{e.name}</span><span class="signature">{signature(e)}</span></button
        >
      {/each}
    {:else}
      <div class="none">{m.ladder_picker_none()}</div>
    {/each}
  </div>
  {#if block}
    <label
      ><span>{m.ladder_instance()}</span><input
        bind:this={instanceField}
        bind:value={instance}
        spellcheck="false"
      /></label
    >
  {/if}
  <label
    ><span>{m.ladder_pins()}</span><input
      bind:value={pins}
      spellcheck="false"
      placeholder="PT := T#2s, ET => Waited"
    /></label
  >
  {#if problem}<p class="problem" role="status">{problem}</p>{/if}
  <div class="actions">
    <button onclick={onClose}>{m.ladder_cancel()}</button>
    <button class="primary" disabled={!ready} onclick={insert}>{m.ladder_insert()}</button>
  </div>
</div>

<style>
  .picker {
    position: absolute;
    z-index: 4;
    top: 36px;
    left: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 360px;
    padding: 6px;
    border: 1px solid var(--color-border);
    border-radius: 5px;
    background: var(--color-surface-2);
    font-size: 11px;
  }
  input {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 2px 4px;
  }
  .list {
    display: flex;
    flex-direction: column;
    max-height: 240px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: 3px;
  }
  .group {
    padding: 4px 8px 1px;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }
  .list button {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    border: 0;
    padding: 2px 8px;
    background: transparent;
    font-family: var(--font-mono);
    font-size: 11px;
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
  }
  .list button:hover,
  .list button.active {
    background: var(--color-surface-3);
  }
  .list button.on {
    background: var(--color-accent-soft);
    color: var(--color-ring);
  }
  .signature {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-text-muted);
    font-size: 10px;
  }
  .none {
    padding: 4px 8px;
    color: var(--color-text-muted);
  }
  label {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--color-text-muted);
  }
  label span {
    min-width: 56px;
  }
  label input {
    flex: 1;
  }
  .problem {
    margin: 0;
    color: var(--color-warning);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
</style>
