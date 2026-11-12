# Spec: the in-app ST editor (ROADMAP 4.7)

Date: 2026-11-12. Status: **draft**, for the user's approval.

ROADMAP 4.7: "In-app ST editor for who has no VS Code (classrooms): Monaco with diagnostics, hover and completion from
truST's `BrowserAnalysisEngine` (`trust-wasm-analysis`) through Tauri commands". It is accepted by:
- the same diagnostics as `trust-lsp` on the default program and on a broken one;
- Monaco measured on WebKitGTK;
- the frontend `rev` moved as the 2.6 policy says.

## What exists (inventory and measurements, 2026-11-12)

### The editor today

- **The redesigned workspace's PLC pane** (`Editor.svelte`) has:
  - an organizer of programs and function blocks;
  - tabs;
  - a `<textarea>` with a hand-made line gutter;
  - Revert.
- **The Problems pane** (`WorkspaceDock.svelte`) lists Build's error and warnings only.
- **Nothing analyses the text while it is typed.**

### truST's analysis engine

- **`trust-wasm-analysis` is a plain Rust library;** wasm is an optional feature. `BrowserAnalysisEngine` offers:
  - `replace_documents` and `diagnostics`;
  - `hover` and `completion`;
  - `definition`, `references`, `document_highlight` and `rename`.

  Positions are `{line, character}`, with byte spans.
- **It is already compiled:** `trust-runtime` depends on it at the pin (`Cargo.lock`).
- **At the pin it gives exactly `trust-lsp` 0.24.75's diagnostics before `trust-lsp.toml`'s filter.** Measured with a
  throwaway test on `two_belts_diverter`:

  | Input | Diagnostics, pin |
  |---|---|
  | `plc/Main.st` | 13: W009, 3 × W001, 9 × W011 |
  | `Main.st` broken | 16: the same plus E203 21:1, E101 22:1, E205 23:19, as `trust-lsp` reports them |
  | the three blocks | 12, 8 and 12, with 3, 2 and 3 × E202 |
  | the blocks with `.twinova/lsp/field.st` | 11, 7 and 11: no W009 |

### The frontend at upstream's v0.39.0 (`5eb7f8c`, 2026-11-12)

- **It builds next to the pinned runtime.** Measured: the probe at `rev = "5eb7f8c…"` compiled in 116 s,
  incrementally.
  - Four crates are compiled twice, at 0.24.75 and 0.39.0: `trust-syntax`, `trust-hir`, `trust-ide`,
    `trust-wasm-analysis`.
  - `Cargo.lock` grows by 67 lines.
- **`trust-wasm-analysis` has the same public API at the pin and at 0.39.0** (a diff of every `pub fn`, `struct` and
  `enum`).
- **148 commits touch the four crates since the pin:** trust-syntax 15, trust-hir 135, trust-ide 27,
  trust-wasm-analysis 2. The 2.6 policy has them read and listed in the PR that moves the `rev`.
- **The diagnostics at 0.39.0 are quieter, and the errors are the same:**

  | Input | Diagnostics, 0.39.0 |
  |---|---|
  | `plc/Main.st` | 4: W009, 3 × W001. W011 no longer flags a plain `AT` variable, and W010 has gone |
  | `Main.st` broken | 7: the same E203, E101 and E205, at the same places |
  | the blocks | E202 and W011 on `AT %I*`, as at the pin |

- **VS Code's Marketplace extension is 0.39.0 too,** the version the user's Windows test used for 4.6. The in-app editor
  at 0.39.0 and VS Code read the same frontend.

### Monaco on WebKitGTK (measured, a throwaway worktree)

`monaco-editor` 0.57.0 (MIT, published 2026-09-24) replaced the textarea: only the editor core
(`editor/editor.api.js`) and its worker, through Vite's `?worker`. It ran in the debug app under Xvfb at 1440×900:

| What | Result |
|---|---|
| Mount | 108 ms; the program is drawn with line numbers, minimap and cursor |
| The editor worker | starts from the app's own protocol and is alive after 2 s; no script error |
| Typing | reaches the project's source; the tab is marked edited; Build takes the text |
| Bundle | JS 816 KB → 3,524 KB (gzip 224 → 920 KB), CSS 31 → 130 KB, a 304 KB worker |

- **WebKitWebDriver drops some spaces from typed keys,** in a plain input too ("a b c" arrives as "a bc"). This is a
  driver quirk, not Monaco's. Text set through Monaco's API keeps them.
- **Monaco draws spaces as no-break spaces (U+00A0).** A test that reads the view must normalise them.

### Highlighting

- Monaco highlights through Monarch tokenizers; TextMate grammars need extra libraries (`vscode-textmate` and an
  Oniguruma wasm).
- truST's VS Code extension ships `syntaxes/st.tmLanguage.json`: 4.7 KB, 8 rules, MIT OR Apache-2.0.

## Decisions (user, 2026-11-12)

