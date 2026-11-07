# Visual Components 5.1: mappa funzionale del Component Modeling (clean-room) — parte 2: feature, geometria, materiali, fisica, note

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


Parte 1 (modello dati e behavior): [02a](02a-component-modeling-modello-behavior.md).

### 2.12 Features (CM-FEAT)

Quasi tutte le feature hanno anche `Physics` (un gruppo di proprietà fisiche che comprende il
`Collider`, vedi CM-PHY-03). Le primitive hanno `Visible: bool` e `Material: ref<Material>`.

| ID | Nome | Cosa fa | Proprietà chiave | Segnali/eventi | Fonte |
|---|---|---|---|---|---|
| CM-FEAT-01 | Angular Clone | Replica le feature figlie attorno al proprio asse Z, di norma insieme a un Transform che dà il raggio. | Count: int (0 nasconde, 1 solo l'originale, n+1 l'originale più n copie); Step: real (gradi); Offset; Physics | — | Features_Angular_Clone.txt |
| CM-FEAT-02 | Block | Parallelepipedo. | Visible; Material; Length (X), Width (Y), Height (Z): real/expr; Physics | — | Features_Block.txt |
| CM-FEAT-03 | Boolean | Operazione CSG fra le prime due feature figlie. Il risultato finisce nei geometry set della prima. | Visible; Operation: enum{Intersection, Union, Subtraction}; Material | — | Features_Boolean.txt |
| CM-FEAT-04 | Cone | Cono o tronco di cono, eventualmente a settore; con poche sezioni diventa una piramide. | Visible; Material; BottomRadius; TopRadius; Height; StartSweep/EndSweep (gradi); Sections: int; Caps: bool; Fill: bool; Physics | — | Features_Cone.txt |
| CM-FEAT-05 | Cylinder | Cilindro o settore cilindrico; con poche sezioni diventa un prisma. | Visible; Material; Radius; Height; StartSweep/EndSweep; Sections; Caps; Fill; Physics | — | Features_Cylinder.txt |
| CM-FEAT-06 | Extrude | Estrude la geometria delle figlie lungo una direzione. | Material; Caps; Reverse; ShowBackFaces; Direction: vector; Length; Physics | — | Features_Extrude.txt |
| CM-FEAT-07 | Frame | Punto di riferimento con posizione e orientamento, usato dai behavior (path, sensori, interfacce, base e tool). | Visible; UseAsGuide: bool | — | Features_Frame.txt |
| CM-FEAT-08 | Geometry | Contenitore di mesh importate, generate o collassate; carica anche CAD da Uri. | Visible; Uri (cambiarlo cancella la geometria esistente); OnDemandLoad: bool; ShowBackfaces; CreaseAngle: real; Pickable: bool; Physics | — | Features_Geometry.txt |
| CM-FEAT-09 | Linear Clone | Replica le figlie lungo una direzione; se ne possono concatenare più per ottenere griglie. | Count; Step: real (distanza); Direction: vector; Physics | — | Features_Linear_Clone.txt |
| CM-FEAT-10 | Mirror | Specchia e scala le figlie su uno, due o tre assi. | HideOriginal: bool; Direction (direzione e fattore di scala); Physics | — | Features_Mirror.txt |
| CM-FEAT-11 | Plane | Superficie piana 2D, per visualizzazione o picking. | Visible; Size (X,Y); Physics | — | Features_Plane.txt |
| CM-FEAT-12 | Python Feature | Genera geometria e/o trasforma le figlie via script. | Script; Physics. Callback OnEvaluate(feature, target, matrix, containers); il default passa la geometria delle figlie al target applicando la matrice | OnEvaluate a ogni rebuild dell'albero | Features_Python_Feature.txt |
| CM-FEAT-13 | Revolve | Rivoluziona la geometria delle figlie attorno a un asse. | Material; Sections; RevolveAngle; RevolveAxis; RevolveOffset; Caps; Reverse; Physics | — | Features_Revolve.txt |
| CM-FEAT-14 | Sphere | Sfera o settore sferico. | Visible; Material; Radius; StartSweep/EndSweep; Sections (longitudinali); Spans (tra i poli); Caps; Physics | — | Features_Sphere.txt |
| CM-FEAT-15 | Switch | Abilita una sola figlia secondo un'espressione. | Choice: expr. Bool: True = prima figlia, False = nessuna. Int: indice a base 1 (0 = nessuna). Real: arrotondato per difetto. String: la figlia con quel nome, altrimenti nessuna. Physics | — | Features_Switch.txt |
| CM-FEAT-16 | Text | Testo 3D calcolato da un'espressione (etichette). | Visible; Material; Text: expr; Size: real; IsFloating: bool; IsReadOnly: bool; Physics | — | Features_Text.txt |
| CM-FEAT-17 | Transform | Applica alle figlie una matrice definita da un'espressione: traslazioni e rotazioni concatenate con `.`, secondo la precedenza degli operatori. | Visible; Expression: expr (matrix); Physics | — | Features_Transform.txt |
| CM-FEAT-18 | Wedge | Cuneo con lunghezze e altezze diverse sopra e sotto, specchiabile. | Visible; Material; BiggerLength, SmallerLength (X); BiggerHeight, SmallerHeight (Z); Width (Y); MirrorBack: bool; MirrorDown: bool; Physics | — | Features_Wedge.txt |

