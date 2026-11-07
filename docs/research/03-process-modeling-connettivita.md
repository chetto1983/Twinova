# Visual Components 5.1: mappa funzionale di Process Modeling e Connectivity

Mappa clean-room per progettare un clone indipendente. Fonte: l'help ufficiale 5.1 Premium (vedi [indice](00-indice.md), §2),
cioè 90 file: 54 `English_Process_Modeling_*` e 36 `English_Connectivity_*`, tutti letti. Le descrizioni
sono riformulate, nessuna frase è copiata. Nella colonna "Fonte" il prefisso `English_` è omesso:
`PM_` sta per `English_Process_Modeling_` e `CN_` per `English_Connectivity_`.

Legenda di verifica:
- **[help]**: affermazione presente nel testo dell'help;
- **[ipotesi]**: mia deduzione o conoscenza esterna, da verificare prima di usarla.

---

## 1. Process Modeling

### 1.1 Concetti

**Prodotti**
- **Product Type (tipo di prodotto)**: il modello di ciò che circola nella simulazione. Ne esistono due
  sottotipi:
  - **Part Product Type**: genera sempre una sola istanza, con un solo componente dinamico;
  - **Assembly Product Type**: definisce come più tipi si combinano; creare un assieme può generare molte
    istanze.
- **Proprietà di un Product Type**, di tre famiglie:
  - di default, obbligatorie: Name, ParentProduct (ereditarietà da un altro tipo), ComponentUri (il
    componente 3D associato, anche via `vcid:`) e OriginFrame;
  - di prodotto: Double, String, Int32, Boolean, Vector, Matrix, URI, Material, oppure un tipo custom via
    API. Viaggiano con il prodotto lungo la produzione e non compaiono nel pannello Properties del
    componente;
  - di componente: proprietà iniettate nel componente dinamico quando l'istanza nasce.
  Prodotto e componente sono oggetti distinti. [help] `PM_Product_Type_Editor.txt`
- **Product Instance**: l'oggetto runtime. Si specializza in Part Instance o Assembly Instance.
  L'Assembly Instance ha un proprio componente dinamico "vuoto", a cui le parti sono attaccate in
  gerarchia, e porta i metadati di struttura dell'assieme.
- **Product Bundle**: più istanze tenute insieme solo agganciando i loro componenti dinamici. Non ha
  metadati di assieme e, per gli statement, si comporta come una parte. Non esiste un "tipo bundle".
  [help] `PM_Terminology.txt`
- **Assiemi**:
  - **Assembly Step**: un passo, cioè un sottoinsieme di parti montato o smontato insieme. Gli step sono
    gerarchici, ognuno con una posa rispetto al padre e proprietà proprie;
  - **Assembly Pattern**: un insieme di **Assembly Slot**, riutilizzabile anche fuori dal tipo;
  - **Assembly Slot**: una posizione e un orientamento, con un filtro sui tipi accettati. Ha queste
    proprietà:
    - OrderIndex: l'ordine di montaggio è crescente e lo smontaggio è decrescente; slot con lo stesso
      indice vengono serviti in parallelo;
    - AcceptAllProductTypes, AcceptProductTypes e AcceptFlowGroups;
    - DefaultProductType;
    - DefaultBoundsDimensions.
  - **Assembly Order**: la selezione di un sottoinsieme di step di un Assembly Type. Si passa agli
    statement come parametro.
  - **Assembly Editor**: un ambiente 3D dedicato per creare, spostare, copiare e centrare gli slot, anche a
    pattern o partendo da template. [help] `PM_Product_Type_Editor.txt`, `PM_Terminology.txt`
- **Flow Group**: ogni tipo appartiene a esattamente un gruppo di flusso, e tutti i tipi di un gruppo
  seguono la stessa sequenza di processi.

**Processi**
- **Process**: un'operazione astratta che una macchina sa fare. Ha due realizzazioni:
  - **Process implementation (routine)**: la sequenza concreta di statement dentro un Process Executor
    (un behavior del componente);
  - **Process Group**: l'insieme di tutte le routine con lo stesso nome, anche su componenti diversi. Il
    flusso punta al gruppo, e le implementazioni del gruppo possono comportarsi in modo diverso.
- **Process Executor**:
  - esegue **una routine alla volta**;
  - sceglie la prossima routine valutando le **Requirements** (precondizioni) di tutte le routine, che
    devono essere soddisfatte contemporaneamente;
  - se nessuna routine ha requisiti, le esegue da sinistra a destra;
  - se nessun requisito è soddisfatto, va in idle;
  - le Requirements vengono rivalutate alla fine di ogni processo. [help] `PM_Process_Editor.txt`

**Flussi**
- **Process flow sequence**: la sequenza di **Flow Step** di un Flow Group. Uno step può contenere più
  Process Group paralleli, con priorità dall'alto in basso.
- **Opzioni di uno step**:
  - lo step può essere **opzionale**: viene eseguito solo se lo step successivo non è disponibile;
  - un gruppo può essere opzionale;
  - un gruppo può avere un limite di **Visit Count**;
  - modalità "first available": viene eseguito il primo gruppo disponibile;
  - modalità "tutti i processi in qualunque ordine": tutti i gruppi vengono eseguiti, a partire da quelli
    disponibili. [help] `PM_Process_Flow_Editor.txt`

**Trasporto**
- **Transport Node**: un luogo logico da cui o verso cui si trasporta. Ha un ComponentContainer associato,
  oppure fa da sensore su un path di un nastro.
- **Transport Link**: una connessione logica monodirezionale tra due nodi, implementata da un solo
  Transport Controller.
- **Transport Solution**: la route calcolata, cioè la lista di link da percorrere.
- **Transport System**: l'insieme di link e controller per ogni flow group. Viene usato solo quando la
  sorgente o la destinazione sono "processo precedente/successivo" o "nodo"; in caso contrario il
  trasporto avviene fuori dal sistema.
- **Transport Controller**:
  - riceve i task di trasporto e di lavoro generati dal PM e li assegna a una risorsa;
  - **ResourcePriority**: Nearest oppure Least Utilized (in base a Statistics.Utilization);
  - **Strategy** di multi-trasporto: One-to-One, One-to-Many, Many-to-One o Many-to-Many. Le risorse prima
    raccolgono tutto e poi consegnano;
  - **LinkDefaults**: PickApproach, PlaceApproach, PickTime e PlaceTime, sovrascrivibili per singolo link
    con UseCustomParameters.
  [help] `PM_Using_Resources_transport_controllers.txt`
- **Resource** (mobile, umano, veicolo):
  - esegue i task;
  - Capacity: quanti pezzi trasporta e dove li tiene a bordo; le posizioni si catturano con PickProduct e
    Add Location;
  - LIFO o FIFO in scarico;
  - un tool per link di trasporto e un tool per il lavoro (tab Work del controller).
  [help] `PM_Using_Resources_resources.txt`
- **Navigation Controller (Beta)**:
  - definisce un'area di percorrenza comune ai controller Mobile, Human e Vehicle (Beta) collegati;
  - costruisce una navmesh, con le visualizzazioni Raw, Standard, Standard with IDs (poligoni ristretti
    marcati) e Detailed;
  - gli ostacoli si dichiarano con la proprietà IsObstacle, aggiunta a tutti i componenti dopo la prima
    generazione;
  - copertura dei percorsi: Max Agent Radius (la mappa si restringe del raggio massimo) oppure Agent Origin
    (i percorsi si espandono, e quelli vicini possono fondersi).
  [help] `PM_Using_Resources_navigation_controller.txt`
