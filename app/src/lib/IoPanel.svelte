<script lang="ts">
  import { Search, Timer } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { backend } from "./backend";
  import { deliveredOf, formatTime, isBehind, type Snapshot, type ScanTiming } from "./snapshot";
  let {
    snapshot,
    onSinkBlocked,
    active = true,
  }: {
    snapshot: Snapshot | null;
    onSinkBlocked: (sink: number, blocked: boolean) => void;
    active?: boolean;
  } = $props();
  let filter = $state("");
  let timing = $state<ScanTiming | null>(null);
  let timingError = $state("");
  const signals = $derived(
    (snapshot?.signals ?? []).filter((s) => `${s.name} ${s.address}`.toLowerCase().includes(filter.toLowerCase())),
  );
  const duration = (us: number | undefined): string =>
    us === undefined ? "—" : us >= 1000 ? `${(us / 1000).toFixed(2)} ms` : `${us.toFixed(1)} µs`;
  $effect(() => {
    if (!active) return;
    let alive = true;
    let pending = false;
    async function update(): Promise<void> {
      if (pending) return;
      pending = true;
      try {
        const next = await backend.scanTiming();
        if (alive) {
          timing = next;
          timingError = "";
        }
      } catch (e) {
        if (alive) {
          timing = null;
          timingError = String(e);
        }
      } finally {
        pending = false;
      }
    }
    void update();
    const timer = setInterval(() => void update(), 500);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  });
</script>

<section class="io">
  <div class="timing" title={timingError || m.ui_task()}>
    <Timer size={14} /><strong>{m.ui_task()}</strong>
    <span>{m.ui_last()} <b>{duration(timing?.scans ? timing.last_us : undefined)}</b></span>
    <span>{m.ui_average()} <b>{duration(timing?.scans ? timing.average_us : undefined)}</b></span>
    <span>{m.ui_maximum()} <b>{duration(timing?.scans ? timing.maximum_us : undefined)}</b></span>
    <span class="spacer"></span>
    <span>{timing ? `${timing.interval_ms} ms · ${timing.scans} scans` : "—"}</span>
  </div>
  <div class="io-body">
    <div class="signal-table">
      <header>
        <h2>{m.io_title()}</h2>
        <label class="search-field"
          ><Search size={12} /><input
            aria-label={m.inspector_signals()}
            placeholder={m.inspector_signals()}
            bind:value={filter}
          /></label
        >
      </header>
      <ul class="bits">
        {#each signals as signal (signal.name)}
          <li class="row">
            <span class="led" class:on={signal.value === "TRUE"}></span><span class="data">{signal.name}</span><span
              class="muted data">{signal.address}</span
            ><span class="spacer"></span><span class="data" class:true-value={signal.value === "TRUE"}
              >{signal.value}</span
            >
          </li>
        {/each}
        {#if snapshot && snapshot.jam !== null && (!filter || "jam".includes(filter.toLowerCase()))}
          <li class="row">
            <span class="led" class:err={snapshot.jam}></span><span class="data">{m.io_jam()}</span><span class="spacer"
            ></span><span class="data">{snapshot.jam ? "TRUE" : "FALSE"}</span>
          </li>
        {/if}
      </ul>
    </div>
    <aside>
      <dl>
        <dt>{m.io_time()}</dt>
        <dd class="data">{formatTime(snapshot?.time_ms ?? 0)}</dd>
        <dt>{m.io_delivered()}</dt>
        <dd class="data">{deliveredOf(snapshot)}</dd>
        {#if snapshot && isBehind(snapshot.lag_ms)}<dt class="warn">{m.io_lag()}</dt>
          <dd class="data warn">{snapshot.lag_ms} ms</dd>{/if}
      </dl>
      {#if snapshot && snapshot.sinks.length}
        <h3>{m.fault_title()}</h3>
        {#each snapshot.sinks as sink, i (sink.id)}<label class="fault"
            ><input
              type="checkbox"
              checked={sink.blocked}
              onchange={(e) => onSinkBlocked(i, e.currentTarget.checked)}
            />{m.fault_sink_blocked({ sink: sink.id })}</label
          >{/each}
      {/if}
      {#if timingError}<p class="muted timing-error" title={timingError}>{m.ui_timing_unavailable()}</p>{/if}
    </aside>
  </div>
</section>

<style>
  .io {
    min-width: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .timing {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
    padding: 7px 12px;
    font-size: 9px;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text-faint);
    background: var(--color-bg);
  }
  .timing strong {
    font-weight: 500;
    color: var(--color-text-muted);
  }
  .timing b {
    color: var(--color-text);
    font: 9px var(--font-mono);
    margin-left: 5px;
  }
  .io-body {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 190px;
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .signal-table {
    min-width: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: space-between;
    padding: 7px 12px;
  }
  h2 {
    font-size: 10px;
    font-weight: 500;
    margin: 0;
    color: var(--color-text-faint);
  }
  .search-field {
    width: 145px;
  }
  .search-field input {
    height: 22px;
    font-size: 10px;
  }
  .bits {
    padding: 0;
    margin: 0;
    list-style: none;
  }
  .bits li {
    gap: 9px;
    min-height: 27px;
    padding: 4px 12px;
    border-top: 1px solid var(--color-border);
  }
  .bits .data {
    font-size: 10px;
  }
  .bits .led {
    width: 5px;
    height: 5px;
  }
  .true-value {
    color: var(--color-success);
  }
  aside {
    border-left: 1px solid var(--color-border);
    padding: 12px;
    font-size: 10px;
  }
  dl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    margin: 0 0 16px;
    color: var(--color-text-faint);
  }
  dd {
    margin: 0;
    color: var(--color-text);
    font-size: 10px;
  }
  h3 {
    text-transform: uppercase;
    font-size: 9px;
    letter-spacing: 0.8px;
    color: var(--color-text-faint);
    margin: 10px 0;
  }
  .fault {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .fault input {
    width: 13px;
    height: 13px;
  }
  .warn {
    color: var(--color-warning);
  }
  .timing-error {
    font-size: 9px;
  }
</style>
