<script lang="ts">
  // The PLC pane in a window of its own (spec 2026-11-13-plc-window-design.md): it shows what the main window sends,
  // and sends back each edit and its Build, Run and Step requests.
  import { onMount } from "svelte";
  import { PanelBottom } from "@lucide/svelte";
  import { m } from "./paraglide/messages.js";
  import { readPreference } from "./lib/preferences";
  import { Outbox, plcWindow, type PlcState, type Request } from "./lib/plcWindow";
  import Editor from "./lib/Editor.svelte";
  import Transport from "./lib/Transport.svelte";

  let shown = $state<PlcState | null>(null);
  const outbox = new Outbox();
  /** The texts as last sent or taken: an edit is a change from them. */
  let last = "";
  document.documentElement.dataset.theme = readPreference("theme") === "light" ? "light" : "dark";

  function receive(state: PlcState): void {
    const current = shown && JSON.stringify($state.snapshot(shown.sources));
    const keep = shown && (!outbox.takes(state) || JSON.stringify(state.sources) === current);
    shown = keep && shown ? { ...state, sources: shown.sources } : state;
    last = JSON.stringify($state.snapshot(shown.sources));
  }
  $effect(() => {
    if (!shown) return;
    const sources = $state.snapshot(shown.sources);
    const json = JSON.stringify(sources);
    if (json === last) return;
    last = json;
    void plcWindow.edit(outbox.next(sources));
  });
  function ask(request: Request): void {
    if (shown) void plcWindow.ask({ ...outbox.next($state.snapshot(shown.sources)), request });
  }

  onMount(() => {
    const unlisten = plcWindow.onState(receive);
    void unlisten.then(() => plcWindow.ready());
    return () => void unlisten.then((stop) => stop());
  });
</script>

<div class="plc-window">
  <header class="bar">
    <div class="transport" aria-label={m.ui_simulation()}>
      <Transport
        canBuild={shown !== null}
        running={shown?.running ?? false}
        ready={shown?.ready ?? false}
        onBuild={() => ask("build")}
        onRun={() => ask("run")}
        onStep={() => ask("step")}
      />
    </div>
    <span class="spacer"></span>
    <button class="btn small" onclick={() => void plcWindow.close()}
      ><PanelBottom size={14} />{m.plc_window_attach()}</button
    >
  </header>
  {#if shown}<Editor
      bind:sources={shown.sources}
      status={shown.status}
      problems={shown.problems}
      reveal={null}
      selected={shown.selected}
      instances={shown.instances}
    />{/if}
</div>

<style>
  .plc-window {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    flex: none;
    padding: 0 10px;
    border-bottom: 1px solid var(--color-border);
  }
  .transport {
    display: flex;
    align-items: center;
    gap: 5px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 2px 5px;
    background: var(--color-bg);
  }
  .spacer {
    flex: 1;
  }
</style>
