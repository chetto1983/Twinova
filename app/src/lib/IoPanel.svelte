<script lang="ts">
  // The line's I/O by scene signal and panel button, with the address each is bound to and its value in the
  // controller's image (engine.rs `Snapshot.signals`); the sinks' counts and their block, M3's jam alarm when the
  // program declares one, virtual time and the lag behind the wall clock.
  import { m } from "../paraglide/messages.js";
  import { deliveredOf, formatTime, isBehind, type Snapshot } from "./snapshot";

  let {
    snapshot,
    onSinkBlocked,
  }: { snapshot: Snapshot | null; onSinkBlocked: (sink: number, blocked: boolean) => void } = $props();
</script>

<section class="panel io">
  <header class="panel-head"><h2 class="panel-title">{m.io_title()}</h2></header>
  <div class="panel-body">
    <ul class="bits">
      {#each snapshot?.signals ?? [] as signal (signal.name)}
        <li class="row">
          <span class="led" class:on={signal.value === "TRUE"}></span>
          <span class="data">{signal.name}</span>
          <span class="muted data">{signal.address}</span>
          <span class="spacer"></span>
          <span class="data">{signal.value}</span>
        </li>
      {/each}
      {#if snapshot && snapshot.jam !== null}
        <li class="row">
          <span class="led" class:err={snapshot.jam}></span>
          <span class="data">{m.io_jam()}</span>
          <span class="spacer"></span>
          <span class="data">{snapshot.jam ? "TRUE" : "FALSE"}</span>
        </li>
      {/if}
    </ul>
    <dl>
      <dt class="muted">{m.io_time()}</dt>
      <dd class="data">{formatTime(snapshot?.time_ms ?? 0)}</dd>
      <dt class="muted">{m.io_delivered()}</dt>
      <dd class="data">{deliveredOf(snapshot)}</dd>
      {#if snapshot && isBehind(snapshot.lag_ms)}
        <dt class="warn">{m.io_lag()}</dt>
        <dd class="data warn">{snapshot.lag_ms} ms</dd>
      {/if}
    </dl>
    {#if snapshot && snapshot.sinks.length > 0}
      <h3 class="sub">{m.fault_title()}</h3>
      {#each snapshot.sinks as sink, i (sink.id)}
        <label class="row">
          <input type="checkbox" checked={sink.blocked} onchange={(e) => onSinkBlocked(i, e.currentTarget.checked)} />
          {m.fault_sink_blocked({ sink: sink.id })}
        </label>
      {/each}
    {/if}
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
