<script lang="ts">
  // A Ladder file of the PLC drawn as its rungs (spec 2026-11-13-ladder-editor-design.md, decision 4): read into its
  // tree by Rust, lowered as Build lowers it for the boxes' types and pins and for what keeps a rung from lowering, and
  // laid out (ladder/layout.ts). Editing comes with the plan's Task 6 next steps; the text stays editable meanwhile.
  import { m } from "../paraglide/messages.js";
  import { ladder } from "./backend";
  import { debounce } from "./analysis";
  import { buildArgs, type Source } from "./sources";
  import { layoutRungs } from "./ladder/layout";
  import { lineOf, resolver, rungErrors } from "./ladder/view";
  import type { Lowering, ParseError, Program } from "./ladder/model";
  import LadderRung from "./LadderRung.svelte";

  let { source, sources }: { source: Source; sources: Source[] } = $props();
  let program = $state<Program | null>(null);
  let lowering = $state<Lowering | null>(null);
  let unreadable = $state("");

  const read = debounce((file: string, text: string, files: [string, string][]) => {
    ladder
      .parse(text)
      .then((tree) => {
        program = tree;
        unreadable = "";
        return ladder.lowering(files, file);
      })
      .then((found) => (lowering = found))
      .catch((e: ParseError | string) => {
        unreadable =
          typeof e === "string" ? e : m.ladder_unreadable({ line: lineOf(text, e.offset), message: e.message });
      });
  }, 200);
  $effect(() => read.call(source.key, source.text, buildArgs(sources).program));

  const layouts = $derived(program ? layoutRungs(program.rungs, resolver(lowering)) : []);
</script>

<div class="ladder" role="region" aria-label={m.ladder_view({ file: source.label })}>
  {#if unreadable}<p class="note" role="status">{unreadable}</p>{/if}
  {#if program}
    {#each program.rungs as rung, i (i)}
      {@const errors = rungErrors(lowering, i)}
      <section class="rung" aria-label={m.ladder_rung({ number: i + 1 })}>
        <header>
          <span class="number">{i + 1}</span>
          {#if rung.label}<span class="label">{rung.label}:</span>{/if}
          {#if rung.comment}<span class="comment">{rung.comment}</span>{/if}
        </header>
        {#if layouts[i]}<LadderRung layout={layouts[i]} />{/if}
        {#each errors as error, k (k)}<p class="error">{error}</p>{/each}
      </section>
    {/each}
  {/if}
</div>

<style>
  .ladder {
    height: 100%;
    overflow: auto;
    padding: 6px 10px 24px;
    background: var(--color-bg);
  }
  .rung {
    border-bottom: 1px solid var(--color-border);
    padding: 4px 0 8px;
  }
  header {
    display: flex;
    gap: 8px;
    align-items: baseline;
    font-size: 11px;
    color: var(--color-text-muted);
  }
  .number {
    font-family: var(--font-mono);
    min-width: 2ch;
  }
  .label {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--color-text);
  }
  .comment {
    white-space: pre-wrap;
    color: var(--color-text-faint);
  }
  .note,
  .error {
    margin: 4px 0;
    font-size: 11px;
    color: var(--color-warning);
  }
</style>
