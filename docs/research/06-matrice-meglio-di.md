# Il meglio dei simulatori commerciali: matrice per il nuovo simulatore open source

Stato: **studio, 2026-11-07**. È l'inventario da cui ricavare visione e requisiti. Non è ancora un design.

## 1. Metodo, e cosa prova e cosa non prova

- **Prodotti studiati:** 14, più 14 progetti open source.
  - Visual Components 5.1: mappe 01–05, dalla guida ufficiale completa.
  - Messa in servizio virtuale: NX MCD, Process Simulate, SIMIT, Emulate3D, realvirtual.io, ISG-virtuos
    ([g1](benchmark/g1-virtual-commissioning.md)).
  - Flusso materiali, cioè simulazione a eventi discreti (DES): Plant Simulation, FlexSim, AnyLogic, Simio
    ([g2](benchmark/g2-flusso-des.md)).
  - Didattici: Factory I/O, CIROS, FluidSIM, più i progetti open source
    ([g3](benchmark/g3-didattici-opensource.md)).
- **Fonti:** ogni fatto nei benchmark ha il suo URL. Le fonti di terzi (forum, aggregatori) sono marcate
  [secondaria]. Le colonne "Idea" qui sotto sono deduzioni.
- **Verificato a campione sulla fonte:**
  - SIMIT V10: 8 time slice con ciclo di almeno 1 ms, ciascuna con 8 sotto-slice da A a H;
  - Factory I/O: `physics.timestep` 1/120 s, 8 substep, 20 iterazioni;
  - docs.flexsim.com reindirizza a help.autodesk.com (FlexSim 2027);
  - PLCSIM Advanced: modi `TimespanSynchronized`;
  - S7 in VC: solo le aree I, Q, M.
- **Non prova:**
  - nessun prodotto è stato installato o provato;
  - i dettagli dietro login (Process Simulate, MCD, help di Plant Simulation, PDF Rockwell e Festo)
    vengono da snippet o da fonti secondarie;
  - nessun prodotto pubblica un listino completo;
  - non esistono benchmark di prestazione confrontabili.

## 2. Matrice: chi fa meglio cosa

