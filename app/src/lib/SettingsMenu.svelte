<script lang="ts">
  // The window's settings, out of the main bar (spec 2026-11-26-ux-consolidation-design.md, decision 8): the language and
  // the theme, which a person sets once, behind one button.
  import { Settings } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { currentLocale, languageName, locales, setLanguage, type Locale } from "./i18n.svelte";
  let { theme = $bindable() }: { theme: string } = $props();
  let open = $state(false);
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="settings">
  <button
    class="icon-button"
    title={m.settings()}
    aria-label={m.settings()}
    aria-haspopup="dialog"
    aria-expanded={open}
    onclick={() => (open = !open)}><Settings size={16} /></button
  >
  {#if open}
    <div class="popover" role="dialog" aria-label={m.settings()}>
      <label
        >{m.language()}<select
          class="lang"
          value={currentLocale()}
          onchange={(e) => setLanguage(e.currentTarget.value as Locale)}
        >
          {#each locales as locale (locale)}<option value={locale}>{languageName(locale)}</option>{/each}
        </select></label
      >
      <label
        >{m.theme()}<select class="theme" bind:value={theme}>
          <option value="light">{m.theme_light()}</option>
          <option value="dark">{m.theme_dark()}</option>
        </select></label
      >
    </div>
  {/if}
</div>

<style>
  .settings {
    position: relative;
  }
  .popover {
    position: absolute;
    z-index: 10;
    top: calc(100% + 6px);
    right: 0;
    display: grid;
    gap: 10px;
    min-width: 220px;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.2);
  }
  label {
    display: grid;
    gap: 4px;
    font-size: 12px;
    color: var(--color-text-muted);
  }
  select {
    height: 28px;
    font-size: 13px;
  }
</style>
