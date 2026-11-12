<script lang="ts">
  import { Box, Search, SlidersHorizontal, Trash2, LockKeyhole } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { headingOf, headingQuat, parseValue, valueText } from "./inspector";
  import InspectorLinks from "./InspectorLinks.svelte";
  import NumberField from "./NumberField.svelte";
  import { familyLabel, signalRows, type Edit, type ParameterDef, type ProjectView, type Vec3 } from "./project";
  let {
    project,
    selected,
    locked = false,
    onEdit,
  }: {
    project: ProjectView;
    selected: string | null;
    locked?: boolean;
    onEdit: (op: Edit) => Promise<boolean>;
  } = $props();
  let filter = $state("");
  const instance = $derived(project.instances.find((i) => i.id === selected) ?? null);
  const def = $derived(instance ? project.definitions[instance.definition] : undefined);
  const axes = ["X", "Y", "Z"];
  const matches = (label: string): boolean => label.toLowerCase().includes(filter.toLowerCase());
  const groupOf = (p: ParameterDef): string =>
    /guard/.test(p.name)
      ? "guards"
      : /^(length|width|height|radius|angle|frame_height)$/.test(p.name)
        ? "size"
        : "parameters";
  const groups = $derived([
    { id: "parameters", label: m.inspector_parameters() },
    { id: "size", label: m.ui_size() },
    { id: "guards", label: m.ui_guards() },
  ]);
  async function commit(field: HTMLInputElement, op: Edit | null, shown: string): Promise<void> {
    if (op === null || !(await onEdit(op))) field.value = shown;
  }
  function move(axis: number, value: number): void {
    if (!instance) return;
    const position = [...instance.position] as Vec3;
    position[axis] = value;
    void onEdit({ op: "move", id: instance.id, position, rotation: instance.rotation });
  }
  function parameter(p: ParameterDef, text: string, field: HTMLInputElement): void {
    if (!instance) return;
    const value = parseValue(p, text);
    void commit(
      field,
      value === undefined ? null : { op: "set_parameter", id: instance.id, name: p.name, value },
      valueText(p, instance.parameters[p.name] ?? p.default),
    );
  }
</script>

