# Spec: the Ladder editor (LD), as the second language of the PLC pane

Date: 2026-11-13. Status: **approved** (user, 2026-11-13: "Approvata, procedi con il piano"), decisions 1 to 8.

The user, 2026-11-13: "poi passerei all'editor ladder". CLAUDE.md: the components' logic is IEC 61131-3 Structured
Text and Ladder. The PLC pane has an LD button, disabled ("Ladder is not available").

## The user's choices (2026-11-13)

- **Monitoring with the power flow along the wires**, not only the contacts' state.
- **A Twinova crate** holding Open Industry Project's Ladder model, ported to Rust, rather than trust-visual extended
  in the fork. This replaces two earlier choices:
  - a first version within trust-visual's limits;
  - trust-visual extended in the fork.
- **The research:**
  - 18: CODESYS LD2;
  - 19: OIP's Ladder;
  - 20: open-source Ladder editors.

## What exists (research 18, 19, 20; inventory 2026-11-13)

### Three bases weighed

- **truST's `trust-visual`** (MIT OR Apache-2.0).
  - A graph of nodes and edges with coordinates.
  - Contacts NO/NC; coils normal, negated, S, R; TON/TOF/TP, CTU/CTD/CTUD; GT/LT/EQ; ADD/SUB/MUL/DIV.
  - Lowered to an ST function block.
  - Measured: it compiles in Twinova, and lowers truST's example with its addresses.
  - No P/N, no other function block, no PLCopen, no monitoring wired.
  - Its profile says it covers no more (truST's spec 16, §3).
- **Open Industry Project** (MIT, TypeScript), research 19.
  - **Model:** a series/parallel tree with no positions, nested without limit.
  - **Elements:**
    - contacts NO/NC/P/N; coils normal, negated, S, R, P, N;
    - any function block or function as a box, the project's own too, with EN/ENO, bare pins and empty pins;
    - jumps forward, RETURN, a comment per rung.
  - **Lowering to ST:**
    - each element is evaluated once, left to right;
    - a condition is captured before a later write changes what it reads;
    - branches run in a fixed order;
    - P/N edges use hidden temporaries.
  - **PLCopen TC6 `<LD>` import and export,** checked against Beremiz's XSD. The round trip is semantic: behaviour
    compared scan by scan.
  - **Monitoring:** the power is recomputed in the view from the end-of-scan image, so it is exact for edges and
    comparisons, approximate within a scan. Toggle, write and force from the view.
  - **Tests:** about 130 unit tests, 30 golden cases, 10 E2E. The Palletizer demo is compared as ST and as LD over
    12,000 random scans.
  - **Its parser extends OIP's ST parser.** It cannot be taken as it is: it would duplicate truST.
- **nautilus** (Apache-2.0, Go plus Svelte 5 with `@xyflow/svelte`), research 20.
  - A series/parallel tree with automatic layout.
  - P/N, any function block; no jumps; coils only at a rung's right end.
  - A pure three-valued power-flow evaluator.
  - A Svelte 5 renderer, the app's own framework.
- **For ideas only** (GPL): OpenPLC Editor, Beremiz, LDmicro.

### How CODESYS LD2 is edited (research 18)

- **The layout:**
  - automatic, from the series/parallel structure;
  - no grid, no wire drawn by hand;
  - reverse flow and short circuits impossible by construction (TIA forbids them too).
- **Insertion markers,** as drop zones and with the keyboard:
  - after (a rectangle);
  - before (a diamond);
  - in parallel below or above (a triangle).
  - The same shortcut, Ctrl+K for a contact, goes in parallel on a triangle.
- **Variants are modifiers:** negated, P/N for contacts, S/R for coils. The toolbox holds only contact, coil, box and
  network.
- **Online:**
  - TRUE wires blue and thick, FALSE black and thick, unknown thin;
  - values beside the operands;
  - a written or forced value marked.
  - CODESYS says its power flow is recomputed in the IDE, not read from the PLC.

### Measured in the plan's Task 1 (probes outside the repository, 2026-11-13)

| What | Result |
|---|---|
| A second PROGRAM beside `Main` in `TestHarness::from_sources`, with no CONFIGURATION | both run every cycle, in the sources' order. `Ld` read a global `Main` had just written: 1 after the first cycle |
| Two programs' outputs, adjacent or interleaved (`%QX1.0`/`%QX1.1`, `%QD4`/`%QD8`/`%QD12`), in either order | every program's outputs survive |
| ST shaped as OIP's lowering: a seal-in rung, a TON box, a P contact through a hidden `R_TRIG` | compiles in truST. Start then Stop gives Motor TRUE then FALSE, Late TRUE after the preset, one pulse counted per rising edge |
| Power temporaries: 20 s of virtual time on the 4.4a line, release, best of 3 | as is 0.837 s; with 60 hidden BOOLs written each scan 0.800 s (within noise); with 200, 0.973 s (+16 %); the fingerprint is unchanged in all three |
| The `plcopen` crate (MIT, 0.3.1) on Beremiz's 5 projects with `<LD>` | each read, written back and read again; contacts, coils and blocks counted the same: 10/2/46, 10/2/46, 3/2/10, 1/0/11, 3/2/9 |

| truST's calls (Task 3, a probe in the crate's tests, not committed) | named arguments to standard functions compile and run: `GT(IN1 := a, IN2 := b)`, `ADD(IN1, IN2, IN3)` by name, `LIMIT(MN, IN, MX)`, `SEL`, `MOVE`, `NOT(t)`. `AND(...)` and `MOD(...)` as calls do not parse; `EN` and `ENO` are unknown parameters |
| What a box's head names (Task 4, a probe in the crate's tests, not committed) | the ST files compiled with a PROGRAM holding only a Ladder file's declarations: the runtime's `programs()` gives each variable's type (`T1` → `TON`), `function_blocks()` the standard and the project's function blocks with each pin's direction and type, `functions()` the project's functions, `stdlib().get()` the standard ones |
| A duplicate PROGRAM name across an ST file and a Ladder file (Task 4) | truST labels the error with both files, but both spans are in the ST file's bytes |

