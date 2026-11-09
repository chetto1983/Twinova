<script lang="ts">
  import { onMount } from "svelte";
  import { m } from "./paraglide/messages.js";
  import { currentLocale, languageName, locales, setLanguage, type Locale } from "./lib/i18n.svelte";
  import { readPreference, writePreference } from "./lib/preferences";
  import { backend, onEngineError, onSnapshot, onStopped, projects } from "./lib/backend";
  import { SPEEDS, formatSpeed, type Snapshot } from "./lib/snapshot";
  import Editor from "./lib/Editor.svelte";
  import Inspector from "./lib/Inspector.svelte";
  import IoPanel from "./lib/IoPanel.svelte";
  import ProjectMenu from "./lib/ProjectMenu.svelte";
  import ProjectScene from "./lib/ProjectScene.svelte";
  import SceneTree from "./lib/SceneTree.svelte";
  import SceneView from "./lib/SceneView.svelte";
  import type { ProjectView } from "./lib/project";
  import twinovaMark from "./assets/twinova-mark.svg";

  let source = $state("");
  let snapshot = $state<Snapshot | null>(null);
  let status = $state<"none" | "ok" | "failed">("none");
  let error = $state("");
  let running = $state(false);
  let speed = $state<number | null>(1);
  let theme = $state(readPreference("theme") === "light" ? "light" : "dark");
  /** An opened project is shown, not run (ROADMAP 4.2); null shows the M3 demo. */
  let project = $state<ProjectView | null>(null);
  let selected = $state<string | null>(null);
  let notice = $state("");

  async function showProject(next: ProjectView | null): Promise<void> {
    if (running) await toggleRun();
    project = next;
    selected = null;
    notice = next?.recovered ? m.project_recovered() : "";
    if (next && next.changed_outside.length > 0) {
      notice = [notice, m.project_changed_outside({ files: next.changed_outside.join(", ") })]
        .filter(Boolean)
        .join(" ");
    }
  }
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

  async function build(): Promise<void> {
    running = false;
    const built = await attempt(() => backend.build(source));
    status = built ? "ok" : "failed";
    if (built) snapshot = built;
  }

  async function toggleRun(): Promise<void> {
    if (running) {
      await backend.pause();
      running = false;
    } else if (await attempt(() => backend.run(speed)).then(() => !error)) {
      running = true;
    }
  }

  async function replace(action: () => Promise<Snapshot>): Promise<void> {
    const next = await attempt(action);
    if (next) snapshot = next;
  }

  onMount(() => {
    // The demo builds first, so a launch folder that fails to open leaves its diagnostics on a usable demo.
    void backend
      .defaultProgram()
      .then((text) => {
        source = text;
        return build();
      })
      .then(() => attempt(projects.launch))
      .then((launched) => launched && showProject(launched));
    const listeners = [
      onSnapshot((s) => (snapshot = s)),
      onStopped(() => (running = false)),
      onEngineError((e) => (error = m.engine_error({ error: e }))),
    ];
    return () => listeners.forEach((l) => l.then((unlisten) => unlisten()));
  });
</script>

