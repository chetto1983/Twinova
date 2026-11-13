# Spec: completion as in PLC IDEs, and navigation (ROADMAP 4.10)

Date: 2026-11-13. Status: **proposed**, decisions 1 to 5 awaiting the user's approval.

ROADMAP 4.10:
- an FB or function call inserted with all its parameters (`:=` inputs, `=>` outputs), optional ones reducible;
- declare an undeclared variable;
- go to definition, references, rename.

Acceptance: E2E on each. Whether truST already provides call templates is measured before any code.

## What exists (inventory and measurements, 2026-11-13)

### What the two IDEs do (research 17, §1 and §6)

- **TIA Portal:**
  - an instruction inserted from the catalogue arrives with its operands' placeholders, mandatory and optional in two
    shades, Tab to the next;
  - Ctrl+Shift+Space shows or hides the optional ones.
- **CODESYS:**
  - "Insert with arguments" puts every formal parameter, `:=` for inputs and `=>` for outputs;
  - Auto Declare (Shift+F2) asks scope, type, initial value and comment;
  - Go to Definition and the Cross Reference List.
- Research 17's conclusion: the FB call with all its parameters, optional ones reducible, and Auto Declare. The `#` and
  `"` prefixes are Siemens SCL, not IEC 61131-3, and are not copied.

### truST's frontend at v0.39.0, already in the app (read in the installed source)

- **`trust-wasm-analysis`**, the engine the editor already uses for diagnostics, hover and completion, also answers
  `definition`, `references` (with `is_write`), `rename` and `document_highlight`.
- **`trust-ide`:**
  - completion inside a call's brackets offers each parameter not yet given, as `NAME := $0` or `NAME => $0`
    (`completion/engine.rs`, `parameter_name_completions`);
  - a standard function completes as `NAME($0)`;
  - `call_signature_info` gives a call's name and its parameters with their directions.
- **No call template** with all the parameters: not in `trust-ide`, not in `trust-wasm-analysis`.
- **No "declare variable" in the analysis crates.**
  - `trust-lsp` has a quick fix "Create VAR declaration" for E101 (`diagnostic_fixes.rs`, `missing_var_text_edit`), but
    it is private to the server crate (`pub(in super::super)`).
  - Its type is inferred, or INT.
- **Twinova already uses truST's frontend directly:**
  - `declarations.rs` renames a variable from the variables table through `trust_ide::rename::rename_checked`, on every
    source the editor holds;
  - `edit_declaration` adds a row (`Change::Add`);
  - `monitor.rs` builds a `trust_hir` project for the inline values.

### Measured (a throwaway probe in a worktree outside the repository, the 4.4a line's `Main.st`)

| What | Result |
|---|---|
| Completion inside `carry(IN := …, PT := …)` | only the parameters not given: `ET => $0` (OUT : TIME), `Q => $0` (OUT : BOOL); then typed literals |
| Completion after a new `carry(` | `ET =>`, `IN :=`, `PT :=`, `Q =>`, in alphabetical order, each with its direction and type |
| Completion of `car` at a statement's start | `carry`, inserted as `carry`: no call |
| Definition of `count` on line 40 | its declaration, line 28 |
| References of `count` | 4: the declaration, the write and the read on line 40, the read on line 42; each says whether it writes |
| Rename of `count` | 4 edits, all in `Main.st` |
| Definition of `TON` (a standard FB) | `plc/Main.st` 0:0, an empty range: the standard library has no source |
| `speed := 3;` with `speed` undeclared | E101 "undefined identifier 'speed'" at its line and column |

**What these do not show:**
- a call's parameters in declaration order (`call_signature_info` was read, not run);
- references and rename across a program file and a block;
- the analysis's speed on a larger program.

## Decisions

1. **Navigation from truST.**
   - Go to definition (F12, Ctrl+click), references (Shift+F12, Monaco's peek list), and the highlight of a symbol's
     occurrences come from `trust-wasm-analysis`: the engine already holds the program, the blocks and the field's
     configuration.
   - A definition with no source, such as a standard FB's (measured: 0:0), goes nowhere.
   - A block's tab navigates too: it is read-only, not closed.
2. **Rename (F2) in the code, as the variables table renames.**
   - The same path: `rename_checked` on every source the editor holds (`declarations.rs`), at the cursor instead of at a
     table row.
   - A rename that would change a block or the field's generated configuration is refused: blocks are read-only (4.8).
3. **Call templates: built by Twinova on truST's signature.**
   - Completing an FB instance or a function at a statement's start offers two more items:
     - `carry(…)` with every parameter;
     - `carry(…)` with the mandatory ones only.
   - Inputs and in-outs take `:=`, outputs `=>`. Each value is a Monaco tab stop, so Tab moves to the next.
   - The parameters come in declaration order, from `trust_ide::call_signature_info` on the text with the call opened
     (`carry(`). That order is measured in Task 1.
   - Mandatory follows IEC 61131-3: an FB's VAR_IN_OUT and every input of a function. The rest is optional.
   - TIA's toggle of the optional parameters inside a call already written is left out: the two items give the same
     choice when the call is inserted.
4. **Auto Declare, as CODESYS's, through the variables table's edit.**
   - On an E101 (measured), a quick fix (the light bulb, Ctrl+.) and Shift+F2 on the name open a bar above the code:
     name, class, type, initial value, comment.
   - The class is one of those the variables table offers.
   - The type is proposed from the other side of an assignment:
     - a literal gives its own type: TRUE or FALSE → BOOL, T#… → TIME, an integer → INT, a real → REAL;
     - an identifier gives the type truST's hover tells;
     - otherwise the field is empty and must be filled.
   - Apply adds the row through `edit_declaration` (`Change::Add`), as the table does.
   - In the program's files only, since blocks are read-only.
5. **Left out:**
   - CODESYS's short declaration (`bStart b` and Ctrl+Enter);
   - "Update inconsistent block calls" and the refactorings;
   - TIA's Call options dialog (single or multi-instance), which creates instance DBs, a Siemens concept.

## Test plan

- **App (Rust):**
  - the signature of `carry` (TON) and of a function, in declaration order;
  - the template text, all parameters and mandatory only;
  - a rename at a position, and its refusal when a block would change;
  - the proposed type for each kind of right-hand side.
- **Unit (vitest):** the template as a Monaco snippet; the quick fix's bar fields to `Change::Add`.
- **E2E, a new `e2e/navigation.mjs`:**
  - F12 on `count` jumps to line 28;
  - Shift+F12 lists 4 references;
  - F2 renames `count` to `cartons` and the line builds;
  - typing `carry` and taking the template inserts `carry(IN := , PT := , Q => , ET => );` with the cursor on IN, and
    the mandatory-only one inserts `carry();`;
  - `speed := 3;` shows E101; the quick fix proposes INT; Apply declares it; the problem goes and the line builds.

## What this would not prove

- A template for a method or an action call.
- References across projects or libraries.
- A type inferred for any expression: only literals and identifiers.
