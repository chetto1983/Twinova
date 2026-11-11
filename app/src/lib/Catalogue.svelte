<script lang="ts">
  // The catalogue (spec 2026-11-11-editor-basics-design.md, "The view"): the library's families by group. A click
  // places one in the scene, pinning its definition in the project; pressed and released over the 3D view, it lands
  // there. Pointer events, not HTML5 drag and drop, so the view keeps its own pointer handling (OIP does the same).
  import { onMount } from "svelte";
  import { m } from "../paraglide/messages.js";
  import { catalogue, type CatalogueEntry } from "./backend";
  import { familyLabel } from "./project";

  let { onPlace, onDrag }: { onPlace: (key: string) => void; onDrag: (key: string) => void } = $props();

  let entries = $state<CatalogueEntry[]>([]);
  const GROUPS = [
    ["conveyors", m.catalogue_conveyors],
    ["devices", m.catalogue_devices],
    ["sensors", m.catalogue_sensors],
    ["flow", m.catalogue_flow],
  ] as const;

  onMount(() => {
    void catalogue().then((list) => (entries = list));
  });
</script>

<section class="panel catalogue">
  <header class="panel-head"><h2 class="panel-title">{m.catalogue_title()}</h2></header>
  <div class="panel-body">
    {#each GROUPS as [group, title] (group)}
      <h3 class="sub">{title()}</h3>
      <div class="cards">
        {#each entries.filter((e) => e.group === group) as entry (entry.key)}
          <button
            class="btn card"
            title={entry.key}
            onpointerdown={() => onDrag(entry.key)}
            onclick={() => onPlace(entry.key)}
          >
            {familyLabel(entry.key)}
          </button>
        {/each}
      </div>
    {/each}
  </div>
</section>

<style>
  .cards {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 0 0 10px;
  }
  .card {
    font-size: 12px;
  }
</style>
