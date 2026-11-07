# La logica dei componenti in IEC 61131-3: cosa esiste già

Stato: **inventario, 2026-11-07**. Decisione dell'utente (2026-11-07): niente Python per la logica dei componenti,
perché chi programma PLC non lo conosce. Si usa **IEC 61131-3**, cioè Structured Text e Ladder.

## Metodo, e cosa prova e cosa non prova

- **Ricerca:** GitHub search API (`search/repositories`, ordinata per stelle) con query su runtime, compilatori e
  interpreti IEC 61131-3, Ladder e PLCopen XML; crates.io per i crate Rust (08).
- **Letto nel sorgente:** di truST ho letto README, `crates/trust-runtime/src/scheduler/clock.rs`,
  `docs/public/concepts/deterministic-harness.md` e `docs/specs/10-runtime-semantics.md` (clone del 2026-11-07,
  commit `89b382b`). Di RuSTy ho letto il `Cargo.toml` (08).
- **Non prova:** niente è stato compilato o eseguito. La conformità allo standard e le prestazioni non sono
  misurate. Il numero di contributori di truST non è verificato, perché il clone è shallow e la search API dei
  commit rifiuta le query senza testo.

## Candidati

| Progetto | Licenza | Linguaggio | Stelle (2026-11-07) | Attività | Cosa fa | Giudizio |
|---|---|---|---|---|---|---|
| **truST Platform** (`johannesPettersson80/trust-platform`) | **MIT OR Apache-2.0** | Rust (~1,3 M righe in 32 crate) | 228, 26 fork | creato 2026-02-03, push 2026-11-07 | ST con OOP; editor visuali **Ladder, SFC, Blockly** con un compagno ST; profili **Siemens SCL**, CODESYS, TwinCAT, GX Works3; PLCopen XML; LSP; debugger; Modbus, OPC UA, ADS, MQTT, EtherCAT; librerie OSCAT e PLCopen Motion; **harness deterministico con tempo virtuale** | **miglior candidato**: stessa licenza e stesso linguaggio di Twinova |
| RuSTy (`PLC-lang/rusty`) | LGPL-3.0 | Rust, LLVM | 373 | dal 2019, push 2026-11-06 | compilatore ST → nativo via LLVM | solo come processo esterno; niente WASM |
| OpenPLC v3 (`thiagoralves/OpenPLC_v3`) | GPL-3.0 | C++ (matiec) | 1 579 | push 2026-04-04 | runtime completo: ST, LD, FBD, IL, SFC; Modbus | solo come processo esterno (GPL), come soft-PLC di test |
| echidna (`61131/echidna`) | BSD-2-Clause | C | 71 | push 2024-12-05 | compilatore e VM IEC 61131-3 | fermo da quasi un anno |
| PLC di Open Industry Project | MIT | TypeScript | – | attivo (07) | ST e LD, PLCopen XML, gira **nel browser** | riferimento per la versione web |
| `iec61131`, `iecst`, `plcopen` (crates.io) | MIT | Rust | < 1 000 download | 2025-12 | solo parser e analisi statica | parser al massimo |
| matiec, Beremiz, esstee | GPL-3.0 | C/C++ | ≤ 44 | – | transpiler ST → C, IDE, interprete | GPL, esclusi |

## truST nel dettaglio: perché combacia con Twinova

1. **Il tempo è iniettabile.** Il runtime pianifica su un trait `Clock` (`now`, `sleep_until`, `wake`). Ci sono
   due implementazioni: `StdClock`, sull'orologio reale, e `ManualClock`, sul tempo virtuale
   (`scheduler/clock.rs`).
2. **L'harness è già un lockstep.** `trust-harness` carica sorgenti ST ed esegue questi comandi:
   - `cycle`, che esegue un ciclo;
   - `set_input` e `get_output`, per gli ingressi e le uscite;
   - `advance_time`, che fa avanzare il tempo virtuale;
   - `run_until`, che fa girare finché una condizione è vera.

   Il protocollo è NDJSON. È lo stesso schema del lockstep con PLCSIM Advanced (05 §6): Twinova comanda il tempo
   e il PLC lo segue. Così lo stesso motore può guidare un soft-PLC open source oppure PLCSIM Advanced.
3. **Parla la lingua del pubblico.** Ha un profilo Siemens SCL e Ladder, SFC e Blockly. I programmi si
   importano da TIA (SCL), CODESYS e TwinCAT, oppure in PLCopen XML.
4. **Licenza e linguaggio** sono gli stessi di Twinova: MIT OR Apache-2.0, Rust.

**Limiti, letti nella fonte:**
- **Solo esecuzione compilata.** ST viene tradotto in Rust e compilato con un `rustc` fissato, che il runtime
  installa al primo build. Il motore a bytecode (VM) è stato **cancellato il 24 ottobre 2026**
  (`10-runtime-semantics.md`).
  - Quindi nel browser c'è solo l'analisi (`trust-wasm-analysis`: diagnostica, hover, completamento), non
    l'esecuzione.
  - Ogni modifica a un programma costa una compilazione.
- **Dimensioni e giovinezza:** circa 1,3 milioni di righe di codice, progetto nato a febbraio 2026.
- **Ladder** è un editor visuale che produce un compagno ST, non un motore LD nativo [inferenza dal README].

## Cosa ne segue [inferenza]

- **Logica dei componenti in ST e Ladder**, eseguita da truST come runtime esterno al motore di Twinova, in
  lockstep tramite l'harness con tempo virtuale. Il behavior di un nastro o di un sensore si scrive come un
  function block ST.
- **Per il browser** serve un'altra strada: un interprete ST in WASM (da scrivere, o da proporre a truST), oppure
  il PLC TypeScript di OIP come riferimento.
- **Prima di decidere, tre prove:**
  1. `trust-harness` su un FB di nastro: costo di `cycle` e `advance_time` per ciclo, cioè quanti cicli al
     secondo regge;
  2. tempo di compilazione di un programma piccolo;
  3. determinismo: stessa sequenza di input, stessi output.
- Lo standard per le estensioni (behavior personalizzati) diventa ST, non Python. Il motore resta in Rust.
