<script lang="ts">
  import {
    Boxes,
    Cpu,
    CircleAlert,
    Activity,
    ChevronDown,
    ChevronUp,
    CircleCheck,
    Maximize2,
    Minimize2,
    ExternalLink,
    PanelBottom,
  } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { m } from "../paraglide/messages.js";
  import { register } from "./control/registry";
  import { dockCommands } from "./control/plcCommands";
  import type { ProjectView } from "./project";
  import type { Source } from "./sources";
  import type { Snapshot } from "./snapshot";
  import { listed, tally, type Listed, type Problems } from "./analysis";
  import Catalogue from "./Catalogue.svelte";
  import Editor from "./Editor.svelte";
  import IoPanel from "./IoPanel.svelte";
  let {
    project,
    sources = $bindable(),
    status,
    error,
    warnings,
    buildHints,
    problems,
    checking = false,
    notice,
    snapshot,
    selected = null,
    detached = false,
    onDetach,
    onAttach,
    onPlace,
    onDrag,
    onSinkBlocked,
    onPress,
  }: {
    project: ProjectView;
    sources: Source[];
    status: "none" | "ok" | "failed";
    error: string;
    warnings: string[];
    /** Build's notes on the scene's signals into the PLC that the program does not read. */
    buildHints: string[];
    /** truST's analysis of the editor's texts, by file. */
    problems: Problems;
    /** The analysis of the texts shown has not answered yet. */
    checking?: boolean;
    notice: string;
    snapshot: Snapshot | null;
    /** The instance selected in the scene, for a block's tab to monitor. */
    selected?: string | null;
    /** The PLC pane is in its own window (spec 2026-11-13-plc-window-design.md): one editor at a time. */
    detached?: boolean;
    /** The PLC pane to its own window, and back; each done when its promise settles. */
    onDetach: () => Promise<unknown>;
    onAttach: () => Promise<unknown>;
    onPlace: (key: string) => void;
    onDrag: (key: string) => void;
    onSinkBlocked: (sink: number, blocked: boolean) => void;
    onPress: (address: string) => void;
  } = $props();
  let tab = $state("parts");
  let collapsed = $state(false);
  /** Laid over the whole workspace (spec 2026-11-13-maximized-dock-design.md). */
  let maximized = $state(false);
  let height = $state(242);
  let resizing = $state<{ y: number; height: number } | null>(null);
  $effect(() => {
    if (error) {
      tab = "problems";
      collapsed = false;
    }
  });
  /** Hints listed too (spec 2026-11-26-ux-consolidation-design.md, decision 7). */
  let hints = $state(false);
  const all = $derived(listed(problems, sources));
  const counted = $derived(tally(all));
  const hintCount = $derived(counted.hints + buildHints.length);
  const rows = $derived(hints ? all : all.filter((r) => r.severity !== "hint"));
  /** The badge counts errors only: the analysis' and the build's. */
  const errors = $derived(counted.errors + (error ? 1 : 0));
  const clear = $derived(!rows.length && !warnings.length && !error && !notice);
  /** The place a clicked row opens in the PLC tab. */
  let reveal = $state<{ key: string; line: number; column: number } | null>(null);
  // The control channel shows a tab, and lays the dock over the workspace, as its buttons do.
  onMount(() =>
    register(
      ...dockCommands({
        show: (panel) => {
          tab = panel;
          collapsed = false;
        },
        maximize: (on) => (maximized = on),
        plcWindow: (own) => {
          if (own && !detached) return onDetach();
          if (!own && detached) return onAttach();
          return Promise.resolve();
        },
      }),
    ),
  );
  function open(row: Listed): void {
    reveal = { key: row.key, line: row.line, column: row.column };
    tab = "plc";
  }
  const tabs = $derived([
    { id: "parts", label: m.ui_parts(), icon: Boxes },
    { id: "plc", label: "PLC", icon: Cpu },
    { id: "problems", label: m.ui_problems(), icon: CircleAlert },
    { id: "watch", label: m.ui_watch(), icon: Activity },
  ]);
  function resize(next: number): void {
    height = Math.max(150, Math.min(window.innerHeight * 0.6, next));
  }
