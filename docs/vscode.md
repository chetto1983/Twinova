# ST in VS Code

A Twinova project is a folder, and its Structured Text sources are files in it. You can edit them in VS Code, with
truST's extension, while Twinova runs the line. Spec: `superpowers/specs/2026-11-12-st-vscode-design.md`.

## Set up

1. Install [VS Code](https://code.visualstudio.com/) and the **truST LSP** extension (`trust-platform.trust-lsp`) from
   the Marketplace.
2. In Twinova, open the project folder, or save the project to one. A project without a folder (the demo, an imported
   archive, a new project) has nothing on disk to follow until it is saved.
3. Open the same folder in VS Code.

## What happens when you save in VS Code

| You save | Twinova |
|---|---|
| `plc/*.st`, the line program | Changes the PLC's program **online**. The plant, the cartons and the components go on. The PLC restarts warm: `RETAIN` variables and the time are kept, every other variable starts again from its initial value, and the outputs hold until the new program's first scan. |
| `components/*.st`, a component's behaviour | Rebuilds the line from zero. If it was running, it runs again. |
| A file that does not compile | Changes nothing: the line runs on with the last good program, and the editor shows the error with its file and line. |

The online change is a warm restart, like a download in STOP → RUN on an S7 (OB100). It is not a download that keeps
every value. A state that must survive a change, such as a latched motor or a count, is declared `VAR RETAIN`.

If a file also has edits in Twinova's own editor that are not built yet, Twinova takes nothing from the disk and says
which file. Build in Twinova to keep its text, or Revert the tab to take the disk's at its next save. A tab with edits
of its own is marked with a dot.

## The files Twinova writes

| File | Why |
|---|---|
| `trust-lsp.toml` | It is written once, then it is yours. It limits indexing to the project's sources: `.twinova/previous` holds copies from the last save, which would declare everything twice. It also keeps the index cache in `.twinova`, and turns off the nondeterminism warnings, which flag every `AT` variable and `TIME` of a simulated line. |
| `.twinova/lsp/field.st` | The program and configuration Twinova generates for the components, so the extension sees each block's instances and their `VAR_CONFIG`. |

Neither is part of the project's manifest, so Twinova does not read them, and Save leaves them alone.

## Known limits

- **Three false E202 errors in each component's block** ("direct address `%I*` requires VAR_CONFIG mapping"). truST
  checks a block's `AT %I*` against the `VAR_CONFIG` of the same file only, while Twinova compiles the blocks and their
  configuration as one source. The line program has no such errors; Build is the referee.
- **Measured versions.** Only trust-lsp 0.24.75 (the version Twinova's runtime is pinned to) and 0.38.0 (the
  Marketplace's on 2026-11-12) were measured; on Twinova's fixtures they give the same diagnostics.
- **Files outside the manifest.** A new `.st` file is not part of the project until Twinova adds it, though the
  extension reads it.
- **Calls between the program and the blocks.** VS Code sees the program and the blocks as one project, but Twinova
  compiles them apart. A program that calls a behaviour block passes in VS Code and fails at Build.
- **Live values and the debugger** in VS Code are not connected to Twinova yet (ROADMAP 4.8).
