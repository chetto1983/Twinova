<script lang="ts">
  // The variables table above the code (spec 2026-11-13-online-monitoring-design.md, decisions 9 and 10): the POU's
  // declarations, edited in place, and while monitoring their values, written or forced, with an instance unfolded
  // into its members.
  import { ChevronRight, Plus, Trash2 } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { CLASSES, labelOf, newRow, type Change, type Declaration, type Row, type Target } from "./monitor";

  let {
    declarations,
    rows,
    values,
    forced,
    unfolded,
    onChange,
    onRename,
    onUnfold,
    onPick,
  }: {
    declarations: Declaration[];
    /** The rows the engine reads while monitoring, members included; null when not monitoring. */
    rows: Row[] | null;
    values: (string | null)[];
    forced: Set<string>;
    unfolded: string[];
    onChange: (change: Change) => void;
    onRename: (index: number, name: string) => void;
    onUnfold: (name: string) => void;
    /** A row's value to write or force. */
    onPick: (target: Target, value: string | null) => void;
  } = $props();

  type Shown = { row: Row | null; decl: Declaration | null; index: number | null; value: string | null };
  const shown = $derived<Shown[]>(
    rows
      ? rows.map((row, i) => ({
          row,
          decl: row.declaration === null ? null : (declarations[row.declaration] ?? null),
          index: row.declaration,
          value: values[i] ?? null,
        }))
      : declarations.map((decl, index) => ({ row: null, decl, index, value: null })),
  );

  function update(index: number, decl: Declaration, field: keyof Declaration, text: string): void {
    const value = text.trim() === "" && field !== "type_name" ? null : text.trim();
    onChange({ op: "update", index, row: { ...decl, [field]: value } });
  }

  const pou = $derived(declarations[0]?.pou ?? "");
  const FIELDS: [keyof Declaration, () => string][] = [
    ["type_name", () => m.vars_type()],
    ["address", () => m.vars_address()],
    ["initial", () => m.vars_initial()],
    ["comment", () => m.vars_comment()],
  ];
</script>

<div class="vars" role="region" aria-label={m.vars_title()}>
  <table>
    <thead>
      <tr>
        <th>#</th>
        <th>{m.vars_name()}</th>
        <th>{m.vars_class()}</th>
        {#each FIELDS as [field, label] (field)}<th>{label()}</th>{/each}
        {#if rows}<th>{m.vars_value()}</th>{/if}
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each shown as item (item.row?.name ?? `d${item.index}`)}
        {@const decl = item.decl}
        <tr class:member={item.row !== null && item.row.declaration === null}>
          <td class="num">{item.index === null ? "" : item.index + 1}</td>
          <td class="name">
            {#if item.row?.instance}<button
                class="unfold"
                class:open={unfolded.includes(item.row.name)}
                title={m.vars_unfold()}
                aria-label={m.vars_unfold()}
                onclick={() => onUnfold(item.row!.name)}><ChevronRight size={11} /></button
              >{/if}
            {#if decl && item.index !== null}
              <input
                aria-label={m.vars_name()}
                value={decl.name}
                onchange={(e) => onRename(item.index!, e.currentTarget.value.trim())}
              />
            {:else}<span class="data">{item.row?.name}</span>{/if}
          </td>
          <td>
            {#if decl && item.index !== null}
              <select
                aria-label={m.vars_class()}
                value={`${decl.class}${decl.qualifier ? ` ${decl.qualifier}` : ""}`}
                onchange={(e) => {
                  const [cls, qualifier] = e.currentTarget.value.split(" ");
                  onChange({
                    op: "update",
                    index: item.index!,
                    row: { ...decl, class: cls, qualifier: qualifier ?? null },
                  });
                }}
              >
                {#each CLASSES.flatMap((c) => [c, `${c} RETAIN`, `${c} CONSTANT`]) as option (option)}
                  <option value={option}>{option}</option>
                {/each}
              </select>
            {/if}
          </td>
          {#each FIELDS as [field, label] (field)}
            <td>
              {#if decl && item.index !== null}
                <input
                  aria-label={label()}
                  value={decl[field] ?? ""}
                  onchange={(e) => update(item.index!, decl, field, e.currentTarget.value)}
                />
              {/if}
            </td>
          {/each}
          {#if rows}
            <td class="value">
              {#if item.row?.target}
                {@const target = item.row.target}
                <button
                  class="data pick"
                  class:forced={forced.has(labelOf(target))}
                  class:true-value={item.value === "TRUE"}
                  onclick={() => onPick(target, item.value)}>{item.value ?? "—"}</button
                >
              {:else}<span class="data">{item.value ?? ""}</span>{/if}
            </td>
          {/if}
          <td>
            {#if item.index !== null}
              <button
                class="icon"
                title={m.vars_delete()}
                aria-label={m.vars_delete()}
                onclick={() => onChange({ op: "delete", index: item.index! })}><Trash2 size={12} /></button
              >
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if pou}
    <button
      class="add"
      onclick={() =>
        onChange({
          op: "add",
          row: newRow(
            pou,
            declarations.map((d) => d.name),
          ),
        })}><Plus size={12} />{m.vars_add()}</button
    >
  {/if}
</div>

<style>
  .vars {
    overflow: auto;
    height: 100%;
    font-size: 11px;
    border-bottom: 1px solid var(--color-border);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th {
    position: sticky;
    top: 0;
    background: var(--color-surface-2);
    color: var(--color-text-muted);
    font-weight: 600;
    text-align: left;
    padding: 3px 6px;
    border-bottom: 1px solid var(--color-border);
    z-index: 1;
  }
  td {
    padding: 1px 4px;
    border-bottom: 1px solid var(--color-border);
    white-space: nowrap;
  }
  td.num {
    color: var(--color-text-faint);
    text-align: right;
    width: 24px;
  }
  tr.member td {
    color: var(--color-text-muted);
  }
  tr.member .data {
    padding-left: 18px;
  }
  input,
  select {
    width: 100%;
    min-width: 60px;
    font: inherit;
    font-family: var(--font-mono);
    color: var(--color-text);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 3px;
    padding: 1px 3px;
  }
  input:hover,
  select:hover,
  input:focus,
  select:focus {
    border-color: var(--color-border);
    background: var(--color-surface);
  }
  .name {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .unfold {
    display: inline-flex;
    background: none;
    border: 0;
    padding: 0;
    color: var(--color-text-muted);
    cursor: pointer;
  }
  .unfold.open :global(svg) {
    transform: rotate(90deg);
  }
  .data {
    font-family: var(--font-mono);
  }
  .value {
    min-width: 80px;
  }
  .pick {
    background: none;
    border: 1px solid transparent;
    border-radius: 3px;
    padding: 0 4px;
    color: var(--color-text);
    cursor: pointer;
  }
  .pick:hover {
    border-color: var(--color-border);
  }
  .pick.true-value {
    color: var(--color-success);
  }
  .pick.forced {
    outline: 1px solid var(--color-warning);
  }
  .icon,
  .add {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: 0;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 2px 4px;
  }
  .add {
    margin: 3px 4px;
  }
</style>
