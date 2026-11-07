# Benchmark G2: simulatori material-flow / DES

Prodotti: Siemens Tecnomatix Plant Simulation, Autodesk FlexSim, AnyLogic, Simio.
Ricerca del 2026-11-07. Il testo è una sintesi clean-room: le funzioni sono descritte con parole mie, senza copiare testi o immagini.

**Convenzioni**
- Ogni affermazione porta una chiave di fonte tra parentesi quadre, ad esempio [F3]. L'URL completo è nella tabella "Fonti" in fondo a ogni scheda.
- [secondaria] indica forum, Wikipedia, blog di terzi o aggregatori.
- [inferenza] indica una mia deduzione, non un fatto documentato.

**Stato delle fonti**
- **FlexSim:** il manuale online (docs.flexsim.com) ora reindirizza a help.autodesk.com. L'indice JSON e le pagine HTML statiche si leggono con curl e li ho letti (versione 2027). www.flexsim.com risponde 403 e non ho aggirato il blocco.
- **AnyLogic:** anylogic.help si legge per intero.
- **Plant Simulation:** l'help Siemens (docs.plm.automation.siemens.com) risponde 400/403 alle richieste automatiche, quindi la scheda si basa su pagine prodotto, blog Siemens, il tutorial TU Darmstadt e Wikipedia.
- **Simio:** il sito non ha un help pubblico raggiungibile. Ho usato pagine prodotto, le note di rilascio in PDF e i paper WSC.

---

## 1. Siemens Tecnomatix Plant Simulation (oggi "Plant Simulation X")

