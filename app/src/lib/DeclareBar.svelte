<script lang="ts">
  // Auto Declare, as CODESYS's (spec 2026-11-13-completion-navigation-design.md, decision 4): an undeclared name's
  // class, type, initial value and comment, added to the variables table on Apply. Opened by the quick fix on E101 or
  // Shift+F2.
  import { X } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { CLASSES, type Declaration } from "./monitor";
  import { declaration, type Proposal } from "./navigation";

  let {
    proposal,
    error = "",
    onApply,
    onClose,
  }: {
    proposal: Proposal;
    /** Why the last Apply was refused, such as a type that does not parse. */
    error?: string;
    onApply: (row: Declaration) => void;
    onClose: () => void;
  } = $props();

  let cls = $state("VAR");
  let typeName = $state("");
  let initial = $state("");
  let comment = $state("");
  $effect(() => {
    cls = "VAR";
    typeName = proposal.type_name ?? "";
    initial = "";
    comment = "";
  });
</script>

<div class="declare-bar" role="group" aria-label={m.declare_title()}>
  <strong class="data">{m.declare_action({ name: proposal.name })}</strong>
  <select aria-label={m.vars_class()} bind:value={cls}>
    {#each CLASSES as option (option)}<option value={option}>{option}</option>{/each}
  </select>
  <input class="type" aria-label={m.vars_type()} placeholder={m.vars_type()} bind:value={typeName} />
  <input class="initial" aria-label={m.vars_initial()} placeholder={m.vars_initial()} bind:value={initial} />
  <input class="comment" aria-label={m.vars_comment()} placeholder={m.vars_comment()} bind:value={comment} />
  <button
    disabled={typeName.trim() === ""}
    onclick={() => onApply(declaration(proposal, cls, typeName, initial, comment))}>{m.bp_apply()}</button
  >
  {#if error}<span class="error" role="alert">{error}</span>{/if}
  <button class="close" aria-label={m.declare_cancel()} onclick={onClose}><X size={12} /></button>
</div>

<style>
  .declare-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-2);
    font-size: 11px;
  }
  input,
  select {
    height: 24px;
    font-size: 11px;
  }
  strong {
    white-space: nowrap;
  }
  select {
    width: 120px;
  }
  .type,
  .initial {
    width: 90px;
  }
  .comment {
    flex: 1;
    min-width: 120px;
  }
  .error {
    color: var(--color-danger);
  }
  .close {
    display: inline-flex;
    border: 0;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }
</style>
