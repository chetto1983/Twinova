# Benchmark G1: simulatori per la messa in servizio virtuale (VC)

Data della ricerca: 2026-11-07. Lavoro clean-room: le funzionalità sono descritte con parole mie,
non c'è testo né immagine copiati. Legenda:
- **[URL]**: fonte primaria (vendor, documentazione ufficiale, manuale);
- **[secondaria]**: forum, blog o pagina di terzi;
- **[snippet]**: dato letto solo nell'estratto del motore di ricerca, perché il documento non si
  poteva scaricare (403 o login). Va verificato;
- **[inferenza]**: mia deduzione.

Contesto comune Siemens. Siemens presenta tre livelli che si accoppiano:
- PLCSIM Advanced simula il controllore;
- SIMIT simula il comportamento elettrico e logico dei dispositivi;
- NX MCD simula la fisica e la cinematica;
- Process Simulate copre le celle robot.

I collegamenti passano per l'API di PLCSIM Advanced e per l'API NX aperta
[https://assets.new.siemens.com/siemens/assets/api/uuid:f6211103-909f-4d43-abd1-1749fe79cfa2/Automation-Days-Discover-the-power-of-simulation.pdf].
PLCSIM Advanced ha la modalità a tempo virtuale (realtime, lento, veloce) e un'API per la
co-simulazione. Si sincronizza a ogni "cycle control point" oppure a ogni OB ciclico
[https://assets.new.siemens.com/siemens/assets/api/uuid:3d8be3e0-00bf-49dc-b2c9-ed3c53f993ae/plcsimadvanced.pdf] [snippet].

---

## 1. Siemens NX Mechatronics Concept Designer (MCD)

**Cosa è e per chi.** MCD è un modulo di NX CAD per il progetto concettuale e funzionale di
macchine. Il modello è multi-corpo e fisico, con attuatori, sensori e comportamento di
automazione. È pensato per costruttori di macchine (meccanici, elettrici e automazione in
parallelo) e si integra con Teamcenter per i requisiti
[https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf].
Il prezzo non è pubblico: il modulo è parte del portafoglio NX, a licenza commerciale.

| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Modello a componenti | Ogni pezzo CAD riceve proprietà fisiche: corpo rigido (massa, inerzia), corpo di collisione (forma semplificata: box, mesh...), giunti, attuatori. Le parti che si muovono sempre insieme si raggruppano in un solo corpo rigido | https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf ; https://blogs.sw.siemens.com/designcenter/nx-tips-and-tricks-mechatronics-concept-designer/ [snippet] |
| Attuatori e controlli | Controllo di posizione e di velocità sui giunti (posizione, direzione, destinazione, velocità), molle, camme | https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf |
| Trasporto materiale | "Transport surface" sui nastri con una velocità assegnata; sorgenti e pozzi di oggetti | https://blogs.sw.siemens.com/designcenter/nx-tips-and-tricks-mechatronics-concept-designer/ [snippet] |
| Sensori | Sensori di collisione (volumi che segnalano il contatto), sensori dalla Reuse Library. Da NX 2024 c'è la "Vision Camera", una vista dedicata che rileva i corpi rigidi | https://blogs.sw.siemens.com/designcenter/whats-new-in-nx-june-2024-multi-discipline-design/ |
| Sequenze ed eventi | Editor delle operazioni: prima un comportamento a tempo, poi controlli a eventi | https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf |
| Segnali e connettività | I segnali MCD si mappano su segnali esterni (finestra Signal Mapping, con auto-mapping). Connettori: OPC UA (da NX 12.01), PLCSIM Advanced (API), PROFINET, SIMIT. I "signal adapter" legano i segnali alle espressioni di runtime | https://www.automation.siemens.com/sce-static/learning-training-documents/tia-portal/digital-twin/sce-150-006-mcd-tia-com-digital-twin-at-education-signal-mapping-mcd-hs-darmstadt-0820-zh.docx [snippet] ; https://i40.fh-aachen.de/courses/8610718/practicum/mcd_opc_ua/index.html [snippet, pagina non raggiunta: 502] |
| Riuso | Gli "oggetti intelligenti" incapsulano dati meccatronici e si mettono in libreria con drag & drop | https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf |
| CAD | È CAD nativo NX. Esporta verso Catia, Pro/E, SolidWorks e JT. La lista di sensori e attuatori si esporta per l'ingegneria elettrica | https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf |
| Interattività | Durante la simulazione si applicano forze o si spostano oggetti con il mouse | https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf |
| VR | Le Vision Camera funzionano anche in NX Immersive Explorer | https://blogs.sw.siemens.com/designcenter/whats-new-in-nx-june-2024-multi-discipline-design/ |

**Architettura del motore.**
- Il motore fisico è di tipo "game physics", con modelli matematici semplificati
  [https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf].
- Nel 2010 era basato sull'SDK NVIDIA PhysX
  [https://www.digitalengineering247.com/article/siemens-plm-software-announces-mechatronics-concept-designer] [snippet].
- Da NX giugno 2024 si può scegliere il solver PhysX V5, con calcolo parallelo su CPU e GPU,
  pensato per le simulazioni di linee intere
  [https://blogs.sw.siemens.com/designcenter/whats-new-in-nx-june-2024-multi-discipline-design/].
- Il passo di integrazione e il modello di threading ufficiali non sono documentati
  pubblicamente: non trovati.

**Punti di forza distintivi.**
- Il modello di simulazione vive dentro il CAD: nessuna conversione, e i requisiti restano
  tracciabili [https://apsindustrial.com.au/wp-content/uploads/2024/04/NX-Mechatronics-Concept-Designer.pdf.pdf].
- È integrato a tre livelli con PLCSIM Advanced e SIMIT. SIMIT ha una modalità "bus-sincrona"
  dedicata a MCD [https://cache.industry.siemens.com/dl/files/317/109759317/att_956837/v1/SIMIT_enUS_en-US.pdf].

**Debolezze e critiche.**
- Un progetto studentesco riporta rallentamenti su modelli complessi: fisica, vincoli e
  comunicazione PLC girerebbero quasi tutti su un solo core
  [https://sites.google.com/view/hubertcholewa/blog-first/performance-issues-faced-during-the-simulation-stage] [secondaria].
  Il solver PhysX V5 parallelo del 2024 sembra una risposta proprio a questo [inferenza].
- Richiede NX, quindi un costo e una curva d'ingresso alti per chi non usa già NX [inferenza].

**Cosa prendere [inferenza].**
- Le proprietà fisiche come "aspetti" attaccati alla geometria CAD: corpo rigido, corpo di
  collisione, giunto, attuatore di posizione o velocità.
- Una finestra Signal Mapping con auto-mapping per nome.
- Il solver fisico sostituibile (semplice o accurato).

---

## 2. Siemens Tecnomatix Process Simulate (Virtual Commissioning)

**Cosa è e per chi.** È lo strumento Siemens per celle e linee robotizzate (soprattutto
automotive): programmazione robot offline, collisioni, tempi ciclo e messa in servizio virtuale
con programmi PLC reali.
- L'offerta è a livelli: X Essentials, X Standard (aggiunge la programmazione robot completa e il
  toolset di VC) e X Advanced.
- Il prezzo non è pubblico. La formula "X" indica il modello Xcelerator in abbonamento
  [https://www.siemens.com/en-us/products/tecnomatix/offerings/process-simulate-x-standard/].

| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Robot | Scarica, carica e ottimizza programmi robot completi. Controller robot emulati. Il "cyclic event evaluator" sincronizza più robot. Pianificazione del moto, collisioni, ottimizzazione del tempo ciclo | https://www.siemens.com/en-us/products/tecnomatix/offerings/process-simulate-x-standard/ |
| Connettività PLC | I modelli si collegano a sistemi di controllo esterni: PLC reale tramite Siemens Simulation Unit o OPC, oppure PLC virtuale PLCSIM Advanced. Sono possibili sia SiL sia HiL | https://blogs.sw.siemens.com/partners/optimizing-complex-production-lines-before-they-are-built/ ; https://blogs.sw.siemens.com/partners/?p=1844 [snippet] |
| Co-simulazione con SIMIT | Process Simulate e SIMIT scambiano dati con un accoppiamento dedicato; SIMIT parla con PLCSIM Advanced | https://support.industry.siemens.com/cs/attachments/109972477/109972477_Getting_Started_with_Drives_in_ProcessSimulate_DOC_V10_en.pdf [snippet, sito non scaricato per policy] |
| Nastri | Dalla v14 i nastri imitano il comportamento reale. Gli skid si creano e si eliminano dinamicamente su condizioni logiche | https://blogs.sw.siemens.com/tecnomatix/?p=371 [snippet] |
| Validazione | Collisioni, sezioni 2D e 3D, revisione condivisa delle celle | https://www.siemens.com/en-us/products/tecnomatix/offerings/process-simulate-x-standard/ |
| Flusso di lavoro dei segnali | Si esportano i tag dal PLC, si importano in Process Simulate e si collegano ai segnali dei dispositivi | https://longtermtec.com/process-simulate-virtual-commissioning-training/ [secondaria] |

**Architettura del motore.** Il partner Simulo parla di simulazione "event-based" di robot e
linea [https://blogs.sw.siemens.com/partners/?p=1707] [snippet]. Non ho trovato nella
documentazione pubblica il passo temporale, il motore fisico né il modello di threading.
La guida ufficiale (docs.sw.siemens.com) è dietro login.

**Punti di forza distintivi.**
- È l'unico del gruppo con programmazione robot offline multimarca nativa e controller robot
  emulati nello stesso ambiente della VC
  [https://www.siemens.com/en-us/products/tecnomatix/offerings/process-simulate-x-standard/].
- Si integra con l'ecosistema Siemens (PLCSIM Advanced, SIMIT, Simulation Unit).

**Debolezze e critiche.**
- Un articolo accademico indica la simulazione free-running e l'interfaccia OPC come fonti di
  jitter, race condition e ritardi nella verifica PLC basata su simulazione. Il punto vale per
  tutti questi tool
  [https://www.academia.edu/28677357/Methods_for_Reliable_Simulation_Based_PLC_Code_Verification] [snippet].
- La documentazione è chiusa e i prezzi sono opachi [inferenza dalla mancanza di fonti
  pubbliche].

**Cosa prendere [inferenza].**
- Il concetto di "event evaluator" ciclico per coordinare più robot o macchine su eventi
  condivisi.
- Le stesse definizioni di segnale riusate in SiL e HiL cambiando solo il trasporto
  (OPC, API PLCSIM, bus fisico).

---

## 3. Siemens SIMIT

**Cosa è e per chi.** È una piattaforma di simulazione per la VC e la formazione degli operatori,
nata nel mondo di processo (PCS 7/PCS neo) e usata anche nel manifatturiero. Modella il campo a
livello di segnale e di dispositivo: I/O, azionamenti, valvole, strumenti.
- Licenze: base, librerie aggiuntive, Component Type Editor (CTE) e Virtual Controller si
  licenziano separatamente
  [https://cache.industry.siemens.com/dl/files/317/109759317/att_956837/v1/SIMIT_enUS_en-US.pdf].
- Siemens annuncia un passaggio a un modello in abbonamento
  [https://resources.sw.siemens.com/en-US/simit-webinar-uk-ireland/] [snippet].

| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Modello a schemi | Il modello è fatto di "chart", con componenti, controlli e connessioni grafiche, su più chart | https://cache.industry.siemens.com/dl/files/317/109759317/att_956837/v1/SIMIT_enUS_en-US.pdf (cap. 1.6, 3.2) |
| Macro componenti | Gerarchie riusabili con I/O, parametri e connettori topologici | idem (cap. 3.4) |
| CTE (tipi di componente) | Il comportamento si descrive in due stili: a equazioni (con equazioni differenziali, vettori, enumerazioni) o a istruzioni (cicli, condizioni). Il comportamento viene convertito in codice C# | idem (cap. 7.6.1-7.6.4) |
| Generazione automatica del modello | Template istanziati da tabelle, import IEA/CMT, import XML/ZIP di chart, bulk engineering (export, modifica, re-import) | idem (cap. 4) |
| Accoppiamenti | SIMIT Unit (PROFIBUS, PROFINET, PROFIsafe su hardware dedicato), PLCSIM, PLCSIM Advanced, Virtual Controller (fino a 32 per sistema), OPC DA client e server, OPC UA client, memoria condivisa (SHM), PRODAVE | idem (cap. 2.5-2.11) ; limite di 32 VC a p. ~1070 del testo |
| Diagnostica | Editor di trend e messaggi, snapshot dello stato del modello e dei controller virtuali, script di test | idem (cap. 5, 6) |
| 3D | Un controllo 3D Viewer lega i movimenti a segnali del modello funzionale | idem (sez. 3D Viewer control) |
| Novità V11.2 | Tipi STRING, LREAL, LINT; guasti di canale e dispositivo con PLCSIM Advanced; componenti da FMU | https://devicebase.net/en/siemens-simit/updates/improved-performance-capability/4oy [secondaria] |

**Architettura del motore.** È documentata bene, ed è il riferimento più solido del gruppo:
- calcolo ciclico a passo fisso con **8 time slice**: ogni slice ha un ciclo assoluto per
  progetto, con minimo 1 ms (dalla V10; prima era 10 ms);
- ogni slice ha **8 sotto-slice (A-H)** con lo stesso ciclo, per distribuire il calcolo su più
  core;
- ogni componente e ogni accoppiamento ha la sua slice; gli accoppiamenti girano sempre nella
  slice principale;
- il rapporto tra tempo simulato e tempo reale si regola (più veloce o più lento);
- la modalità "bus-sincrona" per PLCSIM Advanced e MCD fa avanzare tutti i partecipanti in modo
  sincrono [https://cache.industry.siemens.com/dl/files/317/109759317/att_956837/v1/SIMIT_enUS_en-US.pdf]
  (sez. 1.6.3.2-1.6.3.3, note di rilascio V10).

La modalità bus-sincrona richiederebbe PROFINET IRT e un TO servo isocrono
[https://industrialmonitordirect.com/blogs/knowledgebase/simit-plcsim-adv-bus-synchronous-buttons-greyed-out-fix] [secondaria].

**Punti di forza distintivi.**
- Lo scheduling deterministico e multi-rate (time slice) è esplicito e configurabile.
- Ha la più ricca generazione automatica del modello da liste I/O.
- Il CTE è un linguaggio di comportamento a equazioni.
- Gli snapshot di stato includono i PLC virtuali (tutto dal manuale sopra).

**Debolezze e critiche.**
- La parte 3D e fisica è debole: per quella Siemens rimanda a MCD (lo schema a tre livelli nel
  deck Automation Days citato sopra).
- In modalità demo la simulazione gira solo in tempo reale [manuale, p. ~1068].
- Un utente vede l'OPC UA client aggiornare ogni ~2 s invece dei 250 ms impostati
  [https://support.industry.siemens.com/forum/hn/en/post/1187859] [secondaria, snippet].
- Le licenze sono frammentate (CTE e VC a parte).

**Cosa prendere [inferenza].**
- Lo scheduler multi-rate a time slice e sotto-slice (componente → slice) come cuore
  deterministico.
- Gli snapshot dell'intero stato.
- I template di componente istanziati da una tabella I/O (CSV o export TIA).
- Il comportamento definito a equazioni e compilato.

---

## 4. Rockwell Automation Emulate3D

**Cosa è e per chi.** È una famiglia di prodotti per costruire gemelli digitali dinamici:
- Emulate3D (Controls Testing / Ultimate) per la VC;
- Sim3D per l'analisi del throughput;
- Demo3D per le presentazioni;
- Layout3D.

È forte nel material handling (magazzini, smistamento, bagagli)
[https://www.demo3d.com/technology] [https://www.simplan.de/wp-content/uploads/Comparison-Emulate3D-Editions.pdf].
- Il focus dichiarato è il test della logica di controllo, non il prototipo meccanico dettagliato,
  con lo stesso codice PLC sul gemello e sull'hardware reale
  [https://rockwellautomation.com/content/dam/rockwell-automation/documents/pdf/events/roklive-emea-2024/digital-engineering-summit/device-emulation.pdf] [snippet, 403].
- "Factory Test" è la nuova edizione per gemelli a scala di fabbrica (lancio in Giappone nel
  luglio 2025) [https://monoist.itmedia.co.jp/mn/articles/2508/08/news039.html] [snippet].
- Il prezzo non è pubblico.

| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Connettività PLC | Protocolli nativi: Allen-Bradley (EtherNet/IP, PCCC, RSLinx, Logix Emulate), Beckhoff (ADS, EtherCAT), B&R (PVI), Mitsubishi (MX Component, GX Works2), Omron, Schneider (Modbus), Siemens S7-300/400/1200/1500 (PROFINET, PROFIBUS, PROFIsafe, PUT/GET, Fetch/Write, PLCSIM, PLCSIM Advanced, SIMIT Unit), OPC DA e UA. Fa anche da server PUT/GET e Modbus per testare HMI e SCADA | https://www.demo3d.com/Connectivity |
| Emulazione di alto livello | Si collega a WMS, WCS ed ERP | https://www.demo3d.com/Connectivity |
| Gemello Logix | Con FactoryTalk Logix Echo il programma ControlLogix gira senza modifiche | https://www.controlglobal.com/industrynews/2022/roklive-19 [snippet] |
| Import CAD | Molti formati: STEP, IGES, IFC, SolidWorks, Inventor, DWG e DXF (RealDWG), FBX, SketchUp, Blender, Collada, STL, OBJ, VRML, nuvole di punti (xyz, pts). Strumenti per eliminare i dettagli (viti, molle) | https://www.demo3d.com/cad-files |
| Export | DWG/DXF, STEP, OBJ, STL, PDF 3D animato, file AutoMod, file viewer | https://www.demo3d.com/cad-files |
| Fisica | Modelli fisici in tre modalità: volumetrica, planare e lineare. PhysX aggiornato alla 5.1.3 nella V17 (2024); funzioni con Algoryx AGX | https://www.simplan.de/wp-content/uploads/Comparison-Emulate3D-Editions.pdf ; https://rockwellautomation.com/content/dam/rockwell-automation/documents/pdf/events/roklive-emea-2024/digital-engineering-summit/whats-new-in-E3D-2024.pdf [snippet, 403] |
| Scripting e API | JScript e C#, debugger, DLL esterne, QuickLogic (logica senza codice). Esempi API ufficiali su GitHub con licenza MIT | https://www.simplan.de/wp-content/uploads/Comparison-Emulate3D-Editions.pdf ; https://github.com/RockwellAutomation/ra-emulate3d-core-samples |
| Cataloghi | Cataloghi standard (Sample, QuickStart, People, Robots, FlowControl), cataloghi aziendali, cifratura e licenza dei cataloghi, editor del profilo dei nastri | https://www.simplan.de/wp-content/uploads/Comparison-Emulate3D-Editions.pdf |
| Report ed esperimenti | Gestione degli esperimenti, Sim3D Analyser, distinta base | idem |
| VR | Pagine dedicate alla VR e all'integrazione con Onshape e SolidWorks (Demo3DVR) | https://www.demo3d.com/ ; https://www.solidworks.com/partner-product/demo3dvr-solidworks |
| Robot | Integrazioni con FANUC ROBOGUIDE e ABB RobotStudio citate da fonti di terzi; nessuna doc ufficiale letta | [secondaria, non verificato] |

**Architettura del motore.**
- PhysX, con AGX opzionale ([snippet] sopra).
- La V17 introduce la distribuzione di modelli grandi su più istanze Emulate3D e un profiler
  delle prestazioni (whats-new-in-E3D-2024.pdf [snippet]).
- Il rendering passa a Unreal in Factory Test
  [https://coconote.app/notes/a83d9b2b-1ee3-468a-acf6-b69471fcedb6] [secondaria].

**Punti di forza distintivi.**
- Ha l'elenco di protocolli PLC più ampio del gruppo, multimarca.
- Lo stesso modello passa dalla presentazione al throughput fino al test dei controlli
  [https://www.demo3d.com/technology].
- I cataloghi si possono cifrare e licenziare, quindi i costruttori possono venderli.
- Copre i formati CAD e BIM più ampi.

**Debolezze e critiche.**
- Il bisogno di dividere i modelli su più istanze e il profiler indicano limiti di scala su
  un'istanza singola [inferenza da snippet].
- Rockwell blocca il download automatico della documentazione (403).
- Le recensioni pubbliche sono scarse: non trovate.

**Cosa prendere [inferenza].**
- Il catalogo come pacchetto distribuibile e firmabile.
- Più modalità fisiche per oggetto (lineare, planare, volumetrica) per scalare i costi di
  calcolo.
- Il simulatore che fa anche da server PUT/GET e Modbus per testare HMI e SCADA.
- Un unico modello per tutte le fasi del progetto.

---

## 5. realvirtual.io (Unity)

**Cosa è e per chi.** È un framework sopra Unity per concept design, simulazione, VC e HMI 3D.
È per integratori e costruttori che accettano di lavorare nell'editor Unity
[https://realvirtual.io/doc/unity/llms-full.txt].
- Starter gratis sull'Asset Store. Professional a 1.250 € con licenza perpetua una tantum.
  L'editor Unity si licenzia a parte.
- "realvirtual WEB" (browser, Three.js) è AGPL oppure commerciale a 1.920 € per sviluppatore
  all'anno, più le fee di deploy.
- Pacchetto Research & Education con 20 licenze
  [https://realvirtual.io/en/buy/] (dati schema.org della pagina).

| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Modello a componenti | Componenti Unity: Drive (asse lineare o rotativo, mm e °), DriveBehavior (interfaccia segnali, separata dal moto), Sensor, TransportSurface, Chain, CAM, Kinematic (gerarchia cinematica separata da quella CAD), Grip/Gripper/Fixer, MU con Source e Sink, cambio MU (Cutter, PartChanger) | https://realvirtual.io/doc/unity/llms-full.txt (sezioni Drive, Sensor, Editions) |
| Fisica | Ibrida: le MU sono corpi PhysX liberi, le macchine e i robot sono cinematici. Passo fisso Unity, default 20 ms (consigliato fino a 5 ms). AGX Dynamics opzionale (Algoryx) per forze reali e multi-contatto | idem (sezioni Physics, AGX Physics) |
| Sensori | Collider box (non mesh) o raycast; Measure, MeasureForce, MeasureRaycast | idem (Sensor, Sensors) |
| Connettività | S7 TCP, PLCSIM Advanced (coupler come processo separato, oppure "Native" in-process in preview), SIMIT (accoppiamento sincronizzato nel tempo, memoria condivisa), TwinCAT ADS, OPC UA, Modbus, EtherNet/IP, MQTT, UDP, WebSocket, FMU, Simulink, ctrlX, PLCnext, Keba, Mitsubishi, SEW, OpenCommissioning | idem (sezione Interfaces) |
| Sincronizzazione | FastInterface: un thread di comunicazione in background. Nel FixedUpdate: PreFixedUpdate (PLC→sim), poi fisica e logica, poi PostFixedUpdate (sim→PLC). PLCSIM Advanced usa un "Sync Cycle Ms" | idem (Custom Interfaces, PLCSIM Advanced) |
| Robot | Interfacce KUKA, Fanuc, ABB RobotStudio, Universal Robots, Denso, igus, RoboDK, Wandelbots; IK robot ed export programmi (beta) | idem |
| CAD | CADLink: STEP, JT, 3MF (solo editor Windows); CAD Updater per l'aggiornamento live; Pixyz opzionale; Open Digital Twin Interface (JSON con cinematica accanto a JT/STEP/FBX); AutomationML in WEB | idem (CAD import, Open Digital Twin Interface) |
| Logica | Script C#, Unity Visual Scripting, Behavior Graph, LogicSteps (sequenze senza codice) | idem |
| Collaborazione e VR | Meta Quest 3 (MR), VR Builder; Multiplayer e Industrial Metaverse deprecati | idem |
| Report | Recorder, State Statistics, StatDisplay | idem |
| AI | Server MCP open source (91 tool) per agenti AI in Unity; generazione di dati sintetici e YOLO | idem (sezioni MCP Server, AI Builder) |

**Architettura del motore.**
- Il ciclo FixedUpdate di Unity (passo fisso) è il clock unico della simulazione.
- Le interfacce scambiano i segnali in fasi pre e post a ogni passo; il protocollo gira su un
  thread separato.
- Il "Time Scale" accelera anche la fisica.
- Non c'è un lockstep a tempo virtuale con PLCSIM Advanced: lo scambio avviene a un ciclo in
  ms. La SIMIT coupling invece è dichiarata time-synchronized
  [https://realvirtual.io/doc/unity/llms-full.txt].

**Punti di forza distintivi.**
- Prezzo basso e licenza perpetua.
- Il catalogo di interfacce PLC e robot è enorme.
- Ha una pubblicazione WebGL e web, e il viewer web è open source (AGPL).
- La documentazione è pubblica e leggibile dalle macchine (llms.txt).
- È il primo con un server MCP per l'AI.

**Debolezze e critiche** (dichiarate dalla documentazione stessa):
- con Time Scale alto i sensori perdono i pezzi;
- l'interfaccia PLCSIM Advanced vuole "unsafe code" e solo Windows;
- la variante nativa può far crashare Unity;
- l'import STEP e JT funziona solo nell'editor Windows;
- importare migliaia di segnali rallenta l'avvio;
- i Mesh Collider sono inaffidabili per i sensori
  [https://realvirtual.io/doc/unity/llms-full.txt].

A questi si aggiungono la dipendenza dal ciclo di vita e dalle licenze di Unity e la richiesta di
competenze di Unity [inferenza].

**Cosa prendere [inferenza].**
- Le fasi Pre/Physics/Post a passo fisso, con le I/O raccolte in batch.
- La separazione Drive (moto) / DriveBehavior (I/O).
- La fisica ibrida: oggetti trasportati dinamici, macchine cinematiche.
- La documentazione in formato llms.txt e un server MCP fin dall'inizio.

---

## 6. ISG-virtuos (ISG Industrielle Steuerungstechnik, Stoccarda)

**Cosa è e per chi.** È una piattaforma di simulazione per gemelli digitali e VC, sviluppata dal
2005, che copre MiL, SiL e HiL. La sua caratteristica è il HiL in tempo reale deterministico,
bus di campo compresi. Gli utenti sono costruttori di macchine e automotive (BMW, Mercedes-Benz,
HOMAG, Schuler, SICK...) [https://www.isg-stuttgart.de/en/products/softwareproducts/isg-virtuos].
- Prezzo non pubblico; licenza modulare a pacchetti (base, opzioni, tecnologia).
- Esiste uno store online di modelli (TwinStore)
  [https://www.virtuos-twinstore.de/about].

| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Progettazione 3D → modello | Si trascinano componenti 3D da libreria; il modello di comportamento real-time (schema a blocchi) si genera da solo sotto | https://www.isg-stuttgart.de/en/products/softwareproducts/isg-virtuos |
| Librerie | Logica, matematica, pneumatica, meccanica, azionamenti | idem |
| Fisica real-time del flusso materiale (MFPhys) | Algoritmi deterministici per il flusso materiale fino alla scala di fabbrica. Estensioni: mover e AGV (anche monorotaie), tool handling, asportazione di materiale, accoppiamento e assemblaggio (es. viti), separazione e truciolo | idem |
| HIFIphys (v3.8, maggio 2026) | Motore fisico ad alta risoluzione che affianca quelli esistenti, con nuovi modelli di collisione | https://www.isg-stuttgart.de/en/news/details/isg-virtuos-v38-now-available |
| Solver | Windows Multisolver per SiL (tempo reale Windows, circa >10 ms) con controllori virtuali (PLCSIM Advanced, SINUMERIK ONE, KUKA OfficeLite). Real-time solver su RTOS (<1 ms) per HiL via bus di campo. Multisolver real-time su più core | https://www.isg-stuttgart.de/en/products/softwareproducts/isg-virtuos |
| Bus di campo | Dispositivi di campo virtuali come veri partecipanti EtherCAT (anche safety); cablaggio generato dal progetto TwinCAT. Piattaforme: Windows, TwinCAT, B&R, Linux RT | https://infosys.beckhoff.com/content/1033/appnoteinfosys/html/thirdparty_isg.htm [snippet] ; https://www.sercos.jp/produkt/real-time-simulation-software-isg-virtuos-818/ [snippet] |
| Generazione di componenti per bus virtuali (v3.8) | La mappatura dei dispositivi si crea una volta e si riusa; export di variabili dal Beckhoff Engineering Tool | https://www.isg-stuttgart.de/en/news/details/isg-virtuos-v38-now-available |
| API e scripting | OpenVirtuosAPI (v3.8) in Python e C# per creare e pilotare modelli; ECF/ISGMD e JavaScript per generare modelli; SDK C++ per modelli di comportamento real-time | idem + pagina prodotto |
| Misura | Misure tra oggetti durante la simulazione | pagina prodotto |
| CAD | Import dei "formati CAD comuni" (lista non pubblicata) | pagina prodotto |

**Architettura del motore.**
- Schema a blocchi eseguito ciclicamente da un solver. In SiL gira su Windows a circa ≥10 ms;
  in HiL su un RTOS a <1 ms, anche con la fisica del flusso materiale in quel ciclo.
- Il multisolver distribuisce su più core
  [https://www.isg-stuttgart.de/en/products/softwareproducts/isg-virtuos].
- Hardware dedicato: ISG-Real-Time-Target (anche Powerlink), ISG-Workstation [stessa pagina,
  menu prodotti].

**Punti di forza distintivi.**
- È l'unico del gruppo con fisica del flusso materiale deterministica dentro il ciclo di
  controllo da 1 ms e con emulazione dei dispositivi come slave EtherCAT reali.
- Il modello di comportamento si genera dalla vista 3D.

**Debolezze e critiche.**
- I benefici dichiarati (fino a -80% di tempo di commissioning, avvio fino al 30% prima) sono
  marketing non verificato [https://www.computer-automation.de/engineering-en/birth-of-the-digital-twin.htm] [snippet].
- Il HiL richiede hardware RT dedicato.
- Non ho trovato documentazione pubblica dettagliata né critiche indipendenti.
- Fino alla v3.8 mancava un'API di scripting generale (aggiunta nel 2026)
  [https://www.isg-stuttgart.de/en/news/details/isg-virtuos-v38-now-available].

**Cosa prendere [inferenza].**
- Un modello di comportamento compilato in grafo di blocchi, separato dalla vista 3D, così lo
  stesso modello gira headless su un target RT.
- La fisica del flusso materiale "semplificata ma deterministica", distinta dalla fisica di
  alta fedeltà opzionale.
- La generazione dei dispositivi di bus dal progetto PLC.

---

## Sintesi trasversale [inferenza]

| Tema | Riferimento migliore | Idea per il nuovo simulatore |
|---|---|---|
| Scheduler deterministico | SIMIT (8 slice × 8 sotto-slice, minimo 1 ms) | Clock a passo fisso multi-rate, con assegnazione di componente e accoppiamento a una slice |
| Sync col PLC | PLCSIM Adv tempo virtuale + SIMIT bus-sincrono | Modalità lockstep (il simulatore comanda il tempo del PLC virtuale) oltre al free-running |
| Separazione moto / I/O | realvirtual Drive/DriveBehavior, ISG 3D/schema a blocchi | Il modello di comportamento eseguibile headless, la vista 3D come solo client |
| Fisica a livelli | Emulate3D (lineare, planare, volumetrica), realvirtual (PhysX/AGX), ISG (MFPhys/HIFIphys) | Fisica "di trasporto" economica di default, rigida completa per oggetto, plugin di alta fedeltà |
| Connettività | Emulate3D, realvirtual | Un'interfaccia plugin con fasi pre e post e thread I/O separato; S7, OPC UA, PLCSIM Adv, Modbus, ADS fin dall'inizio |
| Generazione del modello | SIMIT (template da tabelle), ISG (da TwinCAT) | Import della lista I/O dal progetto PLC per generare segnali e componenti |
| Cataloghi | Emulate3D (cifrati e licenziabili), MCD (oggetti intelligenti) | Pacchetti di componenti versionati, con firma |
| Robot | Process Simulate | Lasciare un'interfaccia verso i controller virtuali (RCS/VRC) invece di reimplementare |
| Diagnostica | SIMIT (snapshot, trend) | Snapshot e restore dello stato completo, per test ripetibili |

---

## Cosa non è stato verificato

- **Process Simulate:** non ho letto la documentazione ufficiale, che è dietro login.
  - Mancano: il passo temporale (time interval), il modello a eventi o cicli, il motore fisico,
    i nomi esatti dei moduli (Logic Block, sensori), i protocolli OPC UA o DA supportati e le
    marche di controller robot emulati.
  - I dati vengono dalla pagina di prodotto Siemens e da snippet di blog di partner.
- **MCD:**
  - Il motore PhysX "attuale" per default, il passo e il threading non sono confermati (la fonte
    PhysX 2010 è solo uno snippet).
  - I connettori elencati (OPC UA da 12.01, PROFINET) vengono da snippet di un documento SCE;
    il PDF inglese ha dato 404 e la pagina di Aachen 502.
  - Non verificati: le API (NX Open), l'export FMU e il supporto robot.
- **Emulate3D:** i PDF di Rockwell (whats-new 2024, device-emulation) hanno risposto 426/403 e
  non li ho aggirati.
  - PhysX 5.1.3, AGX, il modello multi-istanza e il profiler vengono solo da snippet di ricerca.
  - Le integrazioni robot (FANUC, ABB, KUKA) sono solo da fonti secondarie.
  - Non è documentato il motore temporale (passo fisso o eventi).
- **SIMIT:** il manuale letto è la **V10.0 (06/2018)**.
  - Le funzioni V11.x vengono da un sito terzo (devicebase).
  - Il passaggio ad abbonamento viene da uno snippet di una pagina webinar Siemens.
  - I requisiti della modalità bus-sincrona (IRT) vengono solo da un blog terzo.
- **realvirtual.io:** i prezzi sono letti dai metadati schema.org della pagina acquisto (data
  2026-11-07) e possono cambiare. L'assenza di un lockstep a tempo virtuale con PLCSIM Advanced è
  dedotta dalla doc ("Sync Cycle Ms"), non dichiarata.
- **ISG-virtuos:** i tempi (<1 ms RT, circa >10 ms su Windows) sono dichiarazioni del vendor,
  non misure. I formati CAD e i prezzi non sono pubblici. Le pagine Beckhoff e Sercos sono state
  lette solo come snippet.
- **Prestazioni comparative:** nessuna misura indipendente e omogenea tra i sei prodotti. I
  numeri di ciclo citati non sono confrontabili tra loro.
