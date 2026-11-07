# Structured Text e Ladder senza compilazione (per il browser e le scuole)

Stato: **misura e opzioni, 2026-11-07**. Richiesta dell'utente: eseguire la logica IEC 61131-3 **senza compilare**,
così si apre una pagina web e la scena gira subito, senza toolchain, adatto alle scuole.

## Il punto di partenza (09)

truST, il miglior runtime IEC 61131-3 open source (MIT OR Apache-2.0, Rust), oggi esegue **solo codice
compilato**. Il suo motore a bytecode, cioè l'encoder STBC più la VM, è stato **cancellato il 24 ottobre 2026**
per scelta del proprietario (`docs/specs/44-native-execution.md`, versione 0.22: "n11, delete it"). Il motivo è
avere un solo motore compilato, anche per i microcontrollori.

## Misure (2026-11-07)

Clone di `johannesPettersson80/trust-platform`, storia recuperata dal 2026-10-18. Ho aperto un worktree al commit
**`64059159`** (2026-10-23), l'ultimo letto che contiene ancora il motore a bytecode.

| Cosa | Risultato | Come |
|---|---|---|
| File del motore a bytecode al commit `64059159` | **337** file sotto `vm/`, `bytecode/` e `runtime/vm/`; **0** in `HEAD` | `git ls-tree` |
| Righe Rust di VM ed encoder | **91 541** | somma dei file `.rs` di quei percorsi |
| `trust-runtime-core`: VM, formato e validazione del bytecode | dichiarato "Portable bytecode execution core", `#![cfg_attr(not(feature = "std"), no_std)]` | `Cargo.toml`, `lib.rs` |
| **Compilazione per WebAssembly** di `trust-runtime-core` | **riuscita**: `cargo build -p trust-runtime-core --no-default-features --target wasm32-unknown-unknown --release` in **22,9 s**, senza modifiche al codice | toolchain fissata dal progetto (Rust 1.99.0), con `RUSTC_WRAPPER` vuoto perché `sccache` qui non c'è |
| Parser e analisi semantica (`trust-syntax`, `trust-hir`) nel browser | **già compilati in WASM oggi**, da `trust-wasm-analysis` (diagnostica, hover, completamento) | `crates/trust-wasm-analysis/Cargo.toml` (`HEAD`) |
| Encoder ST → bytecode (`trust-runtime/src/bytecode`) | **11 328** righe, attivo solo con la feature `full`. Quella feature tira dentro rete, terminale, database, MQTT e un server HTTP, che in WASM non vanno. Le dipendenze interne sono `program_model`, `value`, `memory`, `io`, `harness`, `runtime` e `trust-hir` | `Cargo.toml`, grep sugli `use` |

**Cosa non prova:**
- che la VM compilata in WASM **esegua correttamente** un programma: ho solo compilato, non ho eseguito niente;
- le prestazioni (cicli al secondo in WASM);
- che l'encoder si possa estrarre senza riscrivere parti del runtime;
- la conformità del vecchio motore allo standard IEC rispetto a quello compilato di oggi.

## Le strade

| Strada | Cosa vuol dire | Pro | Contro |
|---|---|---|---|
| **A. Recuperare il motore di truST** | si importa in Twinova la VM (`trust-runtime-core` al commit `64059159`) più l'encoder, estratto da `trust-runtime` | motore completo e testato fino al 23 ottobre; la VM compila già in WASM; licenza compatibile, basta citare la provenienza | **lo manteniamo noi**: upstream l'ha abbandonato; l'encoder va staccato dalle dipendenze del runtime (11 000 righe e 6 moduli interni) |
| **B. Interprete nuovo sul frontend di truST** | si usano `trust-syntax` e `trust-hir` (vivi upstream e già in WASM) e si scrive un interprete che esegue direttamente l'albero semantico | dipende solo dalla parte di truST che resta mantenuta; nuovi profili e dialetti (SCL, CODESYS) arrivano gratis | l'interprete va scritto da zero; più lento di una VM a bytecode [inferenza] |
| **C. Chiedere a truST un backend interprete o WASM** | issue upstream: un backend WASM o interprete accanto a quello compilato | se lo accettano, la manutenzione è condivisa | l'hanno appena tolto per scelta: è improbabile |
| **D. Il PLC di Open Industry Project** | interprete ST e LD in TypeScript (MIT) che gira già nel browser | funziona oggi, Ladder compreso | è in TypeScript, non nel motore Rust; stack doppio |

**Raccomandazione [inferenza]: A come primo passo, B come ripiego.** La prova successiva decide fra le due:
1. estrarre l'encoder in un crate a parte, compilarlo in WASM ed eseguire un FB di nastro sulla VM;
2. misurare quante righe di runtime bisogna portarsi dietro.

Se l'estrazione richiede troppo del runtime, si passa a B.

**Ladder:** in truST il Ladder è un editor visuale che produce un compagno ST [inferenza dal README, 09]. Con A o B
il Ladder gira nel browser dopo la conversione in ST, senza un motore LD separato.
