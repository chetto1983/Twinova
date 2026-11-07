# Visual Components 5.1: mappa funzionale del Component Modeling (clean-room)

**Ambito:** tutte le pagine dell'help il cui file inizia con `English_Component_Modeling` (135 file `.txt`,
verificati contro le 136 voci "Component Modeling" del TOC `toc_premium.txt`: una voce, Routing Rule, compare
due volte nel TOC). Testo scritto con parole nostre, senza frasi o immagini copiate.

**Convenzione Fonte:** nella colonna "Fonte" il prefisso comune `English_Component_Modeling_` è omesso.
Ad esempio `Behaviors_Signals_Boolean_Signal.txt` sta per
`English_Component_Modeling_Behaviors_Signals_Boolean_Signal.txt`.

**Legenda tipi:** `bool`, `int`, `real`, `string`, `enum{...}`, `matrix` (4x4 posa: Tx,Ty,Tz,Rx,Ry,Rz),
`vector` (X,Y,Z), `ref<Behavior X>` (riferimento a un altro behavior), `ref<Frame>` (feature Frame),
`ref<Node>`, `list<...>`, `expr` (espressione valutata), `uri`, `button`. Il tipo è dedotto da quanto
descrive l'help: dove il tipo non è scritto esplicitamente lo segnaliamo con `?`.

---

## 1. Modello dati del componente

### 1.1 Gerarchia degli oggetti

```
Mondo 3D (simulation root node)
└── Component  (file .vcmx, identificato da un VCID univoco)
    └── Root node  (= origine del componente; porta le Properties del componente)
        ├── Properties[]          (variabili globali del componente, tipizzate)
        ├── Behaviors[]           (logica/azioni; possono avere sotto-elementi: porte, Bases, Tools, colliders, pin)
        ├── Feature tree          (root feature -> feature figlie, valutate bottom-up)
        └── Link (child node)[]   (ogni link = nodo con Offset + Joint)
            ├── Behaviors[]
            ├── Feature tree
            └── Link[] ...        (catena cinematica)
```

- **Component.** Oggetto che occupa spazio nel mondo 3D, con un **VCID** univoco. È un archivio
  `.vcmx` con almeno 4 file: `component.rsc` (dati per ricostruire il componente), `component.dat`
  (metadati, VCID compreso), un'immagine di anteprima e un'icona per il catalogo (entrambe `.tga`).
  (`Components.txt`)
- **Node / Link.** Il contenitore base. Ogni componente ha almeno il nodo radice, e ogni nodo agganciato
  a un altro nodo si chiama "link". Gerarchia parent/child/sibling. La radice del componente è di
  norma figlia del nodo radice della simulazione, ma può essere figlia di un nodo di un altro
  componente: per esempio un utensile montato sulla flangia di un robot si muove con essa. Durante la
  simulazione alcuni behavior creano **nodi temporanei** per contenere e trasportare componenti: i pezzi
  su un nastro, per esempio, sono attaccati temporaneamente al nodo del path. (`Nodes.txt`,
  `Behaviors_Sensors_Volume_Sensor.txt`)
- **Property.** Variabile globale del componente, sempre contenuta nel nodo radice. Le si può riferire
  per nome nelle espressioni (offset dei nodi, dimensioni delle feature, scelte dei Switch), il che
  rende il componente **parametrico**. Anche le proprietà dei behavior sono raggiungibili con
  `<NomeBehavior>::<NomeProprietà>`. (`Properties.txt`, `Component_Graph_panel.txt`)
- **Behavior.** Azione o compito che il componente esegue prima o durante la simulazione. Sta in un
  nodo, ma può riferire o connettere behavior di altri nodi dello stesso componente. Verso altri
  componenti si connette **solo tramite interfacce**. (`Component_Graph_panel.txt`)
- **Feature.** Rappresentazione della geometria e di altri dati (frame, testo). Ogni nodo ha un albero
  di feature con in cima una root feature. (`Component_Graph_panel.txt`)

### 1.2 Albero delle feature e sua valutazione

- La valutazione va **dal basso verso l'alto**: ogni feature viene valutata e il risultato passa al
  padre, fino alla root feature del nodo.
- Una feature di tipo **operazione** (Transform, Clone, Extrude, Revolve, Mirror, Switch, Boolean,
  Python Feature) prima valuta le proprie sottofeature, poi applica la sua trasformazione o operazione
  e infine consegna il risultato al proprio padre.
- **Primitive** (Block, Cylinder, Cone, Sphere, Wedge, Plane) contro **Geometry** (contenitore di mesh
  importate o "collassate"). Qualunque feature si può collassare in una Geometry, e la mesh così
  ottenuta diventa editabile.
- La geometria è organizzata in **geometry set** (insiemi di facce, linee e punti) dentro le feature.
- Il "Rebuild" di una property rivaluta le espressioni e rigenera la geometria.

(`Component_Graph_panel.txt`, `Features_*.txt`, `Modeling_Operations_Creating_a_new_feature.txt`)

### 1.3 Giunti, gradi di libertà e cinematica

**Ogni link (esclusa la radice) ha:**
- `Name`;
- `Offset` (matrix rispetto al padre: XYZ in mm, RxRyRz in gradi);
- `Type` (tipo di giunto);
- `Axis`.

Il "rig" (offset più DOF) si visualizza come scheletro, ed è il comando Interact a muovere i giunti.

**Tipi di giunto** (`Nodes.txt`):

