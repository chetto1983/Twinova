# Inventario dell'ecosistema Rust per il nuovo simulatore (strada C)

Stato: **inventario, 2026-11-07**. Decisione dell'utente: **strada C**, cioè un progetto nuovo (07 §4). La proposta di
stack, da confermare, è nucleo in Rust, interfaccia in TypeScript e script in Python.

> **Correzione 2026-11-07 (decisione dell'utente):** niente Python per la logica dei componenti, perché chi
> programma PLC non lo conosce. Si usa IEC 61131-3 (ST e Ladder): vedi [09](09-iec61131-runtime.md). La riga
> `pyo3` qui sotto resta come inventario, ma non fa più parte dello stack.

## Metodo, e cosa prova e cosa non prova

- **Fonte:** l'API di crates.io (`https://crates.io/api/v1/crates/<nome>`), interrogata il 2026-11-07. Da lì vengono
  ultima versione stabile, data di aggiornamento, licenza dichiarata, download totali e download recenti
  (ultimi 90 giorni), con le ricerche per parola chiave. Per RuSTy la fonte è il `Cargo.toml` del repository
  `PLC-lang/rusty` (`license = "LGPL-3.0"`).
- **Prova:** che il crate esiste, con quella licenza dichiarata, e quanto è usato.
- **Non prova:** qualità, completezza dei protocolli, supporto WASM e determinismo. Nessun crate è stato compilato
  né provato. I download sono un indizio di maturità, non una garanzia. Le licenze sono quelle dichiarate su
  crates.io, non lette nei file del sorgente (tranne RuSTy).

## Tabella

Legenda: **R** = riusabile come dipendenza; **V** = da valutare con una prova; **S** = da scrivere (manca o non è
adatto).

| Bisogno | Crate | Versione (data) | Licenza | Download tot. / 90 gg | Giudizio |
|---|---|---|---|---|---|
| Motore a eventi discreti | `nexosim` | 1.0.0 (2026-02-03) | MIT OR Apache-2.0 | 23 091 / 5 344 | **V**: base candidata (r2 §5: ordine deterministico, clock reale con `OutOfSync`, save/restore) |
| | `sim` | 0.13.1 (2025-04-26) | – | 29 549 | sperimentale (r2) |
| Fisica | `rapier3d` | 0.36.1 (2026-10-10) | Apache-2.0 | 2,04 M / 597 k | **R**: Rust puro, gira in WASM, determinismo opzionale (r2 §2) |
| Collisioni, raycast per i sensori | `parry3d` | 0.31.1 (2026-09-18) | Apache-2.0 | 3,22 M / 1,03 M | **R** |
| Fisica Jolt | `rolt`, `joltc-sys` | 0.3.1, Jolt 5.0 (2024-05-19) | MIT OR Apache-2.0 | 7 463 / 884 | **non adatto**: fermo da due anni a Jolt 5.0 |
| Matematica | `glam`, `nalgebra` | 0.34.2 / 0.35.0 (2026) | MIT/Apache, Apache | 181 M, 101 M | **R** |
| ECS (opzionale) | `bevy_ecs` | 0.20.1 (2026-10-28) | MIT OR Apache-2.0 | 10,4 M | **V**: l'ordine dei sistemi va dichiarato (r2 §3) |
| RNG deterministico | `rand_chacha` | 0.10.0 (2026-02-02) | MIT OR Apache-2.0 | 1,63 G | **R** |
| Serializzazione | `serde` | 1.0.229 | MIT OR Apache-2.0 | 1,66 G | **R** |
| OPC UA client/server | `async-opcua` | 0.19.0 (2026-07-18) | **MPL-2.0** | 321 k / 109 k | **R**: copyleft per file, va bene come dipendenza non modificata |
| Modbus TCP/RTU | `tokio-modbus` | 0.17.0 (2025-10-22) | MIT OR Apache-2.0 | 1,99 M / 452 k | **R** |
| | `rodbus` | 1.6.0 (2026-10-29) | non standard | 50 k | da evitare finché la licenza non è letta |
| MQTT | `rumqttc` | 0.25.1 (2025-11-21) | Apache-2.0 | 9,78 M | **R** |
| Beckhoff ADS | `ads` | 0.4.4 (2024-10-03) | MIT/Apache-2.0 | 95 650 / 45 460 | **V** |
| EtherNet/IP (Logix) | `rust-ethernet-ip` | 1.2.1 (2026-08-23) | MIT | 17 946 / 6 768 | **V**: giovane |
| S7 PUT/GET (S7comm) | `snap7-client` (+ `snap7-server`) | 0.1.7 (2026-05-09) | MIT | 1 909 / 1 244 | **V**: dichiara S7Comm e S7CommPlus e ha un server di test, ma è giovanissimo |
| | `tokio-s7` | 0.1.7 (2026-07-23) | Apache-2.0 | 1 190 | **V** |
| | `s7` | 0.1.9 (2019) | non standard | 18 833 | abbandonato |
| S7CommPlus simbolico (DB ottimizzati) | `s7commplus` | 0.1.0 (2026-07-07) | **LGPL-3.0-or-later AND MIT** | 177 | **V/S**: porting di S7CommPlusDriver, LGPL, quasi senza utenti. Il client Node `plc_test/boxline3d` del repository `chetto1983/apr` è un'alternativa da confrontare |
| PLCSIM Advanced (lockstep) | – | – | – | – | **S**: nessun crate. Va scritto un binding verso l'API nativa C++ (Windows) |
| EtherCAT (per l'HiL, più avanti) | `ethercrab` | 0.7.1 (2026-03-23) | MIT OR Apache-2.0 | 88 631 | **V**: master in Rust puro |
| PLC integrato: parser ST | `iec61131`, `iecst` | 0.7.0 / 0.5.1 (2025-12-11) | MIT | 960 / 772 | **V**: solo parser e analisi statica, niente runtime |
| PLC integrato: PLCopen XML | `plcopen` | 0.3.1 (2025-12-14) | MIT | 824 | **V** |
| PLC integrato: compilatore ST | RuSTy (`PLC-lang/rusty`) | 1.1.0-dev | **LGPL-3.0** | – | **non adatto** come base: compila in nativo via LLVM, non in WASM, ed è LGPL |
| Traiettorie a jerk limitato | `ruckig` | 0.19.4 (2026-07-15) | MIT | 1 786 | **V** |
| IK di robot a 6 assi | `rs-opw-kinematics` | 3.0.0 (2026-09-10) | BSD-3-Clause | 35 916 | **V** |
| Catene cinematiche, URDF | `k`, `urdf-rs` | 0.32.0 (2024) / 0.10.0 (2026-09-25) | Apache-2.0 | 314 k / 2,0 M | **R** |
| Import glTF | `gltf` | 1.4.1 (2024-05-10) | MIT OR Apache-2.0 | 11,5 M | **R** |
| Import STEP | `truck-stepio` (+ `ruststep`) | 0.3.0 / 0.4.0 (2024-09-20) | Apache-2.0 | 130 k | **V**: Rust puro, copertura STEP da provare |
| | `opencascade` | 0.3.0 (2026-08-24) | **LGPL-2.1** | 6 504 | alternativa: OCCT completo, ma LGPL e C++ |
| USD | `usd`, `wabi-usd`, `oxideav-usdz` | 2020–2026 | – | ≤ 17 k | **S**: niente di maturo. Prima glTF, USD dopo |
| Python per gli script | `pyo3` | 0.30.1 (2026-10-14) | MIT OR Apache-2.0 | 298 M | **R** |
| WASM per il browser | `wasm-bindgen` | 0.2.130 (2026-10-30) | MIT OR Apache-2.0 | 641 M | **R** |

## Cosa ne segue

1. **Le fondamenta esistono e sono mature**: fisica e collisioni (Rapier e Parry), matematica, RNG,
   serializzazione, Python e WASM. Ci sono anche OPC UA, Modbus e MQTT.
2. **Il motore a eventi discreti ha un candidato serio**, nexosim 1.0. È da provare contro i requisiti di 05 §6:
   tempo intero, iterazione allo stesso istante, nastri analitici.
3. **Il lato Siemens è il punto debole dell'ecosistema**: i crate S7 sono giovanissimi (sotto i 2 000 download),
   S7CommPlus è LGPL con 177 download, e per PLCSIM Advanced non c'è niente. È anche il vuoto che i commerciali
   lasciano (06 §3), quindi è il lavoro più distintivo da fare.
4. **Il PLC integrato non ha un runtime permissivo** in Rust: esistono solo i parser MIT. Un interprete ST/LD va
   scritto. In alternativa, all'inizio ci si collega solo a PLC esterni (PLCSIM, OpenPLC come processo separato).
5. **Per il CAD**: glTF subito; per STEP si prova `truck-stepio` (Apache) prima di valutare OCCT, che è LGPL.
6. **Jolt in Rust non è mantenuto**, quindi conviene Rapier. OIP usa Jolt in WASM da TypeScript, ma per un nucleo
   Rust i binding sono fermi al 2024.

## Prove da fare prima di fissare lo stack

Aggiornamento 2026-11-07: i punti sul WASM qui sotto non valgono più. L'app è desktop (Tauri), senza edizione browser
(decisione dell'utente).

1. nexosim: un nastro analitico con 1 000 pezzi e sensori, confrontato con il passo fisso; ordine degli eventi
   simultanei; compilazione in `wasm32`.
2. Rapier: determinismo tra nativo e WASM con la feature `enhanced-determinism`.
3. `snap7-client` e `tokio-s7` contro PLCSIM Advanced, oppure contro il loro `snap7-server`; lettura dei DB non
   ottimizzati.
4. `truck-stepio` su un STEP di macchina reale.
