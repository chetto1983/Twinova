<script lang="ts">
  import { Play, Pause, Square, StepForward, Hammer, Undo2, Redo2, Save, History, Sun, Moon } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { currentLocale, languageName, locales, setLanguage, type Locale } from "./i18n.svelte";
  import { SPEEDS, formatSpeed } from "./snapshot";
  import type { ProjectView } from "./project";
  import ProjectMenu from "./ProjectMenu.svelte";
  import twinovaMark from "../assets/twinova-mark.svg";

  let {
    project,
    running,
    ready,
    speed = $bindable(),
    theme = $bindable(),
    ratio,
    onBuild,
    onRun,
    onStop,
    onStep,
    onUndo,
    onRedo,
    onSave,
    onRestore,
    onProject,
    onNotice,
    onError,
  }: {
    project: ProjectView | null;
    running: boolean;
    ready: boolean;
    speed: number | null;
    theme: string;
    ratio: number | null;
    onBuild: () => void;
    onRun: () => void;
    onStop: () => void;
    onStep: () => void;
    onUndo: () => void;
    onRedo: () => void;
    onSave: () => void;
    onRestore: () => void;
    onProject: (p: ProjectView) => void;
    onNotice: (text: () => string) => void;
    onError: (s: string) => void;
  } = $props();
</script>

<header class="top">
  <div class="brand">
    <span class="brand-lockup" style="background-color:#101214"
      ><img src={twinovaMark} alt="" /><strong
        class="name"
        style="color:#eef2f6;font-family:'Atkinson Hyperlegible Next',ui-sans-serif,system-ui,sans-serif"
        >Twinova</strong
      ></span
    ><span class="divider"></span>
    {#if project}<ProjectMenu {project} onChange={onProject} {onNotice} {onError} />{/if}
  </div>
  <div class="transport" aria-label={m.ui_simulation()}>
    <button class="icon-button" title={m.build()} aria-label={m.build()} onclick={onBuild} disabled={!project}
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
    <button class="icon-button" title={m.reset()} aria-label={m.reset()} onclick={onStop} disabled={!ready}
      ><Square size={14} fill="currentColor" /></button
    >
    <button class="icon-button" title={m.step()} aria-label={m.step()} onclick={onStep} disabled={!ready}
      ><StepForward size={17} /></button
    >
    <span class="divider"></span>
    <select class="speed" aria-label={m.speed()} bind:value={speed} disabled={running}>
      {#each SPEEDS as option (option)}<option value={option}
          >{formatSpeed(option, currentLocale(), m.speed_max())}</option
        >{/each}
    </select>
    <span class="actual data" title={m.ui_actual_speed()}
      >{running && ratio !== null ? `~${ratio.toFixed(2)}×` : "—"}</span
    >
  </div>
  <div class="utilities">
    <button class="icon-button" title={m.undo()} aria-label={m.undo()} disabled={!project?.can_undo} onclick={onUndo}
      ><Undo2 size={16} /></button
    >
    <button class="icon-button" title={m.redo()} aria-label={m.redo()} disabled={!project?.can_redo} onclick={onRedo}
      ><Redo2 size={16} /></button
    >
    <span class="divider"></span>
    <button class="icon-button" title={m.save_state()} aria-label={m.save_state()} disabled={!ready} onclick={onSave}
      ><Save size={16} /></button
    >
    <button
      class="icon-button"
      title={m.restore_state()}
      aria-label={m.restore_state()}
      disabled={!ready}
      onclick={onRestore}><History size={16} /></button
    >
    <select
      class="lang"
      aria-label={m.language()}
      value={currentLocale()}
      onchange={(e) => setLanguage(e.currentTarget.value as Locale)}
    >
      {#each locales as locale (locale)}<option value={locale}>{languageName(locale)}</option>{/each}
    </select>
    <button
      class="icon-button"
      title={m.theme_switch()}
      aria-label={m.theme_switch()}
      onclick={() => (theme = theme === "dark" ? "light" : "dark")}
    >
      {#if theme === "dark"}<Sun size={16} />{:else}<Moon size={16} />{/if}
    </button>
  </div>
</header>

<style>
  .top {
    height: 52px;
    flex: none;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 14px;
    gap: 12px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }
  .brand,
  .transport,
  .utilities {
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
  }
  .brand img {
    width: 26px;
    height: 26px;
    margin-right: 4px;
  }
  .brand-lockup {
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: 4px;
    padding: 3px 7px 3px 4px;
  }
  .brand strong {
    font-size: 17px;
    font-weight: 650;
    letter-spacing: -0.5px;
  }
  .brand :global(.project) {
    border: none;
    background: transparent;
    max-width: 200px;
    font-size: 12px;
    font-weight: 400;
  }
  .brand :global(.project span:first-child) {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .transport {
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 2px 5px;
    background: var(--color-bg);
  }
  .utilities {
    justify-content: flex-end;
  }
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
  select {
    height: 27px;
    border: 0;
    background-color: transparent;
    font-size: 11px;
  }
  .speed {
    width: 66px;
  }
  .lang {
    width: 87px;
  }
  .actual {
    width: 53px;
    color: var(--color-text-faint);
    font-size: 10px;
  }
  @media (max-width: 1250px) {
    .brand strong {
      display: none;
    }
    .brand :global(.project) {
      max-width: 160px;
    }
    .top {
      gap: 6px;
      padding: 0 8px;
    }
    .utilities {
      gap: 1px;
    }
  }
</style>