Consequences:
- An LD file is lowered to a PROGRAM of its own, listed after `Main.st`'s files: no CONFIGURATION is needed.
- The power temporaries stay in the design: one per element costs nothing measurable at a realistic count.
- Boxes are resolved by truST itself, from that compile: no declaration reader of Twinova's, and the engine needs
  nothing beyond the runtime it links. When the declarations do not compile, every box is lowered by its bare pins
  and the program's own compile reports the problem.
- A function is lowered as a call with named arguments, so the lowering needs no table of argument order. truST
  checks the pins' names, which keeps one compiler. AND, OR, XOR and MOD are written as operators. EN is an `IF`
  around the call, as in OIP.
- ~~PLCopen's XML is read and written with the `plcopen` crate.~~ Amended in Task 5 (2026-11-13), from the crate's
  generated types (`generated/generated_body.rs`, `Body_LD_Inline`): they keep an `<LD>` body's elements grouped by
  kind (every contact, then every coil…), so the order of the file is lost. OIP's reader orders rungs and branches by
  it and attaches a comment or a label to the network after it. So:
  - the body is read with `roxmltree` 0.21.2 (MIT OR Apache-2.0), already in the lockfile through truST's
    `trust-plcopen`, which keeps the document's order;
  - it is written as text, as OIP's writer writes it;
  - OIP's reduction from the graph to the series/parallel tree and its writer are ported as they are (decision 7).
- truST's `trust-plcopen` does not cover LD: its profile refuses graphical bodies ("No import/export for SFC/LD/FBD
  bodies").

What these do not show:
- the scan order across programs with a CONFIGURATION and several tasks;
- PLCopen's meaning, beyond its structure: the semantic round trip is Task 5's;
- the power temporaries' cost on a program of a thousand rungs.

## Decisions

