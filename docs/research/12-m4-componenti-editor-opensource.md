# M4: componenti ed editor — simulatori industriali e codice open source

Data: 2026-11-08. Stato: **ricerca per il brainstorming, non specifica approvata**.

## Metodo e perimetro

Lettura della ricerca 00–11, del codice attuale di Twinova e di sorgenti/documentazione dei riferimenti sotto.
Checkout parziali in `D:\tmp`, senza installarne dipendenze né eseguirne le applicazioni.
Le osservazioni sul codice sono verificate staticamente; prestazioni e usabilità non sono misurate qui.
Le note precedenti restano storiche: le proposte Python/WASM nella matrice 06 sono superate dalle decisioni
Rust + nexosim + truST/ST + Tauri/Svelte/Three.js. Questa ricerca non riapre quelle scelte.

## 1. Cosa prendere dai simulatori industriali

| Riferimento | Evidenza consultata | Indicazione per Twinova, ancora proposta |
|---|---|---|
| Factory I/O | [Edit/Run](https://docs.factoryio.com/manual/edit-and-run/): inserimento dal catalogo, trasformazioni, gruppi, separazione dei modi | Primo percorso breve: inserisci, configura, collega, esegui. Modifiche strutturali nel modo Edit |
| Factory I/O | [Tag](https://docs.factoryio.com/manual/tags/): sensori e attuatori con tag, tipi BOOL/REAL/DINT e forzature | I/O visibili accanto all'oggetto; valori forzati distinguibili da quelli simulati |
| Visual Components | [Interfacce](https://help.visualcomponents.com/4.8/Premium/en/English/Component%20Modeling/Behaviors/Interfaces/Interfaces.htm), documento ufficiale 4.8; mappa locale 5.1 in 02a | Separare montaggio fisico, collegamento di comportamento e trasferimento. Porte con un riferimento spaziale esplicito |
| FlexSim | Documentazione ufficiale indicizzata su [connessioni dei nastri](https://docs.flexsim.com/en/25.2/ConnectingFlows/Conveyors/WorkingWithConveyors/WorkingWithConveyors.html) | Il trasferimento merita una semantica propria, inclusi ingresso laterale, capacità e blocco. Il link ora reindirizza a una pagina Autodesk non trovata: evidenza più debole, da riconfermare prima di dipenderne |
| Plant Simulation | Ricerca precedente [g2](benchmark/g2-flusso-des.md), classi e istanze | Definizione del componente distinta dalle istanze; rendere visibili override e versione. Nessuna nuova prova del prodotto in questa sessione |
| realvirtual-WEB | [Planner](https://github.com/game4automation/realvirtual-WEB/blob/67fbd47ff248f79ed231507f726e42194b7a5994/doc-layout-planner.md) e tipi di connessione nel codice | Scena centrale, catalogo e proprietà contestuali; snap tipizzati con direzione e occupazione. È un riferimento, non una decisione di copiare il prodotto |

Il confronto riguarda comportamenti e flussi operativi. Non sono stati scaricati cataloghi proprietari di componenti.

## 2. Repository letti e cosa è riusabile

| Repository / checkout | Revisione letta | Licenza nei file letti | Valutazione |
|---|---|---|---|
| [Open Industry Project](https://github.com/Open-Industry-Project/Open-Industry-Project), `D:\tmp\Open-Industry-Project` | `05728680455137294bb8e39fb686dcf5c3670c50` | MIT, `LICENSE`; dipendenze e asset hanno verifiche proprie | Miglior candidato da cui valutare piccoli moduli TS e convenzioni di progetto; non è un motore sostituibile senza lavoro |
| [realvirtual-WEB](https://github.com/game4automation/realvirtual-WEB), `D:\tmp\realvirtual-WEB` | `67fbd47ff248f79ed231507f726e42194b7a5994` | AGPL-3.0 per il codice; `CONTENT-LICENSE.md` separata per contenuti elencati | Riferimento funzionale/UI. Secondo la policy Twinova, niente incorporazione del codice AGPL. I contenuti soggetti alla licenza specifica non sono una libreria da importare in Twinova |
| [JaamSim](https://github.com/jaamsim/jaamsim), `D:\tmp\JaamSim` | `d09e4e67a2ac73e418e86782e7fbe84b4d8c125d` | Apache-2.0, `LICENSE` e intestazioni dei sorgenti | Riferimento concreto per trasferimento, accumulo e disponibilità a valle. Java: non una dipendenza diretta per il nostro motore Rust |
| [Three.js](https://github.com/mrdoob/three.js), `D:\tmp\three-reference` | `9c4515e66f479864e39b72f29e10260821576b4a` | MIT, `LICENSE` | Gizmo e loader già disponibili; editor utile per comandi/undo. Non fornisce il modello industriale |

Le licenze sono state lette ai commit indicati; nessun codice o asset di questi checkout è stato copiato in Twinova.
Un eventuale riuso deve conservare gli avvisi del file e verificare dipendenze e asset coinvolti.

### Open Industry Project: separare definizione, istanza e progetto

File letti nel checkout:

- `src/core/types/parts.ts`: istanze con `packageId` e parametri; scena versionata.
- `src/core/project/project.ts` e `projectFolder.ts`: progetto versionato, formato cartella,
  scene JSON, programmi sotto `plc/`, definizioni e asset separati.
- `docs/projects.md`: le definizioni usate sono conservate nel progetto; un aggiornamento del catalogo non
  sostituisce automaticamente le copie del progetto.
- `src/core/parts/partApi.ts`: contesto limitato per i comportamenti; parametri con metadati per l'inspector.
- `docs/architecture.md`: modello indipendente da rendering; comportamento JS e simulazione con Jolt a tick.
- `docs/part-snap-connectors.md` e `src/core/snap/snapConnectors.test.ts`: compatibilità e ranking degli agganci,
  con casi specifici per nastri. Non tutti i collegamenti sono un semplice snap sulla bounding box.

**Applicazione proposta:** definizioni parametriche e istanze con ID stabili, dipendenze locali al progetto,
metadati sufficienti a generare l'inspector. Conservare ST e tempo virtuale di Twinova: il comportamento JS a tick
di OIP non va trasferito automaticamente al DES.

### realvirtual-WEB: porte esplicite e stato serializzato

File letti: `src/plugins/layout-planner/rv-layout-store.ts`, `src/core/connect/snap-name-parser.ts`,
`doc-layout-planner.md`, `LICENSE`, `CONTENT-LICENSE.md`.
Le istanze conservano trasformazione, riferimento al catalogo, mapping dei segnali e override.
I punti di aggancio distinguono tipo e verso: ingressi/uscite compatibili, anche bidirezionali.

**Applicazione proposta:** connessione industriale con ID e tipo espliciti; i nomi visualizzati dei nodi possono
cambiare senza rompere i legami. Non adottare come formato Twinova la convenzione dei nomi `Snap-*`.

### JaamSim: non perdere o duplicare il pezzo al confine

File letti: `src/main/java/com/jaamsim/ProcessFlow/EntityConveyor.java` e `LinkedComponent.java`.
Il nastro tiene traccia dello spazio occupato, della disponibilità all'ingresso e degli eventi successivi;
il componente collegato riceve le entità trasferite. L'accumulo e il blocco a valle hanno regole esplicite.

**Applicazione proposta:** definire proprietà e trasferimento del prodotto prima di disegnare la linea.
Prove necessarie: valle piena, due ingressi nello stesso istante, prodotto più lungo del tratto e deviazione
mentre un prodotto attraversa il confine. L'ID del prodotto resta unico durante ogni trasferimento.

### Three.js: riuso disponibile nell'attuale dipendenza

Verificato anche in `app/node_modules/three/examples/jsm/` di Twinova:

- `controls/TransformControls.js`: spostamento, rotazione e scala; `setTranslationSnap`, `setRotationSnap`,
  `setScaleSnap` e fine gesto `mouseUp`.
- `loaders/GLTFLoader.js` e `exporters/GLTFExporter.js`: disponibili.
- Nell'editor upstream, `editor/js/History.js` e `commands/SetPositionCommand.js` mostrano undo/redo,
  posizione prima/dopo, aggregazione di operazioni successive e serializzazione del comando.

**Applicazione proposta:** riusare controlli e loader; gestire i comandi sul documento di dominio Twinova.
L'undo di una cancellazione deve ripristinare anche segnali e connessioni, non solo l'oggetto Three.js.

## 3. Punto di partenza reale di Twinova

- `app/src-tauri/src/bench.rs` costruisce un nastro e un PLC con porte definite nel codice.
- `app/src-tauri/src/plc.rs` richiede il contratto fisso `sensor/start/stop/motor`.
- `app/src/lib/SceneView.svelte` costruisce direttamente le primitive del nastro e legge snapshot.
- `scene.rs` è il riferimento a passo fisso; il runtime usa il nastro analitico in `belt.rs`.

Il primo salto di M4 è quindi introdurre un documento con componenti e connessioni e materializzarlo nel
motore e nella vista. Cambiare solo la disposizione dei pannelli non renderebbe assemblabile una linea.

## 4. Tre approcci da discutere

| Approccio | Primo risultato | Costo/limite |
|---|---|---|
| A — Catalogo parametrico e gruppi riutilizzabili (consigliato) | Costruire e salvare una linea con componenti funzionanti, configurabili nell'inspector | L'authoring completo di una nuova famiglia arriva dopo; schema già estensibile |
| B — Editor completo del componente | Creare geometria, giunti, porte e comportamento ST dall'app | Aggiunge alla M4 anche un ambiente di authoring; più semantica e più test prima della prima linea |
| C — CAD/GLB prima, poi comportamento | Importare una macchina e attribuire sensori/attuatori | Utile al commissioning di macchine reali; import, pivot e associazione geometria/semantica anticipano lavoro previsto più avanti |

Raccomandazione attuale: A, con un modello capace di crescere verso B. È una proposta dell'assistente;
la scelta dell'utente è ancora aperta. Non restringe automaticamente l'intera milestone M4.

## 5. Proposta di contratto minimo per M4.1

Da validare, non API implementata:

1. **Definizione**: ID/versione, parametri con unità e limiti, nodi, geometria, porte e comportamento.
2. **Istanza**: ID stabile, riferimento alla definizione, trasformazione e override locali.
3. **Montaggio**: trasformazione relativa e compatibilità; un sensore montato segue il nastro.
4. **Trasporto**: ingresso/uscita di prodotti, occupazione, accettazione a valle e spareggio deterministico.
5. **Segnale**: tipo IEC, direzione vista dal componente, valore iniziale e mapping; l'output del sensore
   diventa input del PLC. Nomi rinominabili separati dagli ID usati nei collegamenti.
6. **Stato**: progetto editabile distinto dallo stato runtime. Un gesto di drag produce una sola modifica annullabile.
7. **Persistenza**: JSON versionato + asset glTF/GLB + programmi `.st`; definizioni fissate nel progetto.
   Salvataggio/riapertura e aggiornamento di una definizione devono preservare collegamenti e override.

Prima scena di accettazione proposta: sorgente → nastro A → deviatore → nastro B / uscita di scarto,
due sensori e destinazioni esplicite. Bloccare B, cambiare uscita, salvare/riaprire e ripetere gli stessi ingressi:
nessun pezzo perso/duplicato, stesso risultato a Step, ×1 e non scandito dal tempo reale.

Lettura proposta dell'editor: catalogo/albero a sinistra, scena al centro, inspector a destra; ST e tabella
segnali apribili in basso. Realvirtual informa la centralità della scena; ergonomia e densità vanno provate
con un mockup prima di fissarle.

## 6. Decisioni ancora aperte

La prima domanda inviata all'utente riguarda il livello di authoring: assemblare/configurare, creare famiglie,
oppure partire da CAD importato. Dopo questa scelta si affrontano: gruppi con interfacce pubbliche, aggiornamento
delle definizioni, editing durante Run e vincoli geometrici del trasferimento. Non sono state assunte risposte.

## Limiti della ricerca

- Lettura statica di porzioni nominate, non audit completo né esecuzione dei quattro progetti.
- Nessuna nuova misura di velocità, memoria o fedeltà fisica.
- I test presenti nei riferimenti sono stati letti, non rieseguiti.
- La documentazione commerciale descrive il prodotto; non è una prova pratica.
- Alcuni risultati FlexSim sono indicizzati ma il link corrente non si apre: non usarli come contratto implementativo.
- Un clone pubblico non rende tutti i modelli e tutte le dipendenze riutilizzabili sotto la licenza del codice.
