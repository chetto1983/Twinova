<script lang="ts">
  import { ArrowLeftRight, ChevronUp, ChevronDown } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";

  let {
    value,
    label,
    id,
    unit = "",
    axis = "",
    min,
    max,
    step = 0.01,
    onchange,
    onclear,
    disabled = false,
    integer = false,
  }: {
    value: number;
    label: string;
    id?: string;
    unit?: string;
    axis?: string;
    min?: number;
    max?: number;
    step?: number;
    onchange: (value: number) => void;
    onclear?: () => void;
    disabled?: boolean;
    integer?: boolean;
  } = $props();

  let input: HTMLInputElement;
  let draft = $state<number | null>(null);
  let drag = $state<{ x: number; value: number } | null>(null);
  const shown = $derived(draft ?? value);

  function bounded(next: number): number {
    const rounded = integer ? Math.round(next) : Number(next.toFixed(8));
    return Math.min(max ?? Infinity, Math.max(min ?? -Infinity, rounded));
  }
  function commit(): void {
    const text = input.value.trim();
    const next = Number(text.replace(",", "."));
    if (!text) onclear?.();
    else if (
      Number.isFinite(next) &&
      (!integer || Number.isInteger(next)) &&
      (min === undefined || next >= min) &&
      (max === undefined || next <= max)
    ) {
      if (next !== value) onchange(next);
    }
    input.value = String(value);
  }
  function increment(direction: number, fine = false, base = value): void {
    const next = bounded(base + direction * step * (fine ? 0.1 : 1));
    if (next !== value) onchange(next);
  }
  function key(event: KeyboardEvent): void {
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      const typed = Number(input.value.replace(",", "."));
      increment(event.key === "ArrowUp" ? 1 : -1, event.shiftKey, Number.isFinite(typed) ? typed : value);
    } else if (event.key === "Enter") input.blur();
    else if (event.key === "Escape") {
      draft = null;
      drag = null;
      input.value = String(value);
      input.blur();
    }
  }
  function start(event: PointerEvent): void {
    if (event.button !== 0 || disabled) return;
    event.preventDefault();
    const handle = event.currentTarget as HTMLButtonElement;
    handle.focus();
    handle.setPointerCapture(event.pointerId);
    drag = { x: event.clientX, value };
  }
  function scrub(event: PointerEvent): void {
    if (!drag) return;
    draft =
      Math.abs(event.clientX - drag.x) < 3
        ? null
        : bounded(drag.value + (event.clientX - drag.x) * step * (event.shiftKey ? 0.1 : 1));
  }
  function finish(): void {
    const next = draft;
    drag = null;
    draft = null;
    if (next !== null && next !== value) onchange(next);
    else {
      input.focus();
      input.select();
    }
  }
</script>

<div class="number" class:disabled class:dragging={drag !== null}>
  <button
    class="scrub"
    class:x={axis === "X"}
    class:y={axis === "Y"}
    class:z={axis === "Z"}
    aria-label={m.ui_adjust_value({ name: label })}
    title={m.ui_numeric_hint()}
    {disabled}
    onpointerdown={start}
    onpointermove={scrub}
    onpointerup={finish}
    onpointercancel={() => {
      drag = null;
      draft = null;
    }}
    onkeydown={key}
    >{#if axis}{axis}{:else}<ArrowLeftRight size={12} />{/if}</button
  >
  <input
    bind:this={input}
    {id}
    class="value data"
    aria-label={label}
    role="spinbutton"
    aria-valuenow={shown}
    aria-valuemin={min}
    aria-valuemax={max}
    value={shown}
    {disabled}
    inputmode="decimal"
    onchange={commit}
    onkeydown={key}
  />
  {#if unit}<span class="unit">{unit}</span>{/if}
  {#if !axis}
    <div class="steps">
      <button aria-label={m.ui_increase_value({ name: label })} {disabled} onclick={() => increment(1)}
        ><ChevronUp size={11} /></button
      >
      <button aria-label={m.ui_decrease_value({ name: label })} {disabled} onclick={() => increment(-1)}
        ><ChevronDown size={11} /></button
      >
    </div>
  {/if}
</div>

<style>
  .number {
    display: flex;
    align-items: center;
    min-width: 0;
    width: 100%;
    height: 30px;
    border: 1px solid transparent;
    border-radius: 5px;
    background: var(--color-surface-2);
    transition:
      border-color 120ms,
      background 120ms;
  }
  .number:hover {
    border-color: var(--color-border);
  }
  .number:focus-within,
  .number.dragging {
    border-color: var(--color-ring);
    box-shadow: 0 0 0 1px var(--color-ring);
  }
  button {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: none;
    border: 0;
    padding: 0;
    background: transparent;
    color: var(--color-text-faint);
  }
  .scrub {
    width: 24px;
    height: 28px;
    cursor: ew-resize;
    touch-action: none;
    user-select: none;
    font-size: 10px;
    font-weight: 600;
    border-radius: 4px 0 0 4px;
  }
  .scrub:hover {
    color: var(--color-text);
    background: var(--color-accent-soft);
  }
  .scrub.x {
    color: var(--color-danger);
  }
  .scrub.y {
    color: var(--color-success);
  }
  .scrub.z {
    color: var(--color-ring);
  }
  .value {
    flex: 1;
    width: 0;
    min-width: 0;
    height: 28px;
    padding: 0 4px 0 0;
    border: 0;
    outline: none;
    box-shadow: none;
    background: transparent;
    font-size: 12px;
    font-weight: 400;
    font-variant-numeric: tabular-nums;
  }
  .unit {
    flex: none;
    padding: 0 6px 0 3px;
    color: var(--color-text-faint);
    font-size: 10px;
    white-space: nowrap;
    user-select: none;
  }
  .steps {
    display: grid;
    flex: none;
    width: 16px;
    opacity: 0;
  }
  .steps button {
    height: 13px;
    width: 16px;
    cursor: pointer;
  }
  .steps button:hover {
    color: var(--color-text);
    background: var(--color-accent-soft);
  }
  .number:hover .steps,
  .number:focus-within .steps {
    opacity: 1;
  }
  .disabled {
    opacity: 0.5;
  }
  @media (hover: none) {
    .steps {
      opacity: 1;
    }
  }
</style>