1. **A Twinova crate, `twinova-ladder`** (Rust, in the app's workspace), holds:
   - the Ladder model, ported from OIP's, with MIT attribution in a NOTICE;
   - its lowering to ST;
   - its PLCopen TC6 `<LD>` reader and writer.

   truST compiles and runs only the ST it generates: one compiler, one set of diagnostics. Neither the fork nor
   trust-visual is touched.
2. **The format: a file of the PLC, `plc/<name>.ld`, in OIP's text grammar.**
   - Rungs are separated by `;`, a branch is `[a, b]`, and a comment goes above its rung. The text is versioned with
     git and diffs line by line.
   - Its variables are declared in an ST `VAR` block at its head, as the variables table reads them.
   - Build lowers each LD file to an ST program of the PLC. How it joins the configuration with `Main.st` is
     measured in Task 1.
3. **The element set of OIP's model,** in the first version:
   - contacts NO/NC/P/N;
   - coils normal, negated, S, R, P, N;
   - any function block (TON, CTU, R_TRIG, SR, the project's own…) and function as a box with its pins, and EN/ENO;
   - comparisons and arithmetic as boxes;
   - jumps forward, labels, RETURN;
   - a comment per rung.
4. **The editor in Svelte and SVG,** with an automatic layout from the tree, as LD2's. Interaction as LD2's:
   - insertion markers, which are also the drop zones while dragging;
   - one shortcut per element, in series or in parallel by the marker;
   - modifiers cycled by keys;
   - a box picker (Ctrl+B) over a catalogue built from the project's blocks and the standard library;
   - keyboard navigation along the insertion points;
   - rungs that can be selected, copied, pasted and commented;
   - undo and redo;
   - operands `???` to fill, with completion and Auto Declare (4.10).
   
   nautilus's Svelte renderer and evaluator (Apache-2.0) are read first. What fits is reused with NOTICE and
   attribution, split under 600 lines.
5. **The power flow is the runtime's, not recomputed.**
   - The lowering emits a hidden BOOL per element, its power out, set as the scan reaches it. (Amended in Task 3,
     2026-11-13: the spec said "power in". An element's power in is the power out of the one before it, but the
     power in of every element leaves the end of a branch's path undrawn: in `[|A| |B|, |C|] (X)`, no element's power
     in is `A AND B`. A coil, and a function behind EN, pass their power in on unchanged, so they take no BOOL of
     their own.)
   - Since each element's power is held as the scan reaches it, the lowering evaluates each element there and then.
     OIP keeps a rung's condition as an expression and captures it before a later write changes what it reads; with
     a BOOL per element that capture always holds, so it is not ported. The behaviour is the same, and OIP's cases
     are the tests.
   - The monitoring reads them through the existing watches, at the scan's end, as the other values. A wire is then
     drawn energised exactly as the program ran.
   - Neither OIP nor CODESYS does this; Twinova can, because its runtime is in the app.
   - The cost of the temporaries is measured (BASELINE OR DROP): a rung's scan with and without them, on a reference
     program.
6. **Online, as LD2's:**
   - energised wires thick and coloured, dead ones thick and dark, unknown ones thin;
   - values beside the operands and on the boxes' pins;
   - write and force from the view through the existing monitor (4.8);
   - colour is never the only sign: thickness tells too.
7. **PLCopen TC6 `<LD>` import and export,** ported from OIP's reader and writer.
   - The round trip is semantic, as OIP's: a test exports, imports and runs each fixture, scan by scan.
   - The `plcopen` crate (MIT) is weighed against the port in Task 1. It has no tests, so it is measured on files
     from Beremiz first.
8. **Left out of the first version:**
   - breakpoints and stepping in LD;
   - backward jumps;
   - LD ↔ ST conversion of the user's code;
   - ST blocks inside a rung;
   - free placement (CFC);
   - FBD.

## Test plan (outline)

- **The crate:**
  - OIP's cases ported as Rust tests: parse, lowering, edits, PLCopen;
  - each fixture's lowered ST run by truST, scan by scan, against expected values;
  - the power temporaries against the lowered ST.
- **Measurement:** a rung's scan with and without the power temporaries (decision 5).
- **App:** Build runs an LD program on the 4.4a line's I/O, beside or instead of `Main.st`.
- **E2E:**
  - draw a start/stop rung by dragging and by keys;
  - Build and Run, press Start on the panel;
  - the coil, the output and the wires up to it energised;
  - Stop drops them; a force from the LD view.

## What this would not prove

- PLCopen against CODESYS's and TIA's own exports: only Beremiz's are measured.
- Every FB of every library: the catalogue is the standard library's and the project's.
- The editor's speed on very long programs.
