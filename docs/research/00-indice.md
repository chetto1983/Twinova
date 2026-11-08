# Visual Components 5.1: mappa delle funzionalità (indice)

Stato: **studio, 2026-11-07**. Nessuna decisione di progetto: questo è l'inventario da cui partire per
progettare un **nuovo simulatore open source** che prenda il meglio dei simulatori commerciali (layout 3D
a componenti, flusso materiali, robot, messa in servizio virtuale con PLC). Non è legato alla BoxLine
(correzione dell'utente, 2026-11-07).

Aggiornamento 2026-11-07 (decisione dell'utente): Twinova è un'app desktop Tauri e **non avrà un'edizione browser né
WASM**. Le note che valutano il WASM (07, 08, 09, 10) restano come registro; non condizionano più le scelte.

## 1. Cosa prova e cosa non prova

- **Prova:** che cosa la guida ufficiale di Visual Components 5.1 (Premium, help del 13/10/2026,
  prodotto 5.1.1) **dichiara**, pagina per pagina. Ogni riga delle tabelle cita il file della pagina.
- **Non prova:** come il prodotto si comporta davvero. Il software non è stato installato né provato.
  Dove l'help tace, le mappe lo dicono ("Lacune") e le deduzioni sono marcate *[ipotesi]* /
  *[inferenza]*.
- **Clean-room:** le descrizioni sono riformulate; nessuna frase, immagine, icona o file del prodotto è
  copiato. Nomi di proprietà e comandi compaiono solo come identificatori funzionali. Il clone non dovrà
  usare nome, marchi, libreria componenti o formati proprietari di Visual Components.

## 2. Fonte e metodo (riproducibile)

- Guida online: `https://help.visualcomponents.com/5.1/<Edizione>/en/English/...` (Premium,
  Professional, Essentials, Robotics). Indice RoboHelp in `whxdata/toc*.new.js`, letto con
  [`fonti/vc_help_toc.py`](fonti/vc_help_toc.py) (`python3 -I vc_help_toc.py <base> <out>`; usa `curl`
  perché il sito rifiuta il client HTTP di Python con 403).
- Premium: **1423 voci** di indice. Scaricate e convertite in testo le **500 pagine di manuale**
  (170 000 parole); delle 915 pagine di API Python è stata letta solo la panoramica delle classi.
- Convenzione "Fonte" nelle tabelle: il file è il percorso della pagina con `/` e spazi → `_`, `.htm` →
  `.txt`. Esempio: `English_Connectivity_Using_the_Siemens_S7_plugin.txt` =
  `.../5.1/Premium/en/English/Connectivity/Using_the_Siemens_S7_plugin.htm`.
- Lettura: 4 analisi in parallelo, una per area; affermazioni chiave ricontrollate sul testo originale
  (aree S7 I/Q/M, PDU 240/480 byte, trigonometria in radianti e matrici in gradi, Real/Virtual Time,
  warm-up, contenuto del `.vcmx`, PhysX).

## 3. Le mappe

| File | Area | Righe funzionali |
|---|---|---|
| [01](01-ui-layout-simulazione.md) | Interfaccia, setup, eCatalog, import, layout, operazioni 3D, simulazione, statistiche, disegni, export, espressioni, novità | 176 |
| [02a](02a-component-modeling-modello-behavior.md) | Modello dati del componente, giunti, proprietà, interfacce, 52 behavior (segnali, flusso, sensori, robotica, cinematiche, motion, fisica, process model, varie) | 106 |
| [02b](02b-component-modeling-feature-note.md) | 18 feature, geometria, materiali, fisica, operazioni di modellazione; semantica runtime; lacune | 74 |
| [03](03-process-modeling-connettivita.md) | Process Modeling (48 statement + 11 funzioni), Connectivity (22 connettori) | 81 |
| [04](04-robotica-olp.md) | Programmazione robot (32 funzioni, 22 statement), OLP (92 statement e strumenti) | 146 |
| [05](05-motore-simulazione.md) | Motore di simulazione: come è fatto in VC e negli altri, accoppiamento col PLC, proposta | – |
| [06](06-matrice-meglio-di.md) | Matrice "il meglio di ciascuno" su 14 simulatori commerciali, vuoti comuni, open source riusabile, decisioni aperte | 23 aree |
| [07](07-open-industry-project.md) | Open Industry Project (MIT, browser) come base: copertura della matrice, rischi, tre strade | – |
| [08](08-inventario-rust.md) | Strada C: inventario dei crate Rust (licenza, versione, download) per ogni bisogno, vuoti e prove | – |
| [09](09-iec61131-runtime.md) | Logica dei componenti in IEC 61131-3 (non Python): runtime open source, truST in dettaglio | – |
| [10](10-st-senza-compilazione.md) | ST e Ladder senza compilazione per il browser: il motore a bytecode tolto da truST compila in WASM (misurato); quattro strade | – |
| [11](11-ecatalog-componenti.md) | eCatalog web di Visual Components (indice JSON pubblico, forum "eCat Updates"): 50 categorie e 4177 componenti contati, famiglie e varianti di nastri, sensori, feeder e sink, parametri e segnali citati pubblicamente, distribuzione per versione; ipotesi di parametri minimi per M4.4 | – |
| [benchmark/](benchmark/) | Schede prodotto con URL: virtual commissioning (g1), flusso/DES (g2), didattici e open source (g3) | – |
| [ricerca/](ricerca/) | Note di ricerca con URL: eventi discreti (r1), passo fisso e fisica (r2), virtual commissioning e PLCSIM (r3) | – |

Totale: **583 righe funzionali** mappate.

## 4. Edizioni

Dedotte dalle pagine presenti nella guida di ciascuna edizione (confronto degli indici 5.1, 2026-11-07).
È un'**ipotesi**: la presenza di una pagina nell'help non è la licenza. Le note di rilascio 5.0
confermano solo il punto OPC UA/MQTT in Professional.

| Area | Essentials | Robotics | Professional | Premium |
|---|---|---|---|---|
| Layout, eCatalog, simulazione, statistiche, disegni | sì | sì | sì | sì |
| Process Modeling | sì | **no** | sì | sì |
| Component Modeling (behavior, feature, fisica) | **no** | **no** | sì | sì |
| Programmazione robot base | sì | sì | sì | sì |
| OLP (path, saldatura, spray, post-processor) | **no** | sì | sì | sì |
| Connettività OPC UA, MQTT | no | no | sì | sì |
| Siemens S7, SIMIT, WinMOD, TwinCAT ADS, Allen-Bradley, LS, plugin robot, Stäubli | no | no | **no** | sì |
| Export FBX | no | no | no | sì |

## 5. Architettura dichiarata dall'API Python 3

La panoramica delle classi (`Python_3_API/Overview.html`) è la descrizione più precisa del modello a
oggetti: **10 moduli, 579 classi**.

| Modulo | Classi | Contenuto |
|---|---|---|
| `vcCore` | 193 | applicazione, mondo, componenti/nodi, proprietà, unità, materiali, camera, collisioni, raycast, statistiche, registrazione video/PDF, servizi di selezione e snap, contesti (authoring, configure, drawing, process, teach) |
| `vcBehaviors` | 64 | segnali (bool, int, real, string, matrix, component), mappe segnali, path di moto (one-way, two-way), container, creator, routing rule, sensori (volume, raycast, path), interfacce di simulazione e i loro field, controller di capacità, statistiche, script |
| `vcFeatures` | 25 | primitive (block, cone, cylinder, sphere, wedge, text), extrude, revolve, boolean, clone lineare/angolare, mirror, transform, switch, feature Python |
| `vcGeometry` | 13 | topologia, insiemi di triangoli, linee, punti, frame, testi |
| `vcProcessModel` | 132 | tipi di prodotto e assiemi, process flow/group/executor, ~40 statement di processo, sistema di trasporto (nodi, link, controller a nastro e interpolanti), filtri prodotto, creator di prodotti |
| `vcRobotics` | 57 | controller servo e robot, executor, base/tool frame, statement robot, interpolatore di moto, cinematica Python |
| `vcRobotics2` | 71 | nuova generazione: motion controller, kinematic solver, link joint, target PTP/LIN/multi-driver, programmi e routine, sincronizzazione tra programmi, monitor di moto |
| `vcExecutor`, `vcExecutor2` | 21 | programma, scope, routine, statement, stato di esecuzione |
| `vcExperimental` | 3 | profilazione, mappatore I/O RPC |

Da qui la struttura minima: **mondo → componenti (albero di nodi con giunti) → feature (geometria
parametrica) + behavior (logica) + proprietà + segnali**. Sopra ci sono tre motori: il flusso materiali
e i processi, i robot/motion controller, e la connettività verso il PLC. In più un motore fisico
(NVIDIA PhysX, aggiornato a 5.5.0 nella 5.1; la 5.1.1 corregge risultati non deterministici della 5.1).

## 6. Osservazioni critiche per il clone

Sono deduzioni dalle mappe, da confermare in fase di design.

1. **Il nucleo è il modello a componenti con behavior e segnali** (02a). Senza quello non esiste un
   simulatore: CAD, rendering e catalogo vengono dopo.
2. **Connettività S7 di VC è limitata** (03, verificato sul testo): S7comm PUT/GET, solo aree **I, Q, M**
   (niente DB), tipi fino a REAL, nessun browsing (si importa la tabella dei tag da TIA), PDU di 240 byte
   su S7-1200. Un accesso **simbolico S7CommPlus ai DB** (un client Node esiste in
   il repository `chetto1983/apr`, `plc_test/boxline3d`) farebbe meglio di Premium su questo punto. S7-1200 G2 e PLCSIM classico non sono citati.
3. **Nessun clock sincrono PLC ↔ simulazione** per i PLC: lo scambio è ciclico o a evento in tempo
   reale. Solo l'add-on Stäubli ha una sincronizzazione stretta (20 ms). Un passo sincrono con
   PLCSIM Advanced sarebbe un elemento distintivo.
4. **La simulazione ha due modi:** tempo reale con fattore di scala, oppure tempo virtuale alla massima
   velocità con passo di rendering. Reset riporta lo stato salvato a inizio run e cancella i componenti
   creati (01).
5. **Le espressioni sono un linguaggio a sé** (01, EXPR): tipi, unità tra graffe, distribuzioni
   casuali, matrici. La pagina sulle espressioni negli statement di processo esiste nell'indice, ma la
   sintassi non è descritta da nessuna parte.
6. **Robotica e OLP sono metà del prodotto** (04: 146 righe). Sono il pezzo più costoso da
   rifare.

## 7. Lacune trasversali

- Formati interni (`.vcmx` oltre l'elenco dei file, configurazione della connettività, layout `.vcmx`).
- Scheduler della simulazione: dall'API risulta a eventi discreti con coroutine, vedi [05](05-motore-simulazione.md); restano ignoti coda, spareggi e passo fisica, profili di moto e unità di
  velocità e accelerazione di path e giunti.
- Regole di compatibilità e tolleranze delle interfacce plug-and-play.
- Se i segnali notificano a ogni scrittura o solo al cambio di valore.
- Grammatica completa delle espressioni.
- Metodi dell'API Python: non letti (solo l'elenco delle classi).

Per colmarle servirebbe il prodotto installato (licenza di prova) e una misura: per la regola
"misura, poi registra", una riga va corretta solo dopo una prova sul software reale.

## 8. Prossimo passo proposto (da decidere)

Mappare allo stesso modo gli altri simulatori commerciali e costruire la matrice "il meglio di ciascuno",
da cui ricavare visione e requisiti del nuovo prodotto. Solo dopo: architettura e perimetro del primo
prototipo.
