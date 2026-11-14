<script lang="ts">
  // A Ladder file of the PLC drawn as its rungs and edited there (spec 2026-11-13-ladder-editor-design.md, decision 4):
  // read into its tree by Rust, lowered as Build lowers it for the boxes' types and pins and for what keeps a rung from
  // lowering, and laid out (ladder/layout.ts). An edit is a transform on the tree (ladder/edit.ts) that Rust prints back
  // as the file's text, so the text and the drawing never disagree.
  import { m } from "../paraglide/messages.js";
  import { ladder } from "./backend";
  import { debounce } from "./analysis";
  import { buildArgs, type Source } from "./sources";
  import { layoutRungs } from "./ladder/layout";
  import { keyAction, lineOf, resolver, rungErrors, selectionOf } from "./ladder/view";
  import { addRung, elementAt, modify, place, removeSelected, step } from "./ladder/edit";
  import type { Edit, Modifier, Selection } from "./ladder/edit";
  import { History } from "./ladder/history";
  import type { Lowering, ParseError, Program } from "./ladder/model";
  import LadderRung from "./LadderRung.svelte";
  import LadderToolbar, { type Tool } from "./LadderToolbar.svelte";

  let { source, sources, onText }: { source: Source; sources: Source[]; onText: (text: string) => void } = $props();
  // the tree of the text it was read from: an edit starts only from the text on screen, never from a stale tree
  let read = $state<{ key: string; text: string; program: Program } | null>(null);
  let lowering = $state<Lowering | null>(null);
  let unreadable = $state("");
  let failed = $state("");
  let selection = $state<Selection | null>(null);
  const histories = new Map<string, History>();
  let changes = $state(0);

  const reread = debounce((file: string, text: string, files: [string, string][]) => {
    ladder
      .parse(text)
      .then((program) => {
        read = { key: file, text, program };
        unreadable = "";
        return ladder.lowering(files, file);
      })
      .then((found) => (lowering = found))
      .catch((e: ParseError | string) => {
        unreadable =
          typeof e === "string" ? e : m.ladder_unreadable({ line: lineOf(text, e.offset), message: e.message });
      });
  }, 200);
  $effect(() => reread.call(source.key, source.text, buildArgs(sources).program));

  const program = $derived(read?.key === source.key ? read.program : null);
  const editable = $derived(!!read && read.key === source.key && read.text === source.text && !unreadable);
  const layouts = $derived(program ? layoutRungs(program.rungs, resolver(lowering)) : []);
  const history = $derived.by(() => {
    const known = histories.get(source.key) ?? new History();
    histories.set(source.key, known);
    return known;
  });
  // History is plain: `changes` counts its moves, so these follow them
  const canUndo = $derived(changes >= 0 && history.canUndo);
  const canRedo = $derived(changes >= 0 && history.canRedo);

  async function commit(edit: Edit): Promise<void> {
    if (!editable || !program) return;
    const next: Program = { declarations: program.declarations, rungs: edit.rungs };
    let text: string;
    try {
      text = await ladder.print(next);
      failed = "";
    } catch (e) {
      failed = String(e);
      return;
    }
    selection = edit.selection;
    if (text === source.text) return;
    history.record(source.text);
    changes++;
    read = { key: source.key, text, program: next };
    onText(text);
  }

  function travel(back: boolean): void {
    const text = back ? history.undo(source.text) : history.redo(source.text);
    if (text === null) return;
    changes++;
    selection = null;
    onText(text);
  }

  function onTool(tool: Tool): void {
    if (tool === "undo" || tool === "redo") return travel(tool === "undo");
    if (!program) return;
    if (tool === "rung") void commit(addRung(program.rungs, selection?.rung ?? program.rungs.length - 1));
    else if (selection && tool === "delete") void commit(removeSelected(program.rungs, selection));
    else if (selection && tool !== "delete") void commit(place(program.rungs, selection, tool));
  }

  function onModifier(modifier: Modifier): void {
    if (program && selection) void commit(modify(program.rungs, selection, modifier));
  }

  function onclick(e: MouseEvent): void {
    const target = e.target as globalThis.Element;
    if (target.closest("[role=toolbar]")) return;
    const rung = target.closest<HTMLElement>("[data-rung]")?.dataset.rung;
    selection = selectionOf(rung, target.closest("[data-path]")?.getAttribute("data-path") ?? undefined);
  }

  function onkeydown(e: KeyboardEvent): void {
    const action = keyAction(e);
    if (!action) return;
    e.preventDefault();
    if (action === "undo" || action === "redo" || action === "delete") return onTool(action);
    if (action === "escape") selection = null;
    else if (program && selection) selection = step(program.rungs, selection, action === "left" ? -1 : 1) ?? selection;
  }
</script>

<!-- An application in ARIA's sense, a diagram whose keys the view handles itself (WAI-ARIA 1.2, role application);
     Svelte counts the role among the landmarks, so it asks for neither a tab stop nor listeners on it. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="ladder"
  role="application"
  aria-label={m.ladder_view({ file: source.label })}
  tabindex="0"
  {onclick}
  {onkeydown}
>
  <LadderToolbar
    {editable}
    selected={!!selection}
    element={program && selection ? elementAt(program.rungs, selection) : null}
    {canUndo}
    {canRedo}
    {onTool}
    {onModifier}
  />
  {#if unreadable || failed}<p class="note" role="status">{unreadable || failed}</p>{/if}
  {#if program}
    {#each program.rungs as rung, i (i)}
      {@const errors = rungErrors(lowering, i)}
      <section
        class="rung"
        class:picked={selection?.rung === i && selection.path.length === 0}
        data-rung={i}
        aria-label={m.ladder_rung({ number: i + 1 })}
      >
        <header>
          <span class="number">{i + 1}</span>
          {#if rung.label}<span class="label">{rung.label}:</span>{/if}
          {#if rung.comment}<span class="comment">{rung.comment}</span>{/if}
        </header>
        {#if layouts[i]}<LadderRung layout={layouts[i]} selected={selection?.rung === i ? selection.path : null} />{/if}
        {#each errors as error, k (k)}<p class="error">{error}</p>{/each}
      </section>
    {/each}
  {/if}
</div>

<style>
  .ladder {
    height: 100%;
    overflow: auto;
    padding: 0 0 24px;
    background: var(--color-bg);
    outline: none;
  }
  .rung {
    border-bottom: 1px solid var(--color-border);
    border-left: 3px solid transparent;
    padding: 4px 10px 8px 7px;
  }
  .rung.picked {
    border-left-color: var(--color-ring);
    background: var(--color-accent-soft);
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
    margin: 4px 10px;
    font-size: 11px;
    color: var(--color-warning);
  }
</style>
