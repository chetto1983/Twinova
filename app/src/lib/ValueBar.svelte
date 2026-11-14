<script lang="ts">
  // The value prepared for a variable picked in the code or the table (spec 2026-11-13-online-monitoring-design.md):
  // written once, forced, or released, through the monitor. While a breakpoint holds the line only a write goes.
  import { X } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { monitor } from "./backend";
  import { labelOf, type Force, type Target } from "./monitor";
  import type { Hold } from "./debug";

  let {
    picked = $bindable(),
    hold,
    forced,
    onForces,
    onProblem,
  }: {
    picked: { target: Target; text: string } | null;
    hold: Hold | null;
    forced: Set<string>;
    onForces: (forces: Force[]) => void;
    onProblem: (text: string) => void;
  } = $props();

  async function apply(action: "write" | "force" | "release"): Promise<void> {
    if (!picked) return;
    try {
      if (action === "write") await monitor.writeValue(picked.target, picked.text);
      else
        onForces(
          await (action === "force" ? monitor.force(picked.target, picked.text) : monitor.release(picked.target)),
        );
      onProblem("");
    } catch (e) {
      onProblem(String(e));
    }
  }
</script>

{#if picked}
  <div class="prepare" role="group" aria-label={m.value_prepared()}>
    <strong class="data">{labelOf(picked.target)}</strong>
    <input aria-label={m.value_prepared()} bind:value={picked.text} />
    <button onclick={() => apply("write")} disabled={picked.target.kind === "address"}>{m.value_write()}</button>
    <button onclick={() => apply("force")} disabled={!!hold}>{m.value_force()}</button>
    <button onclick={() => apply("release")} disabled={!!hold || !forced.has(labelOf(picked.target))}
      >{m.value_release()}</button
    >{#if hold}<span class="held-note">{m.value_held()}</span>{/if}
    <button class="close" aria-label="✕" onclick={() => (picked = null)}><X size={12} /></button>
  </div>
{/if}

<style>
  .held-note {
    color: var(--color-warning);
    font-size: 10px;
  }
  .prepare {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-size: 10px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-2);
  }
  .prepare input {
    width: 120px;
    font-family: var(--font-mono);
  }
  .prepare button {
    font-size: 10px;
  }
  .prepare .close {
    margin-left: auto;
    display: inline-flex;
    border: 0;
    background: transparent;
    cursor: pointer;
  }
  .data {
    font-family: var(--font-mono);
  }
</style>