### Cosa è e per chi
- È un simulatore a eventi discreti di sistemi di produzione, flussi di materiale e logistica [P1][P7].
- È orientato agli oggetti, gerarchico e in 3D, e fa parte del portafoglio PLM/Digital Manufacturing Siemens [P1][P7].
- Il mercato tipico, secondo Wikipedia, comprende automotive e fornitori, costruttori di impianti, magazzini automatici, AGV ed elettrovie, cantieri navali e terminal container [P8 secondaria].
- La storia risale a SIMPLE++ (Fraunhofer IPA, anni '80), poi diventato eM-Plant e infine Tecnomatix Plant Simulation [P8 secondaria].

### Licenze ed edizioni
- "Plant Simulation X" è venduto come SaaS [P2][P3]. Le edizioni:

| Edizione | Limite e contenuto |
|---|---|
| Essentials | fino a 500 oggetti [P1][P2] |
| Standard | fino a 4000 oggetti [P1][P2] |
| Advanced | senza limite dichiarato; aggiunge cost analyzer, energy analyzer e sequential sampler [P3] |
| Runtime | esegue e visualizza modelli ed esperimenti già fatti, senza modellazione [P1] |

- Add-on: VSM Library e "Optimize my plant", cioè Plant Simulation come servizio insieme a Mendix [P1].
- C'è un trial cloud gratuito di 30 giorni [P4].
- Il prezzo non compare nelle pagine statiche Siemens. Un aggregatore (G2) citerebbe circa 419 USD al mese per Essentials, ma non sono riuscito ad aprire la pagina, quindi il dato non è verificato.

### Funzionalità principali

| Funzionalità | Cosa fa | Fonte |
|---|---|---|
| Modello a oggetti con ereditarietà | Le classi stanno in librerie. Una modifica alla classe si propaga a tutte le istanze. Gli oggetti si annidano (modelli dentro modelli) | [P1][P8] |
| Oggetti di flusso materiale | Source/Drain, SingleProc, Assembly, Buffer, FlowControl, Connector, Frame/Interface per la gerarchia (tutorial versione 11) | [P6] |
| Operatori e risorse | Worker, Workplace, WorkerPool, FootPath, Broker, ShiftCalendar | [P6] |
| Trasporto | Transporter su Track, AGV a guida libera, conveyor. In 2606: frenata automatica a destinazione, velocità in curva con accelerazione e decelerazione, wizard per chiudere i gap delle linee, carrier multi-pezzo | [P5] |
| Logica: SimTalk | Linguaggio proprietario nei "Method", eseguiti su eventi (entrata pezzo, reset, init). Interpretato riga per riga. Ha l'Event Controller con Reset e Init | [P6][P9] |
| Copilot LLM (2606) | Genera codice SimTalk da linguaggio naturale. Si può usare il LLM ospitato da Siemens oppure un proprio LLM ("bring your own LLM") | [P5] |
| Analisi | Bottleneck analyzer, Sankey, Gantt, throughput, statistiche di macchine, risorse e buffer, energy e cost analyzer | [P1][P3] |
| Esperimenti e ottimizzazione | Experiment manager, algoritmi genetici, reti neurali integrate, sequential sampler per intervalli di confidenza e warm-up. Ottimizzazione con Simcenter HEEDS | [P1][P3][P7] |
| 3D e CAD | Librerie 3D, import di dati CAD (formato JT e tecnologia Siemens per scene grandi). Layout da AutoCAD e MicroStation | [P1][P7] |
| Interfacce | ActiveX, C-Interface, COM, JSON, MQTT, ODBC, OPC Classic, OPC UA, Oracle SQL, Socket, XML | [P1][P2] |
| Integrazione Siemens | NX Line Designer, Teamcenter, HEEDS, Opcenter APS, **TIA Portal, PLCSIM Advanced, SIMIT** | [P1] |
| Virtual commissioning | Si spegne la logica interna del modello e il PLC reale o emulato (TIA Portal e PLCSIM Advanced) prende il controllo. SIMIT copre safety e inverter, WinCC l'HMI | [P10 secondaria] |
| Python / API | Nessuna API Python ufficiale documentata nelle pagine che ho letto. Esistono wrapper della comunità sull'interfaccia COM (pyplantsim) | [P11 secondaria] |
| Collaborazione | Git per i modelli, VR con HTC Vive, cinematiche PLMXML (presentazione roadmap 2021) | [P12] |

### Architettura del motore
- È un DES con Event Controller, che controlla avvio, reset e fine simulazione [P6][P11].
- I Method SimTalk vengono interpretati a runtime, non compilati [P6].
- Esiste un modello fluidi a "PatchMatrix", citato nella roadmap 2021 [P12].
- Non ho trovato nelle fonti accessibili come sono gestiti la event list e gli eventi simultanei, né come funziona il passo continuo. L'help ufficiale è bloccato.

### Punti di forza distintivi
- L'ereditarietà vera tra libreria e istanze, insieme alla gerarchia, è il cuore del prodotto [P1][P8].
- È l'unico del gruppo con un collegamento di prima parte a TIA Portal, PLCSIM Advanced e SIMIT, cioè una catena di virtual commissioning Siemens [P1][P2].
- Analisi industriali pronte all'uso: energia, costi, bottleneck e Sankey [P1][P3].

### Debolezze e critiche note
- Il blog Siemens su 2606 riconosce due limiti delle versioni precedenti [P5]:
  - la scena 3D era meno leggibile del layout 2D a icone;
  - molti utenti dovevano scrivere SimTalk a mano per comportamenti comuni, come la frenata dei trasportatori o i tempi dei carrier multi-pezzo.
- SimTalk è un linguaggio proprietario e interpretato [P6]. Un'API Python ufficiale non risulta documentata [P11].
- I limiti di oggetti per edizione (500 e 4000) spingono verso l'edizione Advanced [P1][P2].

### Cosa prendere per il nuovo simulatore [inferenza]
- Classi di libreria con ereditarietà "live", che propagano le modifiche alle istanze, e modelli annidati.
- Una "modalità emulazione" esplicita: lo stesso modello può girare con la logica interna oppure con I/O verso un PLC esterno (OPC UA, S7 o PLCSIM Advanced).
- Comportamenti cinematici standard (rampe, curve, frenata a destinazione) già nella libreria, invece che negli script.
- Un assistente LLM che genera la logica conoscendo l'API degli oggetti.

### Fonti

| Chiave | URL |
|---|---|
| P1 | https://plm.sw.siemens.com/en-US/tecnomatix/plant-simulation-software/ |
| P2 | https://www.siemens.com/en-us/products/tecnomatix/offerings/plant-simulation-x-essentials/ |
| P3 | https://www.siemens.com/en-us/products/tecnomatix/offerings/plant-simulation-x-advanced/ |
| P4 | https://plm.sw.siemens.com/en-US/tecnomatix/products/compare-plant-simulation-x-software/ |
| P5 | https://blogs.sw.siemens.com/tecnomatix/?p=12529 (Plant Simulation 2606, 2026-06-25) |
| P6 | https://www.iim.maschinenbau.tu-darmstadt.de/kursunterlagen/tecnomatix_plant_simulation_11/method.html (anche index.html) |
| P7 | https://en.wikipedia.org/wiki/Plant_Simulation [secondaria] |
| P8 | https://de.wikipedia.org/wiki/Plant_Simulation [secondaria] |
| P9 | https://docs.plm.automation.siemens.com/content/plant_sim_help/15/plant_sim_all_in_one_html/en_US/tecnomatix_plant_simulation_help/simtalk_reference/simtalk_reference.html (SimTalk 2.0 dalla versione 12.1; dal risultato di ricerca, la pagina risponde 400 a curl) |
| P10 | https://www.simplan.de/en/services/virtual-commissioning-of-control-software-low-level-emulation/ [secondaria, integratore] |
| P11 | https://malun22.github.io/pyplantsim [secondaria] |
| P12 | https://blogs.sw.siemens.com/tecnomatix/tecnomatix-plant-simulation-new-features-and-roadmap |

---

## 2. Autodesk FlexSim (versione 2027)

### Cosa è e per chi
- È un simulatore 3D a eventi discreti per manifattura, movimentazione materiali, sanità, magazzini, miniere e logistica [F1][F2].
- Punta su drag & drop, logica senza codice (Process Flow) e 3D nativo in scala [F2].
- Autodesk ha annunciato l'acquisizione il 23 ottobre 2023 [F20 secondaria]. Dalla versione 2027 il prodotto si chiama "Autodesk FlexSim" [F2].

### Licenze ed edizioni
- Abbonamento con licenza nominale (named-user) [F2].
- Prodotto standalone, non incluso nelle Collection Autodesk [F2].
- Trial di 30 giorni solo per valutazione non commerciale [F2].
- Dalla versione 2027 valgono solo le licenze Autodesk: il licensing legacy è dismesso [F2].
- Dalla 2027 non c'è più un server GIS ospitato: chi fa supply chain deve fornirne uno proprio [F2].
- Nessun prezzo nelle fonti lette, perché flexsim.com risponde 403.

### Funzionalità principali

| Funzionalità | Cosa fa | Fonte |
|---|---|---|
| Oggetti 3D e porte | Fixed Resource (processor, queue...) e Task Executer (Operator, Transporter, Robot, Crane, ASRSvehicle). Connessioni tramite porte, oppure "List" con query SQL per push e pull | [F3][F4] |
| Process Flow | Diagrammi a token con attività e "shared asset". Tipi: General (cervello globale), Object (cervello di un oggetto, con template), Sub-Flow (come una subroutine), Person | [F5] |
| Task sequence | Gli operatori e i veicoli ricevono sequenze di task, con priorità, preemption e coordinamento | [F3] |
| Conveyor | Conveyor discreti con photo eye, decision point, station, power & free, slug building, merge controller, skew dei rulli, pick su un intervallo di posizioni. Conveyor **mass flow**: simulano portate invece dei singoli pezzi, per linee ad alta velocità (bottiglie, alimentare) | [F6] |
| AGV | Rete di percorsi, control point e area, template Process Flow per AGV, ascensori, treni e rimorchi, batteria | [F3][F7] |
| Navigazione | Travel network, A* su griglia, GIS | [F3] |
| Magazzino | Storage System con indirizzi di slot compatibili con un WMS esterno ed eventi sugli slot | [F8] |
| Persone e agenti | Modulo Healthcare/People. Agent System con comportamenti di prossimità e social force | [F3][F9] |
| Fluidi | Libreria Fluid (tank, mixer, pipe, item↔fluid) | [F3] |
| Logica: FlexScript | Linguaggio con sintassi C-like e dot syntax. Supporta coroutine con `await`, per attendere eventi o tempo simulato | [F10] |
| Dati e statistiche | Dashboard e template di grafici, Statistics Collector, Calculated Table, Milestone Collector, tabelle globali, connettori database, interfaccia Excel | [F3] |
| Experimenter | Job di tre tipi: Experiment (scenari × repliche), Optimization (obiettivo e generazione automatica di scenari), Range-based (permutazioni). Multi-core, con nodi distribuiti o cloud. Risultati in un database | [F11] |
| CAD e 3D | Import DWG nativo (RealDWG, dalla 2027), FBX e SketchUp. File Revit .rvt non supportati: si passa da FBX. Import ed export USD, oggetto USD Stage, connettore Omniverse Nucleus (l'export è un'istantanea senza animazione) | [F12][F13][F14][F19] |
| Codice esterno | Python, C#, C/C++ chiamati da trigger o nodi. Module SDK C++ per estendere FlexScript e le classi | [F15][F3] |
| Controllo esterno | Riga di comando senza GUI con script, FlexSimPy (modulo Python da compilare in proprio), Webserver multiutente per eseguire modelli via browser, container Docker (Windows) | [F16][F17][F18] |
| Reinforcement learning | Il modello viene esposto come ambiente Gym tramite Python | [F21] |
| PLC / emulazione | Modulo "Emulation" con OPC DA, OPC UA, Modbus, MQTT, Mitsubishi MC, Beckhoff, **S7 e PLCSIM Advanced**. Dalla 23.0 richiede una licenza Emulation (Modbus e OPC DA esclusi). Dalla 24.2 è un modulo separato dall'installer | [F19] |
| VR | Alcuni visori sono supportati per il walkthrough | [F2] |

### Architettura del motore
- È un DES puro: ogni oggetto ha eventi predefiniti (get item, process, send item), e trigger e listener si agganciano a quegli eventi [F4].
- Gli stati (idle, processing, down, travel, utilized...) servono soprattutto alle statistiche [F4].
- La **Event List** di debug mostra gli eventi pendenti con tempo, oggetto, tipo e dati. Permette di filtrare e di avanzare veloce fino a un evento. I tipi di evento custom si possono registrare con un nome [F22].
- Il continuo è approssimato dal mass flow: il carico di calcolo dipende da quanto spesso cambiano le portate, non dal numero di pezzi. La documentazione stima un giorno di produzione in pochi secondi [F6].
- La FAQ dice esplicitamente che FlexSim **non è progettato per girare in tempo reale** [F2]. Esiste comunque `Delay.realTime()` [F19].

### Punti di forza distintivi
- Libreria di movimentazione molto profonda (conveyor, AGV, magazzino, persone), quasi senza codice [F2][F6].
- Il Process Flow separa la logica dal layout 3D e si attacca per oggetto o globalmente [F5].
- Il conveyor mass flow resta veloce con volumi altissimi [F6].
- Esposizione verso l'esterno ampia: Python, C#, C++, webserver, Docker, Gym, Omniverse e USD [F15][F17][F18][F21].

### Debolezze e critiche note
- Gira solo su Windows x64: niente ARM, macchine virtuali, thin client o GPU virtualizzate [F23].
- Non è pensato per il real-time [F2]. L'emulazione PLC è un modulo a parte con licenza extra [F19].
- FlexSimPy va compilato in proprio, almeno secondo la pagina che parla del 2022 [F16].
- Nei forum utenti, il template AGV va bene solo per logiche vicine al comportamento predefinito: per il resto serve logica custom [F24 secondaria].
- Il supporto ufficiale dice che parte della logica 3D dei conveyor non si riproduce in Process Flow [F24 secondaria].
- La transizione ad Autodesk elimina il server GIS ospitato e il licensing legacy [F2].

### Cosa prendere [inferenza]
- Due livelli di logica: (a) oggetti con eventi e trigger predefiniti; (b) un grafo token-based tipo Process Flow, legabile a istanze e template.
- Una event list ispezionabile, con "fast-forward" fino a un evento.
- Un modello mass flow per linee ad alta cadenza (es. 2 m/s e oltre), con disegno "virtuale" dei pezzi.
- Le "List" con query per l'assegnazione dinamica di task e pezzi.
- Un protocol layer di emulazione che sia **core e open**, non un add-on a pagamento.

### Fonti
Base comune degli URL `F*` del manuale: `https://help.autodesk.com/cloudhelp/2027/ENU/FlexSim-User-Manual/files/`. Indice completo del manuale: `https://help.autodesk.com/view/FLEXSIMIN/2027/ENU/data/toctree.json`.

| Chiave | URL o percorso dopo la base |
|---|---|
| F1 | `introduction/FlexSim_User_Manual_introduction_welcome_html.html` |
| F2 | `introduction/FlexSim_User_Manual_introduction_frequentlyaskedquestionsfaq_html.html` |
| F3 | toctree.json (indice: Operator, Robot, Crane, ASRSvehicle, Fluid, People, A*, GIS, Module SDK, Database Connectors, Excel Interface...) |
| F4 | `modellogic/eventsstatestriggers/FlexSim_User_Manual_modellogic_eventsstatestriggers_keyconceptsevents_html.html` |
| F5 | `modellogic/processflowbasics/FlexSim_User_Manual_modellogic_processflowbasics_typesofprocessflows_html.html` |
| F6 | `connectingflows/conveyors/FlexSim_User_Manual_connectingflows_conveyors_keyconceptsconveyors_html.html` |
| F7 | `workingwithtasks/agvnetworks/FlexSim_User_Manual_workingwithtasks_agvnetworks_keyconceptsagvnetworks_html.html` (titolo dall'indice) |
| F8 | `reference/tools/FlexSim_User_Manual_reference_tools_storagesystem_html.html` |
| F9 | `reference/tools/agentsystem/FlexSim_User_Manual_reference_tools_agentsystem_agentsystemoverview_html.html` |
| F10 | `reference/codinginflexsim/FlexSim_User_Manual_reference_codinginflexsim_writinglogic_html.html` |
| F11 | `reference/tools/FlexSim_User_Manual_reference_tools_experimenter_html.html` |
| F12 | `using3dobjects/FlexSim_User_Manual_using3dobjects_importingcad_html.html` |
| F13 | `using3dobjects/FlexSim_User_Manual_using3dobjects_universalscenedescription_html.html` |
| F14 | `reference/tools/FlexSim_User_Manual_reference_tools_omniverseconnector_html.html` |
| F15 | `reference/developeradvanceduser/FlexSim_User_Manual_reference_developeradvanceduser_connectingtoexternalcode_html.html` |
| F16 | `reference/developeradvanceduser/FlexSim_User_Manual_reference_developeradvanceduser_controllingflexsimexternally_html.html` |
| F17 | `reference/developeradvanceduser/FlexSim_User_Manual_reference_developeradvanceduser_webserver_html.html` |
| F18 | `reference/developeradvanceduser/FlexSim_User_Manual_reference_developeradvanceduser_containerizingamodel_html.html` |
| F19 | `reference/FlexSim_User_Manual_reference_releasenotesandhistory_html.html` (note di rilascio: Emulation, DWG, USD, Python) |
| F20 | https://www.arcweb.com/blog/autodesk-signs-definitive-agreement-acquire-flexsim [secondaria] |
| F21 | `modellogic/reinforcementlearning/FlexSim_User_Manual_modellogic_reinforcementlearning_keyconcepts_html.html` |
| F22 | `reference/tools/FlexSim_User_Manual_reference_tools_eventlist_html.html` |
| F23 | `reference/FlexSim_User_Manual_reference_systemrequirements_html.html` |
| F24 | https://forums.autodesk.com/t5/flexsim-forum/agv-processflow-general/m-p/13579455 [secondaria; contenuto letto solo tramite la sintesi del motore di ricerca] |

---

## 3. AnyLogic

### Cosa è e per chi
- È uno strumento **multi-metodo**: system dynamics, agent-based e discrete-event, combinabili nello stesso modello [A1][A2].
- I modelli sono applicazioni Java [A9].
- Gli usi vanno dalla manifattura alla logistica, pedoni, ferrovia, traffico e sanità. Le librerie: Process Modeling, Material Handling, Pedestrian, Rail, Road Traffic, Fluid [A1][A3].

### Licenze ed edizioni
- Ci sono tre edizioni: PLE gratuita, University Researcher, Professional [A1].
- Funzioni riservate a Professional, cioè assenti in PLE:
  - debug professionale con breakpoint e livello Java;
  - vista della event queue;
  - integrazione SVN;
  - export standalone;
  - snapshot di stato;
  - esperimenti custom [A1].
- I limiti della **PLE gratuita** [A1]:
  - si usa solo per didattica e apprendimento personale;
  - massimo 10 tipi di agente;
  - tempo di modello limitato a 5 h in tutte le librerie tranne Process Modeling;
  - massimo 50.000 agenti dinamici e massimo 200 blocchi per tipo di agente;
  - OptQuest limitato a 500 iterazioni e 7 parametri.
- L'IDE gira su Windows, Mac e Linux [A1].
- Il prezzo non è pubblico. Gli aggregatori riportano cifre discordanti, tra circa 6.000 USD all'anno e circa 19.000 USD per postazione [A15 secondaria, non verificato].

### Funzionalità principali

| Funzionalità | Cosa fa | Fonte |
|---|---|---|
| Flowchart DES | Process Modeling Library: blocchi (Source, Delay, Seize/Release, Batch, Downtime...) con agenti come entità. I punti di estensione accettano codice Java | [A4] |
| Agent-based e statechart | Agenti con statechart, eventi statici e dinamici, messaggi | [A5][A6] |
| System dynamics | Stock e flow risolti da solver numerici: Euler e RK4 per le ODE, Newton per le equazioni algebriche, RK45 Newton ed Euler Newton per le DAE. Precisione e passo sono configurabili | [A7] |
| Material Handling | Conveyor (Convey, ConveyorEnter, ConveyorExit) con stazioni e tavole rotanti. Transporter (AGV) su percorsi guidati o in spazio libero, con evitamento degli ostacoli e collisioni. Gru (MoveByCrane), robot (ProcessByRobot), ASRS (Store, Retrieve, StorageSystem), ascensori, level gate, tugger train | [A3][A8][A16] |
| Fluid Library | Tank, pipeline, valve, conveyor bulk, conversione agente↔fluido, batch | [A17] |
| Esperimenti | Simulation, Optimization (OptQuest o Genetic), Parameter Variation, Compare Runs, Monte Carlo, Sensitivity, Calibration, RL, Custom | [A1][A10] |
| Reinforcement learning | L'esperimento RL esporta il modello verso Azure ML (Plato) o la libreria Alpyne. **Non** addestra dentro AnyLogic | [A11] |
| Python | Pypeline: libreria open source opzionale che collega la JVM a un Python locale. Non ha supporto ufficiale | [A12] |
| Integrazione Java | Modelli lanciati da un'applicazione Java esterna, con o senza UI. Export come applicazione Java standalone (solo Professional) | [A9][A13] |
| Cloud | AnyLogic Cloud con API REST, JavaScript, Python e Java. Private Cloud on-premise con Docker o Kubernetes Helm. Autenticazione LDAP o AD e OAuth2 | [A14][A18][A19] |
| 3D e CAD | Libreria 3D integrata, import di .dae, .gltf/.glb, .fbx, .obj. CAD come sfondo: DXF fino alla 2018, **DWG solo fino alla versione 2000**. Omniverse | [A20][A21][A1] |
| Dati | Database integrato per modello, Excel, database esterni, schedule e calendari, GIS con instradamento | [A1] |
| Team | Integrazione Git, e SVN in Professional | [A1] |
| PLC / OPC UA | **Nessuna** funzione nativa trovata nell'help. Possibile solo tramite librerie Java esterne [inferenza] | [ricerca su anylogic.help, nessun risultato] |

### Architettura del motore (documentata)
- L'esecuzione alterna **time step** ed **event step** [A6]:
  - nel time step avanza l'orologio e i solver integrano le equazioni attive, controllando gli "change event" (condizioni);
  - nell'event step non passa tempo: si eseguono le azioni e si schedulano o cancellano eventi.
- Gli eventi del motore si classificano in current, chosen, enabled, scheduled e pending [A6].
- Gli eventi simultanei vengono serializzati in un ordine interno **non garantito**: il modellista non deve dipenderne [A5].
- Il tempo di modello è un double con unità configurabili. Mese e anno come unità sono fissi (30 e 365 giorni) [A22].
- Si può eseguire in scala con il tempo reale oppure in modalità virtuale [A22].
- Le condizioni sugli elementi SD vengono valutate a ogni passo di integrazione [A2].

### Punti di forza distintivi
- Multi-metodo vero, con un unico spazio dei nomi tra SD, agenti e DES [A2].
- Il motore ibrido discreto/continuo è documentato con precisione [A6][A7].
- Java completo, quindi estendibilità quasi illimitata [A9].
- Un cloud con API in quattro linguaggi e un'opzione on-premise containerizzata [A14][A18].

### Debolezze e critiche note
- L'import DWG si ferma alla versione 2000 [A20].
- Il Python non è ufficiale [A12], e il RL non addestra in locale [A11].
- Per tutto quello che non è standard serve Java [A4][A9] [inferenza: curva di apprendimento alta].
- L'ordine degli eventi simultanei non è garantito [A5].
- La PLE è molto limitata, e le funzioni chiave di debug e deployment sono solo in Professional [A1].
- Non ha emulazione PLC nativa [inferenza dall'assenza nella documentazione].

### Cosa prendere [inferenza]
- Il ciclo time step / event step con rilevamento degli eventi di condizione dal solver continuo: è la base giusta per accoppiare DES e cinematica continua.
- Le API cloud REST, Python e JS, con runner containerizzato.
- Una tabella "edizione gratuita" con limiti chiari. Per un progetto open source vale come esempio di cosa *non* chiudere: debug, export, event queue.
- Rendere **deterministico** l'ordine degli eventi simultanei (priorità e sequenza d'inserimento), a differenza di AnyLogic: è essenziale per il virtual commissioning ripetibile.

### Fonti
Base comune degli URL `A*` dell'help: `https://anylogic.help/`.

| Chiave | URL o percorso dopo la base |
|---|---|
| A1 | `anylogic/ui/editions.html` |
| A2 | `anylogic/multimethod/index.html` |
| A3 | `library-reference-guides/material-handling-library/index.html` |
| A4 | `library-reference-guides/process-modeling-library/index.html` |
| A5 | `anylogic/statecharts/discrete-event-modeling-concept.html` |
| A6 | `anylogic/statecharts/event-processing.html` |
| A7 | `anylogic/experiments/numerical-solvers.html` |
| A8 | `library-reference-guides/material-handling-library/transporters.html` |
| A9 | `advanced/code/integration.html` |
| A10 | `anylogic/experiments/optimization.html` |
| A11 | `anylogic/experiments/rl-experiment.html` |
| A12 | `advanced/code/pypeline.html` |
| A13 | `anylogic/running/export-java-application.html` |
| A14 | `cloud/api/index.html` |
| A15 | https://www.capterra.com/p/95940/AnyLogic/ e https://checkthat.ai/brands/anylogic/pricing [secondarie, non aperte] |
| A16 | `library-reference-guides/material-handling-library/blocks.html` |
| A17 | `library-reference-guides/fluid-library/index.html` |
| A18 | `cloud/private-cloud.html` |
| A19 | `cloud/architecture.html` |
| A20 | `anylogic/presentation/cad.html` |
| A21 | `anylogic/3d/import-3d.html` |
| A22 | `anylogic/experiments/model-time.html` |

---

## 4. Simio

### Cosa è e per chi
- È un framework DES basato su "intelligent objects" [S1].
- Gli oggetti non sono scritti in C++ o Java: si definiscono graficamente con **process** composti di **step** [S1].
- Il posizionamento attuale è "Process Digital Twin" e **APS/scheduling risk-based**, cioè pianificazione e scheduling basati sul modello, con Gantt [S2][S3][S4].
- Il framework supporta paradigmi a eventi, a processi, a oggetti e ad agenti, sia discreti sia continui [S1].

### Licenze ed edizioni
- C'è un trial completo di 30 giorni. I modelli fatti nel trial si aprono solo sulla macchina di creazione [S5].
- Le licenze accademiche sono gratuite e funzionalmente equivalenti a Simio RPS, senza limiti di dimensione. Valgono solo per lavori non commerciali [S6].
- Le edizioni commerciali citate dalle fonti secondarie sono Design, Team, Enterprise, Portal e RPS. I prezzi sono solo su preventivo [S7 secondaria].
- Simio Portal è un prodotto con licenza separata. Richiede SQL Server esterno (versione 2019 o successive) e ha un'immagine Docker [S8].
- Nel 2010 OptQuest era un add-in a pagamento per uso commerciale [S1]. Non ho verificato se lo sia ancora oggi.

### Funzionalità principali

| Funzionalità | Cosa fa | Fonte |
|---|---|---|
| Standard Library | Source, Sink, Server, Workstation, Combiner, Separator, Resource, Worker, Vehicle, BasicNode, TransferNode, Connector, Path (con TimePath e Conveyor nella stessa famiglia [inferenza dal paper]) | [S1] |
| Add-on process | Piccoli flussi di step grafici agganciati a trigger degli oggetti standard: si personalizza senza programmare | [S1] |
| Oggetti custom | Gli utenti costruiscono i propri oggetti (proprietà, stati, eventi, vista esterna) con gli stessi process. Gli step custom si scrivono in .NET | [S1] |
| Esperimenti | Scenari, risposte (KPI), vincoli, OptQuest, add-in di esperimento in .NET, multiprocessore | [S1] |
| Data-driven | I modelli vengono generati e guidati da dati esterni (tabelle, ERP). Template per creazione rapida e aggiornamento automatico | [S2][S8] |
| APS / scheduling | Pianificazione dinamica risk-based con il modello DES come motore e Gantt | [S3] |
| AI | Reti neurali native per l'inferenza nella logica. Raccolta di dati di training sintetici, trainer basato su TensorFlow, import ed esecuzione di modelli ONNX | [S9] |
| Python | Da Simio 19 si possono scrivere step Python nel modello. Il debug live si fa con VS Code. Python si usa anche per gli script di import ed export dati e con la Portal API | [S10][S8] |
| 3D | Import di simboli da Google/Trimble 3D Warehouse (paper 2010). Modellazione 2D e 3D nello stesso passo. Integrazione Omniverse/USD | [S1][S11] |
| Cloud / Web | Simio Portal: esperimenti, coda, Web API, calendario | [S8] |
| PLC / OPC UA | **Nessuna** menzione nelle fonti lette | — |

### Architettura del motore
- I process vengono eseguiti da **token**. Il token è un oggetto con propri stati, distinto dall'entità [S12 secondaria].
- Il paper Simio 2010 dichiara il supporto di paradigmi discreti e continui, ma non descrive la event list [S1].

### Punti di forza distintivi
- La combinazione di oggetti pronti e add-on process grafici risolve il dilemma "facile ma rigido" contro "flessibile ma codice" [S1].
- È l'unico del gruppo che porta il DES dentro uno scheduling operativo e un APS [S3].
- AI integrata con ONNX e TensorFlow [S9]. Python ora nativo [S10].

### Debolezze e critiche note
- Il trial lega i modelli alla macchina [S5].
- Le licenze e i prezzi sono opachi: si conoscono solo su preventivo [S7].
- OptQuest era a pagamento, almeno storicamente [S1].
- Le fonti lette non parlano di emulazione PLC o OPC UA. Nel virtual commissioning il prodotto è debole o assente [inferenza].
- La documentazione tecnica pubblica è scarsa: non c'è un help online accessibile, per cui ho dovuto usare paper e note di rilascio.
- Simio annuncia la fine del supporto alle licenze QLM dopo marzo 2027 [S8].

### Cosa prendere [inferenza]
- Gli "add-on process" come punti di estensione grafici sugli oggetti standard. Si mappano bene sui trigger degli oggetti della nostra libreria.
- La generazione del modello da tabelle o dati: si importa un elenco di macchine e collegamenti e il layout del modello si costruisce da sé.
- Inferenza ONNX dentro la logica di simulazione, per esempio come politica di dispatch addestrata.
- Una modalità "scheduling" che riusi lo stesso modello per un Gantt previsionale.

### Fonti

| Chiave | URL |
|---|---|
| S1 | https://www.informs-sim.org/wsc10papers/003.pdf (Sturrock e Pegden, "Recent innovations in Simio", WSC 2010) |
| S2 | https://www.simio.com/data-integration-framework/ |
| S3 | https://www.simio.com/advanced-planning-scheduling |
| S4 | https://www.simio.com/process-digital-twin |
| S5 | https://www.simio.com/trial-edition-download |
| S6 | https://www.simio.com/academic-licensing/ |
| S7 | https://forum.simio.com/topic/2895-compatibility-between-simio-enterprise-and-team-edition/ e rfp.wiki [secondarie; contenuto letto solo tramite la sintesi del motore di ricerca] |
| S8 | https://cdn.simio.com/software/SimioReleaseNotes290.pdf |
| S9 | https://www.simio.com/ai-optimization |
| S10 | https://www.simio.com/python-simio-revolutionize-your-digital-twin/ |
| S11 | https://www.simio.com/nvidia-omniverse-digital-twin-integration/ |
| S12 | https://textbook.simio.com/Workbook6/using-simio-processes.html [secondaria; contenuto letto solo tramite la sintesi del motore di ricerca] |

---

## 5. Sintesi trasversale per il nuovo simulatore [inferenza]

| Tema | Migliore riferimento | Idea da adottare |
|---|---|---|
| Libreria con ereditarietà | Plant Simulation [P1] | Classe di libreria → istanze con propagazione delle modifiche e override locale |
| Logica senza codice | FlexSim Process Flow [F5], Simio add-on process [S1] | Grafo token-based agganciabile a oggetto o template, più trigger sugli eventi standard |
| Motore ibrido | AnyLogic [A6][A7] | Time step continuo più event step, con rilevamento degli eventi di condizione dal solver |
| Alta cadenza | FlexSim mass flow [F6] | Flusso aggregato con rendering "virtuale" dei pezzi |
| Movimentazione | FlexSim [F6][F7], AnyLogic MHL [A3] | Conveyor con sensori, decision point e merge, AGV con rete e control point, gru e robot come risorse |
| Virtual commissioning | Plant Simulation + PLCSIM Adv [P1], FlexSim Emulation [F19] | Layer I/O core e open (OPC UA, S7, Modbus, MQTT) con interruttore "logica interna ↔ PLC esterno" |
| Esperimenti | FlexSim Experimenter [F11], AnyLogic [A1] | Scenari × repliche, ottimizzazione, multi-core e distribuito, database dei risultati |
| API | AnyLogic Cloud [A14], FlexSim Webserver/Docker [F17][F18] | Esecuzione headless e REST o Python come prima classe, non come add-on |
| Determinismo | Contro-esempio AnyLogic [A5] | Ordine degli eventi simultanei documentato e riproducibile |
| Interop 3D | FlexSim USD [F13], Omniverse in tutti e quattro | USD/glTF in import ed export. DWG via libreria aperta, perché il limite DWG 2000 di AnyLogic mostra il rischio [A20] |

---

## Cosa non è stato verificato
- **Plant Simulation:**
  - l'help ufficiale risponde 400/403, quindi non ho verificato l'event list, la gestione degli eventi simultanei, la lista completa degli oggetti attuali (la lista viene da un tutorial della versione 11), l'API Python ufficiale, la versione del formato CAD e i dettagli dell'interfaccia PLCSIM Advanced e OPC UA;
  - il prezzo non è verificato: cifra G2 solo da sintesi di ricerca.
- **FlexSim:**
  - flexsim.com risponde 403, quindi non ho visto prezzi e pagine prodotto;
  - non so se l'edizione gratuita "Express" esista ancora: la FAQ 2027 cita solo il trial di 30 giorni;
  - l'attribuzione delle voci Emulation alle versioni 23.0 e 24.2 deriva dalla posizione nel testo delle note di rilascio, non da un'intestazione univoca;
  - non so se FlexSimPy sia ora installabile con pip;
  - non ho letto le pagine di riferimento dei singoli oggetti (Crane, Robot, ASRS), solo i loro titoli nell'indice.
- **AnyLogic:**
  - il prezzo non è verificato;
  - l'assenza di OPC UA o PLC è dedotta dall'assenza nell'help, non da un'affermazione esplicita;
  - non ho letto le pagine su gru e robot.
- **Simio:**
  - non c'è un help online accessibile;
  - l'elenco attuale delle edizioni e le differenze tra Design, Team, Enterprise e RPS non sono verificati;
  - non so se OptQuest sia ancora a pagamento;
  - non ho verificato la libreria Flow per il continuo, i conveyor e i requisiti di sistema (Windows/.NET presunti);
  - la descrizione dei token viene da una sintesi di ricerca del textbook, non da una lettura diretta.
- **Critiche:** quelle da forum e paper sono poche. Non ho fatto una ricerca sistematica nella letteratura WSC di confronti tra questi prodotti.
