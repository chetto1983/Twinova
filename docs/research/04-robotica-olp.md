# Mappa funzionale: Robot Programming e OLP (Visual Components 5.1)

Ambito: tutte le 146 pagine `English_Robot_Programming_*` della guida ufficiale (vedi [indice](00-indice.md) §2).
Clean-room: descrizioni scritte con parole nostre, nessuna frase o immagine copiata. I nomi di
proprietà/comandi sono riportati solo come identificatori funzionali.
Prefisso dei file sorgente omesso nelle tabelle: `RP_` = `English_Robot_Programming_`,
`OLP_` = `English_Robot_Programming_OLP_`.

---

## 1. Concetti

### 1.1 Cosa è un "robot" nel modello
- Un componente è trattato come robot quando ha tre comportamenti: un **controller robot**, un
  **executor** (interprete del programma) e un comportamento di **cinematica** (es. cinematica
  scritta in Python). Senza Jog attivo, Select/Move trattano il robot come componente qualunque
  (`RP_Other_Operations_Select_a_robot_and_its_data.txt`).
- Esistono due generazioni di controller: il **Robot Controller** classico (linguaggio a statement
  "RSL") e il nuovo **Motion Controller** introdotto in 5.0, con un proprio Program Editor. La guida
  locale ha **solo il titolo** della sezione Motion Controller (nessuna pagina di contenuto, salvo il
  Program manager) (`RP_Program_Editor.txt`, `toc_premium.txt` righe 391-392).
- Quando un robot ha un executor vecchio, l'editor propone di aggiornarlo ("upgrade executor") prima
  di creare statement (`RP_Program_Create_statements_in_a_robot_program.txt`).

### 1.2 Modello del programma
- **Programma** = il "job" del robot, salvato insieme al robot dentro il layout
  (`RP_Program_Editor.txt`, `RP_Program_Teaching_a_robot.txt`).
- **Routine**: ogni programma ha una routine **Main** obbligatoria + **subroutine** (chiamate
  "Sequence" in OLP). Le subroutine vengono eseguite solo se chiamate (da Main o da altra subroutine)
  con *Call Sequence*. Main non è richiamabile come subroutine. Le routine si possono aggiungere,
  duplicare, cancellare, rinominare (`RP_Program_Create_subroutines_in_robot_program.txt`,
  `RP_Robot_Statements_Call_Sequence.txt`, `OLP_Sub-routines_and_statement_search.txt`).
- **Statement**: comando eseguito in sequenza; alcuni sono contenitori con *scope* annidati
  (If/Then/ElseIf/Else, While, Switch/Case, Path, Statement Group, Conveyor Sync). Riordino con
  drag & drop, con indicatore del livello di annidamento. Durante la simulazione una freccia indica lo
  statement in esecuzione (`RP_Program_Editor.txt`).
- **Variabili** (3 livelli) (`RP_Robot_Statements_Assign_Robot_Variable.txt`,
  `RP_Program_Using_routine_variables.txt`):
  - *di routine*: locali, tipi Boolean/Integer/Real/String, tipo non modificabile dopo la creazione,
    tornano al valore iniziale al reset della simulazione;
  - *di programma*: globali, create via API sull'oggetto programma;
  - *proprietà del componente*: globali; quelle in gruppo si referenziano come `Gruppo::Proprietà`.
- **Espressioni**: usate in condizioni e assegnazioni; possono riferire variabili della stessa
  routine, porte I/O del robot (`IN[n]`) e proprietà del componente; editor con autocompletamento
  (Ctrl+Space). La grammatica completa è in `English_Expressions_Robot_Statements` (fuori ambito).
- **Program manager** (Motion Controller): più programmi per robot; crea, copia, importa/esporta su
  file, ordina, cancella; copia robot→layout e layout→robot, usato anche per trasferire programmi fra
  robot (`RP_Program_manager.txt`).
- **Import/Export programma** dal Program Editor (`RP_Program_Editor.txt`).

### 1.3 Posizioni, frame, tool e base
- **Posizione** = matrice (XYZ + Rx,Ry,Rz) con nome, riferita a una **base** e a un **tool**;
  per PTP anche la **configurazione** (`RP_Robot_Statements_Point-to-Point_Motion.txt`).
- **Base frame**: sistema di riferimento fisso (di norma nel Robot World Frame, alla base o "pancia"
  del robot) ma attaccabile a nodi di altri componenti (pallet, pezzo, posizionatore esterno). Le
  posizioni sono figlie della base: spostando la base si spostano tutte (`RP_Jog_panel.txt`).
- **Tool frame** = TCP; di norma sulla flangia o sulla punta dell'utensile montato. Può stare in un
  altro componente fermo = **TCP esterno** (stationary tool): ruoli invertiti, il tool fa da base e la
  base da TCP.
- **InterpolationMode / IPOMode** del frame: `#NONE`, `#BASE`, `#TCP` – dice se il frame funge da
  base o da TCP e quindi se si è in configurazione a TCP esterno; conflitto se base e tool della
  coppia hanno lo stesso modo (`RP_Robot_Statements_Define_Base.txt`,
  `RP_Other_Operations_Edit_base_and_tool_frames.txt`).
- **ExternalTCP** sugli statement di moto: False = interpolazione base→tool, True = tool→base.
- Modifica frame: Move/Snap/manipolatore, attacco a nodo, cambio modalità di interpolazione.
- **Approach Axis**: asse del tool che viene allineato alla normale della superficie quando si fa
  snap (tipicamente Z+ verso il basso). In OLP anche **Orientation axis**
  (`RP_Program_Teaching_positions_using_snap_options.txt`, `OLP_Via_path.txt`).
- **Lock Positions**: globale; *To World* fissa i target al mondo (non seguono robot/base se questi
  vengono spostati), *To Reference* li lega di nuovo al loro parent
  (`RP_Other_Operations_Lock_robot_positions.txt`).

### 1.4 Configurazioni, turn e assi esterni
- **Configurazione**: nome dipendente dalla cinematica/marca (es. 0 robot a pavimento, 1 a soffitto;
  in OLP indici 0…7) (`OLP_Path_tab.txt`, `OLP_Legacy_Solver_parameters.txt`).
- **Turn** di J4/J6 (anche J1 di default): numero di giri ±360°; tool *Set Turn*, colonne Turn J4/J6
  in Point control, default turn per cablaggio (`OLP_Set_Turn.txt`, `OLP_Setup_tab.txt`,
  `OLP_Point_control.txt`).
- **Assi esterni**: posizionatori pezzo, track/gantry ("robot positioner"), 7° asse del braccio.
  Si collegano con interfacce astratte (wire dalla porta "import joints" del robot al nodo della
  cinematica esterna) (`RP_Other_Operations_Connecting_external_axes.txt`). Nel Jog panel compaiono
  come giunti aggiuntivi. Ordine convenzionale: prima robot positioner, poi workpiece positioner
  (`OLP_Point_control.txt`, `OLP_Add_Process_Statement.txt`).
