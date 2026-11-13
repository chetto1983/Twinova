# Plan: completion as in PLC IDEs, and navigation (ROADMAP 4.10)

## Goal

Go to definition, references, occurrences and rename in the ST editor; FB and function calls inserted with their
parameters; undeclared variables declared from the code.

## Architecture

- **App, `navigation.rs`** (new, + tests): commands over the analysis engine the editor already feeds (`analysis.rs`):
  - `definition`: a source-less definition (0:0, empty) is none;
  - `references` and `highlights`, with `is_write`.
- **App, `declarations.rs`:**
  - `rename` is split so the table's rename (by row) and the code's (by position) share one path through
    `rename_checked`;
  - `rename_at` refuses edits to a block or to the field's configuration.
- **App, `call_templates.rs`** (new, + tests):
  - `call_template(file, text, line, column, name)` runs `trust_ide::call_signature_info` on the text with `name(`
    opened at the cursor, in a `trust_hir` project of the editor's sources, as `declarations.rs` and `monitor.rs` build
    one;
  - it gives the two snippets, all parameters and mandatory only (spec decision 3).
- **App, `declare_proposal`:** the E101 name at a position and the proposed type (spec decision 4). The literal rule is
  local; an identifier's type comes from truST's hover.
- **Webview, `StEditor.svelte`:**
  - Monaco's definition, reference, highlight, rename and code action providers, with their contributions imported;
  - `registerEditorOpener` switches the tab when a definition is in another file;
  - completion items as snippets (`InsertAsSnippet`), and the two template items after an FB instance or a function.
- **Webview, `DeclareBar.svelte`** (new): the Auto Declare bar, applied through `declarations.edit` (`Change::Add`).
  `Editor.svelte` opens it from the quick fix and from Shift+F2.

## Tech Stack

- truST's frontend at v0.39.0 (`5eb7f8c`): `trust-wasm-analysis` (`definition`, `references`, `document_highlight`),
  `trust-ide` (`call_signature_info`, `rename::rename_checked`), `trust-hir`.
- Monaco 0.57 (`gotoSymbol`, `rename`, `codeAction` and `snippet` contributions; `registerEditorOpener`).
- Svelte 5.

## Spec

`docs/superpowers/specs/2026-11-13-completion-navigation-design.md`: approved by the user on 2026-11-13 ("Facciamo
tutto"), decisions 1 to 5.

## Global Constraints

- **Blocks stay read-only:** they navigate, but are never edited.
- **Files** stay under 600 lines. `Editor.svelte` is at 591: the bar goes in a component of its own.
- **Gates** as for 4.9. The E2E runs locally, and in CI on Linux and Windows.

## Review Focus

- **Positions:** Monaco's 1-based lines and UTF-16 columns against the engine's 0-based ones (`analysis.rs`, `position`).
- **The synthetic call:** the template is computed on a copy of the text with `name(` opened; that copy never reaches
  the tab.
- **Snippets:** truST's existing completion items carry `$0` and `${1:…}`. Without `InsertAsSnippet` they go into the
  text as typed (measured in Task 1).

## Decisions taken while writing this plan

1. **The rename at a position reuses `declarations.rs`'s path** (FOLLOW EXISTING PATTERNS), not
   `trust-wasm-analysis`'s `rename`: one rename, one refusal rule.
2. **The quick fix opens the bar; it does not edit the text itself.** The class and type are the user's choice, as in
   CODESYS's Auto Declare.

## File structure

| File | Change |
|---|---|
| `src-tauri/src/navigation.rs` (+ tests), `analysis.rs`, `lib.rs` | definition, references, highlights |
| `src-tauri/src/declarations.rs` (+ tests) | `rename_at`, the shared rename |
| `src-tauri/src/call_templates.rs` (+ tests) | call templates, declare proposal |
| `src/lib/backend.ts` (+ tests), `navigation.ts` (+ tests) | commands; snippets and template items |
| `src/lib/StEditor.svelte`, `DeclareBar.svelte`, `Editor.svelte` | providers, opener, the bar |
| `messages/{en,it}.json` | strings |
| `e2e/navigation.mjs`, `package.json` | the E2E |
| docs | verification, CHANGELOG, ROADMAP |

## Tasks

- [ ] **Task 1: the app (Rust).** Tests first, then a commit.
  - Measure: `call_signature_info`'s order for TON and for a function; a reference from `Main.st` into a block; whether
    a truST snippet goes in as typed today.
  - Then:
    - `definition`, `references` and `highlights`;
    - `rename_at` and its refusal;
    - `call_template` and `declare_proposal`.
- [ ] **Task 2: the webview.** `navigation.ts` tests first, then the providers, snippets, templates, opener and bar.
  Commit.
- [ ] **Task 3: the E2E**, `navigation.mjs` (spec test plan), locally once. Commit.
- [ ] **Task 4: documents.** Verification, CHANGELOG, ROADMAP. The pull request, merged when green.
