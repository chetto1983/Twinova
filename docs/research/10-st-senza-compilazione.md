# Structured Text e Ladder senza compilazione (per il browser e le scuole)

Stato: **misura e opzioni, 2026-11-07**. Richiesta dell'utente: eseguire la logica IEC 61131-3 **senza compilare**,
così si apre una pagina web e la scena gira subito, senza toolchain, adatto alle scuole.

**Decisione successiva (utente, 2026-11-07): niente browser e niente WASM.** Twinova è un'app desktop Tauri e il
motore gira nativo nel backend Rust. L'edizione browser (ROADMAP 9.4) è tolta, quindi il WASM non è più un requisito
né un criterio di scelta. Questa nota resta come registro della misura.

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

## Prototipo (2026-11-07): `prototypes/st-vm-probe`

Il programma di prova è un function block di nastro in ST: start e stop, contapezzi sul fronte della fotocellula,
allarme di inceppamento memorizzato con `TON` di 2 s. Lo esegue `trust_runtime::harness::TestHarness` al commit
`6405915973a7`, in lockstep: a ogni ciclo si scrivono gli ingressi, si avanza il tempo virtuale di 10 ms e si esegue
`cycle()`.

| Misura (nativo, container cloud con 4 core) | Valore |
|---|---|
| Compilazione ST → bytecode, nel processo | **6,1–6,6 ms** |
| Comportamento | **corretto**: start memorizzato, 6 conteggi, inceppamento dopo 2 s virtuali, motore fermo |
| Velocità | **137 421 scansioni/s**, cioè **7,28 µs per scansione**; 200 000 scansioni coprono 2 000 s virtuali in 1,46 s |

**Una nota sulla prima prova.** Il primo programma non memorizzava l'allarme: `TON` scattava a 2,0 s, `Jam` fermava
il motore, `IN := PartAtEnd AND Motor` diventava FALSE e il timer si azzerava. È il comportamento IEC corretto.
L'errore era nel programma di prova, non nella VM, ed è stato corretto memorizzando l'allarme.

**WebAssembly (`wasm32-unknown-unknown`): non compila così com'è.**

1. `getrandom` va configurato con il backend `wasm_js`. Fatto, in `.cargo/config.toml` e `Cargo.toml`.
2. L'harness esiste solo con la feature `full`, che porta tokio e mio (zenoh, rumqttc, mdns-sd). mio non compila per
   wasm32.
3. Ho fatto una prova su una copia usa e getta, con una feature `core` che contiene solo i moduli del percorso
   harness → encoder → VM:
   - 1° giro: **72 errori**, quasi tutti in `control`;
   - 2° giro, tolti anche `control`, `plcopen` e `simulation`: **100 errori**, perché `config`, `runtime` (sottosistemi
     ADS, OPC UA e OpenOT, `installed_services`) e `scheduler` importano direttamente `ads`, `opcua`, `simulation`,
     `hmi`, `historian` e `linux_rt`.

   Il percorso di esecuzione è intrecciato con i servizi del prodotto. Non sono solo dipendenze opzionali.

**Cosa non prova:**
- prestazioni e correttezza in WASM, perché lì non è stato eseguito niente;
- un confronto con il motore compilato di truST di oggi;
- il comportamento su programmi più grandi di un FB.

## Le strade

| Strada | Cosa vuol dire | Pro | Contro |
|---|---|---|---|
| **A. Recuperare il motore di truST** | si importa in Twinova la VM (`trust-runtime-core` al commit `64059159`) più l'encoder, estratto da `trust-runtime` | motore completo e testato fino al 23 ottobre; la VM compila già in WASM; licenza compatibile, basta citare la provenienza | **lo manteniamo noi**: upstream l'ha abbandonato; l'encoder va staccato dalle dipendenze del runtime (11 000 righe e 6 moduli interni) |
| **B. Interprete nuovo sul frontend di truST** | si usano `trust-syntax` e `trust-hir` (vivi upstream e già in WASM) e si scrive un interprete che esegue direttamente l'albero semantico | dipende solo dalla parte di truST che resta mantenuta; nuovi profili e dialetti (SCL, CODESYS) arrivano gratis | l'interprete va scritto da zero; più lento di una VM a bytecode [inferenza] |
| **C. Chiedere a truST un backend interprete o WASM** | issue upstream: un backend WASM o interprete accanto a quello compilato | se lo accettano, la manutenzione è condivisa | l'hanno appena tolto per scelta: è improbabile |
| **D. Il PLC di Open Industry Project** | interprete ST e LD in TypeScript (MIT) che gira già nel browser | funziona oggi, Ladder compreso | è in TypeScript, non nel motore Rust; stack doppio |

**Aggiornamento dopo il prototipo [inferenza].** A, cioè prendere il runtime di truST così com'è, costa più del
previsto. Il percorso di esecuzione va staccato da `config`, `runtime` e `scheduler`, e significa mantenere un fork di
un crate da 286 000 righe. Le varianti realistiche sono due:
- **A′**: tenere la VM (`trust-runtime-core`, che compila in WASM) e **portare solo l'encoder** (circa 10 800 righe)
  in un crate nostro, sostituendo le sue dipendenze interne (`program_model`, `value`, `memory`, `io`);
- **B**: un interprete nuovo su `trust-syntax` e `trust-hir`.

**Correzione 2026-11-07 (misura, spec `2026-11-07-runtime-footprint-design.md`).** La stima di A′ "circa 10 800
righe" vale per il solo `bytecode/`. Per arrivare all'encoder servono anche il lowering (`host/harness`, 12 510 righe
senza test), il `Runtime` (`runtime/`, 14 316) e `io/` (8 104): **almeno circa 45 700 righe**. Seguendo i riferimenti
fra moduli a livello di file si arriva a 119 522 righe su 151 403, perché l'encoder usa `crate::harness` e il
runtime usa `crate::ads` e `crate::debug`. Inoltre `full` pesa poco sul binario (LTO toglie il codice non chiamato):
pesa sul tempo di compilazione, circa un terzo su Linux, con OpenSSL compilato da sorgente in 174 s.

Raccomandazione originale, prima del prototipo:

**Raccomandazione [inferenza]: A come primo passo, B come ripiego.** La prova successiva decide fra le due:
1. estrarre l'encoder in un crate a parte, compilarlo in WASM ed eseguire un FB di nastro sulla VM;
2. misurare quante righe di runtime bisogna portarsi dietro.

Se l'estrazione richiede troppo del runtime, si passa a B.

**Ladder:** in truST il Ladder è un editor visuale che produce un compagno ST [inferenza dal README, 09]. Con A o B
il Ladder gira nel browser dopo la conversione in ST, senza un motore LD separato.