- Assi non sincroni (servo controllati da un altro controller) → statement *External axis motion*
  chiamato con *Macro call* (`OLP_External_axis_motion.txt`).

### 1.5 Tipi di moto
- **PTP / Joint Motion**: interpolazione dei valori giunto; parametri JointSpeed (% della nominale) e
  JointForce (% di acc/dec).
- **LIN**: percorso cartesiano più breve mantenendo la configurazione corrente; velocità, acc, dec
  cartesiane e angolari separate; 4 modi di interpolazione dell'orientamento; 3 modi di gestione
  configurazione.
- **CycleTime** su entrambi: se >0 il moto viene stirato per durare esattamente quel tempo.
- **Accuracy** (zona di approssimazione): metodo + valore (es. ABB Fine/Cont, Yaskawa P1…P8 in OLP)
  (`OLP_Path.txt`).
- **Circolare**: in OLP i punti lineari si possono convertire in punti circolari (Set circular,
  curva per 3 punti) (`OLP_Point_control.txt`).
- **Path statement**: insieme di posizioni LIN continue generate da curve/spigoli.
- In OLP i punti di un percorso hanno un **Kind** (Via, Approach, Near, Start/process, End, Away,
  Departure, ViaDeparture) con distanze e tipi di moto default per tipo (`OLP_Path_tab.txt`,
  `OLP_Path.txt`, `OLP_Spot_path_Importer_options.txt`).

### 1.6 Jogging e teaching
- Comando **Jog**: seleziona il controller, mostra il **manipolatore** sul TCP attivo (frecce grandi =
  sistema di coordinate scelto nel pannello; frecce piccole = sistema tool). Il **toro magenta** al
  centro serve per trascinare con snap (`RP_Jog_panel.txt`).
- **Jog panel**: nome robot, XYZ + orientamento TCP, Base/Tool/Coordinate attivi, Configuration,
  ExternalTCP, giunti (slider con limiti soft, min/max raggiunti nello stato corrente, incremento),
  giunti esterni, opzioni di snap.
- **Snap targets**: Edge, Face, Edge&Face, Frame, Origin, Bound (vertice bounding box), Bisector;
  opzione Set Orientation (allinea Approach Axis alla normale) o mantiene orientamento.
- **Align**: allinea il TCP (o un pezzo afferrato) come si allinea un componente.
- **Teach con valori giunto**: si editano i giunti, poi si crea PTP/PTPHome.
- **Fast Robot Teaching / Teach Overlay menu**: azioni rapide in 3D (PTP, LIN, Path, Halt…); ripetere
  la stessa azione clona i parametri dello statement precedente; menu configurabile via file di
  configurazione o API Python/.NET (`RP_Program_Fast_Robot_Teaching.txt`).
- **Touch-up**: aggiorna lo statement selezionato con la posa corrente del robot.
- Selezionare una posizione nel 3D o nell'editor porta il robot su quella posa; il comando Select
  evita il salto (`RP_Program_Select_statements_in_a_robot_program.txt`).

### 1.7 I/O e azioni su segnale
- Porte di ingresso/uscita numerate; collegamento grafico a segnali di altri componenti
  (Input[+]/Output[+]), cambio porta senza scollegare
  (`RP_Other_Operations_Connecting_signals_to_robot_inputs_and_outputs.txt`).
- **Signal Actions** (mappatura uscita → azione On True / On False). Mappatura di fabbrica tipica:
  1-16 grasp/release, 17-32 trace on/off, 33-48 mount/unmount tool (su tool frame), 49-80 su base
  frame, 81 swept volume; dal 100 in su liberi
  (`RP_Program_Mapping_signals_to_actions_in_robot.txt`, `RP_Program_Sweeping_geometry.txt`).
- Grasp con **volume di rilevamento** (box nel sistema del tool), Release con ricerca di un
  contenitore sotto il pezzo lungo la gravità, Mount con interfaccia compatibile.

### 1.8 Verifiche: raggiungibilità, limiti, collisioni
- Punti/posizioni non raggiungibili evidenziati in rosso; trascinando fuori portata appare una
  freccia rossa verso il TCP (`RP_Jog_panel.txt`).
- Path: segmenti verdi raggiungibili, gradiente verde→rosso non raggiungibili, punti di riferimento
  magenta (`RP_Program_Teaching_a_path_of_positions.txt`).
- **Limiti** (gruppo Limits): singolarità, accelerazione, velocità dei giunti; evidenziazione rossa,
  stop della simulazione, messaggi nel pannello Output
  (`RP_Other_Operations_Limiting_a_robot_and_its_joints.txt`).
- **Collisioni**: detector A vs B, default "Selection vs World"; funziona anche senza simulazione
  (durante il teaching); solo mesh, non point cloud; colore giallo sui nodi in collisione; la
  velocità di simulazione influenza la frequenza di rilevamento
  (`RP_Other_Operations_Detecting_collisions.txt`, `RP_Other_Operations_Collision_Detectors.txt`).
- OLP: **Path check** interpola anche fra statement e mostra due strisce di stato (cinematica /
  collisioni) con 7 colori; **Program validation** per più robot senza simulare.

### 1.9 Flusso OLP (prodotti Premium/Professional/Robotics OLP)
1. **Cell setup** (Setup tab: componenti attivi, robot come WPP, multi-robot sync, tool/base/config
   default, dimensioni torcia per touch sensing, detector).
2. **Creazione percorsi di processo** da CAD: saldatura ad arco (topology analysis sul cordone, free
   path, manual path, multipass, stitch), spot, taglio/foratura (cut path, drill, DXF→path,
   text→path), verniciatura/spray e finitura superfici (free path, zigzag, spiral, slice, recipe).
   Import da file CAM/standard (IFC, XML, CREO 9, Wisconsin, QIF, CSV spot).
3. **Path setup** per ogni statement di processo: tab DB, WPS, Path, Adjust, Aux, Corner, Search,
   Stitch, Test path, Point control, Tool solver.
4. **Ricerche (touch sensing)**: manuali o Auto Search; tipi 1D/2D/3D/circolari; WireSense.
5. **Sequenziamento**, clonazione, template (feature/group/program), riordino e rinomina.
6. **Verifica**: Path check, Program status, Program validation, tracce 3D per stato.
7. **Risoluzione automatica** (add-on Robotics Solver): Path Solver per processo/ricerca/via; Tool
   solver; Via solver per moti in aria senza collisioni.
8. **Post-processing**: Postprocess → Download sotto la marca → file da trasferire al controller;
   Download settings per marca; System routines escluse dalla traduzione; Robot Language e commenti
   verbatim passati come codice; FTP per il trasferimento.

### 1.10 Marche / linguaggi citati
- Marche nominate esplicitamente: **ABB, Fanuc, Yaskawa, OTC, Panasonic** (Circular Search);
  **Fanuc, Yaskawa** (WireSense, Sync base); **ABB** (quaternioni), **IGM** (Euler) in Convert angles;
  **Fronius** (generatore di saldatura, non robot).
- Esempi di sintassi controller citati: ABB variabili `pose…`, Fanuc registri `PR[n]`/`R[n]` e
  routine utility, Yaskawa funzioni utility (es. per RB1).
