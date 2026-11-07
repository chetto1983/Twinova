<script lang="ts">
  import { m } from "../paraglide/messages.js";
  import { formatTime, isBehind, type Snapshot } from "./snapshot";

  let { snapshot, onOutletBlocked }: { snapshot: Snapshot | null; onOutletBlocked: (blocked: boolean) => void } =
    $props();

  const bits = $derived([
    { label: m.io_sensor(), on: snapshot?.sensor ?? false, kind: "on" },
    { label: m.io_motor(), on: snapshot?.motor ?? false, kind: "on" },
    { label: m.io_jam(), on: snapshot?.jam ?? false, kind: "err" },
  ]);
</script>

<section class="panel io">
  <header class="panel-head"><h2 class="panel-title">{m.io_title()}</h2></header>
  <div class="panel-body">
    <ul class="bits">
      {#each bits as bit (bit.label)}
        <li class="row">
          <span class="led" class:on={bit.on && bit.kind === "on"} class:err={bit.on && bit.kind === "err"}></span>
          <span class="data">{bit.label}</span>
          <span class="spacer"></span>
          <span class="data">{bit.on ? "TRUE" : "FALSE"}</span>
        </li>
      {/each}
    </ul>
    <dl>
      <dt class="muted">{m.io_time()}</dt>
      <dd class="data">{formatTime(snapshot?.time_ms ?? 0)}</dd>
      <dt class="muted">{m.io_delivered()}</dt>
      <dd class="data">{snapshot?.delivered ?? 0}</dd>
      {#if snapshot && isBehind(snapshot.lag_ms)}
        <dt class="warn">{m.io_lag()}</dt>
        <dd class="data warn">{snapshot.lag_ms} ms</dd>
      {/if}
    </dl>
    <h3 class="sub">{m.fault_title()}</h3>
    <label class="row">
      <input
        type="checkbox"
        checked={snapshot?.outlet_blocked ?? false}
        disabled={!snapshot}
        onchange={(e) => onOutletBlocked(e.currentTarget.checked)}
      />
      {m.fault_outlet()}
    </label>
  </div>
</section>

<style>
  .bits {
    display: grid;
    gap: 10px;
    margin: 0 0 16px;
    padding: 0;
    list-style: none;
  }
  dl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px 12px;
    margin: 0 0 16px;
  }
  dd {
    margin: 0;
    text-align: right;
  }
  .warn {
    color: var(--color-warning);
  }
  .sub {
    margin: 0 0 8px;
    font-size: 13px;
    color: var(--color-text-muted);
  }
</style>
