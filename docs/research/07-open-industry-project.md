# Open Industry Project come base del nuovo simulatore

Stato: **valutazione, 2026-11-07**, con test e benchmark misurati (§1b). Letto il repository `github.com/Open-Industry-Project/Open-Industry-Project`
(clone del 2026-11-07, commit `142c681`): README, `docs/architecture.md`, `docs/sim-clock.md`, `package.json`,
`THIRD_PARTY_NOTICES.md`, la guida S7 e il codice del collegamento ai PLC. Ho eseguito test e benchmark
headless; l'app nel browser no.

## 1. Cos'è (verificato nel repository)

- **Licenza MIT**, (c) Automation Standard LLC. È un simulatore di magazzino e produzione che gira **nel browser**;
  un'app desktop Electron serve per collegarsi a un PLC esterno.
- **Stack:** TypeScript, three.js (WebGL e WebGPU), **Jolt Physics** compilato in WASM, React, Yjs per la
  collaborazione live, SES per isolare il codice dei componenti, occt-import-js per l'import STEP. Quest'ultima è
  LGPL-2.1, usata non modificata e dichiarata nelle note di terze parti.
- **Dimensioni:** circa 296 000 righe di TS e JS, 527 file di test, layer separati e controllati dal lint
  (`core` headless e deterministico, `sim`, `render`, `plc`, `editor`, `collab`).
- **Componenti:** circa 37 parti pronte (nastri a rulli, a cinghia, curvi e flessibili, diverter, turntable,
  sensori laser, diffusi, a colore, barriere, scanner barcode, robot a 6 assi, gantry, AGV, pallettizzatore,
  avvolgitore, rack, sicurezze). Si scrivono tramite un **Part SDK** pubblico: `rig.json`, GLB, collisioni
  dichiarative e `behavior.ts`. Una parte vede il mondo solo attraverso l'interfaccia `PartContext`.
- **PLC integrato:** IEC 61131-3 in Structured Text e Ladder, import ed export ST e PLCopen XML.
- **PLC esterni (desktop):** Modbus TCP, OPC UA, EtherNet/IP Logix, **S7 PUT/GET** (S7-1200/1500, anche su DB
  non ottimizzati), SLMP, ADS. C'è anche un **server OPC UA** verso SCADA e HMI, e un server MCP.
- **Motore del tempo:** a passo fisso, e il clock lo detta la fisica. Il passo vale da 60 a 1000 Hz ed è un
  parametro del progetto (`simHz`). La scansione del PLC integrato gira nello stesso worker, alternata ai passi.
  - Il **tempo di simulazione è `stepCount / hz`**, quindi deterministico.
  - In sovraccarico il motore rallenta e lo segnala: il pulsante della velocità diventa ambra.
  - Un test runner headless gira senza orologio, alla massima velocità.
- **Attività:** ultimo commit il 2026-11-07. Il primo commit della storia attuale è del **2026-10-19**: 204
  commit, di cui **182 di un solo autore**. Il README dichiara "early development, breaking changes can land in
  any update".

## 1b. Misure (2026-11-07, commit `142c681`)

Container Linux della sessione, 4 core, Node **v24.22.2**. Ho scaricato il binario ufficiale e ne ho verificato
lo SHA-256; prima c'era Node 22, ma il progetto chiede almeno 24. Le dipendenze sono installate con
`npm ci --ignore-scripts`.

- **Test:** `npx vitest run` → **7211 superati, 11 saltati, 0 falliti** su 526 file, in 181,5 s.
- **Benchmark del motore:** `npm run bench-sim`, headless, Jolt WASM con SIMD, mediana di 3 ripetizioni.
  - La scena è il demo di pallettizzazione (`createPlcDemoScene`): 353 parti, circa 1070 box, un programma ST
    di 21 KB con scansione a 10 ms e 206 valori PLC.
  - Per misurarla l'ho esportata in `.oip` con uno spec usa e getta, poi cancellato. Il repository non è stato
    modificato.

| Passo | ms per passo | di cui parti / fisica | Rispetto al tempo reale |
|---|---|---|---|
| 60 Hz | 1,308 | 0,673 / 0,137 | **12,7x** |
| 240 Hz | 1,328 | 0,860 / 0,169 | **3,1x** |
| 1000 Hz | 1,187 | 0,826 / 0,153 | **0,84x** (non tiene il tempo reale) |

Cosa ne segue:
- **Il costo per passo è quasi costante**, quindi il costo totale cresce linearmente con la frequenza.
- **Circa il 65–70 % del tempo va ai behavior dei componenti** in TypeScript. La fisica Jolt pesa solo il
  10–13 %.
- **Con 1 ms di risoluzione I/O la scena demo non regge il tempo reale** su questa macchina. Per una macchina
  veloce, dove a 2 m/s 1 ms vale 2 mm, il passo fisso costa caro.