- La guida **non elenca** i post-processor disponibili né i nomi dei linguaggi (RAPID, KRL, INFORM,
  TP…): dice solo "Download sotto il nome della marca" (`OLP_Translate_robot_program.txt`).
- Applicazioni/preset citati: ARC, CUT, PAINT, SURF-X (`OLP_Hover_Panel.txt`,
  `OLP_Create_a_Free_Path.txt`).

---

## 2. Tabelle

### 2.1 Program Editor, teaching e operazioni generali (RP)

| ID | Nome | Cosa fa (parole nostre) | Parametri chiave | Fonte (.txt) |
|---|---|---|---|---|
| RP-01 | Program Editor (classico) | Albero routine/statement di un robot, toolbar per inserire statement RSL, anteprima moti, indicatore di esecuzione | Main, subroutine, aggiungi/duplica/cancella routine, import/export | RP_Program_Editor |
| RP-02 | Program manager | Gestione di più programmi per robot (Motion Controller) | crea, copia, import/export file, ordina, cancella, copia robot↔layout | RP_Program_manager |
| RP-03 | Jog command + Jog panel | Manipolazione/teach del robot con manipolatore sul TCP e pannello giunti | Base, Tool, Coordinates, Configuration, ExternalTCP, giunti + esterni, limiti, incremento, snap | RP_Jog_panel |
| RP-04 | Teaching a robot | Vista Robot nel tab PROGRAM, Jog per selezionare il controller e aprire il programma | — | RP_Program_Teaching_a_robot |
| RP-05 | Fast Robot Teaching | Menu overlay in 3D che crea statement con un clic; ripetizione clona i parametri | lista azioni configurabile (file config, Python, .NET) | RP_Program_Fast_Robot_Teaching |
| RP-06 | Edit robot positions | Modifica posizioni con Move (solo posa) o Jog (posa + giunti), snap, poi Touch Up | Move/Jog, sistemi Object/Parent | RP_Program_Edit_robot_positions |
| RP-07 | Create statements | Inserimento statement dalla toolbar sulla posa corrente; upgrade executor se necessario | tipo statement | RP_Program_Create_statements_in_a_robot_program |
| RP-08 | Select statements | Selezione diretta o annidata; posizioni selezionabili in 3D; Select evita il salto del robot | — | RP_Program_Select_statements_in_a_robot_program |
| RP-09 | Edit statements | Modifica proprietà, copia/incolla, cancella | pannello Properties | RP_Program_Edit_statements_in_a_robot_program |
| RP-10 | Create subroutines | Aggiunta subroutine con "+" | — | RP_Program_Create_subroutines_in_robot_program |
| RP-11 | Call subroutines | Chiamata subroutine da Main o da altra subroutine | Routine | RP_Program_Call_subroutines_in_robot_program |
| RP-12 | Teach con snap | Trascinamento del toro con snap a geometrie; feedback su tipo superficie e portata | Edge, Face, Edge&Face, Frame, Origin, Bound, Bisector; Set Orientation; Approach Axis | RP_Program_Teaching_positions_using_snap_options |
| RP-13 | Teach con align | Allineamento del TCP o del pezzo afferrato | Base, Tool, Coordinates | RP_Program_Teaching_positions_using_align_options |
| RP-14 | Teach con valori giunto | Imposta i giunti (anche esterni) via slider e crea PTP/PTPHome | Configuration, ExternalTCP, giunti | RP_Program_Teaching_positions_using_joint_values |
| RP-15 | Signal actions | Mappa uscite robot ad azioni (grasp/release, trace, mount/unmount) | Output, OnTrue, OnFalse, tool frame, volume di rilevamento, interfaccia, GravityDirection, attach a mondo/nodo | RP_Program_Mapping_signals_to_actions_in_robot |
| RP-16 | Detection volume (visualizzazione) | Mostra il box/linea di rilevamento del TCP per il grasp; solo visivo, nascosto in simulazione | ShowDetectionVolume, DetectionVolumeGeometry (Box/Line), DetectionVolumeTool, dimensioni XYZ mm | RP_Program_Visualizing_the_tool_detection_volume |
| RP-17 | Visualizza percorsi di moto | Linee di collegamento fra target e tracce durante la simulazione (accese da uscita) | Connecting Lines, Traces | RP_Program_Visualizing_robot_motion_paths |
| RP-18 | Routine variables | Variabili locali tipizzate, editing nome/valore, uso in condizioni | tipo Bool/Int/Real/String, Name, Value | RP_Program_Using_routine_variables |
| RP-19 | Teaching a path of positions | Genera posizioni LIN lungo spigoli/curve con task pane Select Curve; edit, riferimenti, smoothing, offset ugello | Pick Edge/Curve, Auto Expand, Curve Offset, Point Density (deviazione cordale/angolare, Max Distance), Motion Parameters (Approach Axis, Align To, Reverse), Lock Axis, IsReference, Nozzle Offset, Rotation Handles | RP_Program_Teaching_a_path_of_positions |
| RP-20 | Sweeping geometry | Volume spazzato (inviluppo convesso) generato su comando di uscita | SweptMethod (Lofting, Hull from Hulls, Hull from Silhouettes), SweptGeometry, Material, AttachTo (Node/Closest/New Component), StoreGeometry | RP_Program_Sweeping_geometry |
| RP-21 | Touch-up | Aggiorna lo statement di moto selezionato con la posa corrente | — | RP_Program_Editor |
| RP-22 | Collision Detectors panel | Crea/abilita/rimuove detector A-vs-B | First/All, Collision Tolerance, Show Minimum Distance, Ignore Closest Nodes, Ignore Different Components, Use Colliders, materiale in collisione, include/escludi nodi | RP_Other_Operations_Collision_Detectors |
| RP-23 | Detecting collisions | Rilevamento collisioni anche in teach; Selection vs World; detector salvati nel layout | Enable Detectors, liste A (verde) / B (blu) | RP_Other_Operations_Detecting_collisions |
| RP-24 | Connecting external axes | Collega cinematiche esterne al robot via interfaccia grafica (wire) | porta import joints | RP_Other_Operations_Connecting_external_axes |
| RP-25 | Connecting signals to robot I/O | Cablaggio segnali come input/output con numerazione automatica | Input[+], Output[+] | RP_Other_Operations_Connecting_signals_to_robot_inputs_and_outputs |
| RP-26 | Edit base and tool frames | Sposta/snappa/attacca frame, ne imposta il modo di interpolazione | Node, InterpolationMode | RP_Other_Operations_Edit_base_and_tool_frames |
| RP-27 | Exchange robots | Scambia posizione e programma fra due robot compatibili | — | RP_Other_Operations_Exchanging_robots |
| RP-28 | Limits | Controllo singolarità, velocità, accelerazione giunti con evidenziazione/stop/messaggi | Singularity, Acceleration, Speed, Color Highlight, Stop at limits, Message Panel Output | RP_Other_Operations_Limiting_a_robot_and_its_joints |
| RP-29 | Lock robot positions | Blocca i target al mondo o li riaggancia al riferimento | To World, To Reference | RP_Other_Operations_Lock_robot_positions |
| RP-30 | Paint add-on | Preparazione mesh, mappe colore spessore, misura spessore, rimozione, pistole di verniciatura | Max Edge Length, color map (Color/Thickness), Refresh Rate, Spray Min/Max Distance, Gun Efficiency, Spray Distance, profili X/Y, Mean Offset, Flow Rate, Spray Diameter, Remove on Reset | RP_Other_Operations_Painting |
| RP-31 | Select robot and data | Riconoscimento robot, selezione base/tool correnti | socket Base/Tool | RP_Other_Operations_Select_a_robot_and_its_data |
| RP-32 | Via solver (base) | Inserisce PTP/LIN intermedi per evitare collisioni fra due statement | Solution time, Smooth time, Interpolation step, Always add path, PTP/LIN/Near speed, PTP only, preset | RP_Other_Operations_Via_solver |

