<script lang="ts">
  // The instances of an opened project, each with its family; selecting one shows it in the inspector.
  import { m } from "../paraglide/messages.js";
  import { family, type ProjectView } from "./project";

  let { project, selected = $bindable() }: { project: ProjectView; selected: string | null } = $props();
</script>

<section class="panel tree">
  <header class="panel-head"><h2 class="panel-title">{m.tree_title()}</h2></header>
  <ul class="panel-body">
    {#each project.instances as instance (instance.id)}
      <li>
        <button class="item" class:on={instance.id === selected} onclick={() => (selected = instance.id)}>
          <span>{instance.name}</span>
          <span class="muted">{family(instance.definition.split("@")[0])}</span>
        </button>
      </li>
    {/each}
  </ul>
</section>

<style>
  ul {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: 8px;
    list-style: none;
    overflow: auto;
  }
  .item {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }
  .item:hover {
    background: var(--color-surface-2);
  }
  .item.on {
    background: var(--color-accent-soft);
    color: var(--color-accent-text);
  }
</style>
