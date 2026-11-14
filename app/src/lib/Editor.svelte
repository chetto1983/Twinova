<script lang="ts">
  import {
    Cpu,
    ChevronDown,
    FileCode2,
    Braces,
    Clock3,
    Folder,
    Activity,
    Table2,
    Code,
    Workflow,
  } from "@lucide/svelte";
  import { onMount, untrack } from "svelte";
  import { m } from "../paraglide/messages.js";
  import { edited, type Source } from "./sources";
  import { debounce, fileOf, type Problems } from "./analysis";
  import { declarations as declared, monitor, onMonitor } from "./backend";
  import { replaced, toggle, trimmed, type Breakpoint, type Hold, type Log } from "./debug";
  import { Breakpoints } from "./breakpoints.svelte";
  import {
    deviceOf,
    split,
    toggled,
    type Change,
    type Declaration,
    type Force,
    type Target,
    type Watched,
  } from "./monitor";
  import StEditor from "./StEditor.svelte";
  import VariablesTable from "./VariablesTable.svelte";
  import DebugBar from "./DebugBar.svelte";
  import BreakpointBar from "./BreakpointBar.svelte";
  import LogList from "./LogList.svelte";
  import LadderEditor from "./LadderEditor.svelte";
  import ValueBar from "./ValueBar.svelte";
  import { applyChanged } from "./navigation";
  let {
    sources = $bindable(),
    status,
    problems,
    reveal,
    selected = null,
    instances = [],
  }: {
    sources: Source[];
    status: "none" | "ok" | "failed";
    problems: Problems;
    reveal: { key: string; line: number; column: number } | null;
    /** The instance selected in the scene: a block's tab monitors it. */
    selected?: string | null;
    instances?: { id: string; definition: string }[];
  } = $props();
  let active = $state(0);
  // A Problems row opens its tab.
  $effect(() => {
    const at = reveal ? sources.findIndex((s) => s.key === reveal.key) : -1;
    if (at >= 0) active = at;
  });
  const current = $derived(sources[Math.min(active, sources.length - 1)]);
  // A Ladder file is drawn (spec 2026-11-13-ladder-editor-design.md, decision 4), or shown as its text.
  let drawing = $state(true);
  const ladderFile = $derived(!!current?.key.endsWith(".ld"));
  const drawn = $derived(drawing && ladderFile);
  // the button names the view it switches to
  const viewTitle = $derived(!ladderFile ? m.ui_ladder_unavailable() : drawn ? m.ladder_text() : m.ladder_drawing());

  // Online monitoring and the variables table (spec 2026-11-13-online-monitoring-design.md).
  let monitoring = $state(false);
  let showTable = $state(true);
  let watched = $state<Watched | null>(null);
  let values = $state<(string | null)[]>([]);
  let forces = $state<Force[]>([]);
  let unfolded = $state<string[]>([]);
  let visible = $state({ first: 1, last: 60 });
  let rows = $state<Declaration[]>([]);
  let picked = $state<{ target: Target; text: string } | null>(null);
  let problem = $state("");
  const forced = $derived(new Set(forces.map((f) => f.label)));
  const device = $derived(current ? deviceOf(current, selected, instances) : null);
  const blocked = $derived(
    !monitoring || !current
      ? ""
      : edited(current)
        ? m.monitor_edited()
        : current.kind === "behaviour" && !device
          ? m.monitor_device()
          : "",
  );
  const live = $derived(monitoring && status === "ok" && !!current && !blocked);
  const parts = $derived(watched ? split(watched, values) : { spots: [], rows: [] });

  // Breakpoints and HOLD (specs 2026-11-13-breakpoints-hold-design.md and -breakpoints-complete-design.md), in the
  // program's files and the blocks'.
  const breakpoints = new Breakpoints(
    (file) => sources.some((s) => fileOf(s) === file && edited(s)),
    (error) => (problem = error),
  );
  let hold = $state<Hold | null>(null);
  let logs = $state<Log[]>([]);
  let cursor = $state(1);
  /** The breakpoint the bar edits. */
  let editing = $state<Breakpoint | null>(null);
  const debugFile = $derived(current ? fileOf(current) : null);
  // A Build compiles new statements: the breakpoints are set again on them, where their lines resolve now.
  $effect(() => {
    void sources.map((s) => s.base);
    if (status === "ok") untrack(() => breakpoints.resend());
  });
  // A stop shows its file.
  $effect(() => {
    const at = hold ? sources.findIndex((s) => fileOf(s) === hold?.file) : -1;
    if (at >= 0) untrack(() => (active = at));
  });

  const readRows = debounce((file: string, text: string) => {
    declared
      .of(file, text)
      .then((found) => (rows = found))
      .catch(() => (rows = []));
  }, 200);
  $effect(() => {
    if (current) readRows.call(fileOf(current), current.text);
  });

  const rewatch = debounce((file: string, first: number, last: number, dev: string | null, open: string[]) => {
    monitor
      .watchRange(file, first, last, dev, open)
      .then(async (found) => {
        watched = found;
        values = await monitor.watched();
      })
      .catch((e) => (problem = String(e)));
  }, 150);
  /** Whether a breakpoint holds the line: while held the engine cannot read a new table, so letting go watches again. */
  const held = $derived(hold !== null);
  // A new Build, a scroll, a tab, a selection, an unfolded instance or the end of a HOLD watches again; the engine
  // forgets its watches at each Build, so the built text (`base`) is a dependency too.
  $effect(() => {
    if (!live || !current) {
      rewatch.cancel();
      if (untrack(() => watched)) {
        watched = null;
        void monitor.unwatch().catch(() => {});
      }
      return;
    }
    void current.base;
    void held;
    rewatch.call(fileOf(current), visible.first, visible.last, device, [...unfolded]);
  });

  onMount(() => {
    const unlisten = onMonitor((next) => {
      if (watched) values = next;
    });
    // A paused or stepped line sends no event; the forces change from the Watch pane too.
    const timer = setInterval(() => {
      if (!watched) return;
      monitor
        .watched()
        .then((next) => (values = next))
        .catch(() => {});
      monitor
        .forces()
        .then((next) => (forces = next))
        .catch(() => {});
    }, 500);
    return () => {
      void unlisten.then((stop) => stop());
      clearInterval(timer);
      // The pane moving to its own window unmounts this editor: its watches go with it.
      if (watched) void monitor.unwatch().catch(() => {});
    };
  });

  async function change(edit: Change): Promise<void> {
    if (!current) return;
    try {
      current.text = await declared.edit(fileOf(current), current.text, edit);
      problem = "";
    } catch (e) {
      problem = String(e);
    }
  }

  async function rename(index: number, name: string): Promise<void> {
    if (!current) return;
    try {
      const files = sources.map((s): [string, string] => [fileOf(s), s.text]);
      applyChanged(sources, await declared.rename(files, fileOf(current), index, name));
      problem = "";
    } catch (e) {
      problem = String(e);
    }
  }

  const pick = (target: Target, value: string | null): void => {
    picked = { target, text: toggled(value) ?? value ?? "" };
  };

  const task = $derived(
    sources
      .filter((s) => s.kind !== "behaviour")
      .map((s) => s.text)
      .join("\n")
      .match(/\bTASK\s+(\w+)\s*\([^)]*INTERVAL\s*:=\s*(T#[\w.]+)/i),
  );
</script>

<section class="editor">
  <aside class="organizer">
    <h3>{m.ui_organizer()}</h3>
    <div class="tree-line">
      <ChevronDown size={12} /><Cpu size={14} /><strong>PLC</strong><span
        class="led"
        class:on={status === "ok"}
        class:err={status === "failed"}
      ></span>
    </div>
    <div class="tree-line task">
      <Clock3 size={13} /><span>{task?.[1] ?? "MainTask"}</span><small>{task?.[2].replace(/^T#/i, "") ?? "10 ms"}</small
      >
    </div>
    {#each ["program", "behaviour"] as kind (kind)}
      <details open>
        <summary><Folder size={12} />{kind === "program" ? m.ui_programs() : m.ui_function_blocks()}</summary>
        {#each sources as source, i (source.key)}
          {#if (source.kind === "behaviour") === (kind === "behaviour")}
            <button class="source" class:active={source === current} title={source.label} onclick={() => (active = i)}>
              {#if kind === "behaviour"}<Braces size={12} />{:else}<FileCode2 size={12} />{/if}<span
                >{source.label}</span
              >{#if edited(source)}<b>•</b>{/if}
            </button>
          {/if}
        {/each}
      </details>
    {/each}
  </aside>
  <div class="code-pane">
    <header class="code-head">
      <div class="tabs" role="tablist" aria-label={m.editor_title()}>
        {#each sources as source, i (source.key)}
          <button
            role="tab"
            class="tab"
            class:edited={edited(source)}
            title={edited(source) ? `${source.label} · ${m.editor_edited()}` : source.label}
            aria-selected={source === current}
            onclick={() => (active = i)}><FileCode2 size={12} />{source.label}</button
          >
        {/each}
      </div>
      <button
        class="toggle"
        class:on={showTable}
        title={m.vars_title()}
        aria-label={m.vars_title()}
        aria-pressed={showTable}
        onclick={() => (showTable = !showTable)}><Table2 size={13} /></button
      ><button
        class="toggle"
        class:on={monitoring}
        title={m.monitor_toggle()}
        aria-label={m.monitor_toggle()}
        aria-pressed={monitoring}
        disabled={status !== "ok"}
        onclick={() => (monitoring = !monitoring)}><Activity size={13} /></button
      ><DebugBar
        file={debugFile}
        {cursor}
        armed={breakpoints.armed}
        logging={breakpoints.logging}
        ready={status === "ok"}
        bind:hold
        bind:logs
        onError={(text) => (problem = text)}
      /><span class="language">{ladderFile ? "LD" : "ST"}</span><button
        class="ladder"
        class:on={drawn}
        aria-pressed={drawn}
        disabled={!ladderFile}
        aria-label={viewTitle}
        title={viewTitle}
        onclick={() => (drawing = !drawing)}
        >{#if drawn}<Code size={12} />{:else}<Workflow size={12} />{/if}</button
      >
    </header>
    {#if blocked || problem}<p class="note" role="status">{problem || blocked}</p>{/if}
    {#if editing && debugFile}
      {@const file = debugFile}
      <BreakpointBar
        breakpoint={editing}
        onApply={(b) => {
          breakpoints.set(file, replaced(breakpoints.of(file), trimmed(b)));
          editing = null;
        }}
        onRemove={() => {
          breakpoints.set(file, toggle(breakpoints.of(file), editing?.line ?? 0));
          editing = null;
        }}
        onClose={() => (editing = null)}
      />
    {/if}
    <ValueBar bind:picked {hold} {forced} onForces={(list) => (forces = list)} onProblem={(text) => (problem = text)} />
    {#if current}
      {#if showTable}
        <div class="table-area">
          <VariablesTable
            declarations={rows}
            rows={watched?.rows ?? null}
            values={parts.rows}
            {forced}
            {unfolded}
            onChange={change}
            onRename={rename}
            onUnfold={(name) =>
              (unfolded = unfolded.includes(name) ? unfolded.filter((n) => n !== name) : [...unfolded, name])}
            onPick={pick}
          />
        </div>
      {/if}
      <div class="code-area" aria-label={current.label}>
        <StEditor
          bind:sources
          bind:active
          {problems}
          {reveal}
          spots={watched?.spots ?? []}
          spotValues={parts.spots}
          {forced}
          onVisible={(first, last) => (visible = { first, last })}
          onSpot={(i) => {
            const target = watched?.spots[i]?.target;
            if (target) pick(target, parts.spots[i] ?? null);
          }}
          onToggleMonitor={() => (monitoring = status === "ok" && !monitoring)}
          breakpoints={breakpoints.of(debugFile)}
          heldLine={hold && hold.file === debugFile ? hold.line : null}
          onBreakpoint={(line) => {
            if (debugFile) breakpoints.set(debugFile, toggle(breakpoints.of(debugFile), line));
          }}
          onEditBreakpoint={(line) => (editing = breakpoints.of(debugFile).find((b) => b.line === line) ?? { line })}
          onBreakpointsMoved={(list) => {
            if (debugFile) breakpoints.move(debugFile, list);
          }}
          onCursor={(line) => (cursor = line)}
        />
        {#if drawn}<div class="ladder-area">
            <LadderEditor
              source={current}
              {sources}
              onText={(text) => {
                if (current) current.text = text;
              }}
            />
          </div>{/if}
      </div>
    {/if}
    <LogList bind:logs />
    <footer>
      <span class="led" class:on={status === "ok"} class:err={status === "failed"}></span><span
        >{status === "ok" ? m.built_ok() : status === "failed" ? m.build_failed() : m.not_built()}</span
      ><span class="spacer"></span>{#if current && edited(current)}<button
          class="revert"
          onclick={() => (current.text = current.base)}>{m.editor_revert()}</button
        >{/if}<span title={m.editor_hint()}>{drawn ? "Ladder Diagram" : "Structured Text"} · UTF-8</span>
    </footer>
  </div>
</section>

<style>
  .editor {
    display: grid;
    grid-template-columns: 224px minmax(0, 1fr);
    height: 100%;
    min-height: 0;
  }
  .organizer {
    overflow: auto;
    border-right: 1px solid var(--color-border);
    padding: 10px 8px;
  }
  h3 {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--color-text-faint);
    margin: 0 0 10px 6px;
  }
  .tree-line {
    display: flex;
    gap: 7px;
    align-items: center;
    font-size: 11px;
    padding: 4px 5px;
  }
  .tree-line .led {
    width: 5px;
    height: 5px;
    margin-left: auto;
  }
  .task {
    padding-left: 25px;
    color: var(--color-text-muted);
  }
  small {
    margin-left: auto;
    font-size: 9px;
    color: var(--color-text-faint);
  }
  summary {
    font-size: 10px;
    padding: 6px 5px 5px 18px;
    color: var(--color-text-faint);
  }
  summary :global(svg) {
    vertical-align: middle;
    margin-right: 5px;
  }
  .source {
    width: 100%;
    display: flex;
    gap: 6px;
    align-items: center;
    border: 0;
    background: transparent;
    padding: 5px 8px 5px 32px;
    font-size: 10px;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
  }
  .source span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .source :global(svg) {
    flex: none;
  }
  .source.active {
    background: var(--color-accent-soft);
    color: var(--color-ring);
  }
  .code-pane {
    position: relative;
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
  }
  .code-head {
    display: flex;
    height: 32px;
    flex: none;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    padding-right: 8px;
  }
  .tabs {
    display: flex;
    flex: 1;
    overflow: auto;
    height: 100%;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    border: 0;
    border-right: 1px solid var(--color-border);
    background: transparent;
    padding: 0 12px;
    font-size: 10px;
    white-space: nowrap;
    cursor: pointer;
    color: var(--color-text-faint);
  }
  .tab[aria-selected="true"] {
    background: var(--color-bg);
    color: var(--color-text);
    border-top: 2px solid var(--color-ring);
  }
  .tab.edited::after {
    content: " •";
    color: var(--color-ring);
  }
  .language,
  .ladder {
    font-size: 9px;
    padding: 3px 6px;
  }
  .language {
    background: var(--color-accent-soft);
    color: var(--color-ring);
    border-radius: 3px;
  }
  .ladder {
    display: inline-flex;
    border: 0;
    background: transparent;
    color: var(--color-text-muted);
  }
  .ladder:disabled {
    color: var(--color-text-disabled);
  }
  .ladder.on {
    color: var(--color-ring);
  }
  /* over Monaco, which stays mounted with its other tabs */
  .ladder-area {
    position: absolute;
    inset: 0;
    z-index: 2;
  }
  .code-area {
    display: flex;
    flex: 1;
    min-height: 0;
    position: relative;
    overflow: hidden;
    background: var(--color-bg);
  }
  .table-area {
    flex: none;
    height: 35%;
    min-height: 48px;
    max-height: 70%;
    overflow: hidden;
    resize: vertical;
  }
  .toggle {
    display: inline-flex;
    border: 0;
    background: transparent;
    padding: 4px;
    margin-right: 4px;
    border-radius: 3px;
    color: var(--color-text-faint);
    cursor: pointer;
  }
  .toggle.on {
    background: var(--color-accent-soft);
    color: var(--color-ring);
  }
  .toggle:disabled {
    color: var(--color-text-disabled);
    cursor: default;
  }
  .note {
    margin: 0;
    padding: 3px 10px;
    font-size: 10px;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
  }
  footer {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-size: 9px;
    color: var(--color-text-faint);
    border-top: 1px solid var(--color-border);
  }
  footer .led {
    width: 5px;
    height: 5px;
  }
  .revert {
    background: transparent;
    border: 0;
    font-size: 9px;
    color: var(--color-ring);
    cursor: pointer;
  }
  @media (max-width: 1200px) {
    .editor {
      grid-template-columns: 190px minmax(0, 1fr);
    }
  }
</style>
