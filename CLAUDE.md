# CLAUDE.md

Guida per Claude Code su questo repository. Regole derivate da `chetto1983/Aura` attraverso
`chetto1983/ModbusTool` (CLAUDE.md), adattate a Twinova: simulatore di fabbrica 3D e strumento di
messa in servizio virtuale, open source. Lo stack previsto, da confermare con le prove di
`docs/research/08-inventario-rust.md`, è nucleo in **Rust**, editor in **TypeScript** e logica dei
componenti in **IEC 61131-3** (Structured Text e Ladder), non Python: chi programma PLC non lo conosce
(decisione dell'utente, 2026-11-07; `docs/research/09-iec61131-runtime.md`). Oggi il repository contiene la ricerca (`docs/research/00-indice.md`), la prima app desktop (`app/`, M1) e la
fondazione di CI e release (M2); lo stato delle fasi è in `ROADMAP.md`.

## Principio: misura, poi documenta, poi implementa

Una decisione architettonica si scrive **dopo** una misura reale, non su una supposizione.
Si misura (prova contro un PLC o un simulatore vero, build reale, benchmark, run dell'app), poi
si registra la misura nella spec, poi si implementa. Quando una misura contraddice la spec,
**vince la misura** e la spec si corregge citando data ed evidenza. Ogni misura dichiara anche
**cosa NON dimostra**: un numero senza il suo perimetro è un'altra supposizione travestita.

Corollario: **una suite unitaria verde non chiude niente.** La prova è l'E2E vero: scena in
esecuzione, scambio di I/O con un PLC reale o emulato (PLCSIM Advanced, OpenPLC), comportamento
verificato.

## Clean-room

Twinova prende idee dai simulatori commerciali (Visual Components, SIMIT, Emulate3D, FlexSim, …),
mai il loro materiale.

- Le funzionalità si descrivono **con parole proprie**. Non si copiano testo, codice, immagini,
  icone, librerie di componenti o formati di file proprietari.
- Fonti ammesse: documentazione pubblica, paper, standard, codice open source con licenza
  compatibile.
- Non si aggirano login, paywall o blocchi anti-bot: in quei casi si chiede all'utente.
- Il nome e i marchi dei prodotti compaiono solo per descriverli o confrontarli.
- Prima di dipendere da codice di terzi si legge la licenza. MIT, Apache-2.0 e BSD vanno bene.
  MPL e LGPL solo come dipendenza non modificata. GPL e AGPL solo come processo esterno.

## Workflow: spec → piano → esecuzione → verifica

- Una spec per decisione in `docs/superpowers/specs/YYYY-MM-DD-<tema>-design.md`.
- Un piano per implementazione in `docs/superpowers/plans/YYYY-MM-DD-<tema>.md`. Sezioni:
  - `Goal`, `Architecture`, `Tech Stack`, `Spec`;
  - `Global Constraints`, `Review Focus`, `Decisions taken while writing this plan`;
  - `File structure`;
  - task a checkbox con passi TDD e comandi di verifica.
- Esecuzione task per task, un commit atomico per task.
- Ogni run di accettazione lascia un file datato in `docs/superpowers/verification/`: cosa ha
  misurato e cosa non dimostra. Una spec chiusa senza il suo file di verifica non è chiusa.

### 3 gate per ogni task

| Gate | Cosa |
|---|---|
| **1 — Definition of Ready** (prima) | Domande aperte chiuse, acceptance verificabile da macchina, test plan, file target ≤600 LOC |
| **2 — Implementation Q&A** (durante) | Gate di validazione verdi (sotto), refactor-on-touch, niente TODO orfani, 3-strike rule |
| **3 — Definition of Done** (dopo) | Acceptance spuntata, E2E reale verde, coverage ≥85%, mutation ≥70% sui file critici, spec aggiornata, file di verifica scritto |

**Niente shortcut.** Niente "lo aggiusto dopo".

## Behavioral rules (apply to every change)

- **READ, THEN PROBE, THEN IMPLEMENT.** No code, CI change or test edit before these steps, in
  this order:
  1. **Read** the installed source and the documentation of every component involved.
  2. **Reproduce** the failure locally, under the conditions where it fails.
  3. **Measure** the candidate fix in a throwaway copy (a git worktree outside the repo).

  For a CI-only failure, first list how the runner differs from the local run, then try each
  difference locally, one at a time:
  - user (root or not);
  - locale (`LANG`);
  - environment variables exported by setup actions;
  - OS image;
  - library versions.

  A push is not an experiment: never loop "add a diagnostic, push, wait for CI".
- **NEVER SUPPOSE.** Read code before editing. If uncertain about an API contract, stop and ask.
- **READ THE DOCUMENTATION FIRST — ALWAYS.** Before writing a line against any external system or
  library, read its documentation: the PLCSIM Advanced API, S7, OPC UA, Modbus, nexosim, Rapier,
  Tauri, the IEC 61131-3 runtime, three.js. Probing to discover an API is guessing with extra steps. Cite the
  page in a code comment when the behaviour is surprising.
- **INVENTORY BEFORE INVENTION.** Before writing a component, enumerate what the libraries you
  already depend on ALREADY DO. Read the public symbol list of the INSTALLED version
  (`~/.cargo/registry/src/*/<crate>-<ver>/src`, `node_modules/<pkg>`), not the landing page.
  Answer BUILD vs REUSE with a grep over the dependency, never with an assumption in either
  direction.
- **STOP BEFORE BESPOKE. ASK, DO NOT ASSUME.** The moment you are about to write a custom
  component, adapter, wrapper or protocol implementation against a dependency: STOP. Post the
  inventory you actually ran and the gap you believe it leaves, and confirm with the human
  BEFORE a line is written. "The docs do not mention it" is NOT evidence of absence.
- **NOT MY WORK.** Bug or gap found → fix on touch. Never skip.
- **READ BEFORE EDIT.** Re-read a file you haven't touched in the last 5 messages.
- **3-STRIKE RULE.** Same failing approach max 3 times. On strike 3, stop and ask.
- **NEVER MODIFY TESTS TO MAKE THEM PASS** unless the test itself is broken. Fix the code, or
  rewrite the test with explicit justification in the commit message.
- **SCOPE CONTROL.** Do exactly what was asked. No unrequested features, refactors or improvements.
- **FOLLOW EXISTING PATTERNS.** Never invent new approaches when codebase patterns exist.
- **NO GOD FILE.** Never create a file >600 LOC (Rust, TS, ST). Split by concern
  (`<name>_<concern>.rs`, sub-modules). Remove dead code and unused params. DARK CODE IS FORBIDDEN.
- **CODE BASE RULES.** Clean, readable, as simple as the behaviour allows. In case of doubt:
  > Beautiful is better than ugly. Explicit is better than implicit. Simple is better than complex.
  > Complex is better than complicated. Flat is better than nested. Sparse is better than dense.
  > Readability counts. Special cases aren't special enough to break the rules. Although
  > practicality beats purity. Errors should never pass silently. Unless explicitly silenced.
  > In the face of ambiguity, refuse the temptation to guess. There should be one -- and
  > preferably only one -- obvious way to do it. If the implementation is hard to explain, it's a
  > bad idea. If the implementation is easy to explain, it may be a good idea.
- **REUSABLE CODE.** Never duplicate; extract a helper.
- **DEEP REFACTOR ON TOUCH.** Every file you edit gets, in the SAME commit:
  - dead-code removal;
  - duplication folding;
  - LOC ≤600;
  - updated comments.
- **NO COMMENTS UNLESS WHY IS NON-OBVIOUS.** Names explain what. Comments only for hidden
  constraints, workarounds, surprising behaviour (e.g. a PLCSIM Advanced quirk, with the manual
  page).
- **NO TEST BABYSITTING.** Use realistic fixtures (frames from the protocol specifications, real
  PLC captures). Use property-based tests for codecs, data-type conversions and the event queue.
  No sleeps as sync.
- **DETERMINISM IS TESTED.** The engine must replay identical inputs to identical states. Every
  engine change keeps a replay fingerprint test green, and simultaneous-event ordering is
  specified and tested.
- **NO SKIP-AS-GREEN IN CI.** A test that skips under `$CI` exercises nothing. Integration tests
  that need a resource must fail under `$CI` when it is missing, not skip.
- **COVERAGE FLOOR 85%.** No task closes below 85% measured coverage (Rust: `cargo llvm-cov`;
  TS: `vitest --coverage`). Report the combined figure, not a
  cherry-picked one.
- **MUTATION ≥70%** on the critical files of each task (Rust: `cargo mutants`): event scheduler,
  protocol codecs, data-type conversions, conveyor kinematics.
- **BASELINE OR DROP.** Any heuristic or optimisation must be measured against the plain behaviour
  on a reference scene, or it is dropped.
- **DEFINITION OF DONE.** A job is complete only when validated E2E on a real scenario: the scene
  runs and exchanges I/O with a real or emulated PLC, ideally checked against an independent
  reference.
- **GIT PUSH DISCIPLINE.** Push at the end of a task or completed job, then check that all CI runs
  on the pushed SHA are green. Never leave work only in a local branch. CI runs on pushes to `main`
  and on pull requests (the Aura scheme), so a working branch needs an open PR to be checked; a
  change made only of Markdown files or of `docs/` runs nothing (nothing in CI reads them).

## truST, the ST runtime

Policy: `docs/superpowers/specs/2026-11-07-trust-upstream-policy-design.md`.

- **Execution** (`trust-runtime`, `trust-runtime-core`): pinned at `6405915`, the last commit with the bytecode VM.
  Never move that `rev`, because upstream deleted the VM. Port an upstream fix by hand, with a test and the upstream
  commit cited.
- **Language frontend** (`trust-syntax`, `trust-hir`, `trust-ide`, `trust-wasm-analysis`): once Twinova depends on it
  directly, review upstream monthly and move its `rev` in a PR that lists the commits.

## Post-edit validation (Gate 2)

The set comes from `chetto1983/ModbusTool`; CI runs it in `.github/workflows/ci.yml` and
`mutation.yml`, and `lefthook.yml` runs the fast part as git hooks (`cd app && npm run hooks`).

After every Rust edit, in `app/src-tauri`:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` (CI: `cargo llvm-cov --fail-under-lines 85`; Windows is covered by the E2E, not by `cargo test`)
- engine core or protocol changes: `cargo mutants` (scope in `.cargo/mutants.toml`)

After every TypeScript or Svelte edit, in `app`:

- `npm run check` (svelte-check, zero warnings) and `npm run lint` (oxlint, zero warnings)
- `npm run format:check`, `npm run check:i18n`, `npm run deadcode` (knip), `npm run dup` (jscpd)
- `npm run coverage` (85 % thresholds in `vite.config.ts`) and `npm run build`
- `src/lib` helpers: `npm run mutation` (Stryker, breaks below 70 %)
- the real app: `npx tauri build --debug --no-bundle && npm run e2e` (tauri-driver; under `xvfb-run` on Linux)

Repository-wide: `bash scripts/check-file-size.sh` (600-LOC cap, also in CI).

Fix issues before moving on.

## Commit discipline

- One task = one atomic commit (or N with the atomicity stated in the plan).
- Conventional Commits: `type(scope): subject`, imperative subject + body explaining *why*.
- Behaviour changes update `CHANGELOG.md`; the section of a pushed `vX.Y.Z` tag is the release text.
- Co-Authored-By trailer per project convention.
- Spec-amendment commit before the code commit when a task revealed an architectural gap.

## Working with the user

- Reply in **Italian**. Code, comments, commit messages and the README are in **English**. Research
  and specs may be in Italian.
- The user is an experienced automation engineer (Siemens S7, TIA Portal, PLCSIM Advanced): give
  concrete, numeric answers.
- Check claims against sources and always say what is verified (and where) and what is a hypothesis.