### 2.13 Operazioni sulla geometria (CM-GEO)

| ID | Nome | Cosa fa | Parametri chiave | Fonte |
|---|---|---|---|---|
| CM-GEO-01 | Collapse features | Fonde una feature e le sue figlie in un'unica Geometry, eliminando la gerarchia. Con più feature selezionate, ognuna si collassa separatamente. | — | Geometries_Collapsing_features.txt |
| CM-GEO-02 | Create Collision Geometry | Genera geometria di collisione semplificata per i link scelti. | Links to simplify; Method: enum{Convex hulls, Blocks}; Quality; Precision mode: bool; Max geometry set count: int; Preview/Create | Geometries_Creating_Collision_Geometry.txt |
| CM-GEO-03 | Decimate | Riduce la tassellazione fondendo spigoli e vertici entro una tolleranza; il risultato va nell'Output. | Max error; Quality | Geometries_Decimating_geometry.txt |
| CM-GEO-04 | Extract → Component | Sposta le feature o i link selezionati in un nuovo componente (per dividere un assieme CAD). | — | Geometries_Extracting_geometry_into_a_new_component.txt |
| CM-GEO-05 | Extract → Link | Crea un nuovo nodo con le feature selezionate: creazione del link e spostamento della geometria in un solo passo. | — | Geometries_Extracting_geometry_into_new_node.txt |
| CM-GEO-06 | Invert | Inverte le normali della geometria scelta (solo dentro una Geometry). | Invert level: enum{Set, Faces, Face} | Geometries_Inverting_geometry.txt |
| CM-GEO-07 | Merge faces | Salda i punti dei bordi aperti entro un raggio, per chiudere crepe e geometria non-manifold. I bordi devono essere opposti e della stessa feature; i bordi con più vicini vengono saltati. | Tolerance | Geometries_Merging_faces.txt |
| CM-GEO-08 | Merge features | Riunisce più Geometry in una sola. | — | Geometries_Merging_features.txt |
| CM-GEO-09 | Simplify | Sostituisce la geometria con il suo ingombro. | Blockify / Cylindrify / Convex Hull | Geometries_Simplifying_geometry.txt |
| CM-GEO-10 | Slice | Taglia la geometria con un piano e la divide in nuove Geometry; una primitiva tagliata viene collassata. | Snap Type; Plane Offset; Flip Offset; Slice Coplanar | Geometries_Slicing_geometry.txt |
| CM-GEO-11 | Split | Sposta la geometria selezionata in una nuova feature (solo dentro una Geometry). | Split level: enum{Set, Faces, Face} | Geometries_Splitting_features.txt |

### 2.14 Materiali (CM-MAT)

**Modello del materiale** (`Materials_Material_Properties.txt`): un materiale PBR con queste
proprietà:
- `Name` e `Texture Scale`;
- `Base Color` e Base Color Texture;
- `Roughness` e Roughness Texture;
- `Metallic` e Metallic Texture;
- `Normals Texture`;
- `Opacity` e `Index of Refraction`;
- `Clearcoat Strength` e `Clearcoat Roughness`.

Ogni texture ha un valore costante di riserva, da tenere coerente con la texture.

**Ereditarietà** (`Materials_Material_Inheritance.txt`):
- un materiale si può assegnare a un geometry set, a una feature, a un link o a un componente;
- ciò che non ha un materiale proprio lo eredita dalla feature o dal nodo padre;
- un oggetto senza alcun materiale si evidenzia in arancione;
- i materiali stanno in tre gruppi: **Layout** (in uso nella scena, editabili), **User** (libreria
  utente in `Materials.xml`, editabili) e **Library** (libreria di sistema, sola lettura).

