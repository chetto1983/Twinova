<script lang="ts" module>
  export type Tool = "rung" | "contact" | "coil" | "box" | "branch" | "delete" | "undo" | "redo";
</script>

<script lang="ts">
  // The Ladder view's toolbox, as CODESYS LD2's (spec 2026-11-13-ladder-editor-design.md, research 18): a rung, a
  // contact, a coil, a box and a parallel branch, placed by the selection; the variants are modifiers on the selected
  // element; then delete, undo and redo.
  import { ListPlus, Redo2, Trash2, Undo2 } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { hasModifier, takesModifier, type Modifier } from "./ladder/edit";
  import type { Element } from "./ladder/model";

  let {
    editable,
    selected,
    element,
    canUndo,
    canRedo,
    onTool,
    onModifier,
  }: {
    editable: boolean;
    selected: boolean;
    element: Element | null;
    canUndo: boolean;
    canRedo: boolean;
    onTool: (tool: Tool) => void;
    onModifier: (modifier: Modifier) => void;
  } = $props();

  const placing: { tool: Tool; title: () => string }[] = [
    { tool: "contact", title: m.ladder_add_contact },
    { tool: "coil", title: m.ladder_add_coil },
    { tool: "box", title: m.ladder_add_box },
    { tool: "branch", title: m.ladder_add_branch },
  ];
  const modifiers: { modifier: Modifier; mark: string; title: () => string }[] = [
    { modifier: "negated", mark: "/", title: m.ladder_negated },
    { modifier: "rising", mark: "P", title: m.ladder_rising },
    { modifier: "falling", mark: "N", title: m.ladder_falling },
    { modifier: "set", mark: "S", title: m.ladder_set },
    { modifier: "reset", mark: "R", title: m.ladder_reset },
  ];
</script>

<div class="tools" role="toolbar" aria-label={m.ladder_tools()}>
  <button
    class="icon-button"
    title={m.ladder_add_rung()}
    aria-label={m.ladder_add_rung()}
    disabled={!editable}
    onclick={() => onTool("rung")}><ListPlus size={15} /></button
  >
  {#each placing as p (p.tool)}
    <button
      class="icon-button"
      title={p.title()}
      aria-label={p.title()}
      disabled={!editable || !selected}
      onclick={() => onTool(p.tool)}
    >
      <svg width="22" height="14" viewBox="0 0 22 14" aria-hidden="true">
        {#if p.tool === "contact"}
          <path d="M0 7h7M15 7h7M7 1v12M15 1v12" />
        {:else if p.tool === "coil"}
          <path d="M0 7h6M16 7h6M8 1q-4 6 0 12M14 1q4 6 0 12" />
        {:else if p.tool === "box"}
          <path d="M0 7h5M17 7h5" /><rect x="5" y="1" width="12" height="12" rx="1" />
        {:else}
          <path d="M0 4h22M4 4v7M18 4v7M4 11h14" />
        {/if}
      </svg>
    </button>
  {/each}
  <span class="separator"></span>
  {#each modifiers as mod (mod.modifier)}
    <button
      class="icon-button mark"
      class:active={hasModifier(element, mod.modifier)}
      title={mod.title()}
      aria-label={mod.title()}
      aria-pressed={hasModifier(element, mod.modifier)}
      disabled={!editable || !takesModifier(element, mod.modifier)}
      onclick={() => onModifier(mod.modifier)}>{mod.mark}</button
    >
  {/each}
  <span class="separator"></span>
  <button
    class="icon-button"
    title={m.ladder_delete()}
    aria-label={m.ladder_delete()}
    disabled={!editable || !selected}
    onclick={() => onTool("delete")}><Trash2 size={14} /></button
  >
  <button
    class="icon-button"
    title={m.ladder_undo()}
    aria-label={m.ladder_undo()}
    disabled={!canUndo}
    onclick={() => onTool("undo")}><Undo2 size={14} /></button
  >
  <button
    class="icon-button"
    title={m.ladder_redo()}
    aria-label={m.ladder_redo()}
    disabled={!canRedo}
    onclick={() => onTool("redo")}><Redo2 size={14} /></button
  >
</div>

<style>
  .tools {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 3px 6px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-2);
  }
  svg path,
  svg rect {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
  }
  .mark {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 12px;
  }
  .separator {
    width: 1px;
    height: 18px;
    margin: 0 4px;
    background: var(--color-border);
  }
</style>
