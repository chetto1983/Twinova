<script lang="ts">
  // The properties of the selected instance, read-only in 4.2: pose, parameters (defaults, and the instance's own
  // values marked) and signals with their IEC address and direction seen from the PLC.
  import { m } from "../paraglide/messages.js";
  import { eulerDegrees, family, parameterRows, signalRows, type ProjectView } from "./project";

  let { project, selected }: { project: ProjectView; selected: string | null } = $props();

  const instance = $derived(project.instances.find((i) => i.id === selected) ?? null);
  const def = $derived(instance ? project.definitions[instance.definition] : undefined);
  const directions = { plc_input: m.signal_plc_input, plc_output: m.signal_plc_output };
</script>

<section class="panel inspector">
  <header class="panel-head"><h2 class="panel-title">{m.inspector_title()}</h2></header>
  <div class="panel-body">
    {#if instance && def}
      <p class="head"><strong>{instance.name}</strong> <span class="muted">{family(def.id)} · v{def.version}</span></p>
      <h3 class="sub">{m.inspector_transform()}</h3>
      <dl>
        <dt class="muted">{m.inspector_position()}</dt>
        <dd class="data">{instance.position.join(" · ")}</dd>
        <dt class="muted">{m.inspector_rotation()}</dt>
        <dd class="data">{eulerDegrees(instance.rotation).join(" · ")}</dd>
      </dl>
      {#if def.parameters.length > 0}
        <h3 class="sub">{m.inspector_parameters()}</h3>
        <dl>
          {#each parameterRows(instance, def) as row (row.name)}
            <dt class="muted">{row.name}</dt>
            <dd class="data" class:own={row.overridden} title={row.overridden ? m.inspector_overridden() : undefined}>
              {row.text}
            </dd>
          {/each}
        </dl>
      {/if}
      {#if def.signals.length > 0}
        <h3 class="sub">{m.inspector_signals()}</h3>
        <dl>
          {#each signalRows(instance, def) as row (row.name)}
            <dt class="muted">{row.name}</dt>
            <dd class="data">
              {row.address ?? m.signal_unbound()}
              <span class="chip">{directions[row.direction]()} {row.type}</span>
            </dd>
          {/each}
        </dl>
      {/if}
    {:else}
      <p class="muted">{m.inspector_hint()}</p>
    {/if}
  </div>
</section>

<style>
  .head {
    margin: 0 0 8px;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    margin: 0 0 12px;
  }
  dd {
    margin: 0;
  }
  .own {
    color: var(--color-accent-text);
    font-weight: 600;
  }
  .chip {
    margin-left: 6px;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--color-surface-3);
    color: var(--color-text-muted);
    font-size: 11px;
  }
</style>
