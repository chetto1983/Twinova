<script lang="ts">
  // The project button and its menu (research 13, PRJ-01): new, open, save, save to a folder, import, export, and the
  // example lines the app carries, the M3 demo first (spec 2026-11-23-app-mcp-design.md, decision 5). Each action reports
  // through onChange, onNotice or onError.
  import { onMount } from "svelte";
  import { m } from "../paraglide/messages.js";
  import { projects } from "./backend";
  import type { Example, ProjectView } from "./project";

  let {
    project,
    onChange,
    onNotice,
    onError,
  }: {
    project: ProjectView;
    onChange: (next: ProjectView) => void;
    onNotice: (text: () => string) => void;
    onError: (text: string) => void;
  } = $props();

  let open = $state(false);
  let examples = $state<Example[]>([]);

  onMount(() => {
    projects.examples().then(
      (list) => (examples = list),
      (e: unknown) => onError(String(e)),
    );
  });

  async function act(action: () => Promise<void>): Promise<void> {
    open = false;
    try {
      await action();
    } catch (e) {
      onError(String(e));
    }
  }

  async function openFolder(): Promise<void> {
    const next = await projects.open();
    if (next) onChange(next);
  }

  async function saveTo(saved: Promise<ProjectView | null>): Promise<void> {
    const next = await saved;
    if (next) {
      onChange(next);
      onNotice(() => [m.project_saved({ folder: next.folder ?? "" }), next.notice].filter(Boolean).join(" "));
    }
  }

  async function exportArchive(): Promise<void> {
    const file = await projects.exportArchive();
    if (file) onNotice(() => m.project_exported({ file }));
  }

  async function importArchive(): Promise<void> {
    const next = await projects.importArchive();
    if (next) onChange(next);
  }

  const items = $derived([
    { label: m.project_new(), run: async () => onChange(await projects.create()) },
    { label: m.project_open(), run: openFolder },
    ...(project.folder ? [{ label: m.project_save(), run: () => saveTo(projects.save(project.folder!)) }] : []),
    { label: m.project_save_as(), run: () => saveTo(projects.saveAs()) },
    { label: m.project_export(), run: exportArchive },
    { label: m.project_import(), run: importArchive },
  ]);
</script>

<div class="menu">
  <button class="btn project" aria-haspopup="menu" aria-expanded={open} onclick={() => (open = !open)}>
    <span>{project.name}</span>
    <span aria-hidden="true">▾</span>
  </button>
  {#if open}
    <ul class="items" role="menu" aria-label={m.project_menu()}>
      {#each items as item (item.label)}
        <li><button role="menuitem" class="item" onclick={() => act(item.run)}>{item.label}</button></li>
      {/each}
      <li role="separator" class="heading">{m.project_examples()}</li>
      {#each examples as example (example.id)}
        <li>
          <button
            role="menuitem"
            class="item"
            onclick={() => act(async () => onChange(await projects.example(example.id)))}>{example.name}</button
          >
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .menu {
    position: relative;
  }
  .project {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .items {
    position: absolute;
    z-index: 10;
    top: calc(100% + 4px);
    left: 0;
    min-width: 240px;
    margin: 0;
    padding: 4px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.2);
    list-style: none;
  }
  .item {
    width: 100%;
    padding: 8px 10px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }
  .heading {
    padding: 8px 10px 4px;
    border-top: 1px solid var(--color-border);
    margin-top: 4px;
    color: var(--color-text-muted);
    font-size: 0.85em;
  }
  .item:hover {
    background: var(--color-surface-2);
  }
</style>