| ID | Operazione | Cosa fa | Fonte |
|---|---|---|---|
| CM-MAT-01 | Material Properties | Modello PBR descritto sopra, con anteprima nel 3D. | Materials_Material_Properties.txt |
| CM-MAT-02 | Material Inheritance | Regole di ereditarietà e gruppi Layout/User/Library. | Materials_Material_Inheritance.txt |
| CM-MAT-03 | Assign | Assegna un materiale all'oggetto cliccato (scelta del livello: faccia, set, feature, nodo, componente). | Materials_Assigning_Material.txt |
| CM-MAT-04 | Clear | Toglie il materiale assegnato; l'oggetto torna a ereditare. | Materials_Clearing_Material.txt |
| CM-MAT-05 | Pick | Legge il materiale dell'oggetto cliccato (contagocce). | Materials_Picking_Material.txt |
| CM-MAT-06 | Select All Using | Seleziona tutti gli oggetti che usano un dato materiale. | Materials_Selecting_Materials.txt |
| CM-MAT-07 | Set Selected | Sostituisce in blocco un materiale con un altro sugli oggetti selezionati. | Materials_Setting_Materials.txt |
| CM-MAT-08 | Delete from layout | Toglie il materiale da tutti gli oggetti; se non è in libreria, lo rimuove del tutto. | Materials_Deleting_Materials_from_Layout.txt |
| CM-MAT-09 | Add new | Crea un materiale nel layout o nella libreria utente. | Materials_Adding_New_Materials.txt |
| CM-MAT-10 | Add to User Library | Copia un materiale del layout nella libreria utente. | Materials_Adding_Material_to_User_Library.txt |
| CM-MAT-11 | Remove from User Library | Lo toglie dalla libreria utente, ma non dal layout. | Materials_Removing_Material_from_User_Library.txt |
| CM-MAT-12 | Search | Cerca un materiale per parola chiave in Layout, User e Library. | Materials_Searching_for_Material.txt |
| CM-MAT-13 | Display list/grid | Mostra i materiali a lista o a griglia. | Materials_Displaying_Materials.txt |
| CM-MAT-14 | Add Decal | Crea una nuova Geometry rettangolare con UV su una superficie, a cui poi si assegna un materiale con texture. | Width, Height, U/V Tiling, Offset. Fonte: Materials_Adding_a_Decal.txt |
| CM-MAT-15 | Texture Mapping | Assegna coordinate UV con proiezione sferica, cilindrica o planare; la proiezione si muove con un manipolatore. | Mode; Width/Height (spazio UV); U/V Tiling. Fonte: Materials_Assiging_Texture_Coordinates.txt |

### 2.15 Mondo fisico (CM-PHY)