| ID | Tipo giunto | Semantica |
|---|---|---|
| CM-JNT-01 | Fixed | Nessun DOF: il nodo resta vincolato al suo offset. |
| CM-JNT-02 | Rotational | 1 DOF di rotazione attorno a un asse definito. |
| CM-JNT-03 | Translational | 1 DOF di traslazione lungo un asse definito. |
| CM-JNT-04 | Rotational Follower | Rotazione derivata dal valore di un altro giunto (`Driver`) tramite `Value Expression` (giunti passivi, accoppiamenti). |
| CM-JNT-05 | Translational Follower | Come sopra, ma in traslazione. |
| CM-JNT-06 | Custom | Il movimento è dato da espressioni libere: `Joint` (espressione che trasforma l'offset) e `Pivot` (espressione del punto di rotazione). |

**Proprietà di un giunto attivo** (Rotational, Translational, Custom):
- `Name`, che si può usare nelle espressioni;
- `Controller`, il behavior controller a cui il giunto è assegnato;
- `Initial Value`;
- `Value`, letto nelle espressioni come `VALUE`;
- `Value Expression`;
- `Min Limit` / `Max Limit`.

Se il giunto è assegnato a un controller si aggiungono `MaxSpeed`, `MaxAcceleration`, `MaxDeceleration`,
`LagTime` e `SettleTime`. Senza velocità, accelerazione e decelerazione il giunto non si muove
(`Modeling_Operations_Assigning_a_joint_to_a_controller.txt`).

**Proprietà di un giunto follower:** `Name`, `Driver`, `Value Expression`.

**Controller (chi muove i giunti):**
- **Servo Controller:** gestisce una lista ordinata di giunti indipendenti, solo in cinematica diretta
  (moti giunto per giunto). Un Robot Executor collegato a un Servo Controller genera soltanto moti di
  tipo joint.
- **Robot Controller:** come il Servo Controller, più un behavior Kinematics (solver IK) e i parametri
  di interpolazione cartesiana. Permette moti joint e lineari. Elenca come sotto-elementi le Bases
  (sistemi di riferimento) e i Tools (TCP).
- **Motion Controller** (nuova generazione): si accoppia al **Program Executor** e a un behavior
  Kinematics con solver selezionabile.
- **Catena cinematica.** Va da `RootNode` (+`RootOffset`) a `FlangeNode` (+`FlangeOffset`). Il solver
  considera solo i giunti tra i frame di sistema ROOT e FLANGE (`Behaviors_Motion_Kinematics_kinematic_solvers.txt`).
  Il `FlangeNode` serve anche da piastra di montaggio per strutture esterne (assi esterni) o da carrello
  di un track.
- **Assi esterni.** Un posizionatore *esporta* i suoi giunti e il robot li *importa* come giunti esterni
  attraverso il campo d'interfaccia `JointExport`. Allo stesso modo un utensile esporta i tool frame
  (`ToolExport`) e un posizionatore esporta i base frame (`BaseExport`).

**Modello nominale contro spazio utente** (`Behaviors_Motion_Kinematics_kinematic_solvers.txt`):
- ogni solver ha un modello matematico con una posa zero e assi positivi (+Z locale) predefiniti;
- per ogni giunto il solver ricava un `JointZeroOffset` (JZO), sempre espresso nella direzione positiva
  nominale, e un `JointSign` (JS ±1);
- i valori dei giunti vengono mappati nello spazio nominale, il solver li elabora e poi li riconverte
  nello spazio utente;
- per modellare: si costruisce il robot nella posa zero del costruttore, lo si porta nella posa
  equivalente allo zero nominale e si ruotano i joint frame in modo che coincidano. Così JZO e JS
  vengono rilevati correttamente.

### 1.4 Proprietà del componente: tipi e meta-proprietà

Vedi la tabella CM-PROP in §2.1. Ogni property ha queste meta-proprietà:
- **`Name`**, con la sintassi `Tab::Nome` per raggrupparla in una scheda del pannello Proprietà;
- **`Visible`**;
- **`On value change`**, con le opzioni *Rebuild* (rigenera geometria ed espressioni) e *Update
  simulation* (aggiorna lo stato nel mondo 3D, necessario per i giunti e durante la simulazione);
- **`Allow editing when`**, con le opzioni *Disconnected*, *Connected* e *Simulating*;
- **`Value`**;
- **vincoli**: un intervallo `min-max`, un elenco di valori separati da `;`, oppure una stringa per
  riga;
- **quantità e prefisso d'unità**, legati alla "Unit Family" globale.

(`Properties.txt`, `Component_Graph_panel.txt`)

### 1.5 Unità di misura (quanto dice l'help)

- Lunghezze in **mm**: velocità del Physics Path in mm/s, esempio di SegmentSize in `{mm}`.
- Angoli in **gradi**: offset RxRyRz, StartSweep/EndSweep, Step dell'Angular Clone, A45.
- Tempi in **secondi**: Interval del Component Creator, HeartbeatTime, tempi di processo, intervallo
  statistico di default di 60 s.
- Densità in **kg/m³**: Physics Entity, dove l'help indica 0.2 come default di sistema.
- Le unità visualizzate dipendono dalla "Unit Family" globale. Le unità interne di velocità e
  accelerazione dei path e dei giunti **non sono dichiarate** (vedi Lacune).

### 1.6 Connessione tra componenti (interfacce)

- Un'**interfaccia** è un connettore con una o più **section** (la spina) e, dentro ogni section, dei
  **field** (i pin).
- Due interfacce si collegano se sono compatibili e hanno porte libere. Compatibili vuol dire che i
  field coincidono **per tipo e per ordine**.
- Una connessione **fisica** (`IsAbstract=False`) avviene in un punto, il `SectionFrame` di ciascuna
  section, entro `DistanceTolerance` e `AngleTolerance`. Lo usa il comando PnP (plug and play).
- Una connessione **remota** (`IsAbstract=True`) non ha posizione e si cabla da un editor di
  connessioni. Esempi: segnali verso un PLC, assi esterni.
- Dentro un singolo componente il trasferimento dei pezzi usa invece le **porte** (connettori) dei
  behavior di flusso.

(`Component_Graph_panel.txt`, `Behaviors_Interfaces_*.txt`)

---

## 2. Tabelle per categoria

### 2.1 Tipi di proprietà del componente (CM-PROP)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-PROP-01 | Boolean | Valore vero/falso (1/0). | Value: bool | On value change | Properties.txt |
| CM-PROP-02 | Button | Pulsante nel pannello che lancia un'azione. | — | evento di click (gestito da script) | Properties.txt |
| CM-PROP-03 | Distribution | Reale pseudo-casuale estratto da un'espressione di distribuzione. Accetta uno "stream" opzionale per il seed. Funzioni: normal(u,s), exponential(lambda), gamma(k,theta), lognormal(u,s), triangular(min,mode,max), uniform(a,b) con a≤x<b, weibull(k,lambda). | Value: expr | — | Properties.txt |
| CM-PROP-04 | Expression | Valore calcolato da un'espressione. Spesso è nascosta e serve a definire unità o calcoli propri. | Value: expr | — | Properties.txt, Component_Graph_panel.txt |
| CM-PROP-05 | Integer | Intero. | Value: int, vincoli range/set | On value change | Properties.txt |
| CM-PROP-06 | Matrix | Posa 4x4 (Tx,Ty,Tz,Rx,Ry,Rz). | Value: matrix | — | Properties.txt |
| CM-PROP-07 | Real | Reale, con quantità e unità. | Value: real, Quantity/Unit | — | Properties.txt |
| CM-PROP-08 | Simulation Node | Riferimento a un nodo del mondo 3D. | Value: ref<Node> | — | Properties.txt |
| CM-PROP-09 | Simulation Node List | Lista di riferimenti a nodi. | Value: list<ref<Node>> | — | Properties.txt |
| CM-PROP-10 | String | Stringa. Il vincolo può essere un elenco di valori, una stringa per riga. | Value: string | — | Properties.txt |
| CM-PROP-11 | Uri | Riferimento a un file o a una risorsa. | Value: uri | — | Properties.txt |
| CM-PROP-12 | Vector | Vettore X,Y,Z. | Value: vector | — | Properties.txt |

Meta-proprietà comuni a tutti i tipi: `Name` (`Tab::Nome`), `Visible`, `On value change`
(enum{Rebuild, Update simulation}), `Allow editing when` (enum{Disconnected, Connected, Simulating}),
`Value`.

### 2.2 Behaviors: Interfaces (CM-BEH-IF)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-IF-01 | One to One Interface | Connettore che si accoppia a **una sola** interfaccia compatibile. Può avere più section, cioè più punti di connessione indipendenti. | Name; IsAbstract: bool; ConnectSameLevelOnly: bool; AngleTolerance: real (gradi); DistanceTolerance: real; ConnectionEditName: button; InterfaceDescription: string; Sections[]: {Name, SectionFrame: ref<Frame>/NONE/NEW, Fields[]} | connect/disconnect (implicito, nessun evento documentato) | Behaviors_Interfaces_One_to_One_Interface.txt |
| CM-BEH-IF-02 | One to Many Interface | Connettore che si accoppia a **una o più** interfacce compatibili. Ha una sola section, creata automaticamente. Esempio: un nastro che accetta più sensori. | come IF-01, ma con un'unica section | come sopra | Behaviors_Interfaces_One_to_Many_Interface.txt |

**Tipi di field delle interfacce (CM-IF-FLD).** Tutti hanno `Name`. La colonna "Disp." indica se il
field esiste nelle interfacce 1:1 o 1:N.

| ID | Field | Disp. | Cosa fa | Proprietà chiave | Fonte |
|---|---|---|---|---|---|
| CM-IF-FLD-01 | Action | 1:1, 1:N | Scambia "azioni" tra Action Container (risorse client, provider, manager). È tipico delle connessioni remote. | Actions: ref<Action Container>; Connection: enum{Client, Provider, Buffer} | Interfaces_*.txt |
| CM-IF-FLD-02 | Attachment | 1:1 | Attacca e connette oggetti fisici, estendendo il PnP ai giunti. Nel 3D compare come un'icona trascinabile. È usato dai pin dei cavi fisici. | (nessuna proprietà documentata) | One_to_One_Interface.txt, Physics_Cables_in_physics_world.txt |
| CM-IF-FLD-03 | BaseExport | 1:1, 1:N | Esporta o importa base frame, per esempio da un posizionatore verso il robot. | BaseList: ref<Base Container>; Export: bool | Interfaces_*.txt |
| CM-IF-FLD-04 | Flow | 1:1 | Trasferisce componenti tra contenitori di due componenti diversi (nastro → nastro). | Container: ref<behavior contenitore>; Port: int (0=Input, 1=Output, oppure n) | One_to_One_Interface.txt |
| CM-IF-FLD-05 | Hierarchy | 1:1 | Attacca un nodo di questo componente a un nodo di un altro (relazione parent/child). Serve una parte Parent e una Child. | Node: ref<Node>; Frame: ref<Frame> (offset); Parent: bool | One_to_One_Interface.txt |
| CM-IF-FLD-06 | IntegerCompatibility | 1:1, 1:N | Impedisce la connessione se l'intero non coincide (una specie di "codice modello"). | Value: int | Interfaces_*.txt |
| CM-IF-FLD-07 | JointExport | 1:1, 1:N | Esporta o importa giunti. Quelli importati diventano giunti esterni del controller. | Controller: ref<Servo/Robot Controller>; Export: bool | Interfaces_*.txt |
| CM-IF-FLD-08 | Processor | 1:1, 1:N | Aggancia sensori staccabili (Component Path o Process Point Sensor) a un path di un altro componente. Il path fa da padre e il sensore da figlio, posizionato tramite `Distance`. | Path: ref<path>; Sensor: ref<sensor>; Parent: bool | Interfaces_*.txt |
| CM-IF-FLD-09 | Rsl | 1:1, 1:N | Pubblica o sottoscrive il programma di un Robot Executor, per chiamare subroutine remote. | Publish: bool; Subscribe: bool | Interfaces_*.txt |
| CM-IF-FLD-10 | Signal | 1:1, 1:N | Mappa un segnale verso un altro segnale, per esempio da un utensile all'I/O del robot o verso un item PLC. | Signal: ref<signal behavior>; Connection: enum{Normal, Begin, End} (Begin si accoppia solo con End) | Interfaces_*.txt |
| CM-IF-FLD-11 | ToolExport | 1:1, 1:N | Esporta o importa tool frame (TCP dell'utensile verso il robot). | ToolList: ref<Tool Container>; Export: bool | Interfaces_*.txt |
| CM-IF-FLD-12 | Transport | 1:1, 1:N | Scambia compiti di trasporto tra Transport Protocol: quale pezzo, dove prenderlo, chi lo riceve, dove posarlo. | Transport: ref<Transport Protocol>; Connection: enum{Input, Output, Input and Output} | Interfaces_*.txt |

### 2.3 Behaviors: Signals (CM-BEH-SIG)

Proprietà comuni a tutti i segnali con valore: `Name`; `AutomaticReset: bool` (al reset della
simulazione il segnale torna al valore iniziale); `Connections: list<ref<behavior>>` (i behavior
notificati a ogni cambio di valore, tipicamente Python Script tramite `OnSignal`, ma anche sensori in
modalità pulse).

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-SIG-01 | Boolean Signal | Segnale vero/falso, usato per DI/DO, sensori, TransitionSignal. | Name, AutomaticReset: bool, Connections | notifica OnSignal ai connessi | Behaviors_Signals_Boolean_Signal.txt |
| CM-BEH-SIG-02 | Boolean Signal Map | Mappa segnali su porte di uno spazio I/O contiguo e numerato (l'I/O del robot). | Name, PortCount: int, Listeners: list<ref<behavior>> | i listener (Robot o Program Executor) leggono e scrivono le porte | Behaviors_Signals_Boolean_Signal_Map.txt |
| CM-BEH-SIG-03 | Component Signal | Segnale il cui valore è un componente, per esempio il pezzo rilevato da un sensore. | Name, AutomaticReset, Connections | OnSignal | Behaviors_Signals_Component_Signal.txt |
| CM-BEH-SIG-04 | Integer Signal | Segnale intero, per esempio la porta da cui entra un pezzo in una Flow Proxy. | Name, AutomaticReset, Connections | OnSignal | Behaviors_Signals_Integer_Signal.txt |
| CM-BEH-SIG-05 | Matrix Signal | Segnale con valore matrice 4x4 di reali (una posa). | Name, AutomaticReset, Connections | OnSignal | Behaviors_Signals_Matrix_Signal.txt |
| CM-BEH-SIG-06 | Real Signal | Segnale reale, per esempio la distanza misurata da un Raycast. | Name, AutomaticReset, Connections | OnSignal | Behaviors_Signals_Real_Signal.txt |
| CM-BEH-SIG-07 | String Signal | Segnale stringa. | Name, AutomaticReset, Connections | OnSignal | Behaviors_Signals_String_Signal.txt |

### 2.4 Behaviors: Material Flow (CM-BEH-FLOW)

Proprietà ricorrenti dei contenitori:
- `Statistics: ref<Statistics>`;
- `Capacity: int`;
- `CapacityBlocks: list<ref<Capacity Controller>>`;
- `TransitionSignal: ref<Boolean Signal>` (True all'arrivo, False all'uscita di un pezzo);
- le porte (connettori) elencate come sotto-elementi nel grafo, ognuna con `Connection`/`Port` e i
  flag Input/Output.

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-FLOW-01 | Capacity Controller | Limite di capienza condiviso tra più contenitori: una variabile globale contro i deadlock. | Name; Capacity: int | notifica il Component Creator quando si libera capienza (BlockingOptimization) | Behaviors_Material_Flow_Capacity_Controller.txt |
| CM-BEH-FLOW-02 | Component Container | Contenitore statico di componenti, che vi arrivano per trasferimento, posa o script. Di norma solo in ingresso: il prelievo si fa via script. | Name; Statistics; Capacity: int; CapacityBlocks; TransitionSignal; Location: ref<Frame> | TransitionSignal True/False; porte | Behaviors_Material_Flow_Component_Container.txt |
| CM-BEH-FLOW-03 | Component Creator | Genera nuovi componenti in simulazione partendo da un modello. | Name; Statistics; TransitionSignal; Interval: real (s); Limit: int (0 = creazione manuale da script); Part: uri o `vcid:<id>`; BlockingOptimization: bool; PartPooling: bool | TransitionSignal; porta di uscita | Behaviors_Material_Flow_Component_Creator.txt |
| CM-BEH-FLOW-04 | Component Flow Proxy | Smistatore con N porte, ognuna con una porta interna nascosta abbinata a una esterna. Il passaggio interno→esterno è statico, oppure dinamico via script (tavola rotante). | Name; Statistics; ComponentSignal: ref<Component Signal>; PortSignal: ref<Integer Signal>; DynamicConnections: bool; RetainOffset: bool | ComponentSignal e PortSignal quando un pezzo tenta di entrare | Behaviors_Material_Flow_Component_Flow_Proxy.txt |
| CM-BEH-FLOW-05 | One Way Path | Trasporta i pezzi in avanti lungo una polilinea o spline di Frame (il nastro classico). | Name; Statistics; Capacity; CapacityBlocks; TransitionSignal; Speed: real; Acceleration: real (0=off); Deceleration: real (0=off); Interpolation: enum{Step, Linear, Cubic}; Accumulate: bool; SegmentSize: real (0=off); RetainOffset: bool; SpaceUtilization: bool; FastScheduling: bool; Sensors: list<ref<sensor>>; Path: list<ref<Frame>> ordinata; PathAxis: enum{Automatic, X±, Y±, Z±, Generic} | TransitionSignal; porte Input(0)/Output(1); attiva i sensori collegati | Behaviors_Material_Flow_One_Way_Path.txt |
| CM-BEH-FLOW-06 | Two Way Path | Come il One Way Path, ma bidirezionale, con inversione automatica opzionale secondo il lato di arrivo. | come FLOW-05, più Direction: enum{Forward, Backward, Forward (Auto), Backward (Auto)} | come FLOW-05 | Behaviors_Material_Flow_Two_Way_Path.txt |
| CM-BEH-FLOW-07 | Routing Rule | Instrada i pezzi verso le porte di uscita secondo regole annidate (vedi tabella CM-ROUTE). | Name; Statistics; Capacity; Capacity blocks; Transition signal; Rule component: enum{componente proprietario, componente processato}; RetainOffset: bool; Connectors[] (Add Connector); Route: albero di regole | TransitionSignal; porte | Behaviors_Material_Flow_Routing_Rule.txt |

**Regole di instradamento della Routing Rule (CM-ROUTE).** Cambiare una regola annulla le sue regole
figlie. Diverse regole hanno un `Default`.

| ID | Regola | Semantica | Parametri |
|---|---|---|---|
| CM-ROUTE-01 | Boolean Table | Sceglie la porta in base a un valore booleano di una property (del pezzo o del proprietario). | Variable, Default |
| CM-ROUTE-02 | Capacity | Sceglie la prima porta selezionata che ha capienza libera. | lista di porte |
| CM-ROUTE-03 | Cyclic | Alterna in sequenza (round robin) le porte selezionate. | CycleOnlyConnected: bool |
| CM-ROUTE-04 | Fixed | Mappa fissa da una porta d'ingresso a una porta d'uscita. | CycleOnlyConnected |
| CM-ROUTE-05 | Integer Table | Tabella da valore intero a porta. | Variable, Default |
| CM-ROUTE-06 | Percentage | Ripartizione percentuale casuale (somma 100). | RandomStream |
| CM-ROUTE-07 | `<Port>` | Tutto va su una porta (di norma solo al livello 1). | — |
| CM-ROUTE-08 | Product Type Rule | Sceglie per Product Type o Flow Group. | Default / RuleVariable |
| CM-ROUTE-09 | Real Table | Tabella da valore reale a porta. | Variable, Default |
| CM-ROUTE-10 | String Table | Tabella da stringa a porta. | Variable, Default |
| CM-ROUTE-11 | Transport solution rule | Sceglie in base alla soluzione di trasporto (il nodo di trasporto del prodotto). Vale solo per i prodotti contenuti nel router. | RuleVariable |

### 2.5 Behaviors: Sensors (CM-BEH-SEN)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-SEN-01 | Component Path Sensor | Fotocellula "logica" su un path: scatta quando un pezzo raggiunge una posizione lungo il percorso. | Name; Distance: real; Frame: ref<Frame>; TriggerAt: enum{Leading Edge, Origin, Trailing Edge, None}; ResetAt: enum ? (valori non elencati); ComponentSignal: ref<Component Signal>; BoolSignal: ref<Boolean Signal>; BoolSignalValue: bool | scrive ComponentSignal (il pezzo) e BoolSignal (True al trigger, poi reset) | Behaviors_Sensors_Component_Path_Sensor.txt |
| CM-BEH-SEN-02 | Process Point Sensor | Punto di lavoro su un path: rileva il pezzo, ferma il path o il pezzo per un tempo di processo calcolato da regole. | Name; Distance; Frame; TriggerAt (come SEN-01); BoolSignal; ProcessingStops: enum{Path, Component}; RuleComponent: enum{pezzo, proprietario}; Route: albero di regole di tempo (CM-PTIME) | BoolSignal al trigger; ferma e riavvia path o pezzo | Behaviors_Sensors_Process_Point_Sensor.txt |
| CM-BEH-SEN-03 | Raycast Sensor | Lancia un raggio lungo +Z di un Frame: misura la distanza e rileva i pezzi entro una soglia. | Name; UpdateScene: bool; TestParent: bool; ShowRay: bool; UseSampling: bool; SampleTime: real; MaxRange: real; DetectionThreshold: real; DetectionContainer: ref<container>; DetectionContainers: list; RangeSignal: ref<Real Signal>; ComponentSignal; BoolSignal; Frame: ref<Frame> | RangeSignal (distanza se < MaxRange); ComponentSignal e BoolSignal se < DetectionThreshold. In modalità pulse è innescato da un segnale | Behaviors_Sensors_Raycast_Sensor.txt |
| CM-BEH-SEN-04 | Volume Sensor | Volume a forma di box, tra i Frame LowerFrame e UpperFrame, che rileva i componenti al suo interno. | Name; UpdateScene; TestParent; TestSiblings: bool; TestOnlyContained: bool; ShowVolume: bool; VolumeMaterial: ref<Material>; TestMethod: enum{Precise Test, BBox Intersect, BBox Center Inside, BBox Fully Inside}; UseSampling; SampleTime; DetectionContainer; ComponentSignal; BoolSignal; LowerFrame; UpperFrame | ComponentSignal, BoolSignal; in modalità pulse è innescato da un segnale | Behaviors_Sensors_Volume_Sensor.txt |

**Regole di tempo del Process Point Sensor (CM-PTIME).** Il ritardo è in secondi.

| ID | Regola | Semantica | Parametri |
|---|---|---|---|
| CM-PTIME-01 | Boolean Table | Tempo in base a un booleano. | PropertyName, Default |
| CM-PTIME-02 | Integer Table | Tempo in base a un intero. | PropertyName, Default |
| CM-PTIME-03 | Normal Distribution | Tempo con distribuzione normale. | RandomStream, Average, Deviation |
| CM-PTIME-04 | `<Time>` | Tempo costante. | valore |
| CM-PTIME-05 | Uniform Distribution | Tempo con distribuzione uniforme. | RandomStream, Min, Max |
| CM-PTIME-06 | Real Table | Tempo in base a un reale. | PropertyName, Default |
| CM-PTIME-07 | String Table | Tempo in base a una stringa. | PropertyName, Default |

### 2.6 Behaviors: Robotics (CM-BEH-ROB)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-ROB-01 | Base Container | Raccolta di Frame usati come base (sistemi di coordinate utente). Sotto-elemento "Bases". | Name. Per ogni base frame: Name, Transform: expr/matrix, Node: ref<Node> o NULL, InterpolationMode: enum ? | — | Behaviors_Robotics_Base_Container.txt |
| CM-BEH-ROB-02 | Robot Controller | Controller di un robot con cinematica inversa: gestisce giunti, base e tool, e interpola i moti lineari. | Name; Joints: list<joint> ordinata; UseHeartbeat: bool; HeartbeatTime: real (s); RootNode, FlangeNode: ref<Node>; RootOffset, FlangeOffset: matrix; InitialBase, InitialTool; Kinematics: ref<Kinematics behavior>; MaxCartesianSpeed/Accel: real; MaxAngularSpeed/Accel: real; LagTime, SettleTime: real; OrientationInterpolationMode: enum{One Axis, Two Axis, Three Axis (WPR/ZYZ)}; ConfigurationMode: enum{Fixed, Follow frame, Interpolate joint values}; TrackWorldFrameMode: enum{TrackOrigin, RobotWorldFrame} | heartbeat: impulso periodico che genera eventi | Behaviors_Robotics_Robot_Controller.txt |
| CM-BEH-ROB-03 | Robot Executor | Legge, scrive ed esegue il programma robot; gestisce l'I/O digitale con le signal map. | Name; DigitalInputSignals, DigitalOutputSignals: ref<Boolean Signal Map>; IsLooping: bool; StatementHandler: ref<Python process handler>; IsEnabled: bool; Controller: ref<Robot/Servo Controller> | legge e scrive le porte I/O; i comandi possono essere sovrascritti dallo script handler | Behaviors_Robotics_Robot_Executor.txt |
| CM-BEH-ROB-04 | Servo Controller | Pilota giunti indipendenti, solo in cinematica diretta (macchine, assi, posizionatori). | Name; Joints; UseHeartbeat; HeartbeatTime; RootNode; FlangeNode; RootOffset; FlangeOffset | heartbeat | Behaviors_Robotics_Servo_Controller.txt |
| CM-BEH-ROB-05 | Tool Container | Raccolta di Frame usati come tool (TCP). Sotto-elemento "Tools". | Name. Per ogni tool frame: Name, Transform, Node (ref o NULL), InterpolationMode (per esempio il TCP stazionario) | — | Behaviors_Robotics_Tool_Container.txt |

### 2.7 Behaviors: Kinematics (solver "classici") (CM-BEH-KIN)

Questi behavior sono riferiti dalla proprietà `Kinematics` del Robot Controller.

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-KIN-01 | Articulated Kinematics | Cinematica diretta e inversa per un antropomorfo a 6 assi parametrizzato da offset, segni, zeri, accoppiamenti, configurazioni e singolarità. | Name; L01Z…L56Z (L01Z, L23X, L56X…): real; A45: real (gradi, tipicamente ±90); JointZeroOffset1..6; JointSign1..6; Coupling23: bool; ShoulderValid/ElbowValid/WristValid: bool (filtrano le 8 configurazioni front/back × above/below × flip/noflip); ConfigName1..N: string; PalletizingMode: bool; PalletizingNormal: vector; PolarSingularityTolerance: real; WristSingularityTolerance: real (gradi); ExtendedPositionTolerance: real (gradi, ≥0); Coupling456: real | — | Behaviors_Kinematics_Articulated_Kinematics.txt |
| CM-BEH-KIN-02 | Cartesian Kinematics | Cinematica diretta e inversa per un gantry: TX, TY, TZ più polso RZ, RY, RZ. Si usa anche per non-robot (per esempio un carrello elevatore). | Name. La struttura deriva da RootNode, FlangeNode e dagli offset del controller | — | Behaviors_Kinematics_Cartesian_Kinematics.txt |
| CM-BEH-KIN-03 | Delta Kinematics | Cinematica diretta e inversa per un robot delta parallelo. | Name; LinkLength1 (braccio superiore), LinkLength2 (braccio inferiore); Height1, Height2; ShoulderOffset; WristOffset: real | — | Behaviors_Kinematics_Delta_Kinematics.txt |
| CM-BEH-KIN-04 | Parallelogram Kinematics | Cinematica per un pallettizzatore a parallelogramma con polso sempre rivolto in basso. | Name; LinkLength1..5; JointOffset1; JointOffset3: real | — | Behaviors_Kinematics_Parallelogram_Kinematics.txt |
| CM-BEH-KIN-05 | Python Kinematics | Cinematica diretta e/o inversa definita via script. | Name; Script. Callback: OnInitKinObject, OnGetJointCount, OnGetJointName, OnConstrainParams, OnRelaxParams, OnForward, OnInverse | callback richiamate dal controller | Behaviors_Kinematics_Python_Kinematics.txt |
| CM-BEH-KIN-06 | Scara Kinematics | Cinematica diretta e inversa per uno SCARA con configurazioni Left e Right. Esiste anche una variante "old". | Name; L12X, L23X, L04Z: real; ConfigName1..2; JointSign1..4; JointZeroOffset1..5 | — | Behaviors_Kinematics_Scara_Kinematics.txt |

### 2.8 Behaviors: Motion (nuova generazione) e tipi di solver (CM-BEH-MOT, CM-SOLV)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-MOT-01 | Motion Controller | Controller di nuova generazione per i meccanismi del robot. | Name; Visible: bool; Kinematics: ref<Kinematics behavior> | — | Behaviors_Motion_Motion_Controller.txt |
| CM-BEH-MOT-02 | Program Executor | Esecutore di programma abbinato al Motion Controller. | Name; Visible; DigitalInputSignals, DigitalOutputSignals: ref<Boolean Signal Map>; Controller: ref<Motion Controller> | I/O tramite signal map | Behaviors_Motion_Program_Executor.txt |
| CM-BEH-MOT-03 | Kinematics (con solver) | Behavior che crea e possiede un "main solver" del tipo scelto in una proprietà a scelta. Ogni solver espone una propria "Solver Configuration". Dall'API se ne possono creare copie per calcoli di cinematica diretta e inversa. | tipo di solver: enum (CM-SOLV); Solver Configuration (dipende dal solver); JZO e JS per giunto | — | Behaviors_Motion_Kinematics_kinematic_solvers.txt |

| ID | Solver | Requisiti del modello nominale | Fonte |
|---|---|---|---|
| CM-SOLV-01 | Articulated 6 ZYYZYZ | 6 assi con polso sferico, oppure angolato o cavo tramite A45 (rotazione attorno a X nominale, default -90°). Offset firmati tra joint frame nel frame root nominale. | ..._Solver_Types_articulated_6_zyyzyz.txt |
| CM-SOLV-02 | Articulated 6 ZYYZYZ 46Y | 6 assi con offset Y tra gli assi paralleli 4 e 6 (polso con offset). | ..._articulated_6_zyyzyz_46y.txt |
| CM-SOLV-03 | Articulated 6 ZYYYZY | 6 assi con sequenza Z-Y-Y-Y-Z-Y. Le X di tutti i joint frame sono parallele alla X root; un segno negativo si ottiene ruotando il frame di Rx(180). | ..._articulated_6_zyyyzy.txt |
| CM-SOLV-04 | Cartesian | Fino a 3 prismatici ortogonali (ordine e direzione liberi) seguiti da fino a 3 rotoidali ortogonali: polso singolo Z oppure ZYZ sferico. | ..._cartesian.txt |
| CM-SOLV-05 | Delta (Revolute) | Delta a 3 bracci. Il frame root ha X verso il braccio 1 e Z verso la piattaforma; il primo braccio collega la flangia alla root. Lunghezze e offset (SO, L1, L2, WO) sono letti dal modello in posa zero. I valori passivi si leggono come `Passive::ry1` oppure con la ValueSource "FromPassiveValue". | ..._delta_revolute.txt |
| CM-SOLV-06 | Scara | 4 giunti nell'ordine Rz, Rz, Tz, Rz. Lo zero offset vale solo per i rotoidali. | ..._scara.txt |

### 2.9 Behaviors: Physics (CM-BEH-PHY)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-PHY-01 | Physics Cable | Cavo flessibile (fune, catena, cinghia, filo) con due estremi ancorati a nodi. La geometria è fatta di capsule generate in automatico. | Name; Thickness; Cable Length (0 = dinamica); Collider Length; Stiffness: enum{Low, Medium, High}; Node 1/Node 2: ref<Node>; End Type 1/2: enum{Free, Rotating, Fixed}; Total Length Factor; Material. Avanzate: Spring Strength/Damping, Spring Tangential Strength/Damping, Rendering Steps, Collider Overlap, Solver Iter Position/Velocity/Internal/External/Projections, Solver Separation Limit, Material Friction Static/Dynamic, Restitution, Density, Scene Id, Collision Group, Center Position (azione) | pin come nodi, con Physics Joint e interfaccia Attachment | Behaviors_Physics_Physics_Cable.txt |
| CM-BEH-PHY-02 | Physics Joint | Giunto D6 (6 DOF configurabili) tra il nodo che lo contiene e un altro nodo. | Name; Attached to: ref<Node>; Constraints: per ciascuno dei 3 assi T e 3 assi R, enum{Free, Limited, Locked}; Breakage (gruppo, dettagli non documentati); Drive Settings (gruppo, non documentato) | — | Behaviors_Physics_Physics_Joint.txt |
| CM-BEH-PHY-03 | Physics Path | Nastro "fisico": spinge per contatto i corpi lungo una sequenza di Frame (direzione = X di ogni frame). | Name; Statistics; TransitionSignal; Path: list<ref<Frame>>; PushingFeatures: list<ref<feature>>; Speed: real (mm/s, negativo = direzione inversa) | TransitionSignal; porte | Behaviors_Physics_Physics_Path.txt |
| CM-BEH-PHY-04 | Physics Entity | Rende un nodo un corpo rigido del motore fisico. | Name; Physics Type: enum{In Physics, Out of Physics, Kinematic, In Container}; Material Density (kg/m³, 0 = 0.2 di default); Static Friction (0..1, default 0.5); Dynamic Friction (0..1, default 0.5); Restitution (0..1, default 0.1); Linear Damping (0..1, default 0); Angular Damping; Solver Position Iterations (default 4); Solver Velocity Iterations (default 1); Scene Id: int (default 0); Collision Group; Smoothing Steps | — | Behaviors_Physics_Physics_entity.txt |

### 2.10 Behaviors: Process Model (CM-BEH-PM)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-PM-01 | Process Executor | Contiene ed esegue le routine di processo. Parte da sola all'avvio della simulazione e gira finché non viene fermato. | Name; TransportNode: ref<Transport Node> (necessario per gli statement di trasporto in ingresso e in uscita) | riceve dal Transport Node le notifiche di arrivo e partenza dei prodotti | Behaviors_Process_Model_Process_Executor.txt |
| CM-BEH-PM-02 | Product Creator | Crea i prodotti definiti nel Product Type Editor. | Name; Statistics; FeedMode: enum{Single, Batch, Table (Excel), Distribution} | porte | Behaviors_Process_Model_Product_Creator.txt |
| CM-BEH-PM-03 | Transport Controller | Trasporta i prodotti tra Transport Node secondo i transport link; logica personalizzabile in Python (operatori, robot). I controller "Conveyor" e "interpolating" sono interni e non personalizzabili. | Name; Icon: string (percorso SVG nella cartella Icons); IconColor | — | Behaviors_Process_Model_Transport_Controller.txt |
| CM-BEH-PM-04 | Transport Node | Punto dove i prodotti entrano ed escono nel Flow Editor; reagisce agli arrivi in un Component Container associato. | Name; Process Executor: ref; ComponentContainer: ref<Component Container> (obbligatorio, anche in "motion point mode" su un path); VisualPositionFrame, DefaultProductPositionFrame, DefaultResourcePositionFrame: ref<Frame> | notifica arrivi e partenze al Process Executor | Behaviors_Process_Model_Transport_Node.txt |

### 2.11 Behaviors: Misc (CM-BEH-MISC)

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-BEH-MISC-01 | Action Container | Coda di "azioni" da inviare, ricevere e memorizzare tra risorse. | Name; Connections: list<ref<behavior>> | evento OnAction verso gli script connessi | Behaviors_Misc_Action_Container.txt |
| CM-BEH-MISC-02 | Collider Container | Raccolta di collider semplificati (box, sfera, convesso) del componente. | Name; Visible. Per ogni collider: Name, Visible, Padding (convesso), Radius (sfera), Length/Width/Height (box) | — | Behaviors_Misc_Collider_Container.txt |
| CM-BEH-MISC-03 | Document Link | Raccolta di link a file interni o esterni. | Name (le altre proprietà non sono documentate) | — | Behaviors_Misc_Document_Link.txt |
| CM-BEH-MISC-04 | Jog Info | Definisce il DOF interattivo di un oggetto o di una property (maniglia di jog). Ha la precedenza sul jog automatico dei giunti. | Name; Offset: matrix (pivot); DOF: enum{traslazione o rotazione su un asse}; Scale: real (±1); Variable: ref<property/oggetto>; Message: string | — | Behaviors_Misc_Jog_Info.txt, Modeling_Operations_Interacting_with_a_joint.txt |
| CM-BEH-MISC-05 | Note | Documentazione a testo libero del componente, consultabile da un'icona nel 3D. | Name; Note: testo (con link e URI) | — | Behaviors_Misc_Note.txt |
| CM-BEH-MISC-06 | Python Script | Logica utente in Python (Stackless 2.7) con API dell'applicazione; ha un editor con debug. | Name; Script. Eventi base: OnSignal(signal), OnRun(). Editor: Compile, Trace Execution, Run to cursor, Step 1 line, snippet (Ctrl+J), autocompletamento (Ctrl+Space), drag&drop degli elementi del grafo | OnSignal dai segnali connessi; OnRun all'avvio della simulazione; OnAction (dall'Action Container) | Behaviors_Misc_Python_Script.txt |
| CM-BEH-MISC-07 | Statistics | Raccoglie i KPI dei contenitori (pezzi e tempi) e degli stati (utilizzo, guasti…), cumulativi e per intervallo (default 60 s). | Name; Parts{Entered, Exited, Current, Utilization, AverageCount, MinCount, MaxCount, TotalTime, AverageTime, MinTime, MaxTime} e varianti PartsInterval*; Utilization, TotalStateTime, Break/Idle/Busy/Blocked/Failed/Repair/SetupPercentage, AverageActivePeriodTime e varianti Interval*; stati dinamici State<Nome>Percentage e State<Nome>IntervalPercentage; Create default states (8 stati: Blocked, Break, Broken, Busy, Idle, Repair, Setup, Warmup, rinominabili ma legati a uno stato di sistema) | aggiornamento periodico | Behaviors_Misc_Statistics.txt |
| CM-BEH-MISC-08 | Transport Protocol | Contenitore di trasporto che gestisce via script le richieste di trasporto delle risorse. | Name; Statistics; Capacity; CapacityBlocks; TransitionSignal; Location: ref<Frame>; Resourcing: ref<behavior> (tipicamente un Action Container) | TransitionSignal; field d'interfaccia Transport | Behaviors_Misc_Transport_Protocol.txt |
| CM-BEH-MISC-09 | Vehicle | Rende il componente un mobile pilotabile da script (AGV, carrello). | Name (nient'altro documentato) | — | Behaviors_Misc_Vehicle.txt |
| CM-BEH-MISC-10 | Workflow | Procedura guidata a passi associata al componente, con un configuratore. | Name; Visible; RunOnLoad: bool. Passi: titolo, istruzioni (con link), IsSkippable, comandi; import ed export | apertura automatica al caricamento | Behaviors_Misc_Workflow_behavior.txt |