**Cosa non prova:**
- la macchina non è un PC di riferimento;
- l'app nel browser (worker e render) non è stata misurata;
- "0 completed" è il contatore della scena dopo 60 s simulati e non è stato indagato;
- non ho provato scene più grandi del demo, né la build multi-thread di Jolt (`--jolt mt`).

## 2. Confronto con la matrice (06)

| # (06) | Area | OIP | Nota |
|---|---|---|---|
| 1 | Modello del componente | **sì**, Part SDK con contesto isolato | meno ricco del modello VC: niente albero di feature parametriche, segnali tipizzati come behavior o interfacce generiche |
| 3, 4 | Logica | PLC integrato ST e LD, behavior in TS | nessun grafo a token e nessuno scripting Python |
| 5 | Motore del tempo | passo fisso e fisica | **niente eventi discreti**: un turno di lavoro costa tutti i suoi passi |
| 6 | Determinismo | **sì**, tempo per conteggio di passi e RNG con seme | |
| 7 | Sincronizzazione col PLC esterno | **no** | il link è un polling asincrono via socket; PLCSIM Advanced non è citato (0 occorrenze) |
| 8 | Connettività | **ampia**: 6 protocolli più server OPC UA | in linea con Emulate3D e realvirtual |
| 9 | S7 | PUT/GET, I/Q/M più DB **non ottimizzati** | meglio di VC (che non legge i DB), ma niente S7CommPlus simbolico sui DB ottimizzati |
| 11 | Mappatura I/O | tag nel componente, guida per ogni protocollo | da valutare provandolo |
| 13 | Fisica | Jolt per tutto, nastri compresi | latenza minima dell'I/O pari a un passo: 16,7 ms a 60 Hz; a 2 m/s un sensore vede il pezzo a scatti di 3,3 cm (`docs/sim-clock.md`) |
| 15 | Movimentazione | AGV, pallet, rack | |
| 16 | Robot | robot a 6 assi, gantry | niente OLP né controller virtuali |
| 17, 22 | Esperimenti, statistiche | poco | termini quasi assenti nel codice |
| 18 | Headless e API | test runner headless, CLI, MCP | niente REST né Python |
| 19 | CAD e 3D | GLB, STEP (OCCT WASM), viewer USD | |
| 23 | Separazione modello/vista | **sì**, core headless e render come vista | |
| – | Collaborazione | **sì**, Yjs live | nessuno dei commerciali studiati la mette al centro |

## 3. Giudizio

**Pro.** Copre già circa metà della matrice con scelte d'architettura corrette: core headless e deterministico,
interfaccia isolata per i componenti, layer controllati dal lint, test estesi. Ha licenza MIT, gira nel browser
e ha la connettività più ampia tra gli strumenti open source. Come base per il nuovo simulatore è molto più
adatto di SolidCraft, che è un CAD.

**Contro e rischi.**
1. **Il bus factor è 1** e la storia attuale ha 3 settimane. È un'ipotesi: l'organizzazione esisteva già prima
   (vecchia versione Godot) e la storia è stata riscritta, ma non l'ho verificato. Formati e SDK possono cambiare.
2. **Il passo fisso fisico per tutto** è il contrario della scelta di VC, FlexSim e JaamSim. Non regge le
   simulazioni di flusso lunghe e grandi, e la precisione dei sensori dipende dal passo.
3. **Manca il lockstep con PLCSIM Advanced e con i PLC reali**, che è proprio uno dei vuoti da coprire (06 §3).
4. **Il TypeScript nel browser** limita le scene molto grandi. Gli autori stessi scrivono che a 8x serve quasi
   un core intero per la scena demo. Misurato (§1b): 1,2–1,3 ms per passo sul demo, quindi a 1000 Hz non
   si tiene il tempo reale.

## 4. Le tre strade

| Strada | Cosa vuol dire | Pro | Contro |
|---|---|---|---|
| **A. Contribuire a OIP** | PR upstream: modo DES o analitico per i nastri, lockstep PLCSIM Advanced (via l'app desktop), S7 simbolico, statistiche ed esperimenti | si sfrutta un progetto vivo, MIT, con utenti | le decisioni sono del maintainer; l'architettura "la fisica possiede il clock" va cambiata nel cuore |
| **B. Fork di OIP** | si parte da OIP e si diverge | stesso vantaggio iniziale, libertà totale | si mantiene da soli un codebase da 296 000 righe che cambia ogni giorno |
| **C. Nuovo progetto** | si riprendono le idee di OIP (Part SDK, contesto isolato, layer) e il motore si scrive da zero | architettura dal primo giorno (DES ibrido, lockstep) | anni di lavoro per arrivare dove OIP è già |

**Raccomandazione [inferenza]: A.** Prima va aperta una discussione con il maintainer (issue o Discord) su due
punti:
1. un motore ibrido, con eventi discreti più fisica a isole;
2. il lockstep con PLCSIM Advanced.

Se le accetta si contribuisce; altrimenti si passa a B o C con le stesse idee. Test e benchmark sono fatti
(§1b). Manca la prova dell'app nel browser e su un PC Windows con un PLC esterno.