</script>

<svelte:window
  onpointermove={(e) => {
    if (resizing) resize(resizing.height + resizing.y - e.clientY);
  }}
  onpointerup={() => (resizing = null)}
/>

<section class="dock" class:maximized style={maximized ? "" : `height:${collapsed ? 36 : height}px`}>
  <button
    class="resize"
    hidden={maximized}
    aria-label={m.ui_resize()}
    onpointerdown={(e) => {
      resizing = { y: e.clientY, height };
      e.currentTarget.focus();
      e.preventDefault();
    }}
    onkeydown={(e) => {
      if (e.key === "ArrowUp" || e.key === "ArrowDown") {
        resize(height + (e.key === "ArrowUp" ? 20 : -20));
        e.preventDefault();
      }
    }}
  ></button>
  <header class="dock-head">
    <div class="panel-tabs" role="tablist" aria-label="Workspace">
      {#each tabs as item (item.id)}
        <button
          id={`dock-${item.id}`}
          role="tab"
          aria-selected={tab === item.id}
          aria-controls={`panel-${item.id}`}
          onclick={() => {
            tab = item.id;
            collapsed = false;
          }}
        >
          <item.icon size={14} />{item.label}{#if item.id === "problems" && errors}<span class="badge">{errors}</span
            >{/if}
        </button>
      {/each}
    </div>
    <span class="spacer"></span>
    <span class="dock-hint">{tab === "parts" ? m.ui_library_hint() : tab === "plc" ? "IEC 61131-3 · truST" : ""}</span>
    {#if tab === "plc" && !detached}<button
        class="icon-button"
        aria-label={m.plc_window_open()}
        title={m.plc_window_open()}
        onclick={() => void onDetach()}><ExternalLink size={14} /></button
      >{/if}
    <button
      class="icon-button"
      aria-label={maximized ? m.ui_restore() : m.ui_maximize()}
      title={maximized ? m.ui_restore() : m.ui_maximize()}
      onclick={() => {
        maximized = !maximized;
        collapsed = false;
      }}
    >
      {#if maximized}<Minimize2 size={14} />{:else}<Maximize2 size={14} />{/if}
    </button>
    <button
      class="icon-button"
      hidden={maximized}
      aria-label={collapsed ? m.ui_expand() : m.ui_collapse()}
      title={collapsed ? m.ui_expand() : m.ui_collapse()}
      onclick={() => (collapsed = !collapsed)}
    >
      {#if collapsed}<ChevronUp size={15} />{:else}<ChevronDown size={15} />{/if}
    </button>
  </header>
  <div class="dock-content" hidden={collapsed}>
    <div class="tab-content" id="panel-parts" role="tabpanel" aria-labelledby="dock-parts" hidden={tab !== "parts"}>
      <Catalogue {onPlace} {onDrag} />
    </div>
    <div class="tab-content" id="panel-plc" role="tabpanel" aria-labelledby="dock-plc" hidden={tab !== "plc"}>
      {#if detached}<div class="all-clear">
          <ExternalLink size={22} /><strong>{m.plc_window_detached()}</strong>
          <button class="btn small" onclick={() => void onAttach()}
            ><PanelBottom size={14} />{m.plc_window_attach()}</button
          >
        </div>{:else}<Editor bind:sources {status} {problems} {reveal} {selected} instances={project.instances} />{/if}
    </div>
    <div
      class="tab-content problems"
      id="panel-problems"
      role="tabpanel"
      aria-labelledby="dock-problems"
      hidden={tab !== "problems"}
    >
      {#snippet hintsToggle()}<button class="btn small hints" aria-pressed={hints} onclick={() => (hints = !hints)}
          >{hints ? m.ui_hints_hide() : m.ui_hints_show({ count: hintCount })}</button
        >{/snippet}
      {#if clear && checking}<div class="all-clear checking">{m.ui_checking()}</div>
      {:else if clear}<div class="all-clear">
          <CircleCheck size={26} /><strong>{m.ui_no_problems()}</strong>{#if hintCount}{@render hintsToggle()}{/if}
        </div>{:else if hintCount}{@render hintsToggle()}{/if}
      {#each rows as row, i (i)}<button
          class="problem analysis"
          class:err={row.severity === "error"}
          class:hint={row.severity === "hint"}
          onclick={() => open(row)}
          ><CircleAlert size={14} /><span class="where">{row.file}:{row.line}:{row.column}</span><span
            >{row.code} {row.message}</span
          ></button
        >{/each}
      {#if error}<pre class="problem error-line"><CircleAlert size={15} />{error}</pre>{/if}
      {#each warnings as warning, i (i)}<p class="problem"><CircleAlert size={14} />{warning}</p>{/each}
      {#if hints}{#each buildHints as hint, i (i)}<p class="problem hint">
            <CircleAlert size={14} />{hint}
          </p>{/each}{/if}
      {#if notice}<p class="problem">{notice}</p>{/if}
    </div>
    <div
      class="tab-content watch"
      class:operated={project.panel.length > 0}
      id="panel-watch"
      role="tabpanel"
      aria-labelledby="dock-watch"
      hidden={tab !== "watch"}
    >
      <IoPanel {snapshot} {onSinkBlocked} active={tab === "watch" && !collapsed} />
      {#if project.panel.length}<section class="operator">
          <h3>{m.ui_operator()}</h3>
          <div class="operator-buttons">
            {#each project.panel as button (button.address)}<button
                class="btn small"
                disabled={!snapshot}
                onclick={() => onPress(button.address)}>{button.name}</button
              >{/each}
          </div>
        </section>{/if}
    </div>
  </div>
</section>

<style>
  .dock {
    grid-column: 1 / 3;
    min-height: 36px;
    max-height: 60vh;
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
  }
  /* An absolutely positioned grid item is placed in the grid area its grid-column and grid-row name (CSS Grid Level 1,
     9.1), so the dock takes every column and row to cover the workspace. */
  .dock.maximized {
    grid-column: 1 / -1;
    grid-row: 1 / -1;
    position: absolute;
    inset: 0;
    z-index: 6;
    max-height: none;
    border-top: 0;
  }
  .resize {
    position: absolute;
    top: -3px;
    left: 0;
    right: 0;
    height: 5px;
    cursor: ns-resize;
    z-index: 5;
    border: 0;
    padding: 0;
    background: transparent;
  }
  .resize:hover,
  .resize:focus-visible {
    background: var(--color-ring);
  }
  .dock-head {
    display: flex;
    align-items: center;
    height: 35px;
    flex: none;
    border-bottom: 1px solid var(--color-border);
    padding-right: 8px;
  }
  .panel-tabs {
    border: 0;
  }
  .dock-hint {
    font-size: 10px;
    color: var(--color-text-faint);
    padding-right: 12px;
  }
  .badge {
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--color-warning);
  }
  .dock-content,
  .tab-content {
    flex: 1;
    min-height: 0;
    height: 100%;
  }
  .tab-content {
    overflow: auto;
  }
  [hidden] {
    display: none !important;
  }
  .all-clear {
    display: flex;
    height: 100%;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--color-text-faint);
    font-size: 12px;
  }
  .all-clear :global(svg) {
    color: var(--color-success);
  }
  .problem {
    display: flex;
    align-items: start;
    gap: 10px;
    margin: 0;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    font-size: 11px;
    white-space: pre-wrap;
  }
  .problem.analysis {
    width: 100%;
    border-width: 0 0 1px;
    background: transparent;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }
  .problem.analysis:hover {
    background: var(--color-surface-2);
  }
  .problem.analysis.err :global(svg) {
    color: var(--color-danger);
  }
  .problem.hint {
    color: var(--color-text-muted);
  }
  .hints {
    margin: 8px 16px;
  }
  .where {
    flex: none;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }
  .watch {
    display: grid;
    grid-template-columns: 1fr;
  }
  .watch.operated {
    grid-template-columns: 1fr 240px;
  }
  .operator {
    padding: 12px;
    border-left: 1px solid var(--color-border);
  }
  h3 {
    margin: 0 0 12px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--color-text-faint);
  }
  .operator-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .operator .btn:first-child {
    color: var(--color-success);
  }
</style>
