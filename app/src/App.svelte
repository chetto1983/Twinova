<script lang="ts">
  // The app on a project (spec 2026-11-09-component-behaviour-design.md, "The app"): the bundled M3 demo at start, or
  // a project opened from the menu or the command line. Build runs its line with the editor's sources; the panel's
  // buttons come from the project. The catalogue and the inspector edit the scene, each edit one undo step
  // (spec 2026-11-11-editor-basics-design.md), and parts dragged in the view snap together (2026-11-11-snapping); the
  // running line changes at the next Build. Sources saved in the project's folder outside Twinova, as in VS Code, reach
  // the running line (2026-11-12-st-vscode): the program online, a behaviour by a rebuild.
  import { onMount } from "svelte";
  import { m } from "./paraglide/messages.js";
  import { readPreference, writePreference } from "./lib/preferences";
  import { analysis, backend, onDiskSources, onEngineError, onSnapshot, onStopped, projects } from "./lib/backend";
  import { Inbox, hostPlcWindow, plcWindow } from "./lib/plcWindow";
  import { debounce, type Problems } from "./lib/analysis";
  import { deliveredOf, formatTime, type Snapshot } from "./lib/snapshot";
  import {
    buildArgs,
    builtFrom,
    fromDisk,
    mergeSources,
    sourcesOf,
    type DiskSources,
    type Source,
  } from "./lib/sources";
  import Inspector from "./lib/Inspector.svelte";
  import ProjectScene from "./lib/ProjectScene.svelte";
  import SceneTree from "./lib/SceneTree.svelte";
  import SnapBar from "./lib/SnapBar.svelte";
  import { candidate, editOf, type Candidate } from "./lib/snapping";
  import { frame, type Edit, type ProjectView, type Quat, type Vec3 } from "./lib/project";
  import type { GizmoMode } from "./lib/sceneEditing";
  import { shortcutOf } from "./lib/shortcuts";
  import Toolbar from "./lib/Toolbar.svelte";
  import WorkspaceDock from "./lib/WorkspaceDock.svelte";
  import { Box, Move, RotateCw, CircleCheck, AlertCircle } from "@lucide/svelte";
  let hidden = $state<string[]>([]);
  let locked = $state<string[]>([]);
  let history = $state<{ label: string; time: string }[]>([]);
  let ratio = $state<number | null>(null);
  let paceStart: { wall: number; virtual: number } | null = null;
  function receiveSnapshot(s: Snapshot): void {
    const wall = performance.now();
    if (running && paceStart && s.time_ms >= paceStart.virtual) {
      const elapsed = wall - paceStart.wall;
      if (elapsed >= 500) {
        ratio = (s.time_ms - paceStart.virtual) / elapsed;
        paceStart = { wall, virtual: s.time_ms };
      }
    } else paceStart = { wall, virtual: s.time_ms };
    snapshot = s;
  }
  function toggleFlag(list: string[], id: string): string[] {
    return list.includes(id) ? list.filter((x) => x !== id) : [...list, id];
  }

  let project = $state<ProjectView | null>(null);
  let sources = $state<Source[]>([]);
  let snapshot = $state<Snapshot | null>(null);
  let status = $state<"none" | "ok" | "failed">("none");
  let error = $state("");
  let running = $state(false);
  let speed = $state<number | null>(1);
  let theme = $state(readPreference("theme") === "light" ? "light" : "dark");
  let selected = $state<string | null>(null);
  /** What opening the project reported, and what the last Build warned about. */
  let notice = $state<string | (() => string)>("");
  const noticeText = $derived(typeof notice === "function" ? notice() : notice);
  let warnings = $state<string[]>([]);
  /** truST's analysis of the editor's texts, 300 ms after the last change (spec 2026-11-12-st-editor-design.md,
   *  decision 6); a failure of the analysis itself is a notice, since Build stays the referee. */
  let problems = $state<Problems>({});
  const analyzeLater = debounce((program: [string, string][], behaviours: Record<string, string>) => {
    analysis.analyze(program, behaviours).then(
      (found) => (problems = found),
      (e) => (notice = String(e)),
    );
  }, 300);
  $effect(() => {
    const { program, behaviours } = buildArgs(sources);
    analyzeLater.call(program, behaviours);
    return analyzeLater.cancel;
  });
  /** The PLC pane is in its own window (spec 2026-11-13-plc-window-design.md): this window keeps the texts and sends
   *  the window its state, here and when it is ready. */
  let detached = $state(false);
  const inbox = new Inbox();
  const ready = $derived(snapshot !== null);
  const plcState = () => ({
    ...$state.snapshot({ sources, status, problems, selected, running, ready }),
    instances: $state.snapshot(project?.instances ?? []),
    ack: inbox.ack,
  });
  $effect(() => {
    if (detached) void plcWindow.send(plcState());
  });
  function detach(): void {
    inbox.reset();
    detached = true;
    plcWindow.open().catch((e) => {
      detached = false;
      error = String(e);
    });
  }
  /** The scene was edited since the last Build. */
  let changed = $state(false);
  /** Counts the projects shown, so the view frames each when it opens and not at every edit. */
  let framing = $state(0);
  let mode = $state<GizmoMode>("translate");
  /** The catalogue family being dragged into the view. */
  let dragging = $state<string | null>(null);
  /** Snapping: on, the gap a new join leaves (m), and the join a drag would make now. */
  let snapping = $state(true);
  let gap = $state(0);
  let snapTo = $state<Candidate | null>(null);

  $effect(() => {
    document.documentElement.dataset.theme = theme;
    writePreference("theme", theme);
  });

  async function attempt<T>(action: () => Promise<T>): Promise<T | undefined> {
    try {
      error = "";
      return await action();
    } catch (e) {
      error = String(e);
      return undefined;
    }
  }

  /** A Build that fails leaves the running line as it was (commands.rs `build`), so only a success stops Run here. */
  async function build(): Promise<void> {
    const built = await attempt(() => {
      const { program, behaviours } = buildArgs(sources);
      return backend.build(program, behaviours);
    });
    status = built ? "ok" : "failed";
    if (built) {
      sources = builtFrom(sources);
      running = false;
      changed = false;
      snapshot = built.snapshot;
      warnings = built.warnings;
    }
  }

  async function showProject(next: ProjectView): Promise<void> {
    if (running) await toggleRun();
    project = next;
    hidden = [];
    locked = [];
    history = [];
    framing += 1;
    sources = sourcesOf(next);
    selected = null;
    snapshot = null;
    warnings = [];
    notice = () =>
      [
        next.recovered ? m.project_recovered() : "",
        next.changed_outside.length > 0 ? m.project_changed_outside({ files: next.changed_outside.join(", ") }) : "",
        next.notice ?? "",
      ]
        .filter(Boolean)
        .join(" ");
    await build();
  }

  /** Sources saved in the folder outside Twinova (watch.rs): taken unless a changed file has edits here (decision 6); a
   *  program goes online into the running line, a behaviour rebuilds it and Run goes on. Before the first Build there is
   *  no line to change, so it builds. */
  async function fromFolder(disk: DiskSources): Promise<void> {
    const next = fromDisk(sources, disk);
    if (next.held.length > 0) {
      notice = () => m.disk_held({ files: next.held.join(", ") });
      return;
    }
    sources = next.sources;
    if (disk.kind === "program" && snapshot) {
      const result = await attempt(() => backend.changeProgram(disk.program));
      status = result ? "ok" : "failed";
      if (!result) return;
      sources = builtFrom(sources);
      snapshot = result.snapshot;
      warnings = result.warnings;
      notice = () => m.disk_changed_online({ time: (result.snapshot.time_ms / 1000).toFixed(2) });
      return;
    }
    const wasRunning = running;
    await build();
    if (status !== "ok") return;
    notice = () => m.disk_rebuilt();
    if (wasRunning) await toggleRun();
  }

  /** The project after an edit, undo or redo: the editor keeps the texts being edited, and a selection the edit removed
   *  is cleared. */
  function showEdited(next: ProjectView): void {
    project = next;
    sources = mergeSources(sources, next);
    if (!next.instances.some((i) => i.id === selected)) selected = null;
    changed = true;
  }

  /** One edit; false, with the reason shown, when the project refuses it. */
  async function edit(op: Edit): Promise<boolean> {
    if ("id" in op && locked.includes(op.id)) return false;
    const before = new Set(project?.instances.map((i) => i.id));
    const next = await attempt(() => projects.edit(op));
    if (!next) return false;
    showEdited(next);
    history = [
      { label: op.op.replaceAll("_", " ") + ("id" in op ? " · " + op.id : ""), time: new Date().toLocaleTimeString() },
      ...history,
    ].slice(0, 100);
    if (next.notice) notice = next.notice;
    if (op.op === "place") selected = next.instances.find((i) => !before.has(i.id))?.id ?? selected;
    return true;
  }

  /** A catalogue family, placed on the floor under the view's framing centre. */
  function place(family: string): void {
    const [x, , z] = frame(project?.instances ?? []).centre;
    void edit({ op: "place", family, position: [x, 0, z], rotation: [0, 0, 0, 1] });
  }

  function move(id: string, position: Vec3, rotation: Quat): void {
    void edit({ op: "move", id, position, rotation });
  }

  /** A family dropped from the catalogue lands where it is released, then snaps to what it lands near: two undo steps,
   *  as it has no interfaces before it is placed. */
  async function drop(family: string, position: Vec3): Promise<void> {
    dragging = null;
    if (!(await edit({ op: "place", family, position, rotation: [0, 0, 0, 1] }))) return;
    const c = snapping && project && selected ? candidate(project, selected, position, [0, 0, 0, 1]) : null;
    if (c) await edit(editOf(c, gap));
  }

  function onKey(event: KeyboardEvent): void {
    const shortcut = shortcutOf(event);
    if (!shortcut || !project) return;
    event.preventDefault();
    if (shortcut === "undo" && project.can_undo) void step(projects.undo);
    else if (shortcut === "redo" && project.can_redo) void step(projects.redo);
    else if (shortcut === "remove" && selected) void edit({ op: "remove", id: selected });
    else if (shortcut === "translate" || shortcut === "rotate") mode = shortcut;
  }

  async function step(action: () => Promise<ProjectView>): Promise<void> {
    const next = await attempt(action);
    if (next) showEdited(next);
  }

  async function toggleRun(): Promise<void> {
    if (running) {
      await attempt(backend.pause);
      running = false;
    } else if (await attempt(() => backend.run(speed)).then(() => !error)) {
      paceStart = { wall: performance.now(), virtual: snapshot?.time_ms ?? 0 };
      ratio = null;
      running = true;
    }
  }

  const stepLine = () => replace(backend.step).then(() => (running = false));

  async function replace(action: () => Promise<Snapshot>): Promise<void> {
    const next = await attempt(action);
    if (next) snapshot = next;
  }

  onMount(() => {
    // A folder named on the command line opens directly; one that fails to open leaves its diagnostics on the demo,
    // built and usable.
    void attempt(projects.launch).then(async (launched) => {
      const failed = error;
      await showProject(launched ?? (await projects.current()));
      if (failed) error = failed;
    });
    const listeners = [
      onSnapshot(receiveSnapshot),
      onStopped(() => (running = false)),
      onDiskSources((disk) => void fromFolder(disk)),
      onEngineError((e) => (error = m.engine_error({ error: e }))),
      ...hostPlcWindow(inbox, {
        take: (next) => (sources = next),
        request: (r) => void (r === "build" ? build() : r === "run" ? toggleRun() : stepLine()),
        ready: () => void plcWindow.send(plcState()),
        closed: () => (detached = false),
      }),
    ];
    return () => listeners.forEach((l) => l.then((unlisten) => unlisten()));
  });
