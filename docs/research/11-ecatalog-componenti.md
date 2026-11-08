# Visual Components eCatalog: struttura, famiglie e parametri dei componenti (clean-room)

Stato: **studio, 2026-11-08**. Alimenta M4 di `ROADMAP.md`: 4.1 (spec del modello a componenti) e 4.4
(libreria v1). Non ripete [02a](02a-component-modeling-modello-behavior.md),
[02b](02b-component-modeling-feature-note.md) né [01 §3](01-ui-layout-simulazione.md) (pannello eCatalog
nell'app desktop): aggiunge solo ciò che la **versione web pubblica** del catalogo e il forum pubblico
dicono in più.

**Clean-room.** Descrizioni con parole nostre. Nomi di componenti, proprietà e segnali compaiono solo
come identificatori, per descrivere e confrontare. Nessuna immagine, icona, file `.vcmx` o script è stato
scaricato o copiato (l'unica immagine toccata è stata una richiesta `HEAD` per verificare che le
anteprime siano pubbliche; il contenuto non è stato salvato). Twinova non userà nomi, marchi, geometrie o
formati di questa libreria.

## 0. Fonti e metodo

Accesso del 2026-11-08, client `curl` con user agent di browser, **circa 55 richieste** in tutto verso
`visualcomponents.com` e `forum.visualcomponents.com` (più i file dell'indice dell'help 5.1, letti con lo
script di 00 §2).
Nessun login, cookie o token: tutte le risorse sotto sono pubbliche.

| Sigla | URL | Cosa è |
|---|---|---|
| **P** | `https://www.visualcomponents.com/ecatalog/` | Pagina WordPress; il catalogo è un `<div id="app--ecatalog">` vuoto riempito da JavaScript |
| **V** | `https://www.visualcomponents.com/wordpress/wp-content/plugins/meomblocks/build/blocks/ecatalog/view.js` | Script dell'app (Vue 2 + vue-router + axios, 161 KB) |
| **J** | `https://www.visualcomponents.com/components.json` | Indice JSON del catalogo che **V** carica (1,49 MB, nessuna autenticazione) |
| **F‹n›** | `https://forum.visualcomponents.com/t/‹n›` | Topic del forum pubblico (Discourse), letto via `/t/‹n›.json` |
| **FS:‹q›** | `https://forum.visualcomponents.com/search.json?q=‹q›` | Ricerca pubblica del forum (si leggono solo gli estratti) |
| **C17** | `https://forum.visualcomponents.com/c/ecat-updates/17` | Categoria "eCat Updates" del forum (232 topic, annunci del team del catalogo) |
| **H** | `https://help.visualcomponents.com/5.1/Premium/en/whxdata/toc.new.js` | Indice dell'help 5.1 (riletto per cercare pagine sui componenti del catalogo) |

Analisi di **J** con uno script Python usa-e-getta (conteggi per `Type`, `Manufacturer`, `Author`, date,
profondità dei percorsi, ricerca per parole nei nomi). I conteggi sotto sono **misurati su J il
2026-11-08**; cambiano a ogni aggiornamento del catalogo.

## 1. Come è fatta la versione web (nuovo rispetto a 01 §3)

| ID | Elemento | Cosa abbiamo trovato | Fonte |
|---|---|---|---|
| WEB-01 | Architettura | Single-page app Vue montata nella pagina WordPress; all'avvio fa **una sola** `GET /components.json?v=<timestamp>` (il parametro serve solo a evitare la cache). Tutto il filtro avviene nel browser. | P, V |
| WEB-02 | Rotte | Hash routing `#/<Type>/<Manufacturer>`: tipo e produttore scelti finiscono nell'URL e si possono condividere. | V |
| WEB-03 | Filtri | Tre controlli: menu **Type**, menu **Manufacturer** (entrambi con il conteggio degli elementi accanto a ogni voce, ricalcolato in base all'altro filtro), casella **Search** sul nome. | V |
| WEB-04 | Lista | Griglia di miniature con il nome; paginazione "carica altri"; clic → finestra modale con l'immagine grande e il nome. Nessun'altra informazione per elemento. | V |
| WEB-05 | Deprecati | Lo script tiene solo gli elementi con `IsDeprecated` uguale a `"false"`; il 2026-11-08 nessun elemento è deprecato. | V, J |
| WEB-06 | Testata JSON | `Components.$ = {Count: "4183", Version: "3.6"}`, ma gli elementi elencati sono **4177**: 6 in meno del conteggio dichiarato (causa ignota). `Version` è la versione del formato dell'indice, non del prodotto *[inferenza]*. | J |
| WEB-07 | Campi per elemento | Solo 8, tutti stringhe dentro array di un elemento (tipico di una conversione XML→JSON *[inferenza]*): `Name`, `Author`, `Manufacturer`, `Icon`, `PreviewIcon`, `Type`, `DateModified`, `IsDeprecated`. **Nessun** parametro, segnale, interfaccia, dimensione del file, versione, tag, licenza o link di download. | J |
| WEB-08 | Percorsi | Le immagini stanno in `/elib/vccom/5.1/eCat/Components/<Manufacturer>/<Type>/[sottocartelle]/<Name>[_thumbnail].png`: la web mostra **solo il catalogo 5.1**. Profondità variabile: 145 elementi a 8 livelli, 3432 a 9, 175 a 10, 415 a 11, 10 a 12. Le sottocartelle sono famiglie di prodotto del fornitore (es. `VarioFlow plus/Transport Control`, `TS 5`). | J |
| WEB-09 | Immagini | Miniatura e anteprima PNG pubbliche (verificato con `HEAD` → 200 `image/png`). | J |
| WEB-10 | Date | `DateModified` va dal 2025-10-23 al 2026-10-07: 2868 elementi del 2025, 1309 del 2026. Nessuna data di creazione. | J |
| WEB-11 | Dichiarazione | La pagina dichiara "oltre 4000" componenti pronti all'uso, plug-and-play; le richieste di nuovi modelli sono riservate ai clienti (`og:description` della stessa pagina dice ancora "oltre 3000": testo non aggiornato). | P |

## 2. Struttura: i 50 `Type` e i loro conteggi

Il catalogo web ha **un solo livello di categoria** (`Type`) più il produttore; le sottocartelle di WEB-08
non sono esposte come filtro. Raggruppamento in macro-aree **nostro** (*[inferenza]*), conteggi da **J**.

| Macro-area (nostra) | `Type` (conteggio) | Totale |
|---|---|---|
| Robot e periferiche robot | Robots 2481; Robot Tools 120; Robot Workpiece Positioners 115; Robot Positioners 64; Robot Pedestals 19; Robot Cabling 2; Facilities - Robot 39 | 2840 |
| Trasporto materiali | Conveyors 194; Conveyor Utilities 16; Advanced Motion 94; Transport Controllers 10; Cranes 10 | 324 |
| Assi e posizionatori a segnale | Single Axis Linear Actuators 181; Signal Positioners 8 | 189 |
| Mobile robot e navigazione | Mobile Robots 88; Mobile Robot Utilities 18; Navigation 14; Navigation (Beta) 16; PM Navigation 1 | 137 |
| Flusso (creazione, processo, uscita) | Feeders 34; Flow Components 15; Products and Containers 85; Warehousing 10; Packaging 24 | 168 |
| Macchine e postazioni | Machines 53; Machines In-Line 5; Manual Workstations 12; Fixtures 10; Life Science 13 | 93 |
| Segnali, sensori, HMI | Sensors 3; Signal Utilities 7; HMI 6; Cameras 19 | 35 |
| Edifici e arredi | Facilities - Factory 55; Facilities - Interior 33; Basic Shapes 11 | 99 |
| Persone e risorse | Characters 36; Resources 9; Resource Tools 25; Ergonomics 4 | 74 |
| Statistica, dati, varie | Statistics 19; Data Management 5; Text and Media 6; VR Interaction 6; Misc 12; Physics 12 | 60 |
| Esempi e modelli | Layouts 65; Layout Templates 62; Layouts OLP 16; Component Templates 15 | 158 |
| **Totale** | 50 `Type` | **4177** |

Osservazioni misurate su **J**:

- **I robot sono il 59 %** del catalogo (2481/4177, 87 produttori, solo 6 generici). Il trasporto
  materiali (nastri + accessori + motion avanzato) è circa l'8 %.
- **166 produttori**. `Visual Components` (componenti generici, non di marca) firma **553** elementi
  (13 %); le voci "Layouts" (64) e "Layout Templates" (62) usano il tipo stesso come produttore.
- I generici dominano proprio le famiglie di M4.4: 34/194 nastri, **16/16** Conveyor Utilities, 10/34
  feeder, 15/15 Flow Components, 7/7 Signal Utilities, 7/8 Signal Positioners, 9/10 Transport
  Controllers, 12/12 Physics, 15/15 Component Templates.
- **Sensori come `Type` quasi assenti** (3: due Safety Scanner e il Volume Detector). I sensori dei nastri
  vivono altrove: `Conveyor Sensor` in Conveyor Utilities, `Sensor Conveyor` in Conveyors, `VarioFlow
  plus - Sensor` (Bosch Rexroth), `Safety Light Curtain` in Facilities - Factory, `Quality check camera`
  in Facilities - Factory, telecamere 3D in Cameras.
- **Nessun elemento PLC/I-O come categoria.** Gli unici oggetti "controller" sono armadi di sistemi a
  motore lineare (Advanced Motion: XTS, SuperTrak, Planar Motor, B&R…) e controller robot (Facilities -
  Robot). La logica a segnali generica sta in Signal Utilities e HMI (§3.5).
- **Pinze:** Robot Tools 120 elementi, 17 produttori (OnRobot 24, generici 21, Schunk 16, UniGripper 10,
  PushCorp 10, Robotiq 8, …).
- 14 nomi compaiono due volte (es. `Feeder`, `Feeder Process`, `Capacity Pull Process` sia in Feeders sia
  in Flow Components): duplicati voluti, spiegati in F7406 (§5).

## 3. Le famiglie di M4.4 nel catalogo

### 3.1 Nastri generici (`Conveyors`, produttore `Visual Components`, 34 elementi)

Varianti lette dai **nomi** in **J**; la funzione è *[inferenza]* dal nome salvo dove è indicata un'altra
fonte.

| ID | Variante | Funzione (nostre parole) | Fonte |
|---|---|---|---|
| ECV-01 | Conveyor | Nastro rettilineo base | J |
| ECV-02 | Curve Conveyor | Curva orizzontale | J |
| ECV-03 | Incline / Decline; Verticalbend Conveyor | Salita e discesa; raccordo verticale | J |
| ECV-04 | Two Way Conveyor | Nastro bidirezionale; si ferma con `PowerOnSignal = False` o con velocità 0 | J, FS:ConveyorSpeed (topic 3360) |
| ECV-05 | Conveyor X-divert / Y-divert; Conveyor X-merging / Y-merging | Deviatore e confluenza, a croce (90°) o a Y | J |
| ECV-06 | Diverter | Deviatore separato da attaccare al nastro | J |
| ECV-07 | Pusher Conveyor | Nastro con spintore laterale; sceglie cosa spingere con la proprietà `PushThese` (§4) | J, F2230 |
| ECV-08 | Crossing Conveyor | Incrocio con trasferimento ortogonale; supporta il comando a segnali dalla 4.10 | J, F8346, F1439 |
| ECV-09 | Turntable Conveyor; Rotary Table; Rotator Conveyor; Lift-and-Rotate Conveyor | Piattaforma girevole con nastro; tavola rotante; rotazione del prodotto; solleva e ruota secondo una regola di routing (due "preset" di meccanismo) | J, F8346, F7655 |
| ECV-10 | Lift Conveyor; Shuttle Conveyor; Vertical Paternoster; Vertical Stacker Conveyor | Elevatore, navetta laterale, paternoster, impilatore | J |
| ECV-11 | Sensor Conveyor; Processpoint Conveyor | Nastro con sensore integrato che può fermare il prodotto; nastro con punto di processo | J, F4932, F4786 (titolo) |
| ECV-12 | Product Batcher / Product Sorter / Destacker / Flipping / Horizontal Funnel | Raggruppa, smista, disimpila, capovolge, incanala | J |
| ECV-13 | Overhead Conveyor; Spiral Conveyor With Pylon; XYZ Buffer | Aereo, spirale, buffer cartesiano | J |
| ECV-14 | Foil / Stretch Wrapper Conveyor | Nastro con avvolgitrice | J |
| ECV-15 | Conveyor Builder; Material Flow Connector | Strumento che disegna un percorso di nastri; connettore di flusso | J, C17 (topic 6664 "Pathway and Conveyor Builder") |

Nei **Component Templates** ci sono modelli di partenza per chi costruisce componenti: Conveyor,
Curve, Inclining, Declining, Merge, Split, Offset, Sensor Conveyor, Feeder, Python Feeder, Turntable
Machine (J). È l'insieme di famiglie che il fornitore considera "da personalizzare".

In **Physics** (12) esistono le varianti fisiche: `Conveyor (Physics)`, `Pusher Conveyor (Physics)`,
`Physics Pusher`, `Physics Feeder (Shape Feeder)`, `Physics Funnel`, collider a blocco/cilindro/freccia,
contenitori (J). Il nastro fisico accetta velocità negativa per invertire il verso (FS:ConveyorSpeed,
topic 5847), in linea con CM-BEH-PHY-03 di 02a.

### 3.2 Nastri di marca (`Conveyors`, 160 elementi di 11 produttori)

Bosch Rexroth 82, Broich Systemtechnik 23, EWAB 20, Flowsort 13, Schnaithmann 9, Qimarox 4, Kovasat 3,
Ekobal 2, AmbaFlex 2, Apollo 1, Rhein-Nadel 1 (J). Interessano per **come scompongono** una linea:

| ID | Schema | Cosa mostra | Fonte |
|---|---|---|---|
| ECV-V1 | Sistemi a pallet portapezzo (TS 1/2/5, easyflow/easylane/easymove, Schnaithmann) | Un sistema = kit di pezzi: tratto (`Section`), tratto a cinghia, testa motrice (`Drive`), rinvio (`Return Unit`), curve, **arresto** (`Stop Gate`, `Stopper`, `Stopper Indexer`), unità di posizionamento, **unità di sollevamento e traslazione** (trasferimento ortogonale), unità di rotazione, **identificazione** (`ID`, `ID200 Communication Module`, `ID System`), gambe, pallet. | J |
| ECV-V2 | Catene a tapparelle (VarioFlow belt/plus) | Curve orizzontali e verticali, ruota di curva motorizzata, deviatore, deviatore universale, deviatore a croce, confluenza, tratto di trasferimento, **sensore**, arresto, ritegno (`Backstop`), quadro (`Cabinet`). | J |
| ECV-V3 | Curve come **elementi a catalogo fissi** (EWAB PS800) | La curva non è parametrica: angolo, verso e raggio stanno **nel nome**, un elemento per combinazione: 45° sinistra/destra, 90° sinistra/destra con raggio 150 o 225 mm, 180° orario/antiorario con raggio 150 o 225 mm. Più divisore e confluenza CW/CCW, porte a U da 300 e 450 mm, motorizzazione, supporto, pallet. | J |
| ECV-V4 | Taglie come elementi separati | Flowsort SLD/DLD/ZZ in larghezze 400/600/800/1000 (*[inferenza]*: mm, dal nome); Qimarox Prorunner mk1/mk5/mk5-XL/mk9. | J |
| ECV-V5 | Deviatori a ruote (Flowsort) | Si attaccano a inizio o fine di un nastro a rulli; per simulare l'installazione "tra i rulli" si spostano gli offset di inizio e fine del nastro (`ConveyorStartOffset`, `ConveyorEndOffset`). Tutti supportano sia la regola di routing sia il comando a segnali; il comando a segnali permette di usare sensori posizionati per anticipare la rotazione. Il modulo a croce si aggancia in qualsiasi punto del nastro a rulli con il PnP. | F9142 |
| ECV-V6 | Rulli e tavola rotante (Ekobal) | `Roller Conveyor DPN`, `Turntable DPN`: unico nastro **a rulli** dichiarato nel nome. | J |

Nessun nome distingue **motorizzato e a gravità**, né **accumulo** (zero pressure, a zone): queste
varianti non compaiono come elementi del catalogo (ricerca per `roller|gravity|accumul` su J: un solo
nastro a rulli, nessun "gravity", un "Circular Accumulator" in Warehousing).

### 3.3 Accessori dei nastri (`Conveyor Utilities`, 16, tutti generici)

| ID | Elemento | Funzione (nostre parole) | Fonte |
|---|---|---|---|
| ECU-01 | Conveyor Sensor | Sensore da agganciare a un nastro; è un sensore a **raggio** | J, F6572, F7604 |
| ECU-02 | Bladder stopper | Arresto a camera d'aria (fermo del prodotto) | J |
| ECU-03 | Conveyor Motor | Motore separato che comanda uno o più nastri, con stati e guasti (§4) | J, F5586, F5109 |
| ECU-04 | Multi-Conveyor Speed Controller | Velocità comune a più nastri | J |
| ECU-05 | Capacity Control Entry / Exit | Limita quanti prodotti stanno tra due punti | J |
| ECU-06 | Bundler Point / De-bundler Point | Unisce più prodotti in uno e li separa a valle (soluzione suggerita quando la tavola rotante accetta un solo prodotto) | J, F8346 |
| ECU-07 | Process Point; Property Setting Point; Pusher Process | Punto del percorso che lancia logica, scrive proprietà del prodotto, comanda uno spintore | J, F3540 |
| ECU-08 | Grabbing / Dropping Point; Picking Zone | Punti di presa e rilascio per robot e risorse | J |
| ECU-09 | Conveyor Tracking Base | Base per l'inseguimento del nastro da parte del robot | J |
| ECU-10 | End Block | Fine corsa del nastro *[inferenza]* | J |

### 3.4 Creazione e uscita dei prodotti

| ID | Elemento | Dove | Note | Fonte |
|---|---|---|---|---|
| EFL-01 | Feeder, Advanced Feeder, Feeder Process | Feeders e Flow Components (duplicati voluti) | Dalla 4.10 **tutti** i feeder creano *prodotti* (componenti dinamici con informazioni di processo), non più semplici componenti | J, F7406 |
| EFL-02 | Pattern / Shape / Stack / Pallet Feeder, Pallet Filler, Signal Feeder | Feeders | Creazione a schema, per forma, a pile, su pallet, **a comando di segnale** | J |
| EFL-03 | Bowl Feeder; feeder vibranti di marca (AGR, Asyril, RNA, ars) | Feeders | Le versioni AGR hanno verso e velocità della ciotola, un segnale di **attesa** (`Hold`) per fermarle quando un sensore vede l'accumulo, offset sinistro/destro sulle piste lineari | J, F6418 |
| EFL-04 | Product Creator, Box Creator, Point Cloud Creator | Misc, Packaging | Creatori specializzati | J |
| EFL-05 | Sink Process; Terminal Process | Flow Components | **L'unico "sink" del flusso** è un elemento di processo. Attenzione: `Sink` in Facilities - Interior è un lavello (arredo), non un distruttore di prodotti | J |
| EFL-06 | Products and Containers (85) | — | Prodotti campione (scatole, bottiglie, pallet parametrici, contenitori) | J |

### 3.5 Segnali, sensori, logica

| ID | Elemento | Funzione (nostre parole) | Fonte |
|---|---|---|---|
| ESG-01 | Volume Detector | Sensore a volume a forma di **box**: mette a vero un segnale booleano quando un componente entra; espone anche un segnale di tipo componente. Ha un elenco di inclusione e uno di esclusione (un pulsante esclude tutti i componenti statici, così vede solo i prodotti), un intervallo di scansione e un `PowerOnSignal`. Si possono mettere in cascata due detector, uno grande e lento che accende quello piccolo e preciso, per risparmiare calcolo. Pensato per il virtual commissioning e per i nastri con più percorsi. | J, F8350 |
| ESG-02 | Safety Scanner (generico e Hokuyo) | Area di sicurezza; può usare il rilevamento di collisione, quindi forme qualsiasi | J, F8350 |
| ESG-03 | Signal Utilities (7) | Inverter booleano, mappa int↔bool, latch, porta logica AND/OR/XOR, relè di segnale, tester di segnali, timer | J |
| ESG-04 | HMI (6) | Pulsante, manopola, fungo di emergenza, lampada LED, orologi | J |
| ESG-05 | Signal Positioners (8) | Assi comandati da I/O: cilindro, asse traslante, asse rotante, mandrino, tavola a indexaggio, sollevatore | J |
| ESG-06 | Layout Templates a segnali | `Robot Control Handshake Signals`, `Turn Table Integer Signal`, `PM Signals template` | J |
| ESG-07 | Statistics legate ai segnali | `Cycle Time (Signals)`, `Signal Value Recorder` | J |
| ESG-08 | Sensori colore e barcode | **Non esistono** come elementi: la ricerca `barcode|colou?r|photo|beam` sui nomi non trova nulla. Le cose più vicine sono l'identificazione dei pallet (`TS 1 - ID`, `VarioFlow plus - ID System`), la `Quality check camera` e le telecamere 3D di marca. Sul forum il barcode compare solo come dato scambiato da script con un sistema esterno. | J, FS:barcode (topic 3581, 2338) |
| ESG-09 | Fotocellule diffuse e a barriera | **Non distinte** nel catalogo: un solo `Conveyor Sensor` a raggio, più `Safety Light Curtain` (barriera di sicurezza, in Facilities - Factory). | J, F6572 |

## 4. Parametri e segnali visibili pubblicamente

**J** non contiene parametri. Quelli sotto vengono dal **forum**: annunci del team (C17) o risposte a
utenti. Sono nomi di proprietà e segnali citati nel testo, non letti nei file. Unità solo dove la fonte
le scrive.

### 4.1 Nastri

| ID | Proprietà / segnale | Componente | Tipo e unità | Cosa fa (nostre parole) | Fonte |
|---|---|---|---|---|---|
| EP-01 | `ConveyorSpeed` | nastri generici, nastro fisico | real, mm/s (il forum cita "-200 mm/s" e "m/min" nelle domande) | Velocità del nastro; lo script la copia sulla `Speed` del path interno. Sul nastro fisico negativa = verso inverso | FS:ConveyorSpeed (topic 1938, 6934, 5847, 1652), F3540 |
| EP-02 | `ConveyorCapacity` | nastri, tavola rotante | int | Quanti prodotti può tenere; sulla tavola rotante solo 1 funziona correttamente | FS:ConveyorSpeed (topic 1938), F8346 |
| EP-03 | `PowerOnSignal` | Two Way Conveyor e altri | bool | Ferma o avvia il nastro | FS:ConveyorSpeed (topic 3360), F7604 |
| EP-04 | geometria utente | nastri generici | — | Il cambio di tipo o di geometria rigenera il componente (handler sugli eventi di proprietà) | FS:ConveyorSpeed (topic 1938) |
| EP-05 | `ConveyorStartOffset`, `ConveyorEndOffset` | nastro a rulli | real (lunghezza, unità non scritta) | Accorciano il tratto attivo ai due estremi | F9142 |
| EP-06 | `ControlMode` = {Routing Rule, Signal} | Turntable, Shuttle, Crossing, Lift, Flowsort | enum | Con **Routing Rule** decide il componente; con **Signal** decide il PLC o un processo. Il comando a segnali esiste su Crossing, Lift, Shuttle e Turntable dal catalogo 4.10 | F8346, F9142 |
| EP-07 | `ConveyorPowerOn` | Turntable in modo Signal | bool, default falso | Va acceso dal controllore, altrimenti i prodotti restano fermi | F8346 |
| EP-08 | rotazione a una porta | Turntable | segnale per porta (*[inferenza]*: int o bool per porta) | Il segnale porta la tavola verso la porta richiesta; le porte sono numerate; il nastro ha **un solo verso** di ingresso; il simbolo di direzione si può nascondere | F8346 |
| EP-09 | `RoutingRule`, `RuleComponent`, tabella booleana, porta `Default` | Turntable | — | Smistamento per regola; se nessuna regola vale, il prodotto va alla porta di default | F7019 |
| EP-10 | `PushThese` | Pusher Conveyor | string | Valore di `ProdID` dei prodotti da spingere; i feeder base assegnano `ProdID = 111` per default | F2230 |
| EP-11 | `Transport::LockProductOrientation` | risorsa mobile "Lift and Turn" | bool | Mantiene l'orientamento del prodotto nel mondo mentre il veicolo ruota | F6580 |

### 4.2 Motore del nastro (`Conveyor Motor`)

Il caso più completo pubblicamente: un componente separato, con macchina a stati e guasti.

| ID | Proprietà / segnale | Tipo e unità | Cosa fa (nostre parole) | Fonte |
|---|---|---|---|---|
| EP-M1 | `PowerOn` | bool, segnale | Accende (stato Idle) o spegne (stato Off) il motore | F5586 |
| EP-M2 | `RunForward` | bool, segnale | Marcia avanti (nella 4.8 ha preso il nome del vecchio `PowerOn`) | F5586 |
| EP-M3 | `State` | enum {Off, Idle, Running, Fault}, sola lettura | Stato, mostrato anche con un colore (nero, giallo, verde, rosso) | F5586 |
| EP-M4 | `Fault` | bool, sola lettura | Motore guasto | F5586 |
| EP-M5 | `ConveyorType` | enum {One-way, Two-way}, sola lettura | Tipo di nastro collegato | F5586 |
| EP-M6 | `TargetSpeed`, `ActualSpeed`, `RampTime` | real (unità non scritte) | Velocità richiesta, velocità reale, tempo di rampa: il motore **accelera**, non salta di velocità | F5109 |
| EP-M7 | `PresetSpeed1`, `PresetSpeed2` + segnali `SetPresetSpeed1/2` | real + bool | Due velocità preimpostate richiamabili da segnale | F5586 |
| EP-M8 | `Failures::MTBF`, `Failures::MTTR` | real, secondi | Guasti casuali: tempo medio tra guasti e di riparazione | F5586 |
| EP-M9 | `AutomaticRepair` | bool | Torna da Fault a Idle da solo dopo MTTR | F5586 |
| EP-M10 | `ExternalFault`, `FaultReset` | bool, segnali | Guasto forzato dall'esterno e reset | F5586 |

### 4.3 Sensori e feeder

| ID | Proprietà / segnale | Componente | Tipo e unità | Cosa fa (nostre parole) | Fonte |
|---|---|---|---|---|---|
| EP-S1 | `SensorBooleanSignal` | Conveyor Sensor | bool | Vero mentre il raggio vede un prodotto | F7604, F6572, FS:SensorBooleanSignal (topic 3127, 3980) |
| EP-S2 | `TimeLagSignal` + `Time Lag` | Conveyor Sensor | bool + real, s | Diventa vero quando il sensore resta occupato per il tempo impostato (un ritardo all'eccitazione, non allo spegnimento) | F7604 |
| EP-S3 | segnale booleano + segnale componente, volume box (angoli inferiore e superiore + posa), inclusione/esclusione, intervallo di scansione, `PowerOnSignal`, modo di rilevamento (triangoli, bounding box, centro del bounding box) | Volume Detector | bool, component, vector, matrix | Vedi ESG-01 | F8350 |
| EP-S4 | `Interval`, `Part` (scheda ProductCreator) | Feeder Process | real (s *[inferenza]*), riferimento al tipo di prodotto | Cadenza e prodotto da creare; modo batch con tempi per prodotto | FS:feeder interval product (topic 3856, 9527) |
| EP-S5 | `Hold`; verso e velocità; offset sinistro/destro | feeder vibranti AGR | bool; enum + real; real | Vedi EFL-03 | F6418 |

## 5. Distribuzione, versioni e termini (nuovo rispetto a 01 §3)

| ID | Elemento | Cosa abbiamo trovato | Fonte |
|---|---|---|---|
| DIS-01 | Un catalogo per versione | Ogni versione maggiore dell'app ha il suo catalogo (4.7, 4.8, 4.9, 4.10, 5.0, 5.1); quando esce la nuova, la precedente è **congelata**. Motivo dichiarato: si possono aggiornare comportamenti e geometrie senza rompere i layout delle versioni vecchie. | F7406, F5585, F5095, F9110 |
| DIS-02 | Librerie legacy | I cataloghi vecchi restano come sorgenti; dalla 4.9 sono spenti per default (solo il precedente è acceso) per velocizzare l'indicizzazione. Servono ad aprire layout salvati **senza** includere i componenti: in quel caso la geometria si recupera dal catalogo tramite VCID, e se manca compaiono errori di VCID e geometrie mancanti. | F6580, F7406 |
| DIS-03 | "Include Components" | Salvare il layout con i componenti inclusi è il modo raccomandato per migrare tra versioni. | F7406, F9110 |
| DIS-04 | Formato non retrocompatibile | Il `.vcmx` della 5.0 è cambiato molto: i modelli 5.0 non si aprono nelle versioni precedenti; gli script dei componenti sono in migrazione a Python 3. | F9110 |
| DIS-05 | Sorgenti extra | Il nuovo motion controller (beta, 5.0) è distribuito come sorgente separata da abilitare, per tenere separati modelli vecchi e nuovi. | F9110 |
| DIS-06 | Server | Dalla 4.9 il catalogo dell'app è servito da `ecat.visualcomponents.net` (prima `download.visualcomponents.net`); serve aprirlo nel firewall. Una `GET` alla radice restituisce 404: non abbiamo cercato oltre. | F6580 |
| DIS-07 | Sorgenti remote di terzi | Dalla 4.9 una sorgente remota mostra la sua struttura a cartelle; prima si organizzava solo con smart collection. | F7210 |
| DIS-08 | Ricerca | Nell'app la ricerca vale solo dentro la collezione selezionata; per cercare ovunque si sceglie una collezione radice ("Models By Type", "All Components"), più lenta. | F2628 |
| DIS-09 | Evoluzione dei `Type` | Nella 4.10 tolto il prefisso "PM" da 9 categorie (es. PM Flow Components → Flow Components), unificati i tipi di settore, spostati i layout OLP in una categoria propria; nella 4.8 nata "Signal Positioners" per separare gli assi comandati da segnali dagli assi esterni veri; nella 5.1 nata "Navigation (Beta)". | F7406, F5585, F9542 |
| DIS-10 | Annunci | Ogni componente nuovo o rivisto ha un topic nella categoria "eCat Updates" (232 topic dal 2017), spesso con le proprietà nuove. Nella 4.7 c'è un riferimento ai "metadati del componente" come luogo della documentazione delle proprietà. | C17, F5109 |
| DIS-11 | Termini d'uso | Il team non distribuisce i modelli del catalogo sul forum: li considera **parte del prodotto** e un elemento di differenziazione commerciale. Nessun elemento di **J** porta una licenza. I contratti (EULA, termini) sono PDF linkati dalla pagina legale: **non letti**. | F7655, `https://www.visualcomponents.com/legal-information/` |
| DIS-12 | Autori | `Author` ≠ `Manufacturer`: 3044 elementi firmati "Visual Components", 532 senza autore, il resto da produttori o integratori (KUKA in quattro grafie diverse, Bosch Rexroth, Blue Danube Robotics, …). Molti modelli di marca sono fatti dal team in collaborazione col produttore (es. AGR). | J, F6418 |

## 6. Cosa NON dimostra

- **Non dimostra i parametri del catalogo.** La web e il suo JSON non espongono proprietà, segnali,
  interfacce, dimensioni, unità, versioni o licenze. Tutto il §4 viene dal forum: è ciò che team e
  utenti **citano**, non l'elenco completo delle proprietà di un componente. Mancano del tutto, per
  esempio, lunghezza, larghezza e altezza del nastro generico, angolo e raggio del `Curve Conveyor`,
  corsa e tempo dello spintore.
- **Non è stato aperto nessun componente.** I file `.vcmx` non sono scaricabili dalla web; scaricarli
  dall'app richiederebbe un'installazione con licenza, e il fornitore li considera parte del prodotto
  (DIS-11). Non ci abbiamo provato.
- **Funzione dal nome.** In §3 molte funzioni sono dedotte dal nome (*[inferenza]*), non dal
  comportamento.
- **Unità.** Dove la fonte non scrive l'unità non l'abbiamo aggiunta; "mm/s" per `ConveyorSpeed` viene
  da un post del team sul nastro fisico e da script di utenti, non da una scheda del componente.
- **Fotografia di un giorno.** Conteggi del 2026-11-08 sul catalogo **5.1** soltanto; i cataloghi
  precedenti e le sorgenti extra (motion controller beta) non sono nella web. Il conteggio dichiarato
  (4183) e quello elencato (4177) non coincidono e non sappiamo perché.
- **Non letti:** i 232 topic di C17 (letti 16, più 10 topic di domande degli utenti in altre categorie), i PDF dei termini, le pagine Academy, il
  server `ecat.visualcomponents.net` oltre la radice. L'help 5.1 non ha pagine sui singoli componenti del
  catalogo (H: le uniche voci sono il pannello eCatalog e i behavior dei sensori, già in 01 e 02a).
- **Bloccato:** niente. Nessuna risorsa ha chiesto login; il server del catalogo dell'app risponde 404 a
  una richiesta anonima alla radice e non è stato sondato oltre.
- **Forum = utenti.** Le risposte di utenti non del team possono essere sbagliate; dove contava abbiamo
  preferito i post del team (autori degli annunci in C17).

## 7. Implicazioni per M4.4 (ipotesi)

Ipotesi da validare nella spec di 4.1, non decisioni. Criteri ricavati dalle evidenze sopra:

1. **Parametrico, non un elemento per taglia.** I nastri di marca moltiplicano gli elementi (EWAB: 10
   curve per angolo × raggio × verso, ECV-V3); i generici hanno un elemento per famiglia. Per una
   libreria v1 piccola conviene la seconda strada: angolo, raggio e verso come proprietà.
2. **Due modi di comando su ogni componente con una decisione** (deviatore, tavola, navetta, incrocio):
   `Rule` (decide il componente) e `Signal` (decide il PLC), come EP-06. Per Twinova il modo `Signal` è
   quello di default, perché lo scopo è la messa in servizio virtuale.
3. **Il motore separato dal nastro** (EP-M1…M10) è il modello più vicino a un azionamento reale (marcia,
   pronto, guasto, rampa, reset). Proposta: nella v1 i segnali di azionamento stanno nel nastro, con gli
   stessi nomi che avrebbe un motore separato, così un `Motor` separato può arrivare dopo senza
   rinominare.
4. **I sensori sono raggi o volumi** (ESG-01, ESG-09); colore e barcode non esistono come elementi: si
   possono fare come un raggio che legge una **proprietà del prodotto**.
5. **Creator e sink producono e consumano prodotti tipizzati** (EFL-01), con un identificativo di tipo
   che gli altri componenti possono filtrare (`ProdID`/`PushThese`, EP-10).

Insieme minimo proposto. Unità SI del modello di Twinova, mm e s nell'interfaccia; tipi IEC 61131-3
per i segnali (`BOOL`, `INT`, `REAL`, `STRING`), perché la logica è in ST.

| Componente | Parametri (unità) | Segnali verso il PLC (`BOOL` salvo indicato) | Interfacce | Evidenza |
|---|---|---|---|---|
| Nastro a cinghia | `length` (mm), `width` (mm), `height` (mm, quota del piano), `speed` (mm/s, ≥0), `ramp_time` (s), `capacity` (INT, 0 = limitato dalla lunghezza) | in: `run_fwd`, `run_rev` (se bidirezionale), `reset`; out: `running`, `fault` | `flow_in`, `flow_out` (1:1) | EP-01…04, EP-M2/M6/M10, ECV-04 |
| Nastro a rulli | come la cinghia + `roller_pitch` (mm), `start_offset`/`end_offset` (mm) | come la cinghia | come la cinghia + aggancio laterale 1:N per deviatori e moduli | EP-05, ECV-V5/V6 |
| Curva | `angle` (°, con segno = verso), `inner_radius` (mm), `width`, `height`, `speed` | come la cinghia | `flow_in`, `flow_out` | ECV-02, ECV-V3 |
| Deviatore | `mode` (enum {Rule, Signal}), `outputs` (INT, 2 nella v1), `switch_time` (s), `default_output` | in: `select` (INT) o `divert`; out: `in_position` | `flow_in`, `flow_out[n]` | EP-06, EP-09, ECV-05/06, ECV-V5 |
| Arresto (stopper) | `position` (mm lungo il nastro), `release_time` (s) | in: `release`; out: `part_present` | aggancio laterale al nastro | ECV-V1 (`Stop Gate`), ECU-02 |
| Spintore | `stroke` (mm), `push_time` (s), `return_time` (s), `filter` (STRING, tipo di prodotto) | in: `push`; out: `extended`, `retracted` | aggancio al nastro + `flow_out` laterale | EP-10, ECV-07 |
| Tavola rotante | `angles` (lista in °), `rotate_time` (s per 90°), `capacity` = 1, `mode` (Rule/Signal), dimensioni del nastro | in: `target_port` (INT), `belt_run`; out: `at_port` (INT), `in_position`, `part_present` | `flow_port[n]` (porte numerate) | EP-02/06/07/08, ECV-09 |
| Sensore diffuso | `range` (mm), `beam_offset` (posa lungo il nastro), `on_delay` / `off_delay` (s) | out: `detected`, `detected_delayed` | aggancio 1:N al nastro | EP-S1/S2, ECU-01 |
| Sensore a barriera | `emitter`/`receiver` (pose), `on_delay` / `off_delay` (s) | out: `beam_broken` | aggancio 1:N al nastro (o due agganci) | ESG-09 *[ipotesi: la distinzione non è nel catalogo]* |
| Sensore colore | come il diffuso + `property` letta (default `color`) | out: `detected`; `color_code` (INT) | aggancio 1:N | ESG-08 *[ipotesi]* |
| Lettore barcode | come il diffuso + `property` letta (default `barcode`) | out: `read_ok`; `code` (STRING) | aggancio 1:N | ESG-08 *[ipotesi]* |
| Creator | `product_type` (rif.), `interval` (s, o distribuzione), `max_count` (INT, 0 = infinito), `mode` (enum {Interval, Signal}) | in: `create` (se `Signal`), `hold`; out: `blocked` | `flow_out` | EP-S4/S5, EFL-01…03 |
| Sink | `filter` (STRING, opzionale) | out: `count` (INT, contatore) *[ipotesi]* | `flow_in` | EFL-05 |

Restano aperti per la spec di 4.1, perché le fonti tacciono: profilo di moto dello spintore e della
tavola (rampa o tempo fisso), semantica di `capacity` sui nastri lunghi (accumulo a contatto o a
distanza), e se un sensore notifica a ogni scansione o solo sul fronte (già lacuna in 00 §7).
