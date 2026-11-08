<script lang="ts">
  import { onMount } from "svelte";
  import { m } from "./paraglide/messages.js";
  import { currentLocale, languageName, locales, setLanguage, type Locale } from "./lib/i18n.svelte";
  import { readPreference, writePreference } from "./lib/preferences";
  import { backend, onEngineError, onSnapshot, onStopped } from "./lib/backend";
  import { SPEEDS, formatSpeed, type Snapshot } from "./lib/snapshot";
  import Editor from "./lib/Editor.svelte";
  import IoPanel from "./lib/IoPanel.svelte";
  import SceneView from "./lib/SceneView.svelte";
  import twinovaMark from "./assets/twinova-mark.svg";

  let source = $state("");
  let snapshot = $state<Snapshot | null>(null);
  let status = $state<"none" | "ok" | "failed">("none");
  let error = $state("");
  let running = $state(false);
  let speed = $state<number | null>(1);
  let theme = $state(readPreference("theme") === "light" ? "light" : "dark");

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
    void backend.defaultProgram().then((text) => {
      source = text;
      void build();
    });
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
      <img src={twinovaMark} alt="" />
      <span class="name">Twinova</span>
      <span class="muted">{m.app_tagline()}</span>
    </div>
    <div class="row actions">
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
  <main class="grid">
    <Editor bind:source {status} {error} />
    <section class="panel stage">
      <header class="panel-head"><h2 class="panel-title">{m.scene_title()}</h2></header>
      <div class="stage-body"><SceneView {snapshot} {theme} /></div>
    </section>
    <IoPanel {snapshot} onOutletBlocked={(b) => replace(() => backend.setOutletBlocked(b))} />
  </main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: auto 1fr;
    height: 100%;
    gap: 12px;
    padding: 12px;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .brand img {
    width: 26px;
    height: 26px;
    object-fit: contain;
  }
  .name {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
  }
  .actions {
    flex: 1;
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
    display: grid;
    grid-template-columns: minmax(320px, 1fr) minmax(420px, 2fr) minmax(240px, 0.8fr);
    gap: 12px;
    min-height: 0;
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