| # | Area | Migliore riferimento | Cosa fa meglio | Idea per il nuovo simulatore | Fonte |
|---|---|---|---|---|---|
| 1 | Modello del componente | Visual Components | albero di nodi con giunti, feature parametriche, behavior, proprietà e segnali tipizzati, interfacce plug-and-play | stesso schema a oggetti, con formato aperto e documentato | 02a |
| 2 | Libreria con ereditarietà | Plant Simulation | da una classe di libreria nascono le istanze; una modifica alla classe si propaga, l'override resta locale | componenti come classi versionate con propagazione | g2 §1 |
| 3 | Logica senza codice | FlexSim Process Flow, Simio add-on process | grafo a token agganciato a oggetti o template | editor a grafo che compila sullo stesso motore degli script | g2 §2, §4 |
| 4 | Scripting | VC (Python 3 asyncio), Simio (Python dalla v19, debug in VS Code) | logica come coroutine, `await delay` e `await condition` | Python asincrono come linguaggio di prima classe, con debugger esterno | 05 §1, g2 §4 |
| 5 | Motore del tempo | VC (DES più strati campionati), AnyLogic (ibrido), SIMIT (multi-rate fisso) | ciascuno copre un caso: flusso, continuo, ciclo I/O | DES deterministico con tratti analitici, più slice periodiche multi-rate per I/O e fisica | 05, g1 §3, g2 §3 |
| 6 | Determinismo | SimPy, OMNeT++, JaamSim (contro-esempio: AnyLogic non garantisce l'ordine degli eventi simultanei) | ordine `(t, priorità, seq)`, tempo intero | ordine documentato e test di riproducibilità nel CI | r1, g2 §3 |
| 7 | Sincronizzazione col PLC | SIMIT isochronous e bus-sincrono con PLCSIM Advanced; ISG HiL sotto 1 ms | il PLC avanza nel tempo virtuale del simulatore | due modi: libero (con errore `OutOfSync`) e lockstep con `TimespanSynchronized` | 05 §4, g1 §3, §6 |
| 8 | Ampiezza della connettività | Emulate3D (multimarca, PROFIsafe, fa anche da server PUT/GET e Modbus), realvirtual | molti protocolli; il simulatore può fare da server verso HMI e SCADA | interfaccia a plugin con fasi pre/post e thread I/O; S7 simbolico, OPC UA, Modbus, ADS, MQTT nel nucleo | g1 §4, §5 |
| 9 | Connettività S7 | nessuno completo: VC solo I/Q/M fino a REAL | – | S7CommPlus simbolico sui DB, con tutti i tipi | 03 §2.3 |
| 10 | Modello generato dal progetto PLC | SIMIT (template da tabelle I/O), ISG (da TwinCAT) | segnali e componenti creati dalla lista I/O | import della tag table o del progetto PLC, con proposta di mappatura | g1 §3, §6 |
| 11 | Esperienza della mappatura I/O | Factory I/O | tag mappati con drag & drop; export XML della mappatura per TIA | stesso flusso, con l'elenco tag letto direttamente dal PLC | g3 §1 |
| 12 | Guasti e istruttore | Factory I/O (forzature con un clic, pannello istruttore con password), FluidSIM (configurazioni di guasto), CIROS (pezzi difettosi, rumore) | iniezione di guasti ripetibile | guasti come scenari salvabili e scriptabili, utili ai test di regressione PLC | g3 |
| 13 | Fisica a livelli | Emulate3D (lineare, planare, volumetrica), realvirtual (PhysX/AGX), ISG (MFPhys/HIFIphys) | costo proporzionale al bisogno | trasporto cinematico di default, fisica rigida per singolo oggetto, motore ad alta fedeltà come plugin | g1 |
| 14 | Linee ad alta cadenza | FlexSim mass flow; JaamSim (nastro analitico) | nessun passo per pezzo né per frame | nastro a segmenti analitici, con un modo aggregato e pezzi "virtuali" | g2 §2, r1 §4 |
| 15 | Movimentazione | FlexSim (AGV, magazzini, persone, fluidi), AnyLogic Material Handling | reti AGV con control point, gru, scaffali | librerie AGV, magazzino e gru costruite sullo stesso modello a componenti | g2 |
| 16 | Robot e programmazione offline | Process Simulate (controller emulati RCS), VC (OLP completa) | tempi ciclo realistici con il controller del costruttore | interfaccia verso controller virtuali (RCS/VRC), non una reimplementazione; OLP più avanti | g1 §2, 04 |
| 17 | Esperimenti e ottimizzazione | FlexSim Experimenter, AnyLogic (OptQuest, cloud) | scenari per repliche, multi-core | runner headless di scenari e repliche, con un database dei risultati | g2 |
| 18 | Headless, API, cloud | AnyLogic Cloud (REST, Python, JS, Java; privato su Docker/K8s), FlexSim (webserver, Docker, RL Gym) | il modello gira senza interfaccia | motore come libreria e servizio: CLI, REST e Python; l'interfaccia è un client | g2 §2, §3 |
| 19 | Interoperabilità 3D e CAD | VC (34 formati), FlexSim (USD/Omniverse) | import ampio, export USD | glTF e USD nativi; STEP e altri formati tramite librerie con licenza compatibile | 01 IMP, g2 §2 |
| 20 | Cataloghi | VC eCatalog (sorgenti, smart collection), Emulate3D (cataloghi cifrati e licenziabili) | distribuzione dei componenti | pacchetti di componenti versionati e firmati, con registry pubblico | 01 CAT, g1 §4 |
| 21 | Snapshot e ripristino | SIMIT | salvataggio e ripristino dello stato a runtime | snapshot completi, per test ripetibili e per partire da un punto preciso | g1 §3 |
| 22 | Statistiche | VC (dashboard, warm-up), Plant Simulation, FlexSim | KPI e grafici legati alle proprietà | statistiche come stream di eventi esportabili (CSV, Parquet) | 01 SIM |
| 23 | Separazione tra modello e vista | realvirtual (Drive/DriveBehavior), ISG | comportamento eseguibile headless, la vista 3D è un client | stesso principio, con la vista web o desktop come client | g1 §5, §6 |

## 3. Vuoti comuni dei prodotti commerciali, cioè dove un progetto aperto può vincere

1. **L'emulazione PLC è un add-on a pagamento o un'edizione Premium**: FlexSim Emulation, i connettori VC
   fuori da OPC UA e MQTT, Plant Simulation (g2; 00 §4).
2. **Lockstep col PLC raro**: c'è solo in SIMIT, ISG ed Emulate3D "synchronized". VC non ce l'ha per i PLC.
3. **Determinismo non garantito.** AnyLogic sugli eventi simultanei; PhysX in VC 5.1, corretto in 5.1.1.
4. **Solo Windows** per FlexSim, Factory I/O e CIROS; formati e linguaggi proprietari (SimTalk,
   FlexScript, `.vcmx`).
5. **Limiti artificiali**: Factory I/O ha circa 256 DI/DO per driver e nessun import CAD; la PLE di AnyLogic
   si ferma a 5 ore di tempo modello; Essentials di Plant Simulation a 500 oggetti.
6. **Prezzi non pubblici o abbonamenti** per quasi tutti; fa eccezione realvirtual (Starter gratis,
   Professional 1.250 € perpetua, g1 §5).

## 4. Open source riusabile (licenza da rispettare)

Se il progetto è MIT/Apache-2.0 (da decidere):
- vanno bene come dipendenze le librerie con licenza **permissiva**: MuJoCo, JaamSim, SimPy, pymodbus,
  python-snap7, Chrono, Gazebo;
- le **LGPL** (asyncua, FreeCAD) sono utilizzabili solo come librerie collegate, senza copiarne il codice;
- le **GPL** (OpenPLC) solo come processo esterno, per esempio come soft-PLC di test.

Dettagli in [g3 §4](benchmark/g3-didattici-opensource.md). Diverse licenze lì sono marcate
[da verificare].

## 5. Decisioni aperte (dell'utente)

1. **Primo pubblico.** Le possibilità:
   - messa in servizio virtuale industriale (VC, Emulate3D, SIMIT);
   - flusso e logistica (Plant Simulation, FlexSim);
   - didattica (Factory I/O).

   Il modello a componenti è comune a tutte; cambia la prima libreria e la prima interfaccia.
2. **Piattaforma:** desktop nativo, web (WASM e WebGPU), oppure motore headless più un client web.
3. **Linguaggio del nucleo**, e se valutare nexosim (Rust) come base del DES (05 §6).
4. **Licenza del progetto**: MIT/Apache-2.0, come proposto, oppure copyleft.
