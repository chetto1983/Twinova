<script lang="ts">
  // PLCopen TC6 XML from the PLC pane (spec 2026-11-13-ladder-editor-design.md, decision 7), through the system
  // dialogs: the Ladder file shown, exported as the editor has it; a file's Ladder programs imported into the program's
  // tabs, a file it has replaced and a new one added, which Build runs and Save writes.
  import { FileDown, FileUp } from "@lucide/svelte";
  import { m } from "../paraglide/messages.js";
  import { ladder } from "./backend";
  import { withImported } from "./ladder/plcopen";
  import { buildArgs, type Source } from "./sources";

  let {
    sources = $bindable(),
    file,
    onShow,
    onNote,
  }: {
    sources: Source[];
    /** The Ladder file shown, or null. */
    file: string | null;
    /** Shows the tab of program file `key`. */
    onShow: (key: string) => void;
    onNote: (text: string) => void;
  } = $props();

  const fail = (e: unknown): void => onNote(String(e));

  function exportFile(): void {
    if (!file) return;
    ladder
      .exportPlcopen(buildArgs(sources).program, file)
      .then((target) => {
        if (target !== null) onNote(m.plcopen_exported({ file: target }));
      })
      .catch(fail);
  }

  function importFile(): void {
    ladder
      .importPlcopen()
      .then((imported) => {
        if (imported === null) return;
        if (imported.files.length === 0) return onNote([m.plcopen_nothing(), ...imported.notes].join(" · "));
        const next = withImported(sources, imported.files);
        sources = next.sources;
        onShow(imported.files[0]?.[0] ?? "");
        const done = [
          ...(next.replaced.length > 0 ? [m.plcopen_replaced({ files: next.replaced.join(", ") })] : []),
          ...(next.added.length > 0 ? [m.plcopen_imported({ files: next.added.join(", ") })] : []),
        ];
        onNote([...done, m.plcopen_build(), ...imported.notes].join(" · "));
      })
      .catch(fail);
  }
</script>

<button class="icon-button" title={m.plcopen_import()} aria-label={m.plcopen_import()} onclick={importFile}
  ><FileUp size={13} /></button
><button
  class="icon-button"
  title={m.plcopen_export()}
  aria-label={m.plcopen_export()}
  disabled={!file}
  onclick={exportFile}><FileDown size={13} /></button
>

<style>
  /* the size of the debug bar's, beside them in the PLC pane's header */
  button {
    width: 24px;
    height: 24px;
  }
</style>
