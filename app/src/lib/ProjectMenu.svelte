<script lang="ts">
  // The project button and its menu (research 13, PRJ-01): new, open, save, save to a folder, import, export, and back to
  // the bundled M3 demo. Each action reports through onChange, onNotice or onError.
  import { m } from "../paraglide/messages.js";
  import { projects } from "./backend";
  import type { ProjectView } from "./project";

  let {
    project,
    onChange,
    onNotice,
    onError,
  }: {
    project: ProjectView;
    onChange: (next: ProjectView) => void;
    onNotice: (text: string) => void;
    onError: (text: string) => void;
  } = $props();

  let open = $state(false);

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
      onNotice([m.project_saved({ folder: next.folder ?? "" }), next.notice].filter(Boolean).join(" "));
    }
  }

  async function exportArchive(): Promise<void> {
    const file = await projects.exportArchive();
    if (file) onNotice(m.project_exported({ file }));
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
    { label: m.project_back_to_demo(), run: async () => onChange(await projects.demo()) },
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
  .item:hover {
    background: var(--color-surface-2);
  }
</style>
