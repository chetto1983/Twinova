<script lang="ts">
  // Build, Run/Pause, Reset and Step: the toolbar's, and the PLC window's without Reset (spec
  // 2026-11-13-plc-window-design.md, decision 3).
  import { Play, Pause, Square, StepForward, Hammer } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";

  let {
    canBuild,
    running,
    ready,
    onBuild,
    onRun,
    onStep,
    onReset,
  }: {
    canBuild: boolean;
    running: boolean;
    ready: boolean;
    onBuild: () => void;
    onRun: () => void;
    onStep: () => void;
    onReset?: () => void;
  } = $props();
</script>

<button class="icon-button" title={m.build()} aria-label={m.build()} onclick={onBuild} disabled={!canBuild}
  ><Hammer size={16} /></button
>
<span class="divider"></span>
<button
  class="icon-button run"
  class:playing={running}
  title={running ? m.pause() : m.run()}
  aria-label={running ? m.pause() : m.run()}
  onclick={onRun}
  disabled={!ready}
>
  {#if running}<Pause size={17} fill="currentColor" />{:else}<Play size={17} fill="currentColor" />{/if}
</button>
{#if onReset}<button class="icon-button" title={m.reset()} aria-label={m.reset()} onclick={onReset} disabled={!ready}
    ><Square size={14} fill="currentColor" /></button
  >{/if}
<button class="icon-button" title={m.step()} aria-label={m.step()} onclick={onStep} disabled={!ready}
  ><StepForward size={17} /></button
>

<style>
  .divider {
    height: 18px;
    width: 1px;
    background: var(--color-border);
    margin: 0 5px;
    flex: none;
  }
  .run {
    color: var(--color-success);
  }
  .playing {
    background: color-mix(in oklab, var(--color-success) 13%, transparent);
  }
</style>
