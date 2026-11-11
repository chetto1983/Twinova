<script lang="ts">
  // The selected instance's properties, edited in place (spec 2026-11-11-editor-basics-design.md, "The view"): its name,
  // position and heading, its parameters by type, its signals' addresses, and its flow ports and mount
  // (InspectorLinks.svelte). A field commits on Enter or when it loses focus, as one undo step; a value the field cannot
  // read, or one the project refuses, puts the field back to what the project has.
  import { m } from "../paraglide/messages.js";
  import { headingOf, headingQuat, parseValue, valueText } from "./inspector";
  import InspectorLinks from "./InspectorLinks.svelte";
  import { family, signalRows, type Edit, type ParameterDef, type ProjectView, type Vec3 } from "./project";

  let {
    project,
    selected,
    onEdit,
  }: { project: ProjectView; selected: string | null; onEdit: (op: Edit) => Promise<boolean> } = $props();

  const instance = $derived(project.instances.find((i) => i.id === selected) ?? null);
  const def = $derived(instance ? project.definitions[instance.definition] : undefined);
  const directions = { plc_input: m.signal_plc_input, plc_output: m.signal_plc_output };
  const AXES = ["x", "y", "z"] as const;

  /** Sends `op`; if it is refused, the field shows what the project has again. */
  async function commit(field: HTMLInputElement, op: Edit | null, shown: string): Promise<void> {
    if (op === null || !(await onEdit(op))) field.value = shown;
  }

  function move(axis: number, text: string, field: HTMLInputElement): void {
    if (!instance) return;
    const value = Number(text.replace(",", "."));
    const position = [...instance.position] as Vec3;
    position[axis] = value;
    const op = Number.isFinite(value) && text.trim() !== "" ? { position, rotation: instance.rotation } : null;
    void commit(field, op && { op: "move", id: instance.id, ...op }, String(instance.position[axis]));
  }

  function turn(text: string, field: HTMLInputElement): void {
    if (!instance) return;
    const degrees = Number(text.replace(",", "."));
    const op: Edit | null = Number.isFinite(degrees)
      ? { op: "move", id: instance.id, position: instance.position, rotation: headingQuat(degrees) }
      : null;
    void commit(field, op, String(headingOf(instance.rotation)));
  }

  function setParameter(p: ParameterDef, text: string, field: HTMLInputElement): void {
    if (!instance) return;
    const value = parseValue(p, text);
    const op: Edit | null = value === undefined ? null : { op: "set_parameter", id: instance.id, name: p.name, value };
    void commit(field, op, valueText(p, instance.parameters[p.name] ?? p.default));
  }

  const isOwn = (name: string): boolean => instance !== null && Object.hasOwn(instance.parameters, name);
</script>

<section class="panel inspector">
  <header class="panel-head"><h2 class="panel-title">{m.inspector_title()}</h2></header>
  <div class="panel-body">
    {#if instance && def}
      {#key instance}
        <label class="head">
          <span class="muted">{family(def.id)} · v{def.version}</span>
          <input
            aria-label={m.inspector_name()}
            value={instance.name}
            onchange={(e) =>
              commit(e.currentTarget, { op: "rename", id: instance.id, name: e.currentTarget.value }, instance.name)}
          />
        </label>
        <h3 class="sub">{m.inspector_transform()}</h3>
        <dl>
          <dt class="muted">{m.inspector_position()}</dt>
          <dd class="triple">
            {#each AXES as axis, k (axis)}
              <input
                class="data"
                aria-label={`${m.inspector_position()} ${axis}`}
                value={instance.position[k]}
                onchange={(e) => move(k, e.currentTarget.value, e.currentTarget)}
              />
            {/each}
          </dd>
          <dt class="muted">{m.inspector_heading()}</dt>
          <dd>
            <input
              class="data"
              aria-label={m.inspector_heading()}
              value={headingOf(instance.rotation)}
              onchange={(e) => turn(e.currentTarget.value, e.currentTarget)}
            />
          </dd>
        </dl>
        {#if def.parameters.length > 0}
          <h3 class="sub">{m.inspector_parameters()}</h3>
          <dl>
            {#each def.parameters as p (p.name)}
              <dt class="muted" class:own={isOwn(p.name)} title={isOwn(p.name) ? m.inspector_overridden() : undefined}>
                {p.name}{p.unit ? ` (${p.unit})` : ""}
              </dt>
              <dd>
                {#if p.type === "BOOL"}
                  <input
                    type="checkbox"
                    aria-label={p.name}
                    checked={Boolean(instance.parameters[p.name] ?? p.default)}
                    onchange={(e) =>
                      onEdit({ op: "set_parameter", id: instance.id, name: p.name, value: e.currentTarget.checked })}
                  />
                {:else}
                  <input
                    class="data"
                    aria-label={p.name}
                    placeholder={valueText(p, p.default)}
                    value={isOwn(p.name) ? valueText(p, instance.parameters[p.name]) : ""}
                    onchange={(e) => setParameter(p, e.currentTarget.value, e.currentTarget)}
                  />
                {/if}
              </dd>
            {/each}
          </dl>
        {/if}
        {#if def.signals.length > 0}
          <h3 class="sub">{m.inspector_signals()}</h3>
          <dl>
            {#each signalRows(instance, def) as row (row.name)}
              <dt class="muted">{row.name}</dt>
              <dd>
                <input
                  class="data"
                  aria-label={row.name}
                  placeholder={m.signal_unbound()}
                  value={row.address ?? ""}
                  onchange={(e) =>
                    commit(
                      e.currentTarget,
                      { op: "bind_signal", id: instance.id, signal: row.name, address: e.currentTarget.value || null },
                      row.address ?? "",
                    )}
                />
                <span class="chip">{directions[row.direction]()} {row.type}</span>
              </dd>
            {/each}
          </dl>
        {/if}
        <InspectorLinks {project} {instance} {onEdit} />
        <button class="btn danger" onclick={() => onEdit({ op: "remove", id: instance.id })}>
          {m.inspector_remove()}
        </button>
      {/key}
    {:else}
      <p class="muted">{m.inspector_hint()}</p>
    {/if}
  </div>
</section>

<style>
  .head {
    display: grid;
    gap: 4px;
    margin: 0 0 8px;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 6px 12px;
    margin: 0 0 12px;
  }
  dd {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
  }
  .triple input {
    min-width: 0;
  }
  .own {
    color: var(--color-accent-text);
    font-weight: 600;
  }
  .chip {
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--color-surface-3);
    color: var(--color-text-muted);
    font-size: 11px;
    white-space: nowrap;
  }
</style>
