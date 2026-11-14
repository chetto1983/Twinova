# 20 — Editor e motori Ladder open source più avanzati di truST

Data: 2026-11-13. Richiesta dell'utente: "Cerca progetti più evoluti".

## Metodo

- **Come.** Ogni repository è stato clonato con `--depth 1` nella scratchpad della sessione, fuori da Twinova. Ho letto LICENSE, manifest e sorgenti.
- **Date e stelle:** istantanea del 2026-11-13.
- **Non ho potuto leggere** `IOFabrik/ladder-visualizer`, `ControlZebra/ladder-visualizer` e `wangqipov/PLC_LAD`: il clone chiede le credenziali.
- **Cosa non dimostra:**
  - nessun progetto è stato compilato o eseguito;
  - gli elementi sono stati letti nel codice e nei documenti.

## Licenze utilizzabili

### nautilus (joyautomation)

- **Riferimenti.** https://github.com/joyautomation/nautilus, **Apache-2.0**.
- **Stack.**
  - Runtime e compilatori in Go.
  - Diagrammi in **Svelte 5 + `@xyflow/svelte`** (`tools/vscode-iec/webview-ui`).
- **Attività:** ultimo commit 2026-11-13, 1.503 commit.
- **Elementi** (`lang/ld/ld.go`):
  - contatti NO, NC e di fronte, con istanza R_TRIG/F_TRIG implicita;
  - bobine normale, S, R, P, N;
  - FB generici con pin, standard e dell'utente (`lang/ld/catalog.go`);
  - funzioni come contatto, per esempio `GT(TempC, 90.0)`;
  - assegnazione `{ y := expr }`;
  - rami annidati, commenti, bit di word;
  - FB scritti in Ladder.
- **Non ha:**
  - jump e label;
  - bobine a metà rung: il testo dice "coils must sit at the rung's right end".
- **Modello e compilazione.**
  - Il formato è testo (`RUNG … [ a | b ] /Stop ( Run )`), con layout automatico (`ladderLayout.ts`).
  - Il lowering è LD → netlist FBD → ST.
- **Monitoraggio.** `webview-ui/src/ladder.ts` è un valutatore puro del power flow a tre valori: uno stato sconosciuto resta neutro. Ci sono valori live, force e toggle.
- **Interoperabilità:**
  - import Rockwell L5X;
  - export PDF e SVG;
  - PLCopen solo come lavoro futuro.
- **Test:** 15 file Go su LD, 65 test TS, una suite di conformità.
- **Per Twinova.** È lo stesso framework (Svelte 5).
  - Si possono usare modello, power flow e renderer, con NOTICE e attribuzione.
  - `LadderView.svelte` ha 1.846 righe: andrebbe spezzato.

### SoftLadder

- **Riferimenti.** https://github.com/jorgeviloria/softladder, **MIT OR Apache-2.0**.
- **Stack e attività.** Rust + egui; ultimo commit 2026-10-12, autore unico.
- **Origine.** Il NOTICE si dichiara "clean-room reimplementation" di ClassicLadder, che è LGPL. La dichiarazione non è verificata.
- **Elementi:**
  - contatti NO, NC e di fronte;
  - bobine normale, negata, S, R;
  - **jump e call**;
  - timer, contatori, compare, operate, FIFO/LIFO, PID.
- **Modello.**
  - Una griglia di celle con collegamenti verticali.
  - La semantica è normativa e scritta: `docs/SEMANTICS.md`, valutazione per colonne.
- **Contorno e test.** Monitor con power flow, replay deterministico, import `.clp`. 649 `#[test]`.
- **Per Twinova.**
  - Esegue la ladder direttamente e non genera ST, quindi non si allinea a truST.
  - Utile come oracolo o come riferimento semantico per una griglia.

### `plcopen` (radevgit/plc)

- **Riferimenti.** https://github.com/radevgit/plc, crate `plcopen` 0.3.1, **MIT**.
- **Cosa fa.** Tipi serde + quick-xml generati dall'XSD TC6 v2.01: LD, contatti, bobine, rail, blocchi.
- **Limite.** Non ha test, quindi il round-trip in scrittura non è dimostrato.
- **Per Twinova.** Candidato per l'import/export PLCopen XML LD in Rust, da misurare prima su file CODESYS/Beremiz.

### Minori (MIT)

- **ladderlib:** un interprete a celle in C.
- **hiperiondev/ladder-editor:** un unico file HTML generato con l'AI. Le sue icone SVG sono MIT.
- **cdilga/ladder-logic-editor:** solo ST → LD. Il senso inverso lancia "not yet implemented".
- **FUXA:** non ha Ladder.

## Solo idee (copyleft)

- **OpenPLC Editor nuovo** (Autonomy Logic, **GPL-3.0-only**).
  - È il più completo e il più attivo: Electron + React + `@xyflow/react`, 442 stelle.
  - Elementi: fronti P/N, bobine P/N/S/R, blocchi generici con pin.
  - Debugger con power flow e force.
  - LD→ST verificato byte per byte contro un oracolo; PLCopen XML; circa 724 test.
  - Da studiare per la UX del debugger, la tipizzazione dei pin e la fattorizzazione dei percorsi. **Nessuna riga va copiata.**
- **Beremiz / PLCopen Editor** (**GPL-2.0-or-later**; runtime LGPL e GPL).
  - Il formato nativo è PLCopen TC6 XML.
  - LD → ST in `PLCGenerator.py`; debug con propagazione dello stato.
  - È il riferimento per serializzare l'LD in PLCopen.
- **Vecchio OpenPLC Editor** (GPL-2.0): fork di Beremiz.
- **LDmicro** (**GPL-3.0**).
  - Ha il set di istruzioni più ricco, con label, goto e gosub.
  - Simulazione con power flow; ultimo commit nel 2021.
- **ClassicLadder** (LGPL-2.1+), **RuSTy** (LGPL-3.0, solo CFC, senza LD), **RustPLC** (editor GPL-3.0), **NiRuLogic** (AGPL-3.0).
  - Di NiRuLogic vale l'idea degli elementi definiti in JSON.
- **Senza licenza, quindi non utilizzabili:** `plc-ladder-lab`, `plc-ladder-st-simulator`, `Flood-Ladder`.

## Classifica dello studio

**Codice utilizzabile:**
1. nautilus (Apache-2.0, Svelte 5);
2. SoftLadder (MIT/Apache, riferimento semantico a griglia);
3. il crate `plcopen` (MIT, da misurare).

**Solo idee:**
- OpenPLC Editor nuovo;
- Beremiz;
- LDmicro.