</script>

<svelte:window onkeydown={onKey} onpointerup={() => setTimeout(() => (dragging = null))} />

<div class="shell">
  <Toolbar
    {project}
    {running}
    {ready}
    bind:speed
    bind:theme
    {ratio}
    onBuild={build}
    onRun={toggleRun}
    onStop={() => replace(backend.reset).then(() => (running = false))}
    onStep={stepLine}
    onUndo={() => step(projects.undo)}
    onRedo={() => step(projects.redo)}
    onSave={() => attempt(backend.saveState)}
    onRestore={() => replace(backend.restoreState).then(() => (running = false))}
    onProject={showProject}
    onNotice={(text) => (notice = text)}
    onError={(text) => (error = text)}
  />
  {#if project}
    <main class="workspace">
      <aside class="left-sidebar">
        <SceneTree
          {project}
          bind:selected
          {hidden}
          {locked}
          {history}
          onVisibility={(id) => (hidden = toggleFlag(hidden, id))}
          onLock={(id) => (locked = toggleFlag(locked, id))}
        />
      </aside>
      <section class="stage">
        <header class="scene-header">
          <Box size={14} /><span>{project.name}</span>{#if changed}<span class="modified" title={m.scene_changed()}
            ></span>{/if}<span class="spacer"></span><span class="muted">{m.ui_view()}</span>
        </header>
        <div class="stage-body">
          <ProjectScene
            {project}
            {selected}
            {theme}
            {hidden}
            {locked}
            cartons={snapshot?.cartons ?? []}
            actuators={snapshot?.actuators ?? []}
            conveyors={snapshot?.conveyors ?? []}
            {framing}
            {mode}
            dropping={dragging}
            onSelect={(id) => (selected = id)}
            {snapping}
            onMove={move}
            onDrop={drop}
            onSnap={(c) => void edit(editOf(c, gap))}
            onUnmount={(device, position, rotation) => void edit({ op: "unmount", device, position, rotation })}
            onCandidate={(c) => (snapTo = c)}
          />
          <div class="edit-tools">
            <button
              class="icon-button"
              class:active={mode === "translate"}
              title={m.ui_move()}
              aria-label={m.ui_move()}
              aria-pressed={mode === "translate"}
              onclick={() => (mode = "translate")}><Move size={17} /></button
            >
            <button
              class="icon-button"
              class:active={mode === "rotate"}
              title={m.ui_rotate()}
              aria-label={m.ui_rotate()}
              aria-pressed={mode === "rotate"}
              onclick={() => (mode = "rotate")}><RotateCw size={17} /></button
            >
          </div>
          <div class="scene-counters">
            <span># {m.ui_cases()} <b>{snapshot?.cartons.length ?? 0}</b></span><span
              ># {m.ui_delivered()} <b>{deliveredOf(snapshot)}</b></span
            >
          </div>
          <div class="view-footer">
            <span>{m.ui_scene_help()}</span><SnapBar bind:enabled={snapping} bind:gap candidate={snapTo} />
          </div>
        </div>
      </section>
      <aside class="right-sidebar">
        <Inspector {project} {selected} locked={selected !== null && locked.includes(selected)} onEdit={edit} />
      </aside>
      <WorkspaceDock
        {project}
        bind:sources
        {status}
        {error}
        {warnings}
        {problems}
        notice={noticeText}
        {snapshot}
        {selected}
        {detached}
        onDetach={detach}
        onAttach={() => void plcWindow.close()}
        onPlace={place}
        onDrag={(family) => (dragging = family)}
        onSinkBlocked={(sink, blocked) => replace(() => backend.setSinkBlocked(sink, blocked))}
        onPress={(address) => attempt(() => backend.press(address))}
      />
    </main>
  {/if}
  <footer class="statusbar">
    <span class="status"
      ><span class="led" class:on={running}></span>{running
        ? m.ui_running()
        : status === "ok"
          ? m.ui_ready()
          : m.ui_stopped()}</span
    >
    <span class="divider"></span><span class="data">{formatTime(snapshot?.time_ms ?? 0)}</span>
    <p class="status-notice" role="status" title={noticeText}>{noticeText}</p>
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
    <span class="divider"></span><span>{project?.instances.length ?? 0} {m.ui_objects()}</span><span class="divider"
    ></span><span>truST · IEC 61131-3</span>
  </footer>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
  }
  .workspace {
    position: relative;
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 224px minmax(280px, 1fr) 292px;
    grid-template-rows: minmax(200px, 1fr) auto;
  }
  .left-sidebar,
  .right-sidebar {
    min-height: 0;
    overflow: auto;
    background: var(--color-surface);
  }
  .left-sidebar {
    border-right: 1px solid var(--color-border);
  }
  .right-sidebar {
    grid-column: 3;
    grid-row: 1 / 3;
    border-left: 1px solid var(--color-border);
  }
  .stage {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .scene-header {
    height: 35px;
    padding: 0 12px;
    display: flex;
    gap: 8px;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: 11px;
  }
  .modified {
    width: 5px;
    height: 5px;
    background: var(--color-warning);
    border-radius: 50%;
  }
  .stage-body {
    flex: 1;
    min-height: 0;
    position: relative;
  }
  .edit-tools {
    position: absolute;
    top: 12px;
    left: 12px;
    display: flex;
    padding: 3px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    box-shadow: var(--shadow-popover);
  }
  .scene-counters {
    position: absolute;
    top: 65px;
    left: 12px;
    display: flex;
    gap: 6px;
    pointer-events: none;
  }
  .scene-counters span {
    padding: 4px 9px;
    border: 1px solid var(--color-border);
    background: color-mix(in oklab, var(--color-surface) 92%, transparent);
    border-radius: 12px;
    font-size: 10px;
  }
  .scene-counters b {
    margin-left: 8px;
    font-family: var(--font-mono);
  }
  .view-footer {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    justify-content: space-between;
    background: color-mix(in oklab, var(--color-surface) 90%, transparent);
    border-top: 1px solid var(--color-border);
    padding: 5px 10px;
    font-size: 10px;
    color: var(--color-text-faint);
  }
  .view-footer :global(input) {
    height: 23px;
    font-size: 10px;
  }
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
  @media (max-width: 1200px) {
    .workspace {
      grid-template-columns: 190px minmax(260px, 1fr) 260px;
    }
    .view-footer > span {
      display: none;
    }
  }
</style>
