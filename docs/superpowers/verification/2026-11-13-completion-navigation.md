# Verification: completion as in PLC IDEs, and navigation (ROADMAP 4.10)

Date: 2026-11-13. Spec `2026-11-13-completion-navigation-design.md`, plan `2026-11-13-completion-navigation.md`.

## What was measured

- **Before the spec**, with a throwaway probe on the 4.4a line's `Main.st` (spec, "What exists"):
  - truST's completion offers a call's parameters one by one inside its brackets, and `carry` at a statement's start
    as a bare name: no call template;
  - definition, references and rename answer on the analysis engine;
  - a standard FB's definition is an empty range at 0:0;
  - an undeclared name is E101, and the quick fix that declares it is private to trust-lsp.
- **App:** `cargo test -p twinova`, 95/95. The new tests:
  - `count`'s definition is line 28; TON has none to go to;
  - `count`'s four references, and which one writes;
  - a rename at a place, and its refusal in a block;
  - call templates in declaration order:
    - TON: IN, PT, Q, ET, none of them mandatory;
    - a user function: its inputs and in-out mandatory, its output not;
    - LIMIT, a standard function: all mandatory;
  - Auto Declare's proposed type for a literal (INT, BOOL, TIME, REAL, DINT#, 16#), for a declared variable (DINT),
    and for nothing else;
  - Monaco's places against byte offsets in both directions, with characters of two UTF-16 units.
- **Unit:** `navigation.test.ts` and `backend.test.ts`; vitest 233/233.
- **E2E on Linux,** `e2e/navigation.mjs`, 6/6:
  - F12 on `count` lands on line 28;
  - Shift+F12's peek lists 4 references;
  - F2 renames `count` to `cartons` on lines 28, 40 and 42, leaving the comment's "count", and the line builds;
  - typing `carry(` and Enter writes `carry(IN := , PT := , Q => , ET => )` with the cursor on IN's value; the
    mandatory parameters give `carry()`;
  - `speed := 3;` shows E101; the quick fix opens the bar with INT; Apply declares `speed : INT;`; the line builds.

  Regressions: `editor_st` 6/6, `editor` 10/10, `monitor` 7/7, `breakpoints` 15/15, `plc_window` 6/6.

## Measurements that changed the design

- **The template is offered at the typed `(`** (plan, decision 3), not as two more items of every completion list. One
  signature is computed per `(`, on the text as typed.
- **The references' peek needs Monaco's standalone contribution** (plan, decision 6). With contrib's controller alone,
  which is abstract, the editor had no `editor.contrib.referencesController` and Shift+F12 did nothing.

## What this does not prove

- **A definition or a reference across a program file and a block:** the 4.4a program calls no block.
- **A template for a method or an action call**, and TIA's toggle of the optional parameters in a call already written.
- **A type proposed for any expression:** only literals and declared variables.
- **That truST's snippets went in as text before:** the fix follows Monaco's API (`InsertAsSnippet`); the old
  behaviour was not run.
- **Windows,** until CI's windows-2022 E2E.