<div class="shell">
  <header class="top">
    <div class="brand">
      <span class="brand-lockup" style="background-color: #101214">
        <img src={twinovaMark} alt="" />
        <span
          class="name"
          style="color: #eef2f6; font-family: 'Atkinson Hyperlegible Next', ui-sans-serif, system-ui, sans-serif"
          >Twinova</span
        >
      </span>
      <ProjectMenu
        {project}
        onChange={showProject}
        onNotice={(text) => (notice = text)}
        onError={(text) => (error = text)}
      />
    </div>
    <div class="row actions">
      {#if !project}
        <button class="btn" onclick={build}>{m.build()}</button>
        <button class="btn primary" disabled={status !== "ok"} onclick={toggleRun}>
          {running ? m.pause() : m.run()}
        </button>
        <select class="speed" aria-label={m.speed()} bind:value={speed} disabled={running}>
          {#each SPEEDS as option (option)}
            <option value={option}>{formatSpeed(option, currentLocale(), m.speed_max())}</option>
          {/each}
        </select>
        <button
          class="btn"
          disabled={status !== "ok"}
          onclick={() => replace(backend.step).then(() => (running = false))}
        >
          {m.step()}
        </button>
        <button
          class="btn"
          disabled={status !== "ok"}
          onclick={() => replace(backend.reset).then(() => (running = false))}
        >
          {m.reset()}
        </button>
        <button class="btn" disabled={status !== "ok"} onclick={() => attempt(backend.saveState)}>
          {m.save_state()}
        </button>
        <button
          class="btn"
          disabled={status !== "ok"}
          onclick={() => replace(backend.restoreState).then(() => (running = false))}
        >
          {m.restore_state()}
        </button>
        <span class="sep"></span>
        <button class="btn" disabled={status !== "ok"} onclick={() => attempt(() => backend.press(true, false))}>
          {m.start()}
        </button>
        <button class="btn danger" disabled={status !== "ok"} onclick={() => attempt(() => backend.press(false, true))}>
          {m.stop()}
        </button>
      {/if}
    </div>
    <div class="row">
      <select
        class="lang"
        aria-label={m.language()}
        value={currentLocale()}
        onchange={(e) => setLanguage(e.currentTarget.value as Locale)}
      >
        {#each locales as locale (locale)}<option value={locale}>{languageName(locale)}</option>{/each}
      </select>
      <button
        class="btn ghost icon"
        aria-label={m.theme_switch()}
        onclick={() => (theme = theme === "dark" ? "light" : "dark")}
      >
        {theme === "dark" ? "☾" : "☀"}
      </button>
    </div>
  </header>
  {#if project}
    <p class="notice" role="status">
      {m.project_not_runnable()}
      {#if notice}<br />{notice}{/if}
      {#if error}<br /><span class="err">{error}</span>{/if}
    </p>
    <main class="grid">
      <SceneTree {project} bind:selected />
      <section class="panel stage">
        <header class="panel-head"><h2 class="panel-title">{project.name}</h2></header>
        <div class="stage-body"><ProjectScene {project} {selected} {theme} /></div>
      </section>
      <Inspector {project} {selected} />
    </main>
  {:else}
    {#if notice}<p class="notice" role="status">{notice}</p>{/if}
    <main class="grid">
      <Editor bind:source {status} {error} />
      <section class="panel stage">
        <header class="panel-head"><h2 class="panel-title">{m.scene_title()}</h2></header>
        <div class="stage-body"><SceneView {snapshot} {theme} /></div>
      </section>
      <IoPanel {snapshot} onOutletBlocked={(b) => replace(() => backend.setOutletBlocked(b))} />
    </main>
  {/if}
</div>

<style>
  /* A column, so the optional status line does not shift the main grid into an auto-sized row, and min-width: 0 so the
     header's buttons wrap instead of widening the page (the Windows E2E window is 1028 px wide, run 42524833751). */
  .shell {
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    gap: 12px;
    padding: 12px;
  }
  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .brand-lockup {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 38px;
    padding: 4px 10px 4px 6px;
    border: 1px solid #343c46;
    border-radius: 10px;
    box-shadow: 0 5px 16px rgb(0 0 0 / 0.18);
  }
  .brand img {
    width: 26px;
    height: 26px;
    object-fit: contain;
  }
  .name {
    display: inline-flex;
    align-items: baseline;
    font-size: 18px;
    font-weight: 720;
    line-height: 1;
    letter-spacing: -0.02em;
  }
  .actions {
    flex: 1;
    flex-wrap: wrap;
    justify-content: center;
  }
  .sep {
    width: 1px;
    height: 24px;
    background: var(--color-border);
  }
  .lang,
  .speed {
    width: auto;
  }
  .grid {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(320px, 1fr) minmax(420px, 2fr) minmax(240px, 0.8fr);
    gap: 12px;
    min-height: 0;
  }
  .notice {
    margin: 0;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    color: var(--color-text-muted);
  }
  .err {
    color: var(--color-danger);
  }
  .stage {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .stage-body {
    flex: 1;
    min-height: 0;
    padding: 12px;
  }
</style>