1. **Monaco 0.57.0,** the editor core only, as measured.
2. **Highlighting by a Monarch tokenizer translated from truST's `st.tmLanguage.json`** (MIT OR Apache-2.0, cited):
   - keywords and types;
   - comments `(* *)` and `//`;
   - strings and numbers, `T#` literals included;
   - IEC addresses (`%IX0.0`, `%Q*`).
3. **The diagnostics shown are VS Code's,** with the same filter as the `trust-lsp.toml` Twinova writes (no W010 or
   W011). The false E202 "requires VAR_CONFIG mapping" are left out where Twinova's generated configuration places
   that variable. The real errors stay.
4. **Diagnostics, hover and completion,** the ROADMAP's three. Definition, references and rename stay for later.

Proposed by this spec, for approval:

5. **The frontend moves to v0.39.0 (`5eb7f8c`) for the editor only.** The runtime stays at `6405915` (2.6). A construct
   the new frontend accepts and the pinned VM refuses still fails at Build, which remains the referee.
6. **One analysis project, as VS Code's workspace:**
   - it holds the program's files, the blocks, and the field's generated configuration, which is not shown;
   - the webview sends the open texts at most every 300 ms of quiet typing;
   - Rust answers with each file's diagnostics.
7. **The Problems pane lists the analysis's diagnostics of every file, then Build's.** A click opens the file at its
   line.

## The design

### Rust (`twinova` app, `src/analysis.rs`)

- **A `BrowserAnalysisEngine` (0.39.0) held in a managed `Mutex`.**
- **`analyze(program, behaviours) -> BTreeMap<file, Vec<Diagnostic>>`:**
  1. replace the documents: the program's paths, `components/<key>.st` for the blocks, and the field's configuration
     from `field::config_source`;
  2. collect each shown file's diagnostics;
  3. filter them (decision 3).
- **`hover(file, position)` and `completion(file, position)`** on the last texts analysed.
- **The filter is a pure function:**
  - it drops W010 and W011;
  - it drops an E202 when the variable its range names is placed by the configuration's `VAR_CONFIG` for every instance
    of that block;
  - it is unit-tested, and in the mutation scope.
- **Positions.** The engine's `{line, character}` and Monaco's 1-based line and column differ. Whether a character
  counts bytes or UTF-16 units is measured in the plan's first task on a line with a non-ASCII comment, before the
  conversion is written.

### The webview

- **`Editor.svelte` mounts Monaco** in place of the textarea. Its model per tab follows `sources`, so the decisions of
  4.6 (edited tabs, Revert, held disk changes) are unchanged.
- **`st.ts` (Monarch)** registers the `st` language and its tokenizer.
- **`analysis.ts`:**
  - the debounce;
  - the markers from the diagnostics;
  - hover and completion providers calling the Tauri commands;
  - unit-tested with Stryker.
- **The Problems pane** shows the analysis's diagnostics and Build's.
- **The theme follows the app's light and dark,** with two Monaco themes built from the Aura tokens.

### E2E

- **The E2E's `setProgram` sets the text through Monaco's model, not keys,** because of the driver's spaces. It reaches
  the editor through a `data-` hook on its host element, set only in debug builds. Whether that is acceptable, or the
  E2E reads Monaco through its global instead, is settled in the plan.
- **`e2e/editor_st.mjs`:**
  - open the demo: no error, and Main's diagnostics as `trust-lsp` 0.39.0 lists them with Twinova's `trust-lsp.toml`;
  - break the program: E203, E101 and E205 appear as markers and in Problems before any Build;
  - hover a variable: its type;
  - completion offers a declared variable;
  - the blocks show no E202.

## Acceptance

| # | Criterion | Evidence |
|---|---|---|
| 1 | The same diagnostics as `trust-lsp` | On every valid fixture and on the broken diverter, the editor's list equals the probe's on `trust-lsp` 0.39.0 with Twinova's `trust-lsp.toml`, minus the false E202 (decision 3); a script compares them |
| 2 | Hover and completion | `analysis.rs` tests and `e2e/editor_st.mjs`: a variable's type on hover, a declared variable and a keyword offered |
| 3 | Monaco on WebKitGTK | the E2E on Linux, and windows-2022 in CI; the mount time, the bundle and the idle CPU of the release build (`npm run soak`) compared with `main` |
| 4 | The frontend `rev` | `rev = 5eb7f8c` for the four crates; the PR lists the 148 commits; runtime and E2E unchanged |
| 5 | Highlighting | the Monarch tokens of each fixture's sources checked against a table of expected token types |
| 6 | The 4.6 behaviour kept | `e2e/vscode.mjs` 5/5: disk changes, held edits, Revert |
| 7 | Gates | fmt, clippy, coverage ≥ 85 %, cargo mutants on `analysis.rs`, Stryker ≥ 70 % on `analysis.ts` and `st.ts`, svelte-check, oxlint, knip, jscpd, i18n |

## What this will not prove

- **That the new frontend and the pinned VM agree on every construct.** Build stays the referee; a construct only the
  frontend accepts shows as a Build error.
- **Definition, references and rename,** for later.
- **Ladder,** ROADMAP 6.1.
- **Monaco on macOS.**
- **Typing through the keyboard in the E2E,** because the driver drops spaces.
