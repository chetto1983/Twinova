<script lang="ts">
  // The status bar, out of App.svelte: run state, virtual time, notices, errors and build; HOLD at a breakpoint (spec
  // 2026-11-13-breakpoints-hold-design.md, decision 2), asked of the debugger, since a held scan sends no snapshot.
  import { onMount } from "svelte";
  import { CircleCheck, AlertCircle, CirclePause } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { debug } from "./backend";
  import { holdLabel, type Hold } from "./debug";
  import { formatTime, type Snapshot } from "./snapshot";

  let {
    running,
    status,
    snapshot,
    notice,
    error,
    changed,
    objects,
  }: {
    running: boolean;
    status: "none" | "ok" | "failed";
    snapshot: Snapshot | null;
    notice: string;
    error: string;
    changed: boolean;
    objects: number;
  } = $props();

  let hold = $state<Hold | null>(null);
  onMount(() => {
    const timer = setInterval(() => {
      debug
        .hold()
        .then((h) => (hold = h))
        .catch(() => (hold = null));
    }, 300);
    return () => clearInterval(timer);
  });
</script>

<footer class="statusbar">
  {#if hold}<span class="status hold"><CirclePause size={13} />{m.debug_hold({ at: holdLabel(hold) })}</span
    >{:else}<span class="status"
      ><span class="led" class:on={running}></span>{running
        ? m.ui_running()
        : status === "ok"
          ? m.ui_ready()
          : m.ui_stopped()}</span
    >{/if}
  <span class="divider"></span><span class="data">{formatTime(snapshot?.time_ms ?? 0)}</span>
  <p class="status-notice" role="status" title={notice}>{notice}</p>
  {#if error}<AlertCircle size={13} /><span class="error-status" title={error}>{error}</span>{/if}
  {#if changed}<span class="scene-status">{m.scene_changed()}</span>{/if}
  {#if snapshot}<CircleCheck size={13} />{/if}<span class="build-status"
    >{status === "ok"
      ? m.built_ok()
      : snapshot
        ? m.ui_previous_build()
        : status === "failed"
          ? m.build_failed()
          : m.not_built()}</span
  >
  <span class="divider"></span><span>{objects} {m.ui_objects()}</span><span class="divider"></span><span
    >truST · IEC 61131-3</span
  >
</footer>

<style>
  .statusbar {
    min-height: 27px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
    color: var(--color-text-faint);
    font-size: 10px;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--color-text-muted);
  }
  .divider {
    width: 1px;
    height: 12px;
    background: var(--color-border);
    margin: 0 4px;
  }
  .error-status {
    max-width: 40vw;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    color: var(--color-danger);
  }
  .build-status {
    flex: none;
    white-space: nowrap;
    color: var(--color-text-muted);
  }
  .scene-status {
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .status-notice {
    flex: 1;
    min-width: 0;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hold {
    color: var(--color-warning);
    font-weight: 600;
  }
</style>