| ID | Argomento | Contenuto | Fonte |
|---|---|---|---|
| CM-PHY-01 | Physics World | Il mondo di simulazione (cinematico, a eventi) convive con un mondo fisico (NVIDIA PhysX). La fisica è attiva solo a simulazione in corsa e il reset annulla i suoi effetti. I quattro Physics Type stabiliscono a quale mondo appartiene un nodo. | Physics_Physics_world.txt |
| CM-PHY-02 | Cables | La geometria del cavo è fatta di capsule generate in automatico (non stanno nell'albero delle feature). I pin sono nodi con Physics Joint e un'interfaccia Attachment nascosta, interattivi e agganciabili a qualsiasi nodo. Un'interfaccia Attachment creata dall'utente non può fare da estremo del cavo. | Physics_Cables_in_physics_world.txt |
| CM-PHY-03 | Colliders | Ogni feature ha una proprietà Collider: enum{Undefined, #None, #Box, #Boxes (solo sulla root feature), #Precise (mesh)}. La root feature può forzare il collider in tutto il nodo. Se manca una Physics Entity viene creata in automatico. | Physics_Colliders.txt |
| CM-PHY-04 | Collisions | Collidono solo i nodi che hanno sia un collider sia una Physics Entity. Il Collision Group filtra (stesso gruppo = collidono) e lo Scene Id isola dei layer. | Physics_Collisions.txt |
| CM-PHY-05 | Joints | Physics Joint D6 nel nodo il cui offset definisce il giunto. | Physics_Joints_in_physics_world.txt |
| CM-PHY-06 | Paths | Vincoli del Physics Path: i pezzi devono avere una Physics Entity In Physics, e il path stesso una Physics Entity Kinematic. Senza collider i pezzi attraversano il path. Consigli: #Boxes sulla root feature, oppure #Box con padding; #Precise è rischioso. Una Speed negativa inverte il senso. | Physics_Paths_in_physics_world.txt |

### 2.16 Operazioni di modellazione e strumenti (CM-OP)

| ID | Operazione | Cosa fa | Fonte |
|---|---|---|---|
| CM-OP-01 | New component | Crea un componente vuoto nell'origine del mondo; consigliato aggiungere un Frame d'origine. | Modeling_Operations_Creating_a_new_component.txt |
| CM-OP-02 | Create Link | Crea un nodo figlio del nodo selezionato. | Modeling_Operations_Creating_a_new_node.txt |
| CM-OP-03 | New feature | Crea una feature nel nodo selezionato. | Modeling_Operations_Creating_a_new_feature.txt |
| CM-OP-04 | New behavior | Crea un behavior del tipo scelto nel nodo selezionato. | Modeling_Operations_Creating_a_new_behavior.txt |
| CM-OP-05 | New property | Crea una property del tipo scelto nel nodo radice. | Modeling_Operations_Creating_a_new_property.txt |
| CM-OP-06 | Define joint offset | Definisce l'offset del nodo con il manipolatore (che mostra l'asse del giunto), con Snap, con Align o con un'espressione parametrica. | Modeling_Operations_Defining_a_joint_offset.txt |
| CM-OP-07 | Assign joint to controller | Aggiunge giunti, per pick o da elenco, alla lista ordinata Joints di un Servo o Robot Controller; poi vanno impostati Max Speed, Acceleration e Deceleration. | Modeling_Operations_Assigning_a_joint_to_a_controller.txt |
| CM-OP-08 | Interact with joint | Jog automatico dei giunti non Fixed; il Jog Info, se c'è, ha la precedenza. | Modeling_Operations_Interacting_with_a_joint.txt |
| CM-OP-09 | Add/edit base frames | Aggiunge, modifica e rinomina i base frame nel sotto-elemento Bases di un behavior. | Modeling_Operations_Add_and_edit_base_frames.txt |
| CM-OP-10 | Add/edit tool frames | Lo stesso per i tool frame (TCP) nel sotto-elemento Tools. | Modeling_Operations_Add_and_edit_tool_frames.txt |
| CM-OP-11 | Edit node feature tree | Drag&drop di feature tra livelli e nodi (con SHIFT conserva la posizione nel mondo); Move Up in Tree; **Explode** (per geometry set o per connettività, dentro un nuovo Transform); **Group** (fratelli in un nuovo Transform); **Group Main Level**; **Remove Empty**. | Modeling_Operations_Editing_node_feature_tree.txt |
| CM-OP-12 | Edit origin | Sposta l'origine del componente con Snap (con modalità) o Move (manipolatore), poi Apply. | Modeling_Operations_Editing_origin_of_components.txt |
| CM-OP-13 | Unique/Shared | I componenti identici (stesso VCID) possono condividere l'albero delle feature per risparmiare memoria. Ci sono Auto share e Make component unique; il comportamento di default è nell'opzione di configurazione `FeatureTreeAutoSharing`. Prima di modificare conviene rendere unico il componente. | Modeling_Operations_Making_component_unique_or_shared.txt |
| CM-OP-14 | Save / Save As | Salva il componente in un file `.vcmx`. | Modeling_Operations_Saving_a_component.txt |
| CM-OP-15 | Search/filter graph | Ricerca, espandi/comprimi e filtri Behaviors/Properties nel Component Graph. | Modeling_Operations_Searching_component_data.txt |
| CM-OP-16 | Select behaviors | Seleziona un behavior dal grafo. | Modeling_Operations_Selecting_behaviors_in_a_component.txt |
| CM-OP-17 | Select features | Seleziona una feature dal 3D (clic o area) o dal grafo; le feature di operazione solo dal grafo. | Modeling_Operations_Selecting_features_in_a_component.txt |
| CM-OP-18 | Select nodes | Seleziona un nodo; "Structure Show" mostra l'intera struttura cinematica. | Modeling_Operations_Selecting_nodes_in_a_component.txt |
| CM-OP-19 | Select properties | Seleziona una property; si apre un Property task pane con le sue meta-proprietà. | Modeling_Operations_Selecting_properties_in_a_component.txt |
| CM-OP-20 | Transfer within component | Collega le porte di due behavior di flusso dello stesso componente: Connection (il behavior) più Port; Add Connector; flag Input/Output. | Modeling_Operations_Transferring_component_to_other_behaviors.txt |
| CM-OP-21 | Transfer to other components | Crea o usa un'interfaccia, la sua section e un field Flow con Container e Port. | Modeling_Operations_Transferring_components_to_other_components.txt |
| CM-OP-22 | Component Graph panel | Pannello a due alberi: nodi, behavior e property sopra; feature del nodo attivo sotto. Colori: nodo selezionato blu, feature verde, sottofeature oliva. Gli spostamenti con SHIFT conservano la posa nel mondo. | Component_Graph_panel.txt |
| CM-OP-23 | Modeling workflow | Flusso tipico: importa il CAD (consigliato semplificarlo), crea i link, sposta o estrae le feature nei link, aggiunge property, behavior e interfacce, scrive gli script, salva. | Modeling_a_component.txt |
| CM-OP-24 | 3Dfindit (add-on) | Add-on di terze parti (CADENAS) per scaricare modelli CAD da catalogo e inserirli nel layout. Da attivare nelle opzioni, con login. Non rientra nel nucleo del clone. | 3DFindit.txt |

---

## 3. Note per l'implementazione (semantica runtime ricostruita)

Distinguiamo ciò che l'help **afferma** da ciò che è **inferito** (*[inferenza]*).

### 3.1 Motore di simulazione

- L'help parla di "eventi" (FastScheduling "limita gli eventi generati", BlockingOptimization
  "ascolta un evento", heartbeat "un impulso per generare eventi"). Il motore è quindi a **eventi
  discreti**, con processi (gli script hanno `OnRun`, il Process Executor gira dall'avvio) e moti
  pianificati. *[inferenza]* Per il clone serve uno scheduler a eventi con tempo simulato e coroutine
  per gli script, analoghe ai tasklet della Stackless Python che il prodotto usa.
- **Reset della simulazione:**
  - riporta i segnali con `AutomaticReset=True` al valore iniziale;
  - annulla gli effetti della fisica;
  - riporta i giunti a `Initial Value` *[inferenza dal nome della proprietà]*;
  - fa sparire i componenti dinamici creati dai Creator *[inferenza]*.

### 3.2 Path (nastri) e flusso dei pezzi

**Geometria del path:**
- è una lista **ordinata** di Frame (waypoint), dal primo (lato Input) all'ultimo (lato Output);
- la traiettoria fra i waypoint dipende da `Interpolation`:
  - `Step`: il pezzo salta da un waypoint all'altro a intervalli pari a lunghezza / Speed;
  - `Linear`: segmenti rettilinei;
  - `Cubic`: catena di spline cubiche, tanto più fedele quanti più frame ci sono;
- `PathAxis` stabilisce come orientare i pezzi e quanto ingombro considerare: Automatic, gli assi del
  bounding box del componente, oppure Generic (stima pessimistica).

**Cinematica del pezzo:** velocità `Speed` con rampe `Acceleration` e `Deceleration` (0 = rampa
assente). Tutti i pezzi si muovono alla velocità del path, e un pezzo può sorpassarne un altro solo se
uno script ne sovrascrive la velocità.

**Accumulo:**
- `Accumulate=True`: se il pezzo in testa non può uscire (la porta Output è bloccata o la capienza a
  valle è piena), il path si ferma;
- `Accumulate=False`: il path non si ferma e i pezzi si accodano *[il comportamento esatto non è
  descritto]*;
- `SpaceUtilization=True` impedisce le sovrapposizioni: i pezzi si toccano ma non si compenetrano;
- `SegmentSize>0` divide il path in celle da un pezzo ciascuna (avanzamento a "slot").

**Altre regole:**
- **Capienza:** il minimo fra `Capacity` e i `CapacityBlocks` condivisi. Il blocco per capienza
  produce lo stato statistico "Blocked".
- **RetainOffset:** il pezzo mantiene l'offset laterale con cui è entrato; altrimenti viene riportato
  sulla linea del path.
- **Possesso:** mentre è sul path il pezzo è attaccato al nodo del path (un nodo temporaneo).
- **Transizioni:**
  - l'ingresso dalla porta 0 (Input) e l'uscita dalla porta 1 (Output) passano da connettori
    collegati, internamente fra porte e fra componenti con un field Flow;
  - `TransitionSignal` va a True all'arrivo e a False alla partenza.
- **Two Way Path:** `Direction` stabilisce il verso; le varianti (Auto) si invertono quando un pezzo
  arriva dal lato opposto.
- **FastScheduling:** il path calcola in anticipo gli istanti di arrivo ai punti notevoli (sensori,
  fine corsa) invece di avanzare a piccoli passi *[inferenza dalla descrizione]*.

**Component Creator:**
- genera un'istanza di `Part` ogni `Interval` secondi, fino a `Limit` (con 0 la creazione si comanda
  da script), e la spinge sulla porta di uscita;