- **Pathway Area / Pathway Curve (Beta)**:
  - formano l'area navigabile, che il controller ottimizza;
  - OneWay limita il verso di percorrenza;
  - AreaID raggruppa aree logiche con un limite di capacità e di velocità.
  [help] `PM_Using_Resources_pathways.txt`
- **Push e pull**:
  - "To Next Process" e "From Previous Process" sono **pull**: il processo a valle chiede il pezzo quando
    ha capacità;
  - "To Component Container" e "To Transport Node" sono **push**: il nastro fa da buffer.

**Variabili** [help] `PM_Using_process_variables.txt`, `PM_Process_Editor.txt`
- **Scope**: le variabili sono locali alla routine e condivise tra i suoi requisiti e i suoi statement.
- **Statiche**: create dall'interfaccia, persistono nel layout e vengono resettate al default del tipo (False,
  0, 0.0, "") all'avvio e al reset della simulazione, non a ogni esecuzione. Il campo Value è di sola
  lettura; si inizializza con Assign.
- **Runtime**: create automaticamente dagli statement quando la variabile di output non esiste. Vengono
  resettate prima della valutazione dei requisiti, così i requisiti possono passare valori agli statement,
  e cancellate allo start e al reset. Esempio: `ProductIn`.
- **Tipizzazione e riferimenti**: riferimenti e liste sono fortemente tipizzati (un solo tipo per lista).
  Un riferimento a un oggetto cancellato diventa Null, e il Null resta nella lista.
- **Conversione a lista**: molti statement convertono una variabile esistente in lista quando ci
  aggiungono valori.

**Product Creator** (behavior)
- Genera istanze dai tipi.
- Modalità: Single (un tipo a intervallo), Batch (sequenza tipo, quantità e intervallo, anche ripetuta),
  Table (da un file Excel con i tempi) e Distribution (un tipo scelto con probabilità pesata).
- Componenti di esempio: "Source Process" (alimentatore fisso) e "Product Feeder" (in linea).
  [help] `PM_Product_Type_Editor.txt`

**Filtri prodotto** [help] `PM_Product_Filtering.txt`
- Più filtri si combinano con un operatore.
- Tipi di filtro:
  - **Product Type**: per tipi accettati e per flow group accettati;
  - **Product Property**: una proprietà del prodotto, un operatore e un valore, dove il valore è
    un'espressione che può leggere variabili di processo;
  - **Expression**: un'espressione booleana su un alias del prodotto, ad es. `Product.Length <= MaxLength`;
  - **Specific Product**: accetta solo le istanze contenute in una variabile, in modalità Compare To Any
    oppure Compare In Order.
- FilterDiagnosticMode decide se notificare una proprietà mancante. Il prodotto viene comunque scartato.

### 1.2 Tabella degli statement e delle funzioni

Tutti gli statement hanno `IsEnabled`. Il tipo di statement è indicato tra parentesi:
- (R) = requirement, cioè una precondizione;
- (F) = controllo di flusso;
- (T) = trasporto;
- (A) = assiemi;
- (P) = prodotto/variabili;
- (S) = segnali/proprietà/visual;
- (X) = altro.

