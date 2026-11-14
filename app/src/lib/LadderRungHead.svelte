<script lang="ts">
  // A rung's comment, label and exit typed in its header (spec 2026-11-13-ladder-editor-design.md, decision 4: a comment
  // per rung, labels, jumps forward and RETURN). Enter in a field takes them, Ctrl+Enter in the comment too, Escape
  // leaves them as they were.
  import { untrack } from "svelte";
  import { m } from "../paraglide/messages.js";
  import type { RungHead } from "./ladder/edit";

  let {
    head,
    number,
    onCommit,
    onCancel,
  }: { head: RungHead; number: number; onCommit: (head: RungHead) => void; onCancel: () => void } = $props();

  // the fields start from the rung as it was and are the user's from then on
  let typed = $state(untrack(() => ({ ...head })));
  let first: HTMLInputElement | undefined = $state();
  $effect(() => first?.focus());

  function onkeydown(e: KeyboardEvent): void {
    e.stopPropagation();
    const comment = (e.target as HTMLElement).tagName === "TEXTAREA";
    if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    } else if (e.key === "Enter" && (!comment || e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      onCommit(typed);
    }
  }
</script>

<div class="head" role="group" aria-label={m.ladder_head({ number })}>
  <label
    ><span>{m.ladder_label()}</span><input
      bind:this={first}
      bind:value={typed.label}
      spellcheck="false"
      {onkeydown}
    /></label
  >
  <label
    ><span>{m.ladder_exit()}</span><input
      bind:value={typed.exit}
      {onkeydown}
      placeholder={m.ladder_exit_hint()}
      spellcheck="false"
    /></label
  >
  <label class="comment"
    ><span>{m.ladder_comment()}</span><textarea bind:value={typed.comment} rows="2" {onkeydown}></textarea></label
  >
  <div class="actions">
    <button onclick={onCancel}>{m.ladder_cancel()}</button>
    <button class="primary" onclick={() => onCommit(typed)}>{m.ladder_ok()}</button>
  </div>
</div>

<style>
  .head {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 12px;
    margin: 4px 0;
    padding: 6px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-surface-2);
    font-size: 11px;
  }
  label {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--color-text-muted);
  }
  label span {
    min-width: 64px;
  }
  input,
  textarea {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .comment {
    grid-column: 1 / -1;
    align-items: flex-start;
  }
  .actions {
    grid-column: 1 / -1;
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
</style>