- con `BlockingOptimization` non interroga la capienza ma aspetta una notifica di spazio libero;
- `PartPooling` riusa le istanze distrutte.

**Flow Proxy e Routing Rule:**
- la **Flow Proxy** abbina porte interne ed esterne; con `DynamicConnections` uno script ricollega le
  porte a runtime, e i segnali Component e Port dicono chi sta tentando di entrare e da dove;
- la **Routing Rule** valuta un albero di regole sul pezzo (o sul proprietario) e sceglie la porta di
  uscita, con un ripiego Default.

### 3.3 Sensori → segnali

**Sensori di path** (Component Path Sensor, Process Point Sensor):
- **Aggancio:** il sensore è registrato nella lista `Sensors` del path, oppure agganciato da un altro
  componente con un field `Processor` (il path è il padre).
- **Posizione:** è data da `Distance` (ascissa curvilinea) o da un `Frame`.
- **Trigger:**
  - `TriggerAt` stabilisce quale punto del pezzo fa scattare il sensore: il bordo d'attacco, l'origine
    o il bordo d'uscita, misurati lungo l'asse del path;
  - il sensore scrive il pezzo nel `ComponentSignal` e True nel `BoolSignal` (se `BoolSignalValue`
    lo consente);
  - poi si resetta secondo `ResetAt` *[i valori non sono documentati]*.
