<script lang="ts">
  // An element's operand typed in place, over the drawing (spec 2026-11-13-ladder-editor-design.md, decision 4): the
  // names that fit as you type, from truST (ladder/operand.ts). Enter takes the name picked with the arrows, or the
  // text; Tab fills in the picked name or the first; Escape leaves the operand as it was; leaving the field takes the text, as TIA's and CODESYS's fields do.
  import { untrack } from "svelte";
  import { m } from "../paraglide/messages.js";
  import { ladder } from "./backend";
  import { debounce } from "./analysis";
  import { candidates, completedPart, type Role } from "./ladder/operand";

  let {
    value,
    role,
    file,
    text,
    onCommit,
    onCancel,
  }: {
    value: string;
    role: Role;
    file: string;
    text: string;
    onCommit: (operand: string) => void;
    onCancel: () => void;
  } = $props();

  // the field starts from the operand as it was and is the user's from then on
  let typed = $state(untrack(() => value));
  let names = $state<string[]>([]);
  let picked = $state(-1);
  let field: HTMLInputElement | undefined = $state();
  let done = false;

  const ask = debounce((typedNow: string) => {
    const start = completedPart(typedNow, role) ?? "";
    if (start === "") {
      names = [];
      return;
    }
    ladder
      .complete(file, text, start)
      .then((found) => {
        names = candidates(found, start, role).slice(0, 12);
        // nothing picked until an arrow picks it: Enter takes the text, a new name too
        picked = -1;
      })
      .catch(() => (names = []));
  }, 120);
  $effect(() => ask.call(typed));
  $effect(() => {
    field?.focus();
    field?.select();
  });

  function finish(operand: string | null): void {
    if (done) return;
    done = true;
    if (operand === null) onCancel();
    else onCommit(operand);
  }

  function onkeydown(e: KeyboardEvent): void {
    e.stopPropagation();
    const name = names[picked];
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const by = e.key === "ArrowDown" ? 1 : -1;
      picked = names.length === 0 ? -1 : (picked + by + names.length) % names.length;
    } else if (e.key === "Tab" && names.length > 0) {
      e.preventDefault();
      typed = name ?? (names[0] as string);
    } else if (e.key === "Enter") {
      e.preventDefault();
      finish(name ?? typed);
    } else if (e.key === "Escape") {
      e.preventDefault();
      finish(null);
    }
  }
</script>

<div class="operand">
  <input
    bind:this={field}
    bind:value={typed}
    aria-label={m.ladder_operand()}
    spellcheck="false"
    autocomplete="off"
    {onkeydown}
    onblur={() => finish(typed)}
  />
  {#if names.length > 0}
    <ul role="listbox" aria-label={m.ladder_candidates()}>
      {#each names as name, i (name)}
        <!-- mousedown, not click: the field must not lose its focus, which would take the text first -->
        <li
          role="option"
          aria-selected={i === picked}
          class:picked={i === picked}
          onmousedown={(e) => {
            e.preventDefault();
            finish(name);
          }}
        >
          {name}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .operand {
    position: absolute;
    z-index: 3;
    min-width: 120px;
  }
  input {
    width: 100%;
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px 4px;
  }
  ul {
    margin: 2px 0 0;
    padding: 2px 0;
    list-style: none;
    max-height: 180px;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    background: var(--color-surface-2);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  li {
    padding: 2px 6px;
    cursor: pointer;
  }
  li.picked {
    background: var(--color-accent-soft);
    color: var(--color-ring);
  }
</style>