<section class="inspector">
  <header class="panel-head">
    <SlidersHorizontal size={14} />
    <h2 class="panel-title">{m.inspector_title()}</h2>
    {#if locked}<span class="spacer"></span><LockKeyhole size={13} />{/if}
  </header>
  {#if instance && def}
    <div class="identity">
      <span class="object-icon"><Box size={19} /></span>
      <div><strong>{instance.name}</strong><small>{familyLabel(def.id)} · v{def.version}</small></div>
    </div>
    <label class="search-field"
      ><Search size={13} /><input
        aria-label={m.ui_filter_properties()}
        placeholder={m.ui_filter_properties()}
        bind:value={filter}
      /></label
    >
    {#if locked}<p class="locked">{m.ui_locked()}</p>{/if}
    <fieldset disabled={locked}>
      {#key instance}
        {#if !filter || matches(m.inspector_name())}
          <label class="name-field"
            ><span>{m.inspector_name()}</span><input
              aria-label={m.inspector_name()}
              value={instance.name}
              onchange={(e) =>
                commit(e.currentTarget, { op: "rename", id: instance.id, name: e.currentTarget.value }, instance.name)}
            /></label
          >
        {/if}
        {#if !filter || matches(m.inspector_transform()) || matches(m.inspector_position()) || matches(m.inspector_heading())}
          <details open>
            <summary>{m.inspector_transform()}</summary>
            <div class="section-body">
              <span class="field-label">{m.inspector_position()}</span>
              <div class="triple">
                {#each axes as axis, k (axis)}<NumberField
                    {axis}
                    label={`${m.inspector_position()} ${axis.toLowerCase()}`}
                    value={instance.position[k]}
                    onchange={(v) => move(k, v)}
                  />{/each}
              </div>
              <div class="property">
                <span>{m.inspector_heading()}</span><NumberField
                  label={m.inspector_heading()}
                  value={headingOf(instance.rotation)}
                  unit="°"
                  min={-180}
                  max={180}
                  step={1}
                  onchange={(v) =>
                    onEdit({ op: "move", id: instance.id, position: instance.position, rotation: headingQuat(v) })}
                />
              </div>
            </div>
          </details>
        {/if}
        {#each groups as group (group.id)}
          {@const params = def.parameters.filter(
            (p) => groupOf(p) === group.id && (!filter || matches(p.name) || matches(group.label)),
          )}
          {#if params.length}
            <details open>
              <summary>{group.label}<span>{params.length}</span></summary>
              <div class="section-body">
                {#each params as p (p.name)}
                  {@const value = instance.parameters[p.name] ?? p.default}
                  <div class="property" class:overridden={Object.hasOwn(instance.parameters, p.name)}>
                    <label for={`param-${p.name}`} title={p.name}>{p.name.replaceAll("_", " ")}</label>
                    {#if p.type === "BOOL"}
                      <input
                        id={`param-${p.name}`}
                        type="checkbox"
                        aria-label={p.name}
                        checked={Boolean(value)}
                        onchange={(e) =>
                          onEdit({
                            op: "set_parameter",
                            id: instance.id,
                            name: p.name,
                            value: e.currentTarget.checked,
                          })}
                      />
                    {:else if p.type === "REAL" || p.type === "DINT"}
                      <NumberField
                        id={`param-${p.name}`}
                        label={p.name}
                        value={Number(value)}
                        unit={p.unit ?? ""}
                        min={p.min ?? undefined}
                        max={p.max ?? undefined}
                        step={p.type === "DINT" ? 1 : 0.01}
                        integer={p.type === "DINT"}
                        onclear={() => onEdit({ op: "set_parameter", id: instance.id, name: p.name, value: null })}
                        onchange={(v) => onEdit({ op: "set_parameter", id: instance.id, name: p.name, value: v })}
                      />
                    {:else}
                      <input
                        id={`param-${p.name}`}
                        class="data"
                        aria-label={p.name}
                        value={valueText(p, value)}
                        onchange={(e) => parameter(p, e.currentTarget.value, e.currentTarget)}
                      />
                    {/if}
                  </div>
                {/each}
              </div>
            </details>
          {/if}
        {/each}
        {@const signals = signalRows(instance, def).filter(
          (row) => !filter || matches(row.name) || matches(m.inspector_signals()),
        )}
        {#if signals.length}
          <details open>
            <summary>{m.inspector_signals()}<span>{signals.length}</span></summary>
            <div class="section-body">
              {#each signals as row (row.name)}
                <div class="signal">
                  <label for={`signal-${row.name}`}
                    >{row.name}<span
                      class="chip"
                      class:plc={!!row.address && row.direction === "plc_output"}
                      title={row.direction === "plc_output" ? m.signal_plc_output() : m.signal_plc_input()}
                      >{row.address && row.direction === "plc_output" ? "PLC" : "TAG"}</span
                    ></label
                  >
                  <input
                    id={`signal-${row.name}`}
                    class="data"
                    aria-label={row.name}
                    placeholder={m.signal_unbound()}
                    value={row.address ?? ""}
                    onchange={(e) =>
                      commit(
                        e.currentTarget,
                        {
                          op: "bind_signal",
                          id: instance.id,
                          signal: row.name,
                          address: e.currentTarget.value || null,
                        },
                        row.address ?? "",
                      )}
                  />
                  <small
                    >{row.type} · {row.direction === "plc_output" ? m.signal_plc_output() : m.signal_plc_input()}</small
                  >
                </div>
              {/each}
            </div>
          </details>
        {/if}
        {#if !filter || matches(m.ui_connections())}<details open>
            <summary>{m.ui_connections()}</summary>
            <div class="section-body"><InspectorLinks {project} {instance} {onEdit} /></div>
          </details>{/if}
        {#if !filter}<button class="remove" onclick={() => onEdit({ op: "remove", id: instance.id })}
            ><Trash2 size={13} />{m.inspector_remove()}</button
          >{/if}
      {/key}
    </fieldset>
  {:else}
    <div class="empty">
      <Box size={32} strokeWidth={1} /><strong>{m.ui_select_hint()}</strong>
      <p>{m.inspector_hint()}</p>
    </div>
    <div class="project-info">
      <span>{m.tree_title()}</span><strong>{project.name}</strong><span
        >{project.instances.length} {m.ui_objects()}</span
      >
    </div>
  {/if}
</section>

<style>
  .inspector {
    min-height: 100%;
  }
  .identity {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 15px 12px;
  }
  .object-icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-ring);
    background: var(--color-accent-soft);
  }
  .identity strong {
    font-size: 12px;
  }
  .identity small {
    display: block;
    color: var(--color-text-faint);
    font-size: 10px;
    margin-top: 3px;
  }
  .search-field {
    margin: 0 12px 12px;
  }
  fieldset {
    padding: 0;
    border: 0;
    margin: 0;
    min-width: 0;
  }
  .name-field {
    display: grid;
    grid-template-columns: 1fr 1.5fr;
    align-items: center;
    padding: 0 12px 12px;
    color: var(--color-text-muted);
    font-size: 11px;
  }
  details {
    border-top: 1px solid var(--color-border);
  }
  summary {
    padding: 10px 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
    font-size: 10px;
    color: var(--color-text-muted);
    font-weight: 650;
  }
  summary span {
    float: right;
    color: var(--color-text-faint);
    font-weight: 400;
    letter-spacing: 0;
  }
  .section-body {
    padding: 0 12px 12px;
  }
  .field-label {
    display: block;
    margin-bottom: 6px;
    font-size: 10px;
    color: var(--color-text-faint);
  }
  .triple {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 5px;
    margin-bottom: 10px;
  }
  .property {
    display: grid;
    grid-template-columns: 1fr 1.1fr;
    align-items: center;
    gap: 8px;
    margin-top: 7px;
    min-width: 0;
    font-size: 11px;
    color: var(--color-text-muted);
  }
  .property > label {
    overflow-wrap: anywhere;
  }
  .property > input:not([type="checkbox"]),
  .name-field input,
  .signal input {
    background: var(--color-surface-2);
    border-color: transparent;
    font-weight: 400;
  }
  .property input[type="checkbox"] {
    justify-self: end;
  }
  .overridden > label {
    color: var(--color-accent-text);
  }
  .signal {
    display: grid;
    grid-template-columns: 1fr 1.1fr;
    align-items: center;
    gap: 5px 8px;
    margin-top: 9px;
  }
  .signal label {
    font-size: 10px;
    overflow-wrap: anywhere;
  }
  .signal small {
    grid-column: 2;
    font-size: 9px;
    color: var(--color-text-faint);
  }
  .chip {
    margin-left: 5px;
    padding: 1px 3px;
    border-radius: 2px;
    font-size: 8px;
    color: var(--color-text-faint);
    background: var(--color-surface-3);
  }
  .chip.plc {
    color: var(--color-ring);
    background: var(--color-accent);
  }
  .remove {
    display: flex;
    gap: 7px;
    align-items: center;
    margin: 12px;
    background: transparent;
    border: 0;
    color: var(--color-danger);
    cursor: pointer;
    font-size: 11px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 50px 24px 32px;
    color: var(--color-text-faint);
  }
  .empty strong {
    color: var(--color-text-muted);
    margin-top: 14px;
    font-size: 12px;
  }
  .empty p {
    font-size: 11px;
    line-height: 1.6;
  }
  .project-info {
    display: grid;
    gap: 8px;
    margin: 0 18px;
    padding: 16px 0;
    border-top: 1px solid var(--color-border);
    font-size: 10px;
    color: var(--color-text-faint);
  }
  .project-info strong {
    color: var(--color-text-muted);
    font-weight: 500;
  }
  .locked {
    padding: 6px 12px;
    color: var(--color-warning);
    font-size: 10px;
  }
</style>