- **Process Point Sensor:** in più ferma il path o il solo pezzo (`ProcessingStops`) per un tempo
  calcolato dalle regole CM-PTIME, poi lo rilascia.
- *[Inferenza]* Il path conosce in anticipo le ascisse dei sensori, quindi lo scatto è un evento
  pianificato, non il risultato di un campionamento.

**Sensori geometrici** (Raycast, Volume):
- **Modalità sampling:** il sensore esegue un test ogni `SampleTime`.
  - Con `UpdateScene=True` aggiorna le pose del mondo, o solo dei pezzi contenuti in
    `DetectionContainer`, prima del test.
  - Il **Raycast** proietta un raggio lungo +Z del Frame: sotto `MaxRange` scrive la distanza in
    `RangeSignal`, sotto `DetectionThreshold` scrive `ComponentSignal` e `BoolSignal`.
  - Il **Volume** testa il box definito da LowerFrame e UpperFrame con `TestMethod` (mesh precisa,
    intersezione dei bbox, centro del bbox dentro, bbox tutto dentro). I filtri sono TestParent
    (stesso nodo), TestSiblings (nodi fratelli, per esempio i pezzi sul path), TestOnlyContained.
- **Modalità pulse** (`UseSampling=False`): il sensore è nella lista `Connections` di un segnale e
  ogni cambio di quel segnale innesca un test.

### 3.4 Propagazione dei segnali

- Un segnale è un behavior con un valore tipizzato: Boolean, Integer, Real, String, Component,
  Matrix.
- Ogni scrittura (o cambio di valore? *[non specificato]*) notifica tutti i behavior elencati in
  `Connections`: gli script ricevono `OnSignal(signal)`, i sensori in modalità pulse eseguono un test.
- **Fra componenti** i segnali si collegano:
  - in modo fisico, con un field `Signal` in un'interfaccia non astratta (per esempio utensile ↔
    robot);
  - in modo remoto, con un'interfaccia astratta più l'editor "Show Interfaces" o "Connect signals";
  - verso un PLC esterno con la Connectivity, che sta fuori da questo ambito;
- `Begin` si accoppia solo con `End`, `Normal` con qualsiasi altro: è un vincolo di direzione o di
  cablaggio.
- La **Boolean Signal Map** espone N porte numerate (`PortCount`) ai `Listeners` (gli Executor): sono
  gli ingressi e le uscite digitali IN[i] e OUT[i] del robot.
- **AutomaticReset**: al reset della simulazione il valore torna all'iniziale. Con False il valore
  sopravvive al reset.

### 3.5 Interfacce

**Compatibilità:**
- stesso numero, tipo e **ordine** dei field in una section;
- i vincoli dei singoli field devono essere soddisfatti: IntegerCompatibility con lo stesso valore,
  Hierarchy con un Parent e un Child, Export True contro False, Begin contro End, Input contro Output;
- devono esserci porte libere (1:1 contro 1:N);
- `ConnectSameLevelOnly` limita le connessioni allo stesso livello gerarchico.

**Connessione fisica (PnP):**
- quando trascini un componente vicino a un altro, si cercano section compatibili con il
  `SectionFrame` entro `DistanceTolerance` e `AngleTolerance`;