| ID | Nome | Cosa fa (parole mie) | Parametri chiave | Fonte (.txt) |
|---|---|---|---|---|
| PM-01 | Append To List (P) | Accoda il valore o la lista sorgente a una variabile destinazione, trasformandola in lista se non lo è | SourceVariableName, TargetVariableName, AllowDuplicates | PM_Process_Statements_Append_To_List |
| PM-02 | Assign Variable (P) | Scrive in una variabile il risultato di un'espressione. Non accetta istanze di prodotto, ma può azzerare (Null) una variabile prodotto | TargetProperty, ValueExpression | PM_Process_Statements_Assign_Process_Variable |
| PM-03 | Attach (P) | Aggancia un prodotto figlio a un prodotto padre | Parent, Child | PM_Process_Statements_Attach |
| PM-04 | Break (F) | Esce subito dal While | — | PM_Process_Statements_Break |
| PM-05 | Buffer (T) | Magazzino temporaneo a griglia, con ingressi e uscite paralleli. Gira finché la condizione di stop diventa vera e il buffer si è svuotato | BreakCondition, BufferMode FIFO/LIFO, Source (From Previous Process / From Component Container), Destination (To Next Process / To Component Container / To Transport Node), DestinationContainer, DestinationNode, PatternCount/PatternStep/Offset XYZ, ParallelInputLimit, ParallelOutputLimit, Input/OutputResourcePositionFrame, ProductPositionFrame, ReservedProductVariableName, filtro | PM_Process_Statements_Buffer |
| PM-06 | Change Type (P) | Converte un'istanza a un altro tipo, in logica e in grafica. Non supporta gli assiemi | ProductVariableName, NewType, DeleteAttached | PM_Process_Statements_Change_Type |
| PM-07 | Check Assembly (A) | Mette in una variabile booleana l'esito del controllo "tutti gli slot dell'Assembly Order sono pieni" | AssemblyVariableName, AssemblyOrderVariableName, VariableName | PM_Process_Statements_CheckAssembly |
| PM-08 | Comment (X) | Riga di commento, non eseguita | Comment | PM_Process_Statements_Comment |
| PM-09 | Construct (A) | Crea un Assembly Instance e ci inserisce le istanze date negli slot compatibili, nell'ordine dell'assieme, con un moto lineare opzionale. Almeno un pezzo deve entrare | AssemblyOrderVariableName, ProductVariableName, AssemblyTime (s), RemoveAccepted, ReturnVariableName | PM_Process_Statements_Construct |
| PM-10 | Continue (F) | Torna all'inizio del While e rivaluta la condizione | — | PM_Process_Statements_Continue |
| PM-11 | Create (P/A) | Crea un'istanza di un tipo in un container. Se il tipo è un assieme, lo crea completo (gli slot con default vengono riempiti, anche in modo ricorsivo) oppure solo gli step scelti. Senza il primo step produce Product Bundle | ProductType, AssemblySteps, IncludeChildSteps, ProductVariableName, Container, ProductPositionFrame, RetainOffsetFromAssembly | PM_Process_Statements_Create |
| PM-12 | Delay (X) | Attende un tempo di simulazione dato da un'espressione o da una distribuzione con seme (stream) opzionale | TimeSource; distribuzioni normal, exponential, gamma, lognormal, triangular, uniform, weibull; Expression | PM_Process_Statements_Delay |
| PM-13 | Detach (P) | Sgancia dal padre il primo figlio che passa il filtro, oppure tutti | ProductVariableName, DetachMany, DetachedVariableName, filtro | PM_Process_Statements_Detach |
| PM-14 | Find Flow Step (F) | Da un ProcessFlowInfo ne ricava un altro che punta a uno step precedente o successivo della stessa sequenza. Lavora anche su liste | SourceFlowInfoVariableName, TargetFlowInfoVariableName, Mode, ProcessToFind, SearchDirection, LoopingSearch, ReturnStepOffset, SkipOptionalSteps | PM_Process_Statements_FindFlowStep |
| PM-15 | Flow (X) | Registra nelle statistiche l'ingresso o l'uscita di uno o più prodotti | ProductVariableName, Statistics, FlowDirection Enter/Leave | PM_Process_Statements_Flow |
| PM-16 | Get Assembly (A) | Produce un Assembly Order, oppure la lista ordinata (o inversa) delle istanze contenute, partendo da un tipo (step scelti a mano) o da un'istanza (regola: primo step incompleto, vuoto o completo, oppure ultimo step con pezzi) | AssemblyDataSource, AssemblyVariableName, AssemblyStep, Assembly, AssemblySteps, IncludeChildSteps, ReturnAsList, InverseList, ReturnVariableName | PM_Process_Statements_Get_Assembly |
| PM-17 | Get Assembly Step Property (A) | Legge la prima proprietà utente con un dato nome tra gli step selezionati, ad es. un tempo ciclo per step | AssemblyOrderVariableName, PropertyName, OutputVariableName | PM_Process_Statements_Get_Assembly_Step_Property |
| PM-18 | Get Flow Step (F) | Crea uno o più ProcessFlowInfo che descrivono uno step della sequenza, ricavato da uno step fisso o dal prodotto | FlowStepSource, ReturnVariableName, FlowGroup, FlowStep, ProductVariableName | PM_Process_Statements_GetFlowStep |
| PM-19 | Get Products (P) | Mette in una lista i riferimenti alle istanze presenti in un container che passano il filtro. Avvertenza: nessuna protezione contro l'accesso concorrente | Container, MaxCount, VariableName, filtro | PM_Process_Statements_Get_Products |
| PM-20 | Get Property (S) | Lega una variabile runtime a una proprietà di componente o di behavior statico (solo Int, Real, Vector, Matrix, String) | SelectedComponent, SelectedBehavior, SelectedProperty, PropertyVariableName | PM_Process_Statements_Get_Property |
| PM-21 | Halt (X) | Ferma e/o resetta la simulazione | HaltSimulation, ResetSimulation | PM_Process_Statements_Halt |
| PM-22 | If (F) | Esegue il ramo Then o il ramo Else secondo una condizione | Condition | PM_Process_Statements_If |
| PM-23 | Move Joint (S) | Muove un giunto del componente stesso (servo o robot controller) verso un target, con un tempo opzionale. Può superare i limiti se "stop at limits" è disattivo | Controller, Joint, TargetValue, MotionTime | PM_Process_Statements_Move_Joint |
| PM-24 | Print (X) | Scrive un messaggio nell'Output, con un prefisso `Comp::Executor::Routine` opzionale | Message, PrependHeader | PM_Process_Statements_Print |
| PM-25 | Property Condition (R) | Precondizione basata sul valore di una proprietà di componente o di behavior | Selected component/behavior/property, Condition expression | PM_Process_Statements_Property_Condition |
| PM-26 | Python Statement (X) | Statement personalizzato scritto con l'API Python 3 (rimanda alla sezione API) | — | PM_Process_Statements_python_statement |
| PM-27 | Release Product Reservation (P) | Libera le istanze prenotate prima che inizi il loro trasporto | ProductVariableName | PM_Process_Statements_Release_Product_Reservation |
| PM-28 | Remove (P) | Elimina l'istanza dalla simulazione e azzera la variabile (anche per Assembly Order) | ProductVariableName, AssemblyOrderVariableName | PM_Process_Statements_Remove |
| PM-29 | Reserve Product (R) | Precondizione: esiste un prodotto compatibile, che viene prenotato in una variabile. Se ne possono usare più d'uno. Lo consumano Buffer, TransportIn, TransportPatternIn e StartTransportIn | ProductVariableName, filtro | PM_Process_Statements_Reserve_Product |
| PM-30 | Run Robot Routine (S) | Aspetta che il robot sia libero, risolve per nome una routine del robot executor (con un'espressione che può usare variabili prodotto, rivalutata a ogni idle) e la esegue fino in fondo. Permette di condividere un robot tra più executor | Component (Null = sé stesso), RoutineName | PM_Process_Statements_Run_Robot_Routine |
| PM-31 | Select Products (P) | Copia in una nuova lista le istanze di una lista che passano il filtro | ProductVariableName, MaxCount, SplitVariableName, filtro | PM_Process_Statements_Select_Products |
| PM-32 | Send Signal (S) | Emette un segnale (Bool, Int, Real, String, Component) di un componente statico con un valore calcolato | Component, Signal, Value | PM_Process_Statements_Send_Signal |
| PM-33 | Set Node Material (S) | Cambia il materiale di un nodo del componente prodotto, con forzatura opzionale dell'ereditarietà | Material, Force, NodeName, ProductVariableName | PM_Process_Statements_Set_Node_Material |
| PM-34 | Set Node Visibility (S) | Mostra, nasconde o commuta un nodo del componente prodotto (la radice se il nome è vuoto) | ProductVariableName, Visibility Toggle/Visible/Invisible, NodeName | PM_Process_Statements_Set_Node_Visibility |
| PM-35 | Set State (X) | Imposta lo stato statistico della macchina (behavior Statistics) | Statistics, State | PM_Process_Statements_Set_State |
| PM-36 | Set Next Flow Step (F) | Sovrascrive nel prodotto il prossimo step, per fare salti e loop | FlowInfoVariableName, ProductVariableName, ForceExactFlowStep, ForceExactProcessGroup | PM_Process_Statements_SetNextFlowStep |
| PM-37 | Set Product Visit State (F) | Imposta o incrementa il conteggio delle visite del prodotto ai processi | ProductVariableName, TargetProcessMode (All/Specific/Current), ValueMode Absolute/Relative, VisitCount, Processes | PM_Process_Statements_setProductVisitState |
| PM-38 | Start Transport In (T) | Come Transport In ma non bloccante: avvia l'arrivo e prosegue fino a un Wait Transport. Sorgente: solo il transport system | Mode (Transport Products / Process an Assembly), AssemblyOrderVariableName, AssemblyVariableName, ProductPositionFrame, ProductVariableName, ResourcePositionFrame, ReservedProductVariableName, filtro | PM_Process_Statements_Start_Transport_In |
| PM-39 | Start Transport Out (T) | Come Transport Out ma non bloccante | come PM-43, più SendAsProductBundles | PM_Process_Statements_Start_Transport_Out |
| PM-40 | Switch Case (F) | Esegue il primo case la cui condizione (operatore più valore; `==` se l'operatore manca) è soddisfatta dal valore dello switch, altrimenti il case "Default" | Condition, CaseCondition | PM_Process_Statements_Switch_Case |
| PM-41 | Transport In (T) | Fa entrare uno o più prodotti, bloccante. Modo Transport Products: il prodotto finisce nella variabile. Modo Process an Assembly: riempie gli slot di un assieme step per step (in ampiezza, per OrderIndex) e sfrutta i figli dei bundle in arrivo | Mode, Source (From Previous Process, cioè pull via link / From Component Container, cioè arrivo nel container o sul sensore del path), AssemblyOrderVariableName, AssemblyVariableName (se vuoto crea l'assieme), ProductPositionFrame, ProductVariableName, ResourcePositionFrame, ReservedProductVariableName, filtro | PM_Process_Statements_Transport_In |
| PM-42 | Transport Pattern In (T) | Fa entrare i pezzi in sequenza disposti a griglia, usando prima quelli prenotati. Sorgente: solo il transport system | Offset, PatternCount, PatternStep, ProductPositionFrame, ProductVariableName, ResourcePositionFrame, ReservedProductVariableName, filtro | PM_Process_Statements_Transport_Pattern_In |
| PM-43 | Transport Out (T) | Fa uscire i prodotti uno alla volta, oppure svuota gli slot di un assieme (in ampiezza inversa, per OrderIndex decrescente). È bloccante finché il pezzo non è uscito | ProductVariableName, Mode, AssemblyOrderVariableName, ResourcePositionFrame, Destination (To Next Process / To Component Container / To Transport Node), DestinationNode, Container, SendAsProductBundles | PM_Process_Statements_Transport_Out |
| PM-44 | Wait Property Condition (F) | Si blocca finché "variabile + condizione" diventa vera, rivalutando a ogni variazione. Se la variabile non esiste, prosegue | VariableName (variabile di processo o proprietà del componente proprietario), Condition | PM_Process_Statements_Wait_Property_Condition |
| PM-45 | Wait Signal (S) | Attende un segnale (Bool, Int, Real, String) con condizione e timeout opzionali, e può salvare il valore e l'esito del timeout | Component, Signal, WaitTrigger, Condition, Timeout (0 = infinito), TimedOutVariableName, ValueOutputVariableName | PM_Process_Statements_Wait_Signal |
| PM-46 | Wait Transport (T) | Attende la fine dei trasporti avviati con Start Transport In/Out. Fino a quel momento la variabile è in stato indefinito | ProductVariableName | PM_Process_Statements_Wait_Transport |
| PM-47 | While (F) | Ripete il blocco finché la condizione resta vera | Condition | PM_Process_Statements_While |
| PM-48 | Work (T) | Chiede a un transport controller una risorsa (ad es. un operatore) che lavori nel processo per un tempo dato | Controller, ProcessTime, ResourcePositionFrame, WorkPositionFrame | PM_Process_Statements_Work |
| PM-49 | Process Editor | Pannello per creare routine (pulsante "+"), con una sezione dei requisiti e una di definizione. Gli statement si nidificano e si spostano con drag & drop. C'è una galleria degli statement e l'accesso dalle etichette nel viewport 3D | Requirements, Process Definition, Variables, Statement Gallery | PM_Process_Editor |
| PM-50 | Process Flow Editor | Editor della sequenza di step per flow group. Mostra tra gli step un'icona del controller di trasporto: tre puntini se servono più controller, croce rossa se manca una route. I link si disegnano nel 3D cliccando etichette di processo o nodi. Si possono filtrare i link mostrati per gli step selezionati | Flow Group, Process Step, step opzionali, Visit Count, ProcessMode, Hide/Show, Filter Transport Links | PM_Process_Flow_Editor |
| PM-51 | Product Type Editor | Gestisce flow group, tipi, assiemi e le proprietà di default, di prodotto e di componente, con ricerca, copia e incolla tra tipi | Name, ParentProduct, ComponentUri, OriginFrame, proprietà tipizzate | PM_Product_Type_Editor |
| PM-52 | Assembly Editor | Ambiente 3D per definire gli slot di uno step: aggiunta singola o a pattern, template, riuso dei pattern, centratura, strumenti Move, Snap e Align, visualizzazione trasparente degli altri step | OrderIndex, Accept*, DefaultProductType, DefaultBoundsDimensions | PM_Product_Type_Editor |
| PM-53 | Product Creator (behavior) | Generatore di istanze | Modalità Single / Batch / Table (Excel) / Distribution | PM_Product_Type_Editor |
| PM-54 | Product Filtering | 4 tipi di filtro combinabili, usati da TransportIn, TransportPatternIn, StartTransportIn e Assembly Editor (e, nei fatti, da Buffer, Detach, Get/Select Products e Reserve) | Product Type, Product Property, Expression, Specific Product, FilterDiagnosticMode | PM_Product_Filtering |
| PM-55 | Variabili di processo | Variabili statiche e runtime, tipizzazione forte, liste, riferimenti che diventano Null | Boolean, Integer, Real, String più riferimenti | PM_Using_process_variables |
| PM-56 | Transport Controller | Smista i task di trasporto e di lavoro alle risorse | ResourcePriority, Strategy, LinkDefaults, UseCustomParameters | PM_Using_Resources_transport_controllers |
| PM-57 | Resource | Esegue i task, con capacità a bordo e tool | Capacity, LIFO, tool per link e per lavoro | PM_Using_Resources_resources |
| PM-58 | Navigation Controller (Beta) | Navmesh condivisa con gli ostacoli | MeshType, IsObstacle, Pathway Coverage | PM_Using_Resources_navigation_controller |
| PM-59 | Pathway Area/Curve (Beta) | Area navigabile, sensi unici, zone con limite di capacità e velocità | OneWay, AreaID | PM_Using_Resources_pathways |

---

## 2. Connectivity

### 2.1 Architettura generale [help] `CN_Terminology`, `CN_Accessing_Connectivity_feature`, `CN_Pairing_variables`, `CN_Locking_values`, `CN_Supported_data_types`

- **Attivazione**:
  - è un add-on che si accende da File, Options, Add On, Connectivity, e richiede il riavvio;
  - compare una tab ribbon "Connectivity";
  - dalla release note 5.0 (fuori scope, verificata solo con grep): OPC UA e MQTT sono disponibili anche in
    Professional, tutti i connettori solo in Premium.
- **Gerarchia a 3 livelli** nel pannello Connectivity Configuration:
  1. **Plugin**: l'adattatore di protocollo;
  2. **Connection/Server**: un'istanza di connessione verso un PLC o server, con un pulsante on/off;
  3. **Variable Group**: un insieme di coppie di variabili.
- **Variable Group**: ogni gruppo ha
  - una **Transmission direction**:
    - **Simulation → Server**, cioè scrittura verso il PLC;
    - **Server → Simulation**, cioè lettura dal PLC;
  - un **Update mode**:
    - **Cyclic**, con la proprietà "Cyclic update interval";
    - **Event-based**, che aggiorna quando un valore cambia;
  - uno stato attivo on/off;
  - una soglia "Delay warning threshold", citata in `CN_Kawasaki_Robot_Connector`.
  Ogni nuova connessione nasce con **due gruppi**, uno per direzione. Il raggruppamento esiste proprio per
  avere modi, direzioni e attivazione diversi.
- **Variabile di simulazione**: una proprietà o un segnale di un componente: proprietà di componente,
  proprietà di behavior, segnali, valori dei giunti (DOF), basi e tool del robot.
  - ID fino a 4 parti separate da punto: componente, eventuale DOF, behavior o segnale, proprietà.
  - Tipi lato simulazione: boolean, real, integer, string, matrix, vector. Gli interi VC sono **int32 con
    segno**.
- **Pairing**:
  - editor "Create Variable Pairs" a due tabelle: a sinistra la simulazione, filtrabile per componenti
    selezionati, proprietà di componente, proprietà di behavior e segnali; a destra il server, sfogliabile
    se il protocollo lo permette;
  - "Pair Selected" crea la coppia; una variabile di simulazione verso N variabili server si fa una coppia
    alla volta;
  - lato server compaiono un'icona di compatibilità, un "Data Type" (il tipo .NET più vicino) e un "Server
    Type" (il tipo nativo).
- **Conversione automatica dei tipi**, quando simulazione e plugin non coincidono:
  - Matrix: una 4x4 di double, column-major, angoli in gradi. Da string accetta 16 valori, 6 (X, Y, Z, W, P,
    R) o 3 (X, Y, Z); verso string emette sempre 16 valori separati da virgola;
  - Vector: X, Y, Z, W. Da string accetta da 1 a 4 valori, con W = 1 di default.
- **Pannello Connected Variables**, la tabella di debug con queste colonne:
  - Structure: nome della coppia, modificabile;
  - Simulation variable/type/value;
  - **Prepared value**: un valore preparato da scrivere in blocco con "Write Prepared Values";
  - **Latest value**: l'ultimo valore sincronizzato dopo la conversione;
  - Status: l'errore della coppia;
  - Server variable/type.
  Comandi: Add, Write Prepared, Move Variables (tra connessioni dello stesso plugin), Delete Selected,
  Update Selected.
- **Statistiche** nel pannello:
  - Average/Max **update time**: il tempo end-to-end, dal tick ciclico o dall'evento fino alla conferma di
    scrittura;
  - Average/Max **plugin time**: solo la parte del plugin, più vicina alla latenza di rete, esclusi i blocchi
    del thread di simulazione e del rendering;
  - Pairs with errors;
  - Errors on this run.
  In event-based server→sim, l'update time non include la latenza interna del server né quella di rete.
- **Frequenze dichiarate**: le statistiche si calcolano a **10 Hz**, mentre le variabili possono aggiornarsi
  **fino a 1000 Hz**, ed è conservato solo l'ultimo campione per coppia. Non è dichiarato nessun valore di
  default dell'intervallo ciclico.
- **Locking** (forzatura):
  - **Lock Selected** fissa il valore corrente della variabile di simulazione come valore inviato;
  - **Lock Prepared Values** fissa il valore preparato;
  - **Unlock Selected** toglie il blocco.
  A ogni sincronizzazione la destinazione riceve il valore fissato, non quello della sorgente. Si usa per
  simulare guasti di sensori o un motore sempre acceso, e si può fare a simulazione in corsa. È
  l'equivalente della forzatura in una watch table.
- **Configurazione**:
  - è salvata nel layout;
  - Export produce XML di tutte le connessioni; Import accetta XML o CFG;
  - comandi: Clear (rimuove tutte le connessioni), Reconnect, Disconnect, Edit Connection (task pane con
    "Test Connection").
- **Proprietà**:
  - della connessione: "Server Capabilities", con il flag Connected;
  - del plugin: "Connection Plugin Capabilities";
  - opzioni Start/Stop/Reset with simulation, citate per Beckhoff ed Epson, che legano lo stato del PLC
    o del task allo stato della simulazione.

### 2.2 Tabella dei connettori

| ID | Connettore | Protocollo/prodotto | Cosa permette | Requisiti/limiti dichiarati | Fonte |
|---|---|---|---|---|---|
| CON-01 | OPC UA | Client OPC UA verso un server esistente | Browsing dei nodi, pairing, lettura e scrittura, Test Connection | Endpoint sicuro opzionale (certificati X.509 su entrambi i lati), autenticazione username/password o certificato PFX, password salvabile in chiaro nella configurazione. Degli array si accede solo per singolo indice e serve il supporto all'accesso indicizzato. Bug noto se il server gestisce male i ContinuationPoint nel Browse | CN_Connecting_to_an_OPC_UA_Server, CN_Supported_data_types |
| CON-02 | Siemens S7 | S7comm (ISO-on-TCP, porta 102), PUT/GET | Lettura e scrittura delle aree I, Q, M di S7-300, 400, 1200 e 1500 e di S7-1500 simulate con PLCSIM Advanced. Import dei simboli da SDF o Excel esportati da TIA | Nessun browsing, nessuna verifica di esistenza, nessuna cifratura né password. Solo letture cicliche. Sul PLC servono "no protection" e PUT/GET abilitato. Dettagli in 2.3 | CN_Using_the_Siemens_S7_plugin, CN_Supported_data_types |
| CON-03 | Siemens SIMIT | "VC coupling" (external coupling SIMIT, TCP/IP) più plugin VC | Scambio di segnali con SIMIT SP, browsing, modi ciclico ed evento, più connessioni, gestione automatica del runtime | La demo SIMIT non supporta external coupling. Una sola istanza di coupling per progetto. Si accede solo ai segnali definiti nel coupling, con un editor di segnali separato. Bisogna aprire il coupling prima di connettersi. Avviare prima SIMIT. Con picchi di latenza gli eventi si perdono. Tipi: Binary, Integer (int64, ma VC usa int32), Analog | CN_Connecting_to_Siemens_SIMIT |
| CON-04 | WinMOD NET | Libreria proprietaria WinMOD Net (TCP/IP, porta di default 40001) | Più connessioni, browsing, eventi server→sim | WinMOD V7.2 build 40 o successiva, licenza KU200 su ogni installazione, libreria installata sul PC VC. Indirizzamento per progetto/layer/Communication Element/segnale: un rename richiede un nuovo pairing. Tipi Binary, Digital (int32), Analog (scala ±1e6, passo circa 0,0005) | CN_Connecting_to_WinMOD |
| CON-05 | Beckhoff ADS | ADS/AMS verso TwinCAT 2 e 3 | Pairing per nome simbolico completo, autoscan delle porte locali (801/811/821/831 per TC2, 851-860 per TC3) | Componenti ADS installati e route AMS configurata su entrambi i lati se remoto. L'errore "Nullable object" su progetti non PLC si aggira togliendo Start/Stop/Reset with simulation. Tipi IEC fino a LREAL, LINT/ULINT, STRING/WSTRING; REAL80 troncato a 64 bit | CN_Connecting_to_a_TwinCAT_PLC, CN_Supported_data_types |
| CON-06 | Allen-Bradley | EtherNet/IP/CIP [ipotesi: il protocollo non è nominato] | Micro800, CompactLogix e ControlLogix (reali, FactoryTalk Echo, simulatore CCW). Scoperta automatica dei tag globali e locali, membri di UDT e variabili esterne di AOI, PLC Variable Editor | MicroLogix e SLC disabilitati. GuardLogix possibile, ma VC non è safety-rated. Nessun RBAC o password: i tag vanno lasciati anonimi in R/W. Per ControlLogix serve un routing path (default 1.0 = slot 0) | CN_allen-bradley_plc |
| CON-07 | LS Electric | Libreria XGCommLib (COM) di XG5000 | R/W di variabili in area M, import dei simboli da XG5000 | XG5000 sullo stesso PC; default 127.0.0.1:2004. Nessun browsing, nessuna cifratura, nessun evento server→sim. Solo area M, interi trattati come senza segno, niente float, oltre 32 bit fallisce. Errore "Class not registered": registrare la DLL con regsvr32 | CN_ls_electric, CN_Supported_data_types |
| CON-08 | MQTT Client | MQTT 3.1.0 e 3.1.1 (testato sulle porte 1883 e 8883) | Publish/subscribe per topic, template JSON con pairing dei campi, formule di conversione (`Payload.x`, `Value[5]`, `@Array`, `@Object`, `@Json`), trigger su cambio, LWT, QoS, Retain, No Local, registrazione dei messaggi su file, import ed export CSV delle mappature, configurazione in JSON/XML | Nessun wildcard `+`/`#`. Elementi di array non mappabili direttamente (servono formule). "Update only in simulation" = false è possibile solo in event-based | CN_mqtt_client |
| CON-09 | ABB Robot | ABB PC SDK / Robot Communication Runtime | Lettura e scrittura di DI/DO, lettura dei giunti (JointTarget: RobJoint e ExtJoint; il 7° asse sta in ExtAx.Eax_a) | Servono i servizi ABB Communication/Discovery Server e l'opzione PC Interface sul controller reale. Mastership necessaria in Manual (con consenso dal FlexPendant). Eventi solo per i segnali. Nessuna scrittura dei giunti. La versione del Runtime deve corrispondere a RobotWare | CN_ABB_Robot_Connectivity_Plugin |
| CON-10 | Denso Robot | ORiN2 (b-CAP/CAO) | I/O, giunti (lettura), variabili globali I/F/D/V/J/P/T/S in R/W, con tabelle di conversione string↔tipo (separatore ";") | ORiN2 SDK con licenza. RC8 richiede il b-CAP listener e il file WPJ in locale; RC9 usa RCPJ. Solo ciclico. Nessuna scrittura dei giunti | CN_denso_robot |
| CON-11 | Doosan Robot | Doosan Robotics API (DLL incluse) | Lettura dei giunti, DI, AI e I/O del tool; scrittura di DO e AO | Reale o DART Studio/Simulator. Il controllo di autorità va chiesto all'altro client | CN_Doosan_Connectivity_Plugin |
| CON-12 | Epson Robot | RC+ 8.0 API (oggetto SPEL) | DI/DO in R/W, lettura dei giunti, gestione dei task (Start/Stop/Reset with simulation, Task Index), finestre RC+ incorporate, "Add Robot Mappings" automatico | RC+ 8.0 con licenza e API, opzione API sul controller reale. Un solo VRC alla volta e un solo SPEL per PC. Il caricamento del VRC può superare i 60 s | CN_epson_robot |
| CON-13 | FANUC Robot | FANUC PC DK / Robot Server | I/O digitali e analogici, I/O del braccio, variabili di sistema di tipo base, lettura dei giunti. Lo Scattered Access accorpa le richieste; un thread per gruppo | Opzione PC DK (A08B-9410-J535 o J536) e opzione PC interface sul controller. Server→sim solo ciclico. Gli I/O accoppiati passano in modalità simulata e non vengono resettati. Il browse delle variabili di sistema può richiedere 5-10 minuti | CN_Fanuc_robot_plugin |
| CON-14 | Kawasaki KRCC | Kawasaki KRCC library | Lettura di giunti e I/O, scrittura delle uscite (OUT 1-32, IN 1001-1032) | Porte 9105, 9205, 9305, ... Solo K-ROSET in inglese. Su VRC la latenza media supera i 50 ms: alzare il Delay warning threshold | CN_Kawasaki_Robot_Connector |
| CON-15 | MELFA (Mitsubishi) | RT ToolBox3 o scheda Ethernet | DI/DO in R/W, lettura dei giunti con mapping automatico dei giunti, gruppi di segnali per range (ad es. In0, 8 punti) | RT ToolBox3 serve solo per il controller virtuale. CR800. Nessuna scrittura dei giunti | CN_melfa_robot |
| CON-16 | Nachi Robot | Interfaccia Nachi (porta 9876 e successive) | I/O standard in R/W (I/O fissi in sola lettura), giunti per meccanismo, variabili Integer, Real e String | FD on Desk III, controller FD e CFD. Solo ciclico. Nessuna scrittura dei giunti | CN_nachi_robot |
| CON-17 | Omron Robot | ACE API 4.x (porte 43500, 43600, ...) | DI/DO, giunti, pose world/base/tool (matrix) | ACE 4.x sullo stesso PC; ACE 3 non supportato, robot TM esclusi. Solo Viper 650/850 e SCARA. Eventi solo sim→server | CN_omron_robot |
| CON-18 | Techman Robot | TMcraft Shell (locale) / TMcraft Service (remoto, porta 51234) | DI/DO/AI/AO di control box e end module, lettura dei giunti | TMflow/TMcraft 2.16. Per il remoto il servizio va importato **ed eseguito** sul controller. Eventi solo sim→server | CN_techman_robot |
| CON-19 | Universal Robots RTDE | RTDE, binario su TCP 30004, 125 Hz | Tutte le variabili RTDE, con accesso a singoli bit e componenti di vettori (lettura dei vettori come stringa), modello del server hardcoded (controller 3.3) | CB3 con software 3.2.19171 o successivo. Un solo output recipe e fino a 255 input recipe: ogni modifica mette in pausa e riconfigura. Angoli in radianti (il robot VC usa il segnale "RTDEJoints" mappato su target_q). Variabile e maschera nello stesso gruppo. Un solo client per input. Timing di Connectivity non affidabile | CN_Universal_Robots_RTDE_Plugin |
| CON-20 | Yamaha Robot | RCX3-SDK (porte 5000x) | Lettura di DI e SI; R/W di DO, LO, MO, SO e TO (DO_0 è di sistema); giunti convertiti da impulsi in gradi con la formula Lead/(EncPulse·Gear) definita in un file di configurazione | Serve Motor Power ON. RCX 3. Nessuna scrittura dei giunti | CN_yamaha_robot |
| CON-21 | Yaskawa Robot | Plugin MotoSim (VRC API + gRPC) | Lettura di giunti, I/O esterni e variabili di programma; scrittura di ingressi esterni e variabili; file di configurazione Yaskawa.json; workaround con il componente "MotoSim Joint Converter" | Richiede MotoSim EG-VRC, che fa anche da ponte verso il robot reale. Bug dei giunti in MotoSim v2022 | CN_Yaskawa_Robot_Connector |
| CON-22 | Stäubli add-on | SOAP/HTTP verso CS8 e CS9 (emulatore SRS o reale) | Mapping del robot (stesso numero di giunti), I/O mapping (din/dout/ain/aout) con un punteggio automatico per tipo e nome, modi Idle/Virtual/Polling/Jogging, post-processor VAL 3 | CS8 7.0 minimo (7.7 consigliato), SOAP 2.1 o successivo per il modo Virtual. Login di default "Default" (password "" su CS8, "default" su CS9). Porte 5653 e SOAP 851. Servono ioWriteAccess, 123KeysControl e armWriteAccess. Timing di sincronizzazione 20 ms di default (salvato nelle impostazioni utente, non nel layout). La vista degli I/O aggiorna a circa 5 Hz. Nessun accesso alle variabili di programma | CN_Stäubli_Add-on_* (5 file) |

Funzione trasversale ai robot: il **Postprocessor** (ribbon PROGRAM, Postprocess) traduce il programma VC
nel linguaggio nativo del robot. È descritto in modo identico in ABB, Denso, Fanuc, Kawasaki, MELFA, Nachi,
Omron, Techman, UR, Yamaha e Yaskawa. Per Yaskawa ci sono in più l'opzione di gruppo "SYNCHRO OFF" e il
Pulse Setter. Il post-processor VAL 3 Stäubli:
- traduce Assign, Call, Comment, Define Base/Tool, Delay, Halt (→stopMove), If, Lin/PTP, Print, Return,
  Set/Wait Binary e While; Break e Continue non sono supportati;
- inserisce `waitEndMove()` tra blocchi di moto e altre istruzioni;
- riconosce le parole chiave nei commenti (`lastMove`, `timer`, `waitEndMove()`);
- traduce le variabili di routine in dizionari per tipo (ad es. `bPick["Skip"]`).

### 2.3 Note dettagliate: Siemens S7 (rilevante per S7-1200 G2 / TIA V21 / PLCSIM)

Tutto ciò che segue è **[help]**, da `CN_Using_the_Siemens_S7_plugin.txt` e `CN_Supported_data_types.txt`,
salvo dove è marcato.

- **Protocollo**: S7comm classico con le funzioni PUT/GET, su TCP/IP, verso la **porta 102**, IPv4. L'help
  rimanda al "Sharp 7 reference manual" per rack e slot dell'S7-400. [ipotesi] Il plugin è quindi
  probabilmente basato su Sharp7, il porting .NET di Snap7.
- **CPU supportate e parametri di connessione**:

  | CPU | Rack | Slot |
  |---|---|---|
  | S7-300 | 0 | 2 |
  | S7-400 | dipende dalla configurazione | dipende dalla configurazione |
  | S7-1200 | 0 | 0 oppure 1 |
  | S7-1500 | 0 | 0 oppure 1 |
  | S7-1500 simulata con **PLCSIM Advanced** | come S7-1500 | come S7-1500 |

  **PLCSIM classico non è citato**. Nemmeno l'**S7-1200 G2** è citato esplicitamente: l'help parla
  solo di "S7-1200".
- **Configurazione del PLC richiesta in TIA**:
  1. interfaccia Ethernet con l'IP;
  2. Protection & Security con livello di accesso "no protection" (full access);
  3. abilitazione di **PUT/GET**;
  4. (opzionale) disabilitazione dell'aggiornamento degli ingressi fisici, così la simulazione può scrivere
     nell'immagine di processo degli ingressi senza che il ciclo di I/O la sovrascriva.
- **Indirizzamento**: **assoluto per area di memoria**. Le uniche aree supportate sono:
  - **I/E**: immagine di processo degli ingressi;
  - **Q/A**: immagine di processo delle uscite;
  - **M**: merker.
  **I DB non sono elencati tra le aree supportate**. L'help non li esclude esplicitamente, ma l'elenco è
  chiuso.
- **Simbolico**: il protocollo non fa browsing. I nomi simbolici arrivano **importando la tag table** di TIA
  esportata in **SDF o Excel** (comando "Load PLC symbols from file").
  - I tag importati sono salvati nella configurazione e nell'XML esportato.
  - Dopo un nuovo import, le coppie esistenti vengono **riallineate per nome del tag**: se cambiano
    indirizzo o tipo, il pairing resta valido.
  - I nomi duplicati non vengono aggiornati e producono un errore nell'Output.
  - Va ricaricato l'albero del server nella finestra Add Variables.
- **Tipi STEP 7 supportati**:

  | Tipo STEP 7 | Tipo .NET |
  |---|---|
  | BOOL | Boolean |
  | BYTE | Byte |
  | WORD | UInt16 |
  | DWORD | UInt32 |
  | SINT | SByte |
  | USINT | Byte |
  | INT | Int16 |
  | UINT | UInt16 |
  | DINT | Int32 |
  | UDINT | UInt32 |
  | REAL | Single |

  **Non elencati**: LREAL, LINT, ULINT, LWORD, STRING, WSTRING, TIME, DTL, CHAR, gli array e gli UDT.
- **Limiti dichiarati**:
  - non si scoprono le variabili del PLC;
  - non si verifica che un indirizzo sia davvero usato dal programma;
  - non c'è cifratura né password, quindi va usato solo su una rete sicura.
- **Prestazioni e ciclo**:
  - le richieste S7comm vengono servite tra un ciclo PLC e l'altro: con il carico CPU vicino al 100% la
    comunicazione rallenta. Il consiglio è **aumentare il tempo di ciclo minimo** del PLC;
  - **PDU**: 240 byte su S7-1200, 480 su S7-1500. Ogni richiesta ha un costo minimo quasi fisso, quindi
    conta il numero di richieste;
  - sull'S7-1200 fisico, S7comm è dichiarato **molto più veloce dell'OPC UA integrato**;
  - la rete aggiunge jitter: preferire il cavo diretto. Un PLC simulato in una VM degrada le prestazioni.
  - Nessun numero di update time in ms è dichiarato.
- **Letture**: **solo cicliche**, perché il protocollo non ha sottoscrizioni. A ogni ciclo vengono letti
  tutti i tag del gruppo. Il plugin **accorpa in blocchi** i tag adiacenti o sovrapposti della stessa area,
  dentro richieste MultiRead.
  - Sono adiacenti: MW0 e MW2; I0.7 e I1.0.
  - Non sono adiacenti: I0.0 e I0.2; I0.0 e M0.1.
  - Si sovrappongono: IW0 e I1.2.
  L'ottimizzazione vale per singolo variable group, quindi conviene un solo gruppo server→sim.
- **Scritture**:
  - usare il modo **ciclico**, perché accorpa le scritture in MultiWrite; scegliere un intervallo
    ragionevole;
  - evitare valori che cambiano di continuo, come i giunti;
  - scrivere gli I/O come **interi senza segno** (BYTE o WORD) invece che come singoli BOOL. Serve uno
    script VC che impacchetti i bit; i tag possono sovrapporsi, quindi si possono avere anche i singoli
    bit;
  - gli interi VC sono int32 con segno, quindi si possono scrivere al massimo 31 bit. In pratica conviene
    una variabile ogni 16 bit.
- **[ipotesi, da verificare sul campo]** Punti da controllare per S7-1200 G2, TIA V21 e PLCSIM:
  - se la G2 (FW 1.x) espone ancora PUT/GET con "full access / no protection", visto il nuovo modello di
    sicurezza e la confidential PLC configuration delle CPU recenti;
  - se gli slot 0 e 1 valgono anche per la G2;
  - se il PDU della G2 è 240 come per la 1200 classica o più grande;
  - se è raggiungibile **PLCSIM V21 non-Advanced**: normalmente non ha uno stack TCP/IP esterno, quindi per
    S7comm servirebbe PLCSIM Advanced, che però l'help cita solo per S7-1500;
  - che i DB non siano accessibili, cosa che obbliga a lavorare su I, Q e M e a copiare nel programma PLC.

### 2.4 Note dettagliate: OPC UA

- **Ruolo**: VC è un **client** OPC UA verso un server già in esecuzione (ad es. il server OPC UA integrato in
  una CPU S7-1500 o S7-1200, oppure un gateway). [help]
- **Connessione**: si imposta l'indirizzo del server (endpoint URL) e si possono attivare:
  - "Use secure endpoint": sceglie l'endpoint migliore, con certificati X.509 accettati reciprocamente;
  - autenticazione Username/Password oppure Certificato (file PFX più password). Con "Save password" la
    password viene salvata **in chiaro** nella configurazione, mai la chiave.
  Poi Test Connection, e l'attivazione con il pulsante Power (Connected = True). [help]
- **Indirizzamento**: **simbolico per nodo**, con browsing dell'albero. Il nome visualizzato è il nome del
  nodo più un indice per gli array (`Example[0]`), se il nodo sta sul server principale. [help]
- **Tipi**:

  | Tipo OPC UA | Tipo .NET |
  |---|---|
  | Boolean | Boolean |
  | Byte | Byte |
  | SByte | SByte |
  | Int16 | Int16 |
  | Int32 | Int32 |
  | Int64 | Int64 |
  | UInt16 | UInt16 |
  | UInt32 | UInt32 |
  | UInt64 | UInt64 |
  | Float | Single |
  | Double | Double |
  | Integer | Int32 |
  | UInteger | UInt32 |
  | String | String |
  | LocalizedText | String (lingua di default) |

  Gli **array** si leggono e si scrivono solo per elemento e richiedono un server che supporti l'accesso
  indicizzato su array statici; un array intero non si trasferisce. Non sono menzionati struct o UDT, né
  DateTime. [help]
- **Update rate**: non è dichiarato per OPC UA. Valgono i modi generali ciclico ed evento. L'help non
  specifica se l'evento usa subscription o monitored item, né i parametri di sampling o publishing.
  [help: assenza]
- **Troubleshooting**: se il server pagina male il Browse (ContinuationPoint, nell'esempio batch da circa
  166 elementi), il client non riesce a leggere i metadati dei nodi. Va corretto il server. [help]
- **PLCSIM Advanced**: non è nominato nella pagina OPC UA. [ipotesi] Una CPU S7-1500 simulata in PLCSIM
  Advanced con il server OPC UA attivo è raggiungibile come qualunque server; per la S7-1200 G2 va
  verificato se e come è disponibile il server OPC UA (licenza runtime, porta 4840).

---

## 3. Note per l'implementazione

1. **Nucleo PM come VM a coroutine**: ogni Process Executor è un interprete con una routine attiva.
   - Gli statement bloccanti (TransportIn/Out, Delay, Wait*, Work, Run Robot Routine, Move Joint) vanno
     modellati come `await` sul tempo di simulazione.
   - Start/WaitTransport sono future esplicite.
   - Le Requirements sono predicati valutati in modo atomico, cioè tutti veri nello stesso istante, con
     prenotazione (ReserveProduct) per evitare race.
2. **Routing**: serve un grafo orientato di Transport Node e Link per ogni flow group, con un controller per
   link e il calcolo di una Transport Solution (cammino minimo). La semantica è pull (richiesta a monte da
   parte del ricevente con capacità) oppure push (container o nodo). Da riprodurre fedelmente:
   - la croce rossa quando la route non esiste;
   - i tre puntini quando servono più controller.
3. **Dispatcher delle risorse**: politiche Nearest e LeastUtilized, strategie di raccolta e consegna
   1:1, 1:N, N:1 e N:N, capacità e LIFO. La navmesh (Recast/Detour o equivalente) è di livello "beta" in VC:
   può arrivare in una seconda fase.
4. **Assiemi**: albero di step, pattern di slot con OrderIndex (stesso indice = parallelo), Assembly Order
   come bitmask di step. Il riempimento è in ampiezza, lo svuotamento in ampiezza inversa. I bundle sono
   prodotti senza metadati.
5. **Espressioni**: serve un linguaggio di espressioni comune (condizioni, CaseCondition "op valore",
   distribuzioni con stream di seme, accesso a `Product.Prop`). La pagina "Process Statement Expressions"
   **non è nel dump**: la sintassi va riprogettata (vedi Lacune).
6. **Connectivity**: astrazione a plugin con le interfacce `Connect`, `Browse` (opzionale), `ReadMany`,
   `WriteMany` e `Subscribe` (opzionale), più le capabilities dichiarate: eventi in una o entrambe le
   direzioni, browsing, sicurezza. Il core gestisce:
   - i variable group, con direzione, modo, intervallo, enable e soglia di ritardo;
   - la conversione dei tipi;
   - lock e prepared value;
   - le statistiche (update e plugin time, 10 Hz).
   Un thread di I/O per gruppo, come fa FANUC, separato dal thread di simulazione.
7. **S7 per il target utente** (S7-1200 G2):
   - VC si ferma alle aree I, Q e M con REAL massimo. Un clone guadagnerebbe valore supportando **DB
     (anche ottimizzati, via S7CommPlus simbolico o OPC UA)**, LREAL e i tipi lunghi. [ipotesi] Il workspace
     ha già un client S7CommPlus simbolico (`plc_test/boxline3d` nel repository `chetto1983/apr`), da valutare come base;
   - l'import della tag table da SDF o Excel, con riallineamento delle coppie per nome, va replicato;
   - [ipotesi] un import da TIA via Openness (tiaportal-mcp) eliminerebbe l'export manuale;
   - l'accorpamento delle letture per blocchi contigui e le MultiRead/MultiWrite vanno replicati,
     rispettando il limite PDU;
   - il bit-packing degli I/O in WORD è nativo nel clone, non uno script utente.
8. **Coerenza con PLCSIM**:
   - prevedere un modo **sincrono o lock-step** (simulazione e PLC sullo stesso clock virtuale), come il
     modo "Virtual" di Stäubli, dove il controller avanza con il clock della simulazione. VC lo offre solo
     per Stäubli e non per i PLC;
   - [ipotesi] PLCSIM Advanced espone un'API di sincronizzazione del tempo virtuale, utile per un
     lock-step deterministico.
9. **Sicurezza**:
   - non salvare password in chiaro, come invece fa VC per OPC UA;
   - avvisare quando il PLC deve essere in "no protection" con PUT/GET;
   - forzare una conferma prima di scrivere su I/O "di sistema" dei robot.
10. **Robot**: lo schema ricorrente è "leggo i giunti, leggo e scrivo l'I/O, non scrivo mai i giunti
    (sicurezza)". Conviene un plugin generico "robot" con mapping automatico dei DOF, come in Epson "Add
    Robot Mappings" e MELFA, e conversioni di unità (radianti per UR, impulsi per Yamaha e Yaskawa).
11. **Configurazione**: va salvata nel layout ed esportabile e importabile (XML in VC; nel clone JSON o YAML
    con versioning). Import ed export CSV delle mappature come in MQTT.

## 4. Lacune

- **Pagine assenti nel dump o rinviate**:
  - "Process Statement Expressions": la sintassi, gli operatori e la precedenza delle espressioni PM sono
    citati ovunque ma la pagina non c'è;
  - "How to use – Python Statement" (API Python 3, fuori scope);
  - la pagina "Translate robot program" (OLP).
- **Statement parzialmente documentati**:
  - in FindFlowStep e GetFlowStep mancano i valori enumerati di `Mode`, `SearchDirection` e
    `FlowStepSource`;
  - in Set Product Visit State la descrizione di ProductVariableName sembra un copia-incolla errato
    dell'help;
  - "Run Robot Routine" ha una descrizione troncata ("A.") nella galleria;
  - la pagina Run Robot Routine cita "istruzioni" per l'uso dallo stesso robot in Python, ma il testo non
    è nel dump.
- **Connectivity**:
  - nessun valore di default dell'intervallo ciclico, nessun minimo, nessun update rate tipico in ms per S7
    e OPC UA;
  - non è descritto il formato XML/CFG della configurazione;
  - non è detto se l'OPC UA event-based usi subscription, né con quale sampling interval; non sono
    menzionati namespace, NodeId e tipi strutturati;
  - non sono descritti i formati SDF ed Excel accettati per la tag table S7;
  - **i DB S7 non sono menzionati**: l'elenco delle aree è chiuso (I, Q, M);
  - l'S7-1200 G2 non è citato; PLCSIM classico non è citato; PLCSIM Advanced è citato solo per S7-1500;
  - per Allen-Bradley non è nominato il protocollo;
  - per Doosan, Kawasaki e Yaskawa mancano le tabelle dei tipi;
  - le immagini (schermate delle impostazioni TIA, coupling SIMIT) non sono nel dump: i passi TIA sono
    noti solo a parole.
- **Process Modeling**:
  - mancano le proprietà complete dei controller Mobile, Human e Vehicle (Beta) e dei componenti eCatalog;
  - il formato Excel della modalità Table del Product Creator non è specificato;
  - non sono descritte le statistiche (behavior Statistics, stati).
