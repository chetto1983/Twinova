<script lang="ts">
  // A breakpoint's condition, hit count and log message, as VS Code's Edit Breakpoint (spec
  // 2026-11-13-breakpoints-complete-design.md, decision 7): opened by a right click on its dot or Shift+F9.
  import { X } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import type { Breakpoint, Hit } from "./debug";

  let {
    breakpoint,
    onApply,
    onRemove,
    onClose,
  }: {
    breakpoint: Breakpoint;
    onApply: (b: Breakpoint) => void;
    onRemove: () => void;
    onClose: () => void;
  } = $props();

  // A copy to edit: the breakpoint changes only on Apply.
  let condition = $state("");
  let op = $state<Hit["op"]>("equal");
  let count = $state("");
  let log = $state("");
  $effect(() => {
    condition = breakpoint.condition ?? "";
    op = breakpoint.hit?.op ?? "equal";
    count = breakpoint.hit ? String(breakpoint.hit.count) : "";
    log = breakpoint.log ?? "";
  });

  function apply(): void {
    const hits = Number.parseInt(count, 10);
    onApply({ line: breakpoint.line, condition, log, hit: hits > 0 ? { op, count: hits } : undefined });
  }
</script>

<div class="breakpoint-bar" role="group" aria-label={m.bp_title({ line: String(breakpoint.line) })}>
  <strong class="data">{m.bp_title({ line: String(breakpoint.line) })}</strong>
  <input class="condition" aria-label={m.bp_condition()} placeholder={m.bp_condition()} bind:value={condition} />
  <select aria-label={m.bp_hit_op()} bind:value={op}>
    <option value="equal">=</option>
    <option value="at_least">≥</option>
    <option value="greater_than">&gt;</option>
  </select>
  <input class="count" aria-label={m.bp_hit()} placeholder={m.bp_hit()} inputmode="numeric" bind:value={count} />
  <input class="log" aria-label={m.bp_log()} placeholder={m.bp_log_hint()} bind:value={log} />
  <button onclick={apply}>{m.bp_apply()}</button>
  <button onclick={onRemove}>{m.bp_remove()}</button>
  <button class="close" aria-label="✕" onclick={onClose}><X size={12} /></button>
</div>

<style>
  .breakpoint-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-2);
    font-size: 11px;
  }
  input,
  select {
    height: 24px;
    font-size: 11px;
  }
  .condition {
    width: 160px;
  }
  .count {
    width: 50px;
  }
  .log {
    flex: 1;
    min-width: 120px;
  }
  .close {
    display: inline-flex;
    border: 0;
    background: transparent;
  }
</style>