- alla connessione il componente viene allineato sul frame *[inferenza: lo snap non è descritto]*;
- se un field è `Hierarchy`, il nodo figlio diventa figlio del nodo padre (montaggio dell'utensile).

**Effetti dei field a connessione avvenuta:**
- **Flow:** collega la porta di uscita di un contenitore alla porta d'ingresso dell'altro;
- **JointExport, ToolExport, BaseExport:** copiano o rendono visibili giunti, TCP e basi nel
  controller dell'altro componente;
- **Signal:** cablano i segnali;
- **Processor:** aggancia un sensore a un path;
- **Rsl:** condivide le subroutine fra robot;
- **Transport e Action:** canali di messaggi fra risorse.

### 3.6 Controller e interpolazione

**Servo Controller:**
- i moti sono **joint-space**;
- ogni giunto ha `MaxSpeed`, `MaxAcceleration` e `MaxDeceleration`, il che suggerisce un **profilo di
  velocità trapezoidale** per giunto *[inferenza: la forma del profilo non è dichiarata]*;
- `LagTime` e `SettleTime` aggiungono un ritardo e un tempo d'assestamento a fine moto;
- non è documentato se i giunti di un moto multi-asse vengano sincronizzati (arrivo simultaneo).

**Robot Controller:**
- per i moti **lineari** usa `MaxCartesianSpeed/Accel` e `MaxAngularSpeed/Accel` sul TCP;
- l'orientamento si interpola secondo `OrientationInterpolationMode`:
  - asse unico di rotazione minima, cioè slerp *[inferenza]*;
  - due assi;
  - tre angoli WPR o ZYZ;
- la configurazione si gestisce con `ConfigurationMode`: fissa, segue il frame, oppure interpolata in
  joint space;
- l'IK viene dal behavior Kinematics;
- `UseHeartbeat` e `HeartbeatTime` generano un impulso periodico per gli eventi di monitoraggio
  *[lo scopo esatto non è documentato]*;
- `InitialBase` e `InitialTool` sono la base e il tool attivi all'avvio;
- con un track, `TrackWorldFrameMode` stabilisce dove si trova il frame World.

**Solver cinematici:**
- **Articulated:** sceglie fra 8 configurazioni (spalla front/back, gomito above/below, polso
  flip/noflip), che le maschere Shoulder/Elbow/WristValid filtrano;
- **tolleranze di singolarità:** polare (raggio), polso (angolo del giunto 5 vicino a 0), estensione;
- **PalletizingMode** forza l'asse utensile lungo una normale fissa;
- **accoppiamenti:** Coupling23 (asse 3 che compensa il 2) e Coupling456 (frazionario, polso cavo);
- **Python Kinematics:** fornisce un'interfaccia standard (forward, inverse, constrain e relax delle
  soluzioni) per cinematiche arbitrarie;
- **solver di nuova generazione:** mappano i valori dei giunti nello spazio nominale con JZO e JS,
  risolvono e riconvertono.

**Robot Executor:**
- interpreta il programma (con loop opzionale), legge e scrive l'I/O tramite le signal map;
- può delegare gli statement a uno `StatementHandler` Python;
- ha un'abilitazione globale `IsEnabled`.

### 3.7 Fisica (PhysX)

**Due mondi coesistenti:**
- il **mondo di simulazione** è cinematico e a eventi;
- il **mondo fisico** è a passi PhysX;
- un nodo passa da uno all'altro secondo `Physics Type`:
  - `In Physics`: corpo dinamico soggetto alla gravità;
  - `Kinematic`: corpo pilotato da path o servo, che spinge gli altri ma non subisce forze;
  - `Out of Physics`: fuori dalla fisica;
  - `In Container`: dinamico, ma il sistema ne traccia l'appartenenza ai contenitori, e quindi
    TransitionSignal e statistiche funzionano anche in fisica.

**Corpi e collisioni:**
- un corpo esiste se c'è una Physics Entity;
- la forma di collisione viene dalla proprietà `Collider` di ogni feature, ereditata o forzata dalla
  root feature: box, insieme di box per geometry set, mesh precisa;
- in alternativa si usano collider espliciti (Collider Container: box, sfera, convesso con padding);
- impostare un collider senza Physics Entity ne crea una in automatico;
- filtri di collisione: lo `Scene Id` (layer completamente separati, il pavimento è nella scena 0) e
  il `Collision Group`.

**Physics Path:** il nodo del path è `Kinematic` e le feature in `PushingFeatures` (o, di default,
tutte quelle con un collider) applicano una forza di contatto, una velocità di superficie, lungo la X
dei frame del path, a `Speed` mm/s *[inferenza: "nastro a velocità superficiale"]*.

**Physics Joint:** un D6 con 6 vincoli Free/Limited/Locked, rottura (breakage) e azionamento (drive)
*[parametri non documentati]*.

**Cavi:** catene di capsule tenute da molle (strength e damping assiali e tangenziali), con pin come
nodi con giunto. Gli estremi Free, Rotating e Fixed corrispondono a gradi di vincolo dell'ancoraggio.

**Reset:** ripristina lo stato iniziale di tutto ciò che la fisica ha mosso.

### 3.8 Albero delle feature ed espressioni

