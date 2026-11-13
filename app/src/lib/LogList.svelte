<script lang="ts">
  // The logpoints' lines under the code (spec 2026-11-13-breakpoints-complete-design.md, decision 3), where they cover
  // no line and no breakpoint's dot.
  import { m } from "../paraglide/messages.js";
  import { holdLabel, type Log } from "./debug";

  let { logs = $bindable() }: { logs: Log[] } = $props();
</script>

{#if logs.length}
  <section class="log-area" aria-label={m.debug_log()}>
    <button class="clear" onclick={() => (logs = [])}>{m.debug_log_clear()}</button>
    <ol>
      {#each logs as log, i (i)}<li>
          <span class="where">{holdLabel({ file: log.file, line: log.line, frames: [] })}</span>{log.message}
        </li>{/each}
    </ol>
  </section>
{/if}

<style>
  .log-area {
    position: relative;
    flex: none;
    max-height: 90px;
    overflow: auto;
    border-top: 1px solid var(--color-border);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--color-text-muted);
  }
  ol {
    margin: 0;
    padding: 4px 10px;
    list-style: none;
  }
  .where {
    color: var(--color-text-faint);
    margin-right: 8px;
  }
  .clear {
    position: sticky;
    float: right;
    top: 2px;
    margin: 2px 6px;
    font-size: 10px;
  }
</style>
