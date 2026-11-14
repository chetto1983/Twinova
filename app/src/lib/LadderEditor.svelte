<script lang="ts">
  // A Ladder file of the PLC drawn as its rungs and edited there (spec 2026-11-13-ladder-editor-design.md, decision 4):
  // read into its tree by Rust, lowered as Build lowers it for the boxes' types and pins and for what keeps a rung from
  // lowering, and laid out (ladder/layout.ts). An edit is a transform on the tree (ladder/edit.ts) that Rust prints back
  // as the file's text, so the text and the drawing never disagree.
  import { m } from "../paraglide/messages.js";
  import { declarations, ladder } from "./backend";
  import { debounce } from "./analysis";
  import { buildArgs, type Source } from "./sources";
  import { layoutRungs } from "./ladder/layout";
  import { keyAction, lineOf, resolver, rungErrors, selectionOf } from "./ladder/view";
  import {
    addRung,
    elementAt,
    insertElement,
    modify,
    place,
    removeSelected,
    replaceElement,
    setOperand,
    step,
  } from "./ladder/edit";
  import { declarable, declared, operandOf, roleOf } from "./ladder/operand";
  import type { CatalogEntry } from "./ladder/catalog";
  import type { Edit, Modifier, Selection } from "./ladder/edit";
  import { History } from "./ladder/history";
  import type { Element, Lowering, ParseError, Program } from "./ladder/model";
  import LadderRung from "./LadderRung.svelte";
  import LadderToolbar, { type Tool } from "./LadderToolbar.svelte";
  import LadderOperand from "./LadderOperand.svelte";
  import LadderPicker from "./LadderPicker.svelte";

  let { source, sources, onText }: { source: Source; sources: Source[]; onText: (text: string) => void } = $props();
  // the tree of the text it was read from: an edit starts only from the text on screen, never from a stale tree
  let read = $state<{ key: string; text: string; program: Program } | null>(null);
  let lowering = $state<Lowering | null>(null);
  let unreadable = $state("");
  let failed = $state("");
  let selection = $state<Selection | null>(null);
  // the element whose operand is being typed, and a contact's or coil's operand typed that nothing declares
  let editing = $state<Selection | null>(null);
  let undeclared = $state<string | null>(null);
  let view: HTMLDivElement | undefined = $state();
  // the box picker, open on its catalogue and the names the file declares
  let picker = $state<{ entries: CatalogEntry[]; taken: string[]; problem: string } | null>(null);
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

  const selected = $derived(program && selection ? elementAt(program.rungs, selection) : null);
  const pou = $derived(source.key.replace(/^.*\//, "").replace(/\.ld$/, ""));

  /** The file's text after `edit`, given to the editor; null when it changed nothing or could not print. */
  async function commit(edit: Edit): Promise<string | null> {
    if (!editable || !program) return null;
    const next: Program = { declarations: program.declarations, rungs: edit.rungs };
    let text: string;
    try {
      text = await ladder.print(next);
      failed = "";
    } catch (e) {
      failed = String(e);
      return null;
    }
    selection = edit.selection;
    if (text === source.text) return null;
    keep(text);
    read = { key: source.key, text, program: next };
    return text;
  }

  /** `text` as the file's, the one it replaces kept for Undo. */
  function keep(text: string): void {
    history.record(source.text);
    changes++;
    onText(text);
  }

  /** The one box `call` reads as (`T1(IN, PT := T#2s)`), read by the grammar Rust reads files with. */
  async function boxOf(call: string): Promise<Element> {
    const tree = await ladder.parse(`${call};`).catch((e: ParseError) => Promise.reject(e.message));
    const [only, ...more] = tree.rungs[0]?.elements ?? [];
    if (only?.kind !== "box" || more.length > 0 || tree.rungs.length !== 1) throw m.ladder_not_a_box({ call });
    return only;
  }

  async function typedIn(at: Selection, operand: string): Promise<void> {
    editing = null;
    view?.focus();
    if (!program) return;
    if (roleOf(elementAt(program.rungs, at)) === "box") {
      try {
        await commit(replaceElement(program.rungs, at, await boxOf(operand)));
      } catch (e) {
        failed = String(e);
      }
      return;
    }
    const text = await commit(setOperand(program.rungs, at, operand));
    const name = operand.trim();
    if (text === null || !declarable(name)) return;
    // Auto Declare (4.10): what truST knows by this name, the file's variables and the globals, is declared already
    const known = await ladder.complete(source.key, text, name).catch(() => []);
    const same = known.some((c) => c.kind === "variable" && c.label.toLowerCase() === name.toLowerCase());
    undeclared = same ? null : name;
  }

  async function declare(name: string, type = "BOOL"): Promise<void> {
    undeclared = null;
    try {
      keep(await declarations.edit(source.key, source.text, { op: "add", row: declared(pou, name, type) }));
    } catch (e) {
      failed = String(e);
    }
  }

  async function openPicker(): Promise<void> {
    if (!editable || !selection) return;
    try {
      const [entries, rows] = await Promise.all([
        ladder.catalog(buildArgs(sources).program),
        declarations.of(source.key, source.text),
      ]);
      picker = { entries, taken: rows.map((r) => r.name), problem: "" };
    } catch (e) {
      failed = String(e);
    }
  }

  /** A box picked: placed by the selection, and a block's instance declared when nothing declares it yet. */
  async function picked(choice: { entry: CatalogEntry | null; head: string; pins: string }): Promise<void> {
    const open = picker;
    if (!program || !selection || !open) return;
    try {
      const box = await boxOf(`${choice.head}(${choice.pins})`);
      picker = null;
      view?.focus();
      const text = await commit(insertElement(program.rungs, selection, box));
      const known = open.taken.some((t) => t.toLowerCase() === choice.head.toLowerCase());
      if (text !== null && choice.entry?.block && !known) await declare(choice.head, choice.entry.name);
    } catch (e) {
      picker = { ...open, problem: String(e) };
    }
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
    else if (tool === "box") void openPicker();
    else if (selection && tool === "delete") void commit(removeSelected(program.rungs, selection));
    else if (selection && tool !== "delete") void commit(place(program.rungs, selection, tool));
  }

  function onModifier(modifier: Modifier): void {
    if (program && selection) void commit(modify(program.rungs, selection, modifier));
  }

  function onclick(e: MouseEvent): void {
    const target = e.target as globalThis.Element;
    if (target.closest("[role=toolbar], [role=dialog], .operand, .declare")) return;
    const rung = target.closest<HTMLElement>("[data-rung]")?.dataset.rung;
    selection = selectionOf(rung, target.closest("[data-path]")?.getAttribute("data-path") ?? undefined);
  }

  function onkeydown(e: KeyboardEvent): void {
    const action = keyAction(e);
    if (!action) return;
    e.preventDefault();
    if (action === "undo" || action === "redo" || action === "delete" || action === "box") return onTool(action);
    if (action === "edit") editing = roleOf(selected) ? selection : null;
    else if (action === "escape") selection = null;
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
  bind:this={view}
  {onclick}
  ondblclick={() => (editing = roleOf(selected) ? selection : null)}
  {onkeydown}
>
  <LadderToolbar {editable} selected={!!selection} element={selected} {canUndo} {canRedo} {onTool} {onModifier} />
  {#if picker}
    <LadderPicker
      entries={picker.entries}
      taken={picker.taken}
      problem={picker.problem}
      onInsert={picked}
      onClose={() => {
        picker = null;
        view?.focus();
      }}
    />
  {/if}
  {#if unreadable || failed}<p class="note" role="status">{unreadable || failed}</p>{/if}
  {#if undeclared}
    {@const name = undeclared}
    <p class="declare" role="status">
      {m.ladder_undeclared({ name })}
      <button onclick={() => declare(name)}>{m.ladder_declare_bool({ name })}</button>
      <button aria-label="✕" onclick={() => (undeclared = null)}>✕</button>
    </p>
  {/if}
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
        {#if layouts[i]}
          {@const layout = layouts[i]}
          {@const at = editing?.rung === i ? editing : null}
          {@const node = at && layout.nodes.find((n) => "path" in n && n.path.join() === at.path.join())}
          <div class="drawing">
            <LadderRung {layout} selected={selection?.rung === i ? selection.path : null} />
            {#if at && node && program}
              {@const el = elementAt(program.rungs, at)}
              {@const role = roleOf(el)}
              {#if role}
                <div
                  class="place"
                  style="left: {node.x - 4}px; top: {node.y - 20}px; width: {Math.max(
                    node.w + 40,
                    role === 'box' ? 300 : 0,
                  )}px"
                >
                  <LadderOperand
                    value={operandOf(el)}
                    {role}
                    file={source.key}
                    text={source.text}
                    onCommit={(operand) => typedIn(at, operand)}
                    onCancel={() => {
                      editing = null;
                      view?.focus();
                    }}
                  />
                </div>
              {/if}
            {/if}
          </div>
        {/if}
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
  .drawing,
  .place {
    position: relative;
  }
  .place {
    position: absolute;
  }
  .declare {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 4px 10px;
    font-size: 11px;
    color: var(--color-warning);
  }
  .declare button {
    font-size: 10px;
  }
  .note,
  .error {
    margin: 4px 10px;
    font-size: 11px;
    color: var(--color-warning);
  }
</style>