**Rebuild:**
- un cambio di property con "Rebuild" rivaluta tutte le espressioni (dimensioni, Transform, Switch,
  Text, offset dei nodi, Value Expression dei giunti) e rigenera la geometria bottom-up;
- la Python Feature riceve `OnEvaluate(feature, target, matrix, containers)` e deve spostare nel
  `target` la geometria delle sottofeature (`containers`) applicando `matrix`.

**Espressioni:**
- riferiscono le property del componente per nome, quelle dei behavior come `Behavior::Prop`, i giunti
  per nome e il valore corrente come `VALUE`;
- le espressioni di Transform concatenano le operazioni con `.`, rispettando la precedenza degli
  operatori.

**Condivisione:** i componenti con lo stesso VCID possono condividere l'albero delle feature (stile
flyweight). Va reso unico prima di modificarlo.

---

## 4. Lacune (cosa l'help non spiega)

1. **Grammatica delle espressioni.** Mancano gli operatori, le funzioni disponibili (Tx(), Rz()…?) e
   la "Operator Precedence" citata dalla pagina Transform, che non è in questo ambito. La sintassi
   esatta di Value Expression, Joint e Pivot nei giunti Custom non è data.
2. **Unità interne.** Mancano le unità di Speed e Acceleration dei path (solo il Physics Path dice
   mm/s), di MaxSpeed e MaxAcceleration dei giunti (gradi/s? rad/s?), di LagTime e SettleTime, di
   AngleTolerance e DistanceTolerance.
3. **Profilo di moto.** La forma del profilo dei servo (trapezoidale? jerk limitato?), la
   sincronizzazione multi-asse, il significato preciso di LagTime e SettleTime e la pianificazione
   dei moti lineari (blending, zone e precisione) non sono spiegati.
4. **Valori di `ResetAt`** del Component Path Sensor: non elencati.
5. **Semantica di Accumulate=False**, cioè come si comporta la coda quando il path non si ferma, e
   l'algoritmo di "SpaceUtilization" sulle sovrapposizioni.
6. **Segnali.** Non è detto se la notifica parta a ogni scrittura o solo a un cambio di valore, in
   quale ordine vengano notificati i connessi, né quale sia il "valore iniziale" a cui si resetta un
   segnale (manca una proprietà InitialValue).
7. **Interfacce.** Mancano l'algoritmo di matching (come vengono confrontati i field, cosa succede con
   più section), lo snap o allineamento alla connessione e le proprietà del field Attachment.
8. **Physics Joint.** Il contenuto dei gruppi `Breakage` e `Drive Settings` non è documentato; lo step
   di simulazione PhysX e il sotto-campionamento rispetto al motore a eventi nemmeno.
9. **Behavior quasi vuoti.** Vehicle, Document Link, Motion Controller e Program Executor sono
   descritti solo per nome e per pochi riferimenti. Della guida veicolo, dei link e delle proprietà del
   nuovo Motion Controller non c'è nulla.
10. **Solver di nuova generazione.** Le pagine dei solver dipendono da immagini, assenti nel dump
    testuale. Mancano gli elenchi degli offset (a1x, a4y…), la "Solver Configuration" di ciascun
    solver e l'elenco completo dei tipi selezionabili.
11. **InterpolationMode di base e tool frame:** i valori dell'enum non sono elencati.
12. **API Python.** Le classi `vcScript`, `vcFeature`, `vcPythonKinematics`, `vcMatrix` e `vcVector`
    sono solo citate: niente metodi, niente eventi oltre a OnSignal, OnRun, OnAction ed OnEvaluate,
    niente semantica di scheduling (delay, condition) degli script.
13. **Process Model.** Le routine e gli statement del Process Executor, il formato della tabella
    Excel del Product Creator, i parametri delle modalità Batch e Distribution e il funzionamento del
    Flow Editor e dei transport link stanno in altre sezioni o non sono descritti.
14. **Statistics.** Manca come un behavior entra negli stati (chi li imposta, a parte gli script) e
    come si configura l'intervallo (si sa solo il default di 60 s).
15. **Formato file.** Il contenuto e la sintassi di `component.rsc` e `component.dat` non sono
    documentati. Per un clone serve definire un formato proprio.
16. **Routing Rule.** L'ordine di valutazione con più livelli e il comportamento quando la porta scelta
    è bloccata (attesa o ripiego) non sono spiegati. "Product Type Rule" compare due volte con
    descrizioni leggermente diverse.
17. **Collider.** I parametri di "Quality" e "Precision mode" della Create Collision Geometry e il
    rapporto fra il Collider Container e la proprietà Collider delle feature non sono chiariti.
