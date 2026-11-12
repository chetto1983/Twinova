<script lang="ts">
  import { onMount } from "svelte";
  import { Search, Plus, PackageOpen } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { catalogue, type CatalogueEntry } from "./backend";
  import { familyLabel } from "./project";
  let { onPlace, onDrag }: { onPlace: (key: string) => void; onDrag: (key: string) => void } = $props();
  let entries = $state<CatalogueEntry[]>([]);
  let previews = $state<Record<string, string>>({});
  let category = $state("equipment");
  let search = $state("");
  let error = $state("");
  let loading = $state(true);
  const groups = $derived([
    { id: "equipment", label: m.ui_equipment() },
    { id: "devices", label: m.ui_devices() },
    { id: "products", label: m.ui_products() },
    { id: "structure", label: m.ui_structure() },
  ]);
  const categoryOf = (e: CatalogueEntry): string =>
    e.group === "conveyors" || e.group === "devices" ? "equipment" : e.group === "sensors" ? "devices" : "products";
  const filtered = $derived(
    entries.filter((e) => categoryOf(e) === category && familyLabel(e.key).includes(search.toLowerCase())),
  );
  onMount(() => {
    let alive = true;
    void catalogue()
      .then(async (list) => {
        if (!alive) return;
        entries = list;
        const { partPreviews } = await import("./partPreviews");
        previews = await partPreviews(list.map((e) => e.key));
      })
      .catch((e) => {
        if (alive) error = String(e);
      })
      .finally(() => {
        if (alive) loading = false;
      });
    return () => {
      alive = false;
    };
  });
</script>

<section class="catalogue">
  <div class="catalogue-bar">
    <div class="categories" role="tablist" aria-label={m.catalogue_title()}>
      {#each groups as group (group.id)}<button
          role="tab"
          aria-selected={category === group.id}
          onclick={() => (category = group.id)}>{group.label}</button
        >{/each}
    </div>
    <span class="spacer"></span>
    <label class="search-field"
      ><Search size={13} /><input
        aria-label={m.ui_search_parts()}
        placeholder={m.ui_search_parts()}
        bind:value={search}
      /></label
    >
  </div>
  <div class="cards">
    {#each filtered as entry (entry.key)}
      <button class="card" title={entry.key} onpointerdown={() => onDrag(entry.key)} onclick={() => onPlace(entry.key)}>
        <div class="preview">
          {#if previews[entry.key]}<img src={previews[entry.key]} alt="" draggable="false" />{:else}<PackageOpen
              size={25}
              strokeWidth={1}
            />{/if}<span class="add"><Plus size={12} /></span>
        </div>
        <span class="name">{familyLabel(entry.key)}</span><span class="version">Twinova · v1</span>
      </button>
    {/each}
    {#if !filtered.length && !loading}<p class="empty">
        <PackageOpen size={24} />{search ? m.ui_no_results() : m.ui_no_parts()}
      </p>{/if}
  </div>
  {#if error}<p class="error-line">{m.ui_catalogue_error()} {error}</p>{/if}
</section>

<style>
  .catalogue {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .catalogue-bar {
    display: flex;
    gap: 14px;
    align-items: center;
    min-height: 42px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--color-border);
  }
  .categories {
    display: flex;
    gap: 3px;
  }
  .categories button {
    border: 1px solid transparent;
    padding: 4px 11px;
    border-radius: 4px;
    font-size: 10px;
    background: transparent;
    color: var(--color-text-faint);
    cursor: pointer;
  }
  .categories button[aria-selected="true"] {
    color: var(--color-accent-text);
    border-color: var(--color-accent-muted);
    background: var(--color-accent-soft);
  }
  .search-field {
    width: 190px;
  }
  .cards {
    display: flex;
    gap: 9px;
    overflow: auto;
    flex: 1;
    min-height: 0;
    padding: 12px;
    align-items: flex-start;
  }
  .card {
    width: 126px;
    flex: none;
    padding: 0 0 9px;
    text-align: left;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 5px;
    overflow: hidden;
    cursor: grab;
  }
  .card:hover {
    border-color: var(--color-ring);
    background: var(--color-accent-soft);
  }
  .preview {
    height: 76px;
    display: grid;
    place-items: center;
    position: relative;
    color: var(--color-text-faint);
    background: radial-gradient(ellipse at 50% 70%, var(--color-surface-3), transparent 75%);
  }
  .preview img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    pointer-events: none;
  }
  .add {
    position: absolute;
    top: 5px;
    right: 5px;
    opacity: 0;
    color: var(--color-ring);
  }
  .card:hover .add {
    opacity: 1;
  }
  .name {
    display: block;
    padding: 2px 9px;
    font-size: 10px;
    text-transform: capitalize;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .version {
    display: block;
    padding: 0 9px;
    font-size: 9px;
    color: var(--color-text-faint);
  }
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    width: 100%;
    margin: auto;
    font-size: 11px;
    color: var(--color-text-faint);
  }
</style>
