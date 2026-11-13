<script lang="ts">
  // Continue, the steps and Run to cursor, their keys (VS Code's, decision 5) and the call stack, in the PLC pane's bar
  // and the PLC window's (spec 2026-11-13-breakpoints-hold-design.md). A held scan sends no event, so while breakpoints
  // are set the bar asks where the program is held (plan, decision 5).
  import { onMount } from "svelte";
  import { Play, RedoDot, ArrowDownToLine, CornerUpLeft, TextCursor } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { debug } from "./backend";
  import { holdLabel, keyAction, type Hold, type Resume } from "./debug";

  let {
    file,
    cursor,
    armed,
    ready,
    hold = $bindable(null),
    onError,
  }: {
    /** The shown tab's file when it is the program's: only the program has breakpoints (decision 3). */
    file: string | null;
    cursor: number;
    /** Some breakpoint is set. */
    armed: boolean;
    /** The program is built. */
    ready: boolean;
    hold?: Hold | null;
    onError: (text: string) => void;
  } = $props();

  const fail = (e: unknown): void => onError(String(e));
  const resume = (how: Resume): void => void debug.resume(how).catch(fail);
  const toCursor = (): void => {
    if (file) void debug.runToCursor(file, cursor).catch(fail);
  };
  const steps: { how: Resume; label: () => string; icon: typeof Play }[] = [
    { how: "continue", label: m.debug_continue, icon: Play },
    { how: "over", label: m.debug_over, icon: RedoDot },
    { how: "into", label: m.debug_into, icon: ArrowDownToLine },
    { how: "out", label: m.debug_out, icon: CornerUpLeft },
  ];

  function onKey(e: KeyboardEvent): void {
    const action = keyAction(e);
    if (!action || action === "toggle") return;
    if (action === "cursor") {
      if (!file || !ready) return;
      toCursor();
    } else if (hold) resume(action);
    else return;
    e.preventDefault();
  }

  onMount(() => {
    const timer = setInterval(() => {
      if (!armed && !hold) return;
      debug
        .hold()
        .then((h) => (hold = h))
        .catch(() => (hold = null));
    }, 200);
    return () => clearInterval(timer);
  });
</script>

<svelte:window onkeydown={onKey} />

<div class="debug-bar" role="toolbar" aria-label={m.debug_title()}>
  {#each steps as step (step.how)}<button
      class="icon-button"
      title={step.label()}
      aria-label={step.label()}
      disabled={!hold}
      onclick={() => resume(step.how)}><step.icon size={13} /></button
    >{/each}<button
    class="icon-button"
    title={m.debug_cursor()}
    aria-label={m.debug_cursor()}
    disabled={!file || !ready}
    onclick={toCursor}><TextCursor size={13} /></button
  >
</div>
{#if hold}
  <ol class="stack" aria-label={m.debug_stack()}>
    {#each hold.frames as [frameFile, frameLine], i (i)}<li>
        {holdLabel({ file: frameFile, line: frameLine, frames: [] })}
      </li>{/each}
  </ol>
{/if}

<style>
  .debug-bar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 6px;
    border-left: 1px solid var(--color-border);
  }
  .debug-bar button {
    width: 24px;
    height: 24px;
  }
  .stack {
    position: absolute;
    right: 8px;
    bottom: 30px;
    z-index: 2;
    margin: 0;
    padding: 4px 8px 4px 22px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--color-text-muted);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }
</style>