### 2.2 Statement RSL del controller classico (RP-STM)

| ID | Nome | Cosa fa (parole nostre) | Parametri chiave | Fonte (.txt) |
|---|---|---|---|---|
| RP-STM-01 | Assign Variable | Scrive in una variabile il risultato di un'espressione | TargetProperty, ValueExpression | RP_Robot_Statements_Assign_Robot_Variable |
| RP-STM-02 | Break | Esce subito dal While | — | RP_Robot_Statements_Break |
| RP-STM-03 | Call Sequence | Esegue una subroutine (non Main) | Routine | RP_Robot_Statements_Call_Sequence |
| RP-STM-04 | Comment | Riga non eseguibile | Comment | RP_Robot_Statements_Comment |
| RP-STM-05 | Continue | Salta alla prossima iterazione del While | — | RP_Robot_Statements_Continue |
| RP-STM-06 | Define Base | Ridefinisce a runtime un base frame del proprio robot (ripristino al reset); tipico per pallettizzazione | Base, IsRelative, Position, IPOMode (#NONE/#BASE/#TCP), Node | RP_Robot_Statements_Define_Base |
| RP-STM-07 | Define Tool | Ridefinisce a runtime un tool frame (cambio utensile, TCP esterno) | Tool, IsRelative, Position, IPOMode, Node | RP_Robot_Statements_Define_Tool |
| RP-STM-08 | Delay | Attesa a tempo | Delay [s] | RP_Robot_Statements_Delay |
| RP-STM-09 | Halt | Ferma (ed eventualmente resetta) la simulazione | HaltSimulation, ResetSimulation | RP_Robot_Statements_Halt |
| RP-STM-10 | If | Ramo Then / ElseIf multipli / Else secondo condizione | Condition (variabili, IN[n], proprietà) | RP_Robot_Statements_If |
| RP-STM-11 | Linear Motion (LIN) | Moto cartesiano rettilineo con config corrente | Position, Name, Base, Tool, ExternalTCP, MaxSpeed, Acceleration, Deceleration, CycleTime, AccuracyMethod/Value, OrientationInterpolationMode (One/Two/Three Axis, Wrist), ConfigurationMode (Fixed, Follow frame, Interpolate joint values), MaxAngularSpeed, AngularAcceleration, AngularDeceleration | RP_Robot_Statements_Linear_Motion |
| RP-STM-12 | Path | Contenitore di posizioni LIN continue generate da geometria; Name/Base/Tool condivisi; non accetta altri statement annidati | vedi RP-19 | RP_Robot_Statements_Path |
| RP-STM-13 | Point-to-Point Motion (PTP / Joint Motion; anche PTPHome) | Moto in spazio giunti | Position, Name, Configuration, Base, Tool, ExternalTCP, CycleTime, JointSpeed %, JointForce %, AccuracyMethod/Value | RP_Robot_Statements_Point-to-Point_Motion |
| RP-STM-14 | Print | Messaggio nel pannello Output | Message | RP_Robot_Statements_Print |
| RP-STM-15 | Program Synchronize | Rendez-vous fra programmi di più componenti tramite messaggio | SyncMessage, WaitSync, SyncComponents | RP_Robot_Statements_Program_Synchronize |
| RP-STM-16 | Return | Termina la routine corrente | — | RP_Robot_Statements_Return |
| RP-STM-17 | Set Binary Output | Imposta un'uscita digitale (anche per azioni su segnale) | OutputPort, OutputValue | RP_Robot_Statements_Set_Binary_Output |
| RP-STM-18 | Set Statistics | Cambia lo stato statistico del robot (es. busy/idle) | Statistics, State | RP_Robot_Statements_Set_Statistics |
| RP-STM-19 | Switch Case | Esegue il primo case la cui espressione è vera; case "default" | Condition, CaseCondition | RP_Robot_Statements_Switch |
| RP-STM-20 | Wait for Binary Input | Attende un valore su un ingresso, ciclicamente o su evento (evento perso se arriva prima) | InputPort, InputValue, WaitTrigger | RP_Robot_Statements_Wait_for_Binary_Input |
| RP-STM-21 | While | Ciclo finché la condizione è vera | Condition | RP_Robot_Statements_While |
| RP-STM-22 | Toggle statement | Disabilita uno statement: escluso da simulazione, path check e post-processing | on/off | OLP_Enabling_and_disabling_statements |

### 2.3 OLP – statement, tool e impostazioni (OLP)

| ID | Nome | Cosa fa (parole nostre) | Parametri chiave | Fonte (.txt) |
|---|---|---|---|---|
| OLP-01 | Hover Panel | Barra flottante in 3D con gli strumenti più usati, configurabile per applicazione | Preset (ARC, CUT, PAINT…), Action list, spacer, "+" | OLP_Hover_Panel |
| OLP-02 | Path Visibility | Filtra quali percorsi sono disegnati in 3D | ALL, Sub, Sel, Scope | OLP_Path_Visibility |
| OLP-03 | Program Status | Stato OK/Fail per statement di processo, icona colorata | Update statuses, Auto set status | OLP_Program_Status |
| OLP-04 | Visualizzazione tracce 3D per stato | Colora linee/tracce verde/rosso e annota Pass/Fail; stato impostabile a mano | Line color by status, 3D trace by status, annotations, OK/Fail/Unchecked/Custom/Clear | OLP_Visualization_of_3D_tracesLines |
| OLP-05 | Robotics Solver | Add-on: risolve automaticamente collisioni, singolarità, limiti, raggiungibilità dal Path check | pulsanti processo/ricerca/via | OLP_Robotics_solver |
| OLP-06 | Path Solver | Finestra del solver con preset, scope e override; report risultati | Process/Search/Via tab, preset, scope | OLP_Path_Solver |
| OLP-07 | Solver parameters (nuovo) | Preset e gruppi di preset con vincoli e offset ammessi/ottimali | Solution/Optimize time, Interpolation step, Tool, Config, Polar/Wrist singularity, Via Clearance, Stretch, External joints (Constant/Original/Moving/keep positioner mode), Position offsets, Approach tolerance, Pre-solve with tool, Optimal joints, Safe states | OLP_Solver_parameters |
| OLP-08 | Legacy Solver parameters | Parametri JSON del vecchio planner a campionamento e del via planner bi-albero | PathPlanSettings, SearchPlanSettings, ViaPlannerSettings, SafeStates (vedi §3) | OLP_Legacy_Solver_parameters |
| OLP-09 | Translate robot program | Post-processing: Postprocess → Download per marca → file per il controller | robot/dispositivo, cartella/nome file | OLP_Translate_robot_program |
| OLP-10 | Download settings | Opzioni di traduzione per marca su robot e posizionatori | STATION, MechanismNumber, parametri marca-specifici | OLP_Download_settings |
| OLP-11 | Sub-routines & statement search | Sequence = subroutine; ricerca testuale di statement in tutte le routine | nome, identificatori (W1, S2, LIN, PTP…) | OLP_Sub-routines_and_statement_search |
| OLP-12 | Paint tools (Surf Analysis) | Preparazione componenti, mappe colore, ispezione accumulo (cursore/linea con export JSON), pulizia, parametri di simulazione, gestione brush | Sim. rate, occlusion, ignore size, Brush Setup, Edit Statement Brush, New Nozzle, Brush Table | OLP_Paint_tools |
| OLP-13 | Test path | Navigazione e modifica punto per punto di un percorso, undo/redo, modi di applicazione dei parametri | slider, salti a primo/ultimo/inizio-fine processo, add/delete point, touch-up/undo, Full/Single/Multi mode, interpolatore S | OLP_Test_path |
| OLP-14 | DB tab | Salva/applica template di parametri di path setup | filtro, include WPP, include TRACK | OLP_DB_(Database)_tab |
| OLP-15 | WPS tab | Parametri di simulazione (cordone, weaving) e di processo (velocità, arc on/off, job) | Speed, Process on, Process off, valori lista, lock | OLP_WPS_(Welding_Procedure_Specification)_tab |
| OLP-16 | Path tab | Base/tool/config/pezzo, punti via con distanze, lead-in/out, rifinitura | Sync base, Via/Approach/Near/Away/Departure, Lead in/out, Set start/end, Repick, Reanalyze, Reverse, Multipass, Change start, Stitch, Auto smooth, Auto align | OLP_Path_tab |
| OLP-17 | Adjust tab | Angoli torcia, offset del percorso, flare a inizio/fine | Wire roll, Wire angle, Rake angle (push/pull), offset Wire/Wall tangent/Base tangent/Bisector/Base normal/Wall normal, Start/End offset, Max point distance, Auto flare, Flare angle/distance | OLP_Adjust_tab |
| OLP-18 | Aux tab | Gestione track/gantry e posizionatore pezzo, posizioni di saldatura normalizzate | Move/Avg/Lock, Mode, Azimuth, Elevation, Stretch, Link limits, Flip, PA/PB/PD-PE/PF-PG, Direction, Lock Grill, turn, Link3, Joint 7 | OLP_Aux_(auxiliary_axes)_tab |
| OLP-19 | Corner tab | Trattamento angoli interni/esterni e loop sugli spigoli (taglio) | Examine corners, Auto flare, Flare distance, Corner offset, Min offset angle, Sharp corner mode, Loop mode, Distance, Radius, Min angle | OLP_Corner_tab |
| OLP-20 | Point control | Tabella di tutti i punti di un percorso con edit multiplo | Kind, Motion, Config, Search (memory slot), Turn J4/J6, Aux, Speed, WPS, set orientation, Smooth XYZ, smooth aux, XYZ in World/Base/Part, Set circular, 3-point curve, resequence; operatori + - * / | OLP_Point_control |
| OLP-21 | Tool Solver | Regola automaticamente angoli/offset della torcia per evitare collisioni con il pezzo | Tool Collider (o tool sulla flangia) | OLP_Tool_Solver_1 |
| OLP-22 | Program Import/Export | Riusa programmi fra layout con compensazioni | Workpiece, Sequence, ExtMap, MotionTransMat, PathTransMat, ExtTrans | OLP_Program_ImportExport |
| OLP-23 | Feature template | Riusa saldature su parti ripetute riconosciute per nome geometria; scala/orienta | Set feature, Export/Import, Sequence, Scale feature, Fix orientation, Auto-select, Family table, Name split symbol | OLP_Feature_template |
| OLP-24 | Group template | Esporta/importa il contenuto di un Statement Group con origine | Set origin, Export, Import, Description | OLP_Group_template |
| OLP-25 | Import tools (CAD to path) | Genera saldature da file CAM/standard | IFC, XML, CREO 9 XML, Wisconsin, QIF | OLP_Import_tools |
| OLP-26 | WPS Rule Manager | Crea e mantiene regole WPS | — (pagina sintetica) | OLP_WPS_Rule_Manager |
| OLP-27 | WPS Rule Selector | Trova e applica regole WPS alle saldature con filtri | scope Selected/Sequence/Program, filtri spessore, a-value, angolo, posizione, tipo; Apply All/Selected | OLP_WPS_Rule_Selector |
| OLP-28 | Common tab (Robot settings) | Impostazioni generali OLP | Hide handle, Auto check points (raggiungibilità, singolarità, limiti), cartella programmi, Process speed override, controllo velocità simulazione per tipo moto, aspetto torcia | OLP_Common_tab |
| OLP-29 | Path (Robot settings) | Default di processo, ricerca, velocità/tipo moto/accuracy per tipo di punto | Simulate weaving, 3D trace, WPS file, Proc On/Off template, Multipass file, Sparks, Custom search, Speeds, Motion types, Accuracy values | OLP_Path |
| OLP-30 | Auto-Search Advanced Settings | Parametri JSON di generazione automatica ricerche | SaveCandidates, CreateVias, ViasToCreate, ViasDistances, AssignSearchVariableToKinds, SearchVariableString, Start/Mid/EndSearch, CastPattern X/Y/Z/Step/Safety, DistanceToPreviousSearchFilter, HitAngleFilter, set per tipo (Wire2D…ExtCamera) | OLP_Auto-Search_Advanced_Settings |
| OLP-31 | Setup tab | Configurazione cella e torcia | Robot as WPP, Run setup, Multi-robot Sync, handling guide, default Config/Base/Tool/Search tool/Turn, tool index→geometria, dimensioni ugello a–e, tandem (WirePushAngle, NozzleDistance, Torch type, TCP location), approach/orientation axis, detector (+WP) | OLP_Setup_tab |
| OLP-32 | Calibration tools | Strumenti di calibrazione robot (dettagli solo su extranet) | — | OLP_Calibration_tools |
| OLP-33 | Path check | Validazione del programma interpolando anche fra statement; strisce colorate; statistiche tempo | Scope, check limiti/singolarità/collisioni, Step size, Auto set Turns & Config, Auto set status, pin, statistiche Via/Process time e lunghezza | OLP_Path_check |
| OLP-34 | Combine paths | Unisce saldature selezionate (stessa direzione) | — | OLP_Combine_paths |
| OLP-35 | Sequencing | Ordina saldature, ricerche, multipass | Scope, Inside out / Sorting only, Keep search with weld, Group by positioner, sort primario/secondario, layers/passes, Recalculate variables | OLP_Sequencing |
| OLP-36 | Manual path | Saldatura definita scegliendo parete, base, inizio, fine | — | OLP_Manual_path |
| OLP-37 | Re-Index Paths | Rinumera percorsi e ricerche secondo il flusso | — | OLP_Re-Index_Paths |
| OLP-38 | Shift Aux | Trasla i valori degli assi ausiliari in uno scope | scope | OLP_Shift_Aux |
| OLP-39 | Replace Search variable | Sostituisce le variabili di ricerca in uno scope | scope | OLP_Replace_Search_variable |
| OLP-40 | Set Turn | Imposta i turn J4/J6 in uno scope | scope | OLP_Set_Turn |
| OLP-41 | Set Point Names | Rinomina con indice progressivo i punti PTP/LIN | Root name, scope | OLP_Set_Point_Names |
| OLP-42 | Add Process Statement (LockExternalAxes) | Statement di processo che blocca alcuni assi esterni durante una routine | lista indici assi da bloccare | OLP_Add_Process_Statement |
| OLP-43 | 2D Offset Path | Crea percorsi traslati su un piano, anche multipli | piano, offset, count | OLP_2D_Path_Offset |
| OLP-44 | Path Statement → Process path | Converte un Path RSL in percorso di processo gestibile da Path setup | parametri di conversione | OLP_Path_Statement_to_Process_path |
| OLP-45 | Program validation | Interpolazione multi-robot con slider globale e per robot, senza simulare | slider All/singoli | OLP_Program_validation |
| OLP-46 | Split path | Divide un percorso in N parti | Path count, Start/End/Middle length, Overlap, Push to start, Keep original | OLP_Split_path |
| OLP-47 | Text to path | Percorso da testo digitato con altezza data | testo, altezza | OLP_Text_to_path |
| OLP-48 | Hide features | Nasconde o inverte feature (es. cordoni già modellati) | — | OLP_Hide_features |
| OLP-49 | Add surface | Aggiunge superfici/cilindri/frame ausiliari al pezzo per migliorare il picking | dimensioni, orientamento, colore, posizione | OLP_Add_surface |
| OLP-50 | Explode Component | Separa le feature di un componente in componenti distinti | — | OLP_Explode_Component |
| OLP-51 | Create Path Geometry | Genera la geometria dei cordoni per tutto il programma senza simulare | — | OLP_Create_Path_Geometry |
| OLP-52 | Remove Path Geometry | Rimuove geometrie di saldatura | — | OLP_Remove_Path_Geometry |
| OLP-53 | Simplify geometries (Collider Wizard) | Crea collider semplificati (convesso/box/cilindro) per collisioni più veloci | nodi, tolleranza mm/% | OLP_Simplify_geometries |
| OLP-54 | Torch cloud | Crea componenti "nuvola" della torcia/giunti lungo il percorso per ispezione ed export | Scope, Reference, Nodes, Cloud mode (Single/Path/Point), Simplify | OLP_Torch_cloud |
| OLP-55 | System routines | Subroutine simulate ma non tradotte (già presenti sul controller) | Add/Remove | OLP_System_routines |
| OLP-56 | Table motion | Rotazione di una tavola esterna non sincronizzata | angolo, velocità | OLP_Table_motion |
| OLP-57 | External axis motion (statement) | Muove servo non sincroni dal robot, da subroutine di un altro componente | Motion type (Absolute/Incremental), Joint Value (numero o variabile), Motion speed | OLP_External_axis_motion |
| OLP-58 | Convert angles | Conversione quaternioni/Euler ↔ WPR | gradi; output nel pannello | OLP_Convert_angles |
| OLP-59 | FTP Export/Import | Connessione FTP per trasferire file | indirizzo, utente, password | OLP_FTP_Export_Import |
| OLP-60 | Fronius Job Editor | Connessione al generatore, lettura/clonazione/modifica/upload job | IP, utente, password, Clone, Delete, Upload | OLP_Fronius_Job_Editor |
| OLP-61 | Topology analysis | Riconoscimento del cordone dal punto pickato e regole di propagazione | lunghezza sx/dx, centra, Corner angle, Corners to pass, Distance from last corner, Stop to corner, Step size, Cavity, gap, Circular + sensitivity, Spline mode (distance, split angle), long steps, Step offset, base/wall, Outside edge, preset P1-P4, Auto | OLP_Topology_analysis |
| OLP-62 | Free Path | Percorso a punti pickati su superficie con riempimento, snap, smoothing, fanning, filtraggio | Process speed, Tool rotation, Max point distance, Extend, Max snap distance, smoothing pos/rot + Angle limit, Fanning ampiezza/angolo, Filtering, Template, Brush | OLP_Free_Path, OLP_Create_a_Free_Path |
| OLP-63 | Multipass weld | Genera passate multiple da una passata di radice | a (gola), α, Flip, per passata: Layer, offset Base/Wall/Bisector/Wire, Wire angle, Speed, Start/End offset, Reverse, WPS; template | OLP_Multipass_weld |
| OLP-64 | Stitch welds | Saldatura intermittente secondo ISO/AWS | Standard, Balancing (Start/Mid/End/Stretch), LS, L1, LE, L2, Via height, Max Count, Push to start, Staggered | OLP_Stitch_welds |
| OLP-65 | Search tab | Definizione ricerca a contatto (touch sensing) | Touch with (Wire/Torch/External), Memory slot, tipo 1D / 2D in-out / 3D combinazioni / Orientation, Corner oriented, Search distance, Wire roll, Wire angle V/H, rotazione, Repick | OLP_Search_tab |
| OLP-66 | Auto Search | Genera automaticamente le ricerche per una saldatura con allocazione memorie | Device, Touch type, Drive distance 1-3, Tangent distance 1-3, StartOffset 1-2, Start/Middle/End search, Min/Max memory slot, Search each point | OLP_Auto_Search |
| OLP-67 | Circular Search | Ricerche per cordoni circolari interni/esterni | 2D.CIRC[IN/OUT], 3D.CIRC[in+h/out+h], drive distance, twist angle, HeightFirst, Variant (per marca) | OLP_Circular_Search |
| OLP-68 | Fronius WireSense | Ricerca con filo tramite funzione del generatore (Fanuc, Yaskawa) | proprietà avanzata "WireSense", modo 2D, funzioni utility sul controller, registri | OLP_Fronius_WireSense |
| OLP-69 | Spot path | Punto di saldatura a resistenza pickato con topology analysis | Auto | OLP_Spot_path |
| OLP-70 | Spot path importer | Import CSV di punti spot con parole chiave | Position X/Y/Z, Robot, Sequence, Comment, WpsSpeed/Thickness/ProcessTime/Process, AdjustWireRoll/Offset, AuxGunValue1-2, AuxExternalValue1-12, PathBase/Tool/Configuration/ApproachAxis/OrientAxis, distanze Via/Approach/Near/Away/Departure | OLP_Spot_path_Importer_options |
| OLP-71 | Drill path | Percorso di foratura/levigatura dal centro di un foro | stickout, start/end offset | OLP_Create_a_Drill_Path |
| OLP-72 | DXF to Path | Converte contorni 2D importati in percorsi di processo | Distance/angle per archi, Remove longest path, Auto split routine size | OLP_DXF_to_Path |
| OLP-73 | Spray brush | Modello di spruzzo calibrato | Name, Flow Rate, Efficiency, Min Z/Max Z, Brush designer | OLP_Create_a_spray_brush |
| OLP-74 | Circular brush | Pennello circolare uniforme con forza centro/bordo interpolata | Name, Radius, Min/Max Z, Center/Edge strength | OLP_Create_a_circular_brush |
| OLP-75 | Spray Designer | Calibrazione multi-layer del modello di spruzzo da colpi di prova | Max Size, Flow Rate, Tool Speed, layer per Nozzle Distance, stroke H/V (X, Y, Peak Thickness), grafici moving/stationary | OLP_Spray_Designer |
| OLP-76 | Spray calibration (calibrated spray) | Fitting del modello su misure CSV di spessore | Type Normal/Bell, flow, speed, max width/distance, sim flow, CSV H/V, resample, shift, integration steps, iterations, sampling points, learning rate, count, efficienza vs angolo (Default/Custom) | OLP_Spray_calibration |
| OLP-77 | Using brushes in simulation | Brush sui punti dei path: simulazione accumulo + parametri applicatore nel programma tradotto | Brush Table (Name, Brush, Applicator, Sim. Brush, Sim. Applicator, References), Start/End Brush, Trigger distance | OLP_Using_Brushes_in_Simulation |
| OLP-78 | Set Brush (statement) | Accende/spegne lo spruzzo fra punti PTP/LIN | on/off | OLP_Set_Brush |
| OLP-79 | Conveyor Sync (statement) | Scope sincronizzato con un nastro; memorizza stati in simulazione e gestisce basi convogliatore nel post | Conveyor, Sensor, Workpiece, Trigger offset [mm] | OLP_Conveyor_Sync |
| OLP-80 | Zigzag Pattern | Strisce parallele su superficie con trimming, snap, estensione, smoothing | pick 2 punti + normale, offsets, Stroke Length/Width, Fit, Tool rotation, Point distance, Split at gaps, Zigzag/Zig-zig, Reverse, Continuous, feature/line trimming, snap posizione/orientamento, ray height/depth, Extend (Off/Tangent/Plane), Process speed, Single path, Workpiece, Template, Brush | OLP_Zigzag_Pattern |
| OLP-81 | Spiral Pattern | Spirale proiettata sulla superficie | centro, Width, Radial step, Point distance, Rotate, speed, Tool angle, Reverse, distanza proiezione | OLP_Spiral_Pattern |
| OLP-82 | Slice Pattern | Percorsi da intersezione superfici/piano su più facce con loop di transizione | superfici, piano, numero/offset slice, one/each stroke, angolo spigolo, offset H/V del loop, Template | OLP_Create_a_Slice_Pattern_Path |
| OLP-83 | Recipes (surface processing) | Ricetta di passi di finitura → programma generato con cambi utensile/abrasivo | Recipe DB, righe (index, surface/tool priority), definizioni superfici/utensili/parametri custom On/Off, Recipe_Source, ViaMotions, routine tooling (Switch per utensile), Generate → Recipe_Output | OLP_Create_recipes_for_surface_processing |
| OLP-84 | Assembly operations | Statement Grab/Release che raggruppano punti in aria e Assembly status | grab pick, tab Grab/Release/Process/DB, Assembly + feature corrispondente, regrab station | OLP_Assembly_operations |
| OLP-85 | Assembly status (statement) | Memorizza posizione/visibilità dei componenti per fasi di saldatura o pick&place | Set current positions, Show all, Hide feature, Status description | OLP_Assembly_status |
| OLP-86 | Cloning | Duplica statement/percorsi con trasformazioni, anche verso altro robot | Distance, Plane-to-plane, Mirror, Vector, Circle (numero, distanza, angolo), Recalculate track/positioner, Joint offsets, Shift no clone, Move path (sequence, robot, base, tool) | OLP_Cloning |
| OLP-87 | Comment (OLP) | Commento tradotto come commento o, se verbatim, come riga di codice | Verbatim | OLP_Comment |
| OLP-88 | Macro call (statement) | Esegue routine di altri componenti (multi-robot, tavole) | componente, macro | OLP_Macro_call |
| OLP-89 | Robot Language (statement) | Inserisce comandi nativi del controller da una lista salvata in nota del robot | nota RobotLang | OLP_Robot_Language_statement |
| OLP-90 | Start Marker (statement) | Fa partire la simulazione da un punto del programma | posizione | OLP_Start_Marker |
| OLP-91 | Statement Group | Cartella di statement con origine, spostabile e esportabile | nome, Set origin (3 superfici), Move origin, Move group | OLP_Statement_Group |
| OLP-92 | Via path (statement) | Raccolta di punti in aria PTP/LIN modificabili in blocco | Base, Tool, Configuration, LIN/PTP speed, Parent, Approach/Orientation axis, Reverse | OLP_Via_path |

---

## 3. Note per l'implementazione

### 3.1 Semantica del motion planner (moti singoli)
- **LIN – orientamento** (`RP_Robot_Statements_Linear_Motion.txt`, technical notes):
  - *One Axis* = SLERP su quaternione (asse di rotazione minima; per 180° si sceglie X per
    convenzione). Default.
  - *Two Axis* (X/Y/Z): prima si estrae la rotazione attorno all'asse scelto del tool, poi SLERP sul
    residuo; le due componenti sono interpolate in modo indipendente. Utile per utensili assiali
    (torce, mandrini) perché minimizza la rotazione attorno all'asse utensile.
  - *Three Axis*: scomposizione in WPR (yaw-pitch-roll) o ZYZ Euler, interpolando ogni angolo.
  - *Wrist*: interpolazione in giunti dei soli polsi e correzione degli assi principali per tenere
    la traiettoria della posizione rettilinea; risultato dipendente dalla cinematica.
- **Profili**: profilo trapezoidale (vel/acc/dec) separato per posizione e per orientamento (sulla
  rotazione più ampia); in Wrist si usano i limiti dei giunti polso. Poi **sincronizzazione** di
  tutti i profili (posizione, orientamento, assi esterni) sul più lento, rendendo uguali le durate
  di accelerazione/crociera/decelerazione.
- **CycleTime > 0**: scala il moto a una durata imposta (override velocità).
- **ConfigurationMode** su LIN: Fixed (blocca config), Follow frame (IK più vicina ai giunti
  precedenti), Interpolate joint values (config dedotta da interpolazione PTP).
- **PTP**: interpolazione giunti, velocità in % della nominale, acc/dec in % (JointForce).
- **Accuracy/zone**: metodo + valore (blending); semantica precisa non descritta → vedi Lacune.
- **Frame dinamici**: Define Base/Tool cambiano frame a runtime e si ripristinano al reset;
  posizioni figlie della base si muovono con essa (pattern pallettizzazione).

### 3.2 Generazione percorsi da geometria
- Frame del punto curva: X lungo la curva, Z normale alla superficie, Y di conseguenza; l'orientamento
  finale dipende da Approach Axis e Align To. Densità punti da deviazione cordale/angolare e distanza
  max (`RP_Program_Teaching_a_path_of_positions.txt`).
- Punti di riferimento con blending dell'orientamento fra riferimenti in base all'indice di curva,
  Lock Axis e interpolazione; "Apply Smoothing" e "Apply Nozzle Offset".
- Topology analysis: avanzamento a passi dal punto pickato lungo l'intersezione base/parete, con
  regole su angoli, gap, cavità, circolarità e spline (`OLP_Topology_analysis.txt`).
- Pattern superficie: proiezione su mesh con raggi (altezza/profondità), rimozione punti non
  proiettabili, smoothing con soglia angolare, filtraggio a deviazione max (Douglas-Peucker-like:
  ipotesi nostra).

### 3.3 Solver (dal Legacy Solver, descritto in dettaglio)
- **Process/Search path solver** (`OLP_Legacy_Solver_parameters.txt`): verifica prima il percorso
  originale; se fallisce, campionamento casuale di N candidati validi per il primo punto
  (StartCandidatesCount=20), poi per ogni punto successivo M candidati (MidCandidatesCount=20) validi
  solo se collegabili con moto valido a un candidato del punto precedente; si sceglie poi il
  cammino a costo minimo (costi su moto giunti robot/esterni, stretch, offset). Variabili libere:
  assi esterni, configurazione (0…7), offset di orientamento torcia (WireRoll, WireAngle, RakeAngle,
  StickOut, FlareAngle) entro range progressivi. Validazione a due passi di interpolazione
  (100 mm/° poi 10). Fase di ottimizzazione/smoothing finale.
- **Vincoli cinematici** riutilizzabili: wrist singularity (|J5| ≥ 1°), polar singularity (polso a
  ≥100 mm dall'asse J1), stretch J5 10-90 %, distanza dai limiti di turn J4/J6 (180+n·360),
  padding limiti, rapporti di corsa assi esterni / TCP, track kinematics (gantry segue il TCP).
- **Via path planner**: bi-RRT (alberi da start e goal, connessione dei vicini, campioni attirati
  verso l'altro albero, "leap" sugli assi esterni), poi shortcut (skip di stati) e smoothing angoli;
  **safe states** predefiniti (livello 0/1) per accelerare; tolleranze di collisione decrescenti dal
  start/goal; parametri di velocità applicati ai punti generati (PTP 50 %, LIN 500 mm/s, near
  100 mm/s). I LIN non sono supportati come estremi nel legacy (manca config/turn).
- **Nuovo solver**: preset con range "ammesso" e "ottimale" (verde/giallo/grigio), gruppi di preset
  in cascata (veloce → lento), modalità assi esterni Constant/Original/Moving (+ vincolo modalità
  posizionatore PA/PB…).
- **Tool solver**: ottimizza solo angoli/offset dell'Adjust tab contro un collider dell'utensile.

### 3.4 Verifica e stima del tempo ciclo
- Path check: interpolazione a passo fisso fra tutti gli statement, classificazione di ogni passo
  in 7 stati (OK, non verificabile, irraggiungibile, rotazione TCP > 180° in LIN, limiti giunto,
  singolarità, collisione); auto-impostazione turn/config dopo ogni LIN.
- Statistiche: tempo via vs processo, tempo totale, lunghezza processo (mm) → base per stima tempo
  ciclo offline senza simulazione completa.
- Program validation: lo stesso per più robot con timeline comune (slider).
- Collisioni: detector A/B su mesh, tolleranza (clearance), distanza minima, esclusione
  parent-child e componenti attaccati, collider semplificati.

### 3.5 Post-processing
- Architettura implicita: post-processor per marca con impostazioni di download per robot e per
  posizionatore (stazione, numero meccanismo); routine di sistema escluse; righe native (Robot
  Language, commenti verbatim); statement disabilitati esclusi; Conveyor Sync richiede una
  simulazione preventiva per avere i dati da tradurre; brush/applicatori e trigger distance tradotti
  in comandi della marca; variabili di ricerca/memory slot con tipo dipendente dalla marca.
- Conversioni d'angolo: VC usa **WPR**; servono conversioni da/verso quaternioni (ABB) ed Euler.

### 3.6 Simulazione di processo (spray/paint)
- Accumulo come coordinate di texture sulle mesh preparate (tassellazione a lunghezza spigolo max);
  modello di spruzzo da profili X/Y (gaussiane con offset medio e deviazione, o "bell"), layer a
  diverse distanze ugello con interpolazione, efficienza di trasferimento anche in funzione
  dell'angolo di incidenza, occlusione/mascheramento; spessore in µm; mappe colore a gradiente o a
  intervalli.

---

## 4. Lacune (cosa la guida locale NON dice)

1. **Program Editor per Motion Controller**: solo il titolo nel TOC; nessuna pagina su statement,
   differenze col controller classico, modello dati. Solo Program manager documentato.
2. **Grammatica delle espressioni** (operatori, funzioni): rimandata a
   `English_Expressions_Robot_Statements` (fuori da questo ambito).
3. **AccuracyMethod**: elenco dei metodi (distanza, velocità, tempo…) e semantica del blending non
   descritti.
4. **Elenco dei post-processor / linguaggi** supportati: non presente; solo marche citate per
   singole funzioni (ABB, Fanuc, Yaskawa, OTC, Panasonic; IGM per angoli).
5. **Algoritmo IK, gestione singolarità durante l'esecuzione** e semantica del "Singularity" limit:
   nessun dettaglio numerico salvo le soglie del solver (J5, distanza polare).
6. **Circolari** nel controller classico: non esiste uno statement CIRC RSL documentato; solo
   punti circolari in OLP Point control.
7. **WPS Rule Manager**: pagina di una riga, formato delle regole non descritto.
8. **Calibration tools**, **Import tools** (CAD-to-path) e advanced parameters Panasonic: rimandati
   a extranet/reseller.
9. **Costi del legacy solver**: la guida stessa li dichiara poco efficaci.
10. **Nuovo solver**: descritti solo i parametri GUI, non l'algoritmo.
11. Pagine OLP di sola intestazione o quasi: Explode Component, Create/Remove Path Geometry, Text to
    path, Re-Index, Shift Aux, Replace Search variable, Set Turn, Table motion, FTP.
12. Pagina Via solver (`RP_Other_Operations_Via_solver.txt`) troncata all'ultima voce ("PTP only").
13. Nessuna informazione su tempo ciclo "ufficiale" (stima vs simulazione), su modelli dinamici
    (coppie, payload) o su RCS/Virtual Robot Controller.
