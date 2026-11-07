# R3: come i tool di virtual commissioning (VC) organizzano il motore di simulazione e lo sincronizzano col PLC

Ricerca del 2026-11-07. Ogni affermazione porta la fonte tra [ ]. [inferenza] = deduzione mia, non scritta nella fonte.
Le fonti Siemens sono PDF scaricati da `cache.industry.siemens.com` e letti con `pdftotext`. `support.industry.siemens.com`
non è stato usato (blocca con 403). Legenda: FM = Function Manual.

Fonti principali, con le abbreviazioni usate sotto:
- FM6 = S7-PLCSIM Advanced Function Manual V6.0 UPD1, 03/2024 (A5E37039512-AG) [https://cache.industry.siemens.com/dl/files/578/109955578/att_1267815/v1/s7-plcsim_advanced_function_manual_en-US.pdf]
- FM3 = S7-PLCSIM Advanced Function Manual 11/2019 (A5E37039512-AC, V3.0); contiene il capitolo API completo [https://cache.industry.siemens.com/dl/files/484/109773484/att_1004713/v1/s7-plcsim_advanced_function_manual_en-US_en-US.pdf]
- RM4 = Readme PLCSIM Advanced V4.0 [https://cache.industry.siemens.com/dl/files/483/109773483/att_1004710/v1/s7plcsim_advanced_readme_en-US_en-US.pdf]
- RM5 = Readme PLCSIM Advanced V5.0 Upd2 [https://cache.industry.siemens.com/dl/files/215/109823215/att_1150293/v2/s7plcsim_advanced_readme_en-US.pdf]
- GL = Guideline "Virtual Commissioning with SIMIT, NX MCD and S7-PLCSIM Advanced", Entry-ID 109963864, V1.0, 04/2024 [https://cache.industry.siemens.com/dl/files/864/109963864/att_1277519/v1/Guideline_VC_SIMIT_PSA_MCD_V1.0.0.pdf]
- FAQD = FAQ "Drive Simulation and Optimization in SIMIT, S7-PLCSIM Adv. & NX", Entry-ID 109990858, V1.0.0, 07/2025 [https://cache.industry.siemens.com/dl/files/858/109990858/att_1335079/v1/FAQ_Drive_Simulation_and_Optimization_Manual_V1_0_0_en.pdf]
- PS19 = S7-PLCSIM (classico) Operating Manual / Help V19, 11/2023 [https://cache.industry.siemens.com/dl/files/203/109826203/att_1161267/v1/PLCSIM_Help_en-US.pdf]
- PS21 = S7-PLCSIM Readme V21, 11/2025 (A5E51611959-AP) [https://cache.industry.siemens.com/dl/files/163/109997163/att_1348297/v1/Readme_enUS_en-US.pdf]
- G2SM = S7-1200 G2 System Manual V4.1, 12/2025 (copia locale `manuali_S7-1200_G2/S71200_G2_system_manual_V4.1_2025-12_en-US.pdf`)

> **Nota 2026-11-07:** il supporto della G2 in PLCSIM Advanced V8, dato sotto da fonti secondarie, è
> già **misurato** nel progetto BoxLine (il repository `chetto1983/apr`, design §12; `plc_test/PlcSimAdv.ps1`, API V8 A5E53246567-AE).
> Non è misurato il funzionamento dei modi SingleStep e TimespanSynchronized su un'istanza G2.

---

## 1. Siemens: NX MCD, Process Simulate, SIMIT e l'accoppiamento con PLCSIM Advanced

**Architettura**
- SIMIT è l'"interfaccia centrale di simulazione e orchestratore": sceglie il modo operativo e la sincronizzazione con gli altri partecipanti (PLCSIM Advanced, NX MCD, Process Simulate, robot virtuali) [GL §2.2.3, p.17; Fig. 2-2].
- I partecipanti sono tre: PLCSIM Advanced per il PLC, SIMIT per il comportamento di segnali, azionamenti e quadro elettrico, NX MCD per la cinematica e il flusso materiale ("rigid body simulation with physics engine") [GL §2.3, p.19].
- I tool di VC sono "di solito applicazioni single-core". Contano quindi le prestazioni del singolo core [GL §1.5.1, p.7].
- PLCSIM Advanced chiede un core libero per ogni istanza per comportarsi nel modo più deterministico possibile [FM6 §2.7].

**I tre modi operativi di SIMIT** [GL §2.2.3, pp.17-18; i diagrammi usano un passo di 2 ms di tempo virtuale]
- *Asynchronous*: nessun partecipante aspetta gli altri. Se un calcolo non finisce in tempo, il passo viene saltato e il valore nel coupling non si aggiorna.
- *Synchronous*: il calcolo del modello cade sempre tra la lettura degli ingressi e la scrittura delle uscite. Se il calcolo sfora, SIMIT aspetta: la simulazione rallenta per quel ciclo e poi prosegue con l'offset accumulato.
- *Isochronous*: tutti i partecipanti avanzano con lo stesso tempo virtuale, indipendente dal tempo reale, e i passi si eseguono insieme. È il modo "preferito per la VC per avere risultati deterministici". MCD aspetta SIMIT e SIMIT aspetta MCD.
- Con PLCSIM Advanced, SIMIT offre la sincronizzazione *bus-synchronous* o *event-synchronous*. Quale delle due dipende dagli OB e dalle partizioni di immagine di processo (PIP). "Per la maggioranza dei casi basta la bus-synchronous su MC-Servo OB91" [GL p.18, Fig. 2-14].
- SIMIT definisce le "time slice" con il loro tempo di ciclo in *Project Manager > Times & operating modes*. Nel caso d'esempio il modo è isochronous e il coupling PLCSIM Advanced è impostato su "Bus synchronous" [FAQD §3, Fig. 3-19 e 3-20].
- Nell'ottimizzazione del caso d'esempio c'è una time slice da 1 ms per i componenti SIMIT legati ai segnali MCD; la FAQ la chiama "il ciclo più basso, 1 ms, in SIMIT" [FAQD §4.1, Fig. 4-1/4-2].
- Gli FMU entrano in SIMIT solo se sono eseguibili in tempo reale a passi temporali discreti [GL p.~16, "Functional Mock-Up Interface"].

**Numeri misurati da Siemens (FAQD, Tab. 3-3; setup con azionamento REALE)**
- Ritardo tra setpoint e valore attuale, per ciclo OB91:
  - 1 ms: 5 ms;
  - 2 ms: 10 ms;
  - 4 ms: 12 ms.
- A 4 ms il ritardo è di 3 cicli. Con cicli più corti c'è un po' più di jitter e di overshoot [FAQD pp.~22-24].
- Questi numeri non provano niente sulla latenza del coupling simulato: la tabella è dichiarata "real drive setup" [inferenza].

**NX MCD**
- Il passo fisico di MCD non risulta in nessuna fonte Siemens primaria che io abbia potuto leggere.
- Un articolo di terze parti consiglia un passo di 10 ms, allineato e sottomultiplo del ciclo SIMIT, ma non è verificato [https://cielo.com/C135/post/resolving-simit-and-nx-mechatronics-concept-designer-mcd-signal-lI6jKZwSAT8mS7x].
- La signal mapping diretta "PLCSIM Advanced" richiede PLCSIM Advanced sulla stessa macchina [https://community.sw.siemens.com/s/question/0D54O00006Y1U6iSAF/i-am-unable-see-the-plcsim-advance-in-signal-mapping-please-do-the-need-full-i-am-using-nx110].
- GL avverte che il comportamento fisico reale differisce da quello simulato e che i problemi multifisici vanno trattati a parte, per esempio con Simcenter AMESIM [GL §2.3].

**Process Simulate**
- Non ho trovato una fonte primaria con il tempo di ciclo di Process Simulate verso PLCSIM Advanced: le pagine Siemens sono su support.industry, bloccato.
- Un paper riporta che il tempo avanza sincronizzato tra PLCSIM Advanced e Process Simulate, con scaling da 0,01 a 100 [https://www.matec-conferences.org/articles/matecconf/pdf/2019/48/matecconf_mtem2019_02005.pdf]. Il paper parla di Plant Simulation/PS, quindi va verificato.
- JR Automation (2019): OPC "troppo lento", tra 300 e 500 ms; il canale PLCSIM Advanced permette scambi sincronizzati in 15-20 ms [https://www.automationworld.com/products/control/blog/13320025/virtual-commissioning-of-a-robotic-cell]. È un dato aneddotico.

**Esempio Siemens Simulink ↔ PLCSIM Advanced (S-function)**
- I tag si scambiano al *cycle control point*: se il PLC non lo ha ancora raggiunto, la S-function aspetta la fine del suo ciclo.
- Con "Enable time synchronization", Simulink rallenta o accelera il proprio pseudo-real-time per seguire il tempo virtuale del PLC.
- Il ciclo dell'OB ciclico (20 ms) deve coincidere con il Ts del modello [Use Case 1, Entry 109749187, §3.2.1/3.2.2, https://cache.industry.siemens.com/dl/files/187/109749187/att_935975/v1/109749187_DIGI_Usecases_API_DOC_V10_en.pdf].

## 2. API PLCSIM Advanced e gestione del tempo

**Due orologi**
- Il *virtual clock* è la base del programma utente: OB ciclici, sorveglianza del tempo di ciclo, tempo minimo di ciclo, ora di sistema virtuale. Anche il tempo tra due cycle control point (o single-step point) si misura in tempo virtuale.
- Il *real clock* resta invariato e serve, per esempio, alla comunicazione con STEP 7 [FM6 cap.6, p.74].
- Se Windows sospende il processo, si fermano entrambi gli orologi [FM6 cap.6].

**Scaling**
- `ScaleFactor`, `SetScaleFactor()`: valori validi da 0,01 a 100, default 1.
- Una modifica a runtime ha effetto solo al cycle control point [FM3 §7.6.6, pp.246-247].
- Lo scaling non cambia la velocità di esecuzione del codice macchina, ma solo la frequenza degli eventi temporizzati.
- Se il fattore è troppo alto, la sorveglianza del ciclo manda la CPU in STOP. Il manuale consiglia di salire per gradi [FM6 §6.1, p.75].
- Ci sono anche le API `GetSystemTime()` e `SetSystemTime()` [FM3 p.245].

**Freeze state e sync point** [FM3 §6.2, pp.90-91]
- Nel freeze state:
  - il tempo virtuale è fermo;
  - non girano OB né timer;
  - l'I/O è consistente;
  - la CPU resta raggiungibile da TIA.
- Il freeze state non è uno stato operativo: i LED mostrano ancora RUN o STOP, l'API riporta `SROS_FREEZE`.
- C'è un sync point prima di ogni lettura degli ingressi: al cycle control point (inizio OB1) o all'inizio di un OB ciclico.
- In modo *Default* la CPU non va mai in freeze.

**Enum `EOperatingMode`** [FM3 §7.8.7.4, p.370]
- Valori: `Default=0`, `SingleStep_C=1`, `SingleStep_CT=2`, `TimespanSynchronized_C=3`, `SingleStep_P=4`, `TimespanSynchronized_P=5`, `SingleStep_CP=6`, `SingleStep_CPT=7`, `TimespanSynchronized_CP=8`.
- `SingleStep_Bus` è stato aggiunto in V4.0 [RM4].

**Modi SingleStep (sincronizzazione a cicli)** [FM3 §6.3.1, p.92; RM5]
- Le lettere indicano dove la CPU si ferma:
  - C = al cycle control point;
  - P = prima della lettura di una PIP;
  - T = con tempo minimo di ciclo sovrascritto (`OverwrittenMinimalCycleTime_ns`, default 100 ms virtuali).
- La CPU va in freeze e manda `OnSyncPointReached`. `RunToNextSyncPoint()` la fa ripartire fino al sync point successivo.
- L'evento porta `TimeSinceSameSyncPoint_ns` e `TimeSinceAnySyncPoint_ns`, cioè il tempo virtuale dall'ultimo sync point della stessa PIP o di una qualsiasi [FM3 p.93].
- `SingleStep_Bus` si sincronizza sul send clock dell'IO system PROFINET o PROFIBUS.
  - Usa `RunToNextSyncPoint(ioSystemId, busTimeStamp_ns)`, con `ioSystemId` 1..32 (PROFIBUS) o 100..115 (PROFINET).
  - Richiede uscite configurate e `SYNC_PO` nell'OB di sincronismo di clock [RM4].
- Siemens sconsiglia di cambiare modo SingleStep durante l'esecuzione [RM5].
- `SingleStep_P` genera un `SyncPointReached` per ogni PIP e per ogni IO system contenuto in quella PIP [FM6 §7.22].

**Modi TimespanSynchronized (sincronizzazione a tempo)** [FM3 §6.3.2, pp.94-95; §7.6.7, p.252]
- `StartProcessing(in_MinimalTimeToRun_ns)` lascia girare la CPU almeno per quel tempo virtuale. Poi la CPU va in freeze al sync point successivo e riporta il tempo effettivamente usato.
- Serve un *sync master*: un client API che dà il "via" a ciascun client a turno. Lo schema è Gauss-Seidel nel senso della co-simulazione [inferenza].
- Un client API può gestire il tempo solo di istanze PLC: non riceve eventi da altri client API.
- La granularità reale è il ciclo del PLC: la CPU si ferma al sync point *successivo* al tempo richiesto, quindi il passo effettivo è almeno t e arriva fino a circa t + un ciclo OB [inferenza dal testo "at least as long as required"].

**Eventi e I/O**
- `OnSyncPointReached` gira in un thread separato. Ci sono anche `RegisterOnSyncPointReachedCallback()` e `WaitForOnSyncPointReachedEvent()` [FM3 pp.284-287].
- L'area I/O dell'API si sincronizza con l'immagine di processo al cycle control point e alle chiamate degli OB (PIP, interrupt).
- Se API e programma scrivono la stessa area, vince la scrittura dell'API [FM6 §5.1.5, p.59].
- `ProcessEvent(hwId, channel, type)` simula gli hardware interrupt dei moduli di ingresso [FM3 §7.6.8.4, p.263]. Potrebbe servire a emulare gli eventi HSC, ma va verificato sulla G2 [inferenza].
- *Strict Motion Timing* attiva o disattiva il rilevamento degli overrun dell'OB MC-Servo [FM6 p.~53].

**Supporto S7-1200 / S7-1200 G2**
- V6.0: "S7-PLCSIM Advanced does not simulate S7-1200 CPUs"; per l'S7-1200 si usa S7-PLCSIM [FM6 §2.7, p.21]. La tabella di confronto elenca per V6.0 solo S7-1500, ET 200SP/pro e Software Controller [FM6 §2.2, p.17].
- V7.0: un estratto del manuale V7.0 dice ancora "does not simulate S7-1200 or S7-1200 G2" (visto solo come snippet di ricerca) [https://www.scribd.com/document/854490123/S7-PLCSIM-Advanced-Manual-V7-0].
- V8.0 (con TIA V21) aggiunge "tutti i numeri d'articolo S7-1200 G2 con firmware V4.1" e anche il FW G2 V1.0, i progetti TIA da V14 a V21 e le versioni API da V3.0 a V8.0 [https://www.infoplc.net/descargas/siemens/software-step7-tiaportal/tia-portal/descarga-s7-plcsim-advanced-demo-trial]. Lo conferma PLC-Lab: "Starting with S7-PLCSIM Advanced V8 … S7-1200 G2 can be simulated" [https://www.mhj-wiki.de/en/plc-lab/device-plcsim-advanced/].
  - Mancano un manuale o un readme Siemens V8 leggibile: queste fonti sono secondarie.
  - Il testo non dice se i modi SingleStep o Timespan valgono anche per le istanze G2. Va misurato [inferenza].

**PLCSIM classico (V19/V21)**
- La tabella di FM6 indica "API for co-simulation: –" e "Virtual time: –" per S7-PLCSIM [FM6 p.17].
- L'S7-1200 gira solo come istanza "Standard", non configurabile come Advanced [PS19 §2.5, p.~13].
- PLCSIM non simula, per l'S7-1200, conteggio e misura, PID e motion [PS19 §2.4.3, p.12].
  - Notizie secondarie dicono che il PID su G2 con PLCSIM V21 si simula. Non è verificato [https://industrialmonitordirect.com/blogs/knowledgebase/resolving-pid-compact-v2x-plcsim-simulation-failure-on-s7-1200].
- Gli HSC onboard e i PTO/PWM delle 1500C non sono simulabili [PS19 §5.11].
- Per le istanze S7-1200, la tabella eventi e le sequenze permettono di generare a mano gli eventi "HSC CV=RV1", "HSC CV=RV2", "HSC direction changed" e "HSC reset" [PS19 p.~28].
- V21 introduce:
  - un "Global Properties" tab "con le funzioni del control panel PLCSIM Advanced";
  - sequenze in formato tabella;
  - il supporto G2 (1212/1214, V4.1). Per "Upload as new station" serve il modo TCP/IP [PS21 p.1-2].
- Non ho trovato una API pubblica documentata per PLCSIM classico. PLCSIM classico può caricare istanze avviate dal control panel o dall'API di PLCSIM Advanced [PS19 cap.2]. Probabilmente il runtime è condiviso, ma non è verificato [inferenza].

## 3. Altri tool: passo di simulazione e scambio di I/O

**Emulate3D (Rockwell)**
- Fonte: deck RokLive 2025, "FMI, Cosimulation, and Controls Testing". Il PDF risponde 403 a curl e WebFetch, quindi ho letto solo gli snippet di ricerca [https://www.rockwellautomation.com/content/dam/rockwell-automation/documents/pdf/events/roklive-emea-2025/digital-engineering-summit/augment-your-digital-model-with-cosimulation.pdf].
- Ci sono due modi:
  - "Scheduled (co-sim disabled)": il tempo è sincronizzato col tempo reale e l'I/O segue uno scheduler real-time;
  - "Synchronized (co-sim enabled)": il tempo è virtuale.
- Il protocollo legacy "PLCSIM Advanced" sincronizza al cycle control point. Il nuovo "S7-PSA" ha sync point configurabili, sincronizzazione del tempo in co-simulazione e notifiche durante il download. Richiede PLCSIM Advanced v6/v7 o successivo.
- Esiste un video di training "controls testing at sub-10 millisecond timeframes" [https://store.sim3d.com/training_videos/plcsim_advanced_cosimulation_example]. Non l'ho visto.
- Emulate3D offre 4 motori fisici [https://www.simplan.de/de/]. Non ho trovato una fonte primaria sul passo fisso.

**ISG-virtuos**
- Simulazione in "harter Echtzeit (deterministisch <1 ms)" sul Realtime Target, con ciclo deterministico fino a 250 µs.
- Bus reali: PROFINET (anche PROFIsafe), PROFIBUS, CANopen, EtherCAT, POWERLINK.
- Il tempo di ciclo si fissa in modo indipendente per ogni core di simulazione.
- La figura confronta "z.B. TCP/IP >10 ms" con il bus reale "<1 ms" [https://www.isg-stuttgart.de/fileadmin/media/5_Hardware/Broschuere_RTTarget_2021_DE.pdf].
- Per l'HiL via bus reale servono hardware dedicato e un RTOS [https://www.isg-stuttgart.de/en/products/hardware-products].

**Visual Components**
- Il pannello Connectivity misura "update time" (dalla modifica della variabile alla conferma di scrittura dal server) e "plugin time" [https://help.visualcomponents.com/4.9/Premium/en/English/Connectivity/Accessing_Connectivity_feature.htm].
- Dal forum:
  - gli aggiornamenti della connectivity seguono il tempo reale, non il tempo di simulazione;
  - c'è un modo "event-based" (OPC UA subscription) e uno ciclico;
  - un utente misura circa 20 ms per 15-20 bool. [https://forum.visualcomponents.com/t/vc-plc-signals-update-rates/3666] È un dato aneddotico.

**realvirtual.io (Unity, ex game4automation)**
- Il PLCSIM Advanced gira tramite un exe "coupler" esterno: le DLL Siemens non si compilano in Unity. Lo scambio con Unity passa per shared memory.
- I parametri sono "Sync Cycle Ms" e "Pause With Unity". Lo scambio si sospende durante il download e si riconnette.
- La pagina dichiara PLCSIM Advanced solo per S7-1500 [https://realvirtual.io/doc/unity/components-and-scripts/interfaces/plcsim-advanced].
- L'interfaccia SIMIT ha "Allow Time Sync: SIMIT pilota il FixedUpdate di Unity e il ciclo segnali" [https://realvirtual.io/doc/unity/components-and-scripts/interfaces/siemens-simit-interface-pro].
- `ITimeSyncedPhysics.CalcFixedUpdate(deltaTime)` serve a sincronizzare dall'esterno i componenti Drive e TransportSurface [https://realvirtual.io/apidoc/interfacerealvirtual_1_1_i_time_synced_physics.html].
- Unity, alla base:
  - la fisica gira a `Time.fixedDeltaTime` fisso;
  - in pratica l'intervallo reale tra fixed update non è fisso, perché il fixed update gira dentro un frame e recupera il backlog [https://docs.unity.com/en-us/engine/6000.0/manual/scripting/managing-time-and-frame-rate/fixed-updates.md];
  - `maximumDeltaTime` (default 0,333 s) limita il catch-up: oltre quel limite la fisica "rallenta" invece di recuperare [https://docs.unity.com/en-us/engine/6000.0/manual/scripting/managing-time-and-frame-rate/time-handling-variations.md].

**FluidSIM (Festo)**
- Il solver numerico continuo usa un passo automatico. Si può imporre un incremento ("display interval", per esempio 1e-6 per i µs), ma con incrementi piccoli il tempo reale non è più garantito.
- Il fattore slow-motion va da 1:10 a 1:0,1.
- La percentuale di tempo reale raggiunta compare nella barra di stato. OPC/DDE ed EasyPort servono per l'I/O esterno [FluidSIM 5 User Guide §18.10, p.308; p.307; §9.12, https://eitcdn.eit.edu.au/files/eit/FluidSim/fluidsim5_enus_user%20guide.pdf].
- Non viene indicato un ciclo I/O.

**Sintesi** [inferenza]
- Tutti usano un motore a passo fisso (Unity FixedUpdate, MCD, time slice SIMIT, ISG) o un solver continuo con passo adattivo (FluidSIM).
- Lo scambio I/O avviene una volta per passo di comunicazione, mai in modo event-driven puro.
- Il determinismo arriva solo se il PLC è nel loop del tempo virtuale (freeze/sync point) o con un RTOS e un bus reale (ISG).

## 4. FMI 2.0/3.0 e SSP

- In FMI 3.0 il co-simulation algorithm resta all'importer: lo standard non definisce un master [FMI 3.0 §1.2, §2.2.6, https://fmi-standard.org/docs/3.0/].
  - L'importer chiama `fmi3DoStep(currentCommunicationPoint, communicationStepSize>0)`.
  - Gli eventi possono accorciare il passo, anche a 0.
- Gli schemi master tipici sono:
  - *Jacobi*: tutte le unità avanzano in parallelo sullo stesso intervallo e gli ingressi si scambiano a fine passo;
  - *Gauss-Seidel*: un'unità alla volta, ciascuna usa le uscite più recenti delle precedenti.
  - Passo di comunicazione, iterazione ed estrapolazione degli ingressi sono parametri dell'orchestratore [Gomes et al., "Co-simulation: State of the art", https://arxiv.org/abs/1702.00686; tutorial arXiv 1809.08463].
- Novità di FMI 3.0 per la co-simulazione [FMI 3.0 §1.1, §1.2.4, §2.2.6]:
  - Event Mode;
  - *early return* (`earlyReturn`, `earlyReturnRequested`);
  - *Intermediate Update*, per cambiare gli ingressi a metà passo e interpolare.
- Clock [FMI 3.0 §2.2.8.2]:
  - periodici a tempo, con intervallo costante salvo `intervalVariability = tunable`;
  - *triggered*, anche da interrupt hardware lato importer. Le uscite clock devono essere triggered.
- *Scheduled Execution* [FMI 3.0 §1.2.3, §2.2.6]:
  - espone partizioni del modello attivate da uno scheduler esterno con `fmi3ActivateModelPartition(activationTime)`;
  - "time is under the sole control of the importer";
  - è pensata per simulatori real-time e vECU.
  - Rilevanza per il PLC-in-the-loop [inferenza]: lo schema è lo stesso di un PLC con task ciclici (OB30..), cioè partizioni con clock periodici e un interrupt come triggered clock. PLCSIM Advanced in TimespanSynchronized è l'equivalente di una "unità" a passo variabile minimo, mentre un FMU dell'impianto è un'unità a passo fisso.
- In FMI 2.0 c'è solo `fmi2DoStep` a passo di comunicazione, senza clock né event mode in co-simulazione. Non l'ho riletto in questa sessione [inferenza da FMI 3.0 §1.1, che elenca queste funzioni come novità].
- *SSP* (Modelica Association) fissa formati XML e ZIP per la struttura del sistema (rete di componenti, connessioni, parametri). È complementare a FMI e non limitato agli FMU [https://ssp-standard.org/docs/main/; https://www.prostep.org/mediathek/fact-sheets/system-structure-and-parameterization-ssp].
  - Nessuna fonte gli attribuisce un algoritmo di esecuzione o di stepping [inferenza].
  - SSP 2.0 (12/2024) aggiunge il supporto a FMI 3.0 e lo scambio di "architetture di simulazione astratte" [https://www.nafems.org/about-us/media/news/industrynews1003/ssp-2-0-released-standardized-exchange-of-composite-system-models-and-simulation-architectures/].

## 5. Requisiti di latenza e jitter, limiti di OPC UA e S7comm

**Requisiti**
- Guida VDMA sulla VC:
  - calcolo dei tempi "nell'ordine di 1-10 ms per processi PLC e calcoli di collisione in robotica, CNC o motion";
  - "<1 ms per la simulazione di un bus di campo" [VDMA/Forum Industrie 4.0, "Virtual Commissioning Guideline", Tab. criteri, https://www.vdma.eu/c/document_library/get_file?uuid=12b18c33-bb36-4f68-0770-5987b8cbf837&groupId=34570].
- GL: nel SiL il requisito real-time si soddisfa sincronizzando i cicli, anche con time scaling. Nell'HiL il determinismo richiede che la base tempi della simulazione coincida con quella della CPU reale [GL §1.x, p.~5].
- Scheifele, Verl, Riedel, "Real-time co-simulation for the virtual commissioning of production systems", Procedia CIRP 79 (2019) 397-402, doi:10.1016/j.procir.2019.02.104.
  - La co-simulazione real-time si distribuisce su più task. Il partizionamento è manuale; il progetto aCoSi lo automatizza [https://www.isw.uni-stuttgart.de/en/research/projects/aCoSi/].
  - Non ho letto il testo completo.

**Encoder, HSC e G2 (dati hardware)**
- Gli ingressi HSC della G2 contano fino a 100 kHz in single phase su Ia.0..Ia.5; su alcune CPU o signal board fino a 200 kHz [G2SM, dati tecnici, righe ~17267 e ~21589 del testo estratto].
- Conseguenze [inferenza]:
  - Nessun canale software (API PLCSIM, OPC UA, S7comm) emula impulsi a 10-100 kHz.
  - Va emulato il valore di conteggio (CV) e gli eventi CV=RV, non i fronti.
  - Il PLCSIM classico non simula affatto l'HSC dell'S7-1200 (PS19 §2.4.3): gli eventi vanno iniettati con event table, sequenze o `ProcessEvent`, dove supportato.

**OPC UA sulla G2** [G2SM §7.10.2, pp.274-275]
- Limiti:
  - 2 server interface;
  - 2000 nodi OPC UA;
  - 50 subscription concorrenti, 5 per sessione;
  - 201 metodi server.
- Nessuna "standard SIMATIC server interface": l'interfaccia va definita in STEP 7.
- Gli intervalli minimi di sampling e publishing della G2 non risultano dal system manual: sono rimandati al Communication Function Manual 59192925, su support.industry, quindi non letto.
- Le cifre dai blog (10 ms di sampling, 50-100 ms di publishing su S7-1500) non sono verificate [https://industrialmonitordirect.com/blogs/knowledgebase/opc-ua-server-sampling-interval-vs-publishing-interval-in-tia-portal-s7-1500].
- Visual Components legge `MinSupportedSampleRate` dal server; un caso riportato è di 1000 ms [https://forum.visualcomponents.com/t/vc-plc-signals-update-rates/3666].

**Latenze riportate da progetti** (non verificate)
- OPC: 300-500 ms. Pipe PLCSIM Advanced: 15-20 ms [automationworld 2019, sopra].
- S7comm PUT/GET con snap7: 5-50 ms di round trip a seconda del ciclo e della rete. Sono solo stime da blog [https://industrialmonitordirect.com/blogs/knowledgebase/real-time-siemens-s7-plc-data-access-via-snap7-and-tcpip].
- La G2 ha 10 risorse di connessione riservate e 78 dinamiche [G2SM §7.x, riga ~10193], con un carico di comunicazione di default del 20 % [G2SM §5.4.7, p.128].

**Implicazioni per un tool S7-1200 G2 + PLCSIM** [inferenza]
- Con un PLC reale e uno scambio via OPC UA o S7comm (asincrono, decine di ms, con jitter Windows), il tempo virtuale non è sincronizzabile. Ha senso solo per logiche lente, quindi non per la BoxLine a 120 m/min, dove 1 ms vale 2 mm.
- Con PLCSIM Advanced V8 e una G2 (da verificare), un master in `TimespanSynchronized_*` o `SingleStep_*` rende il loop deterministico: il passo fisico è fisso, `StartProcessing(Δt)`, sync point, scambio dell'immagine di processo. La velocità reale non conta.
- Con PLCSIM classico V21, senza API né tempo virtuale documentati, il loop è asincrono come quello verso un PLC reale.

## Da misurare (nessuna fonte lo prova)
1. Che PLCSIM Advanced V8 accetti la G2 dell'impianto (FW V4.1) e che `OperatingMode = TimespanSynchronized_CP` funzioni su un'istanza G2.
2. La granularità effettiva di `StartProcessing(Δt)` con un ciclo OB1 reale: quanto supera Δt.
3. Il comportamento degli HSC G2 in PLCSIM Advanced V8: CV scrivibile? `ProcessEvent` CV=RV?
4. La latenza e il jitter reali di un client S7CommPlus (es. `plc_test/boxline3d` nel repository `chetto1983/apr`) verso PLCSIM e verso la CPU.
