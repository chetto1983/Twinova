# Mappa funzionale Visual Components Premium 5.1 — UI, layout, simulazione, catalogo, import/export, disegni, espressioni

Fonte: help ufficiale 5.1 Premium, vedi [indice](00-indice.md) §2 (versione help del 13/10/2026, prodotto 5.1.1).
Descrizioni scritte con parole nostre (clean-room): nessuna frase o immagine copiata.
Convenzione: "Fonte" = nome del file `.txt` senza il prefisso `English_`.

---

## 1. Getting Started — interfaccia, navigazione, selezione, file di layout (UI)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| UI-01 | Ribbon a schede contestuali | Barra comandi divisa per contesto di lavoro: Home (layout), Modeling, Program (robot), Process, Drawing, Help, più il backstage File. | Schede: HOME, MODELING, PROGRAM, PROCESS, DRAWING, HELP, FILE | Getting_Started_UI_Overview_Tabs_Layout_View.txt, _Modeling_View, _Robot_View, _Process, _Help_tab |
| UI-02 | Vista Layout (Home) | Contesto di configurazione dell'impianto: aggiungere/manipolare componenti, simulare, registrare, esportare. Pannelli di default: eCatalog, Cell Graph, Properties. | Gruppi: Clipboard, Manipulation, Grid snap, Tools, Dimension, Connect, Hierarchy, Import, Export, Statistics, Camera, Origin, FactoryLens | Getting_Started_UI_Overview_Tabs_Layout_View.txt |
| UI-03 | Vista Modeling (sintesi) | Contesto di modellazione del componente: albero nodi (catena cinematica), comportamenti, feature geometriche, proprietà parametriche, script Python. | Pannello Component Graph (Node Tree + Feature Tree); comandi New/Save/Save As, Create Link, Behaviors, Features, Properties, Tools (Split, Invert, Merge, Merge faces, Collapse, Slice, Select Identical, Remove Holes, Component, Link, Decimate, Cylindrify, Blockify, Assign materiale, Add Decal, Mapping UV), Wizards (Action Script, Convex Hull, Conveyor, Count Triangles, End Effector, IO-Control, Machine Wizard), Hierarchy/Selected, Structure Show | Getting_Started_UI_Overview_Tabs_Modeling_View.txt |
| UI-04 | Vista Program/Robot (sintesi) | Contesto di programmazione robot: teach di posizioni e percorsi, editor programmi, jog, collisioni e limiti. | Program Editor, Jog panel, Teach overlay; Detectors (prima/tutte le collisioni, tolleranza, distanza minima, liste A/B), Stop on collision, Stop at limits, limiti Joint/Speed/Acceleration/Singularity, Color Highlights, Connecting Lines, Traces, Exchange Robots, Move Robot World Frame, To Reference / To World | Getting_Started_UI_Overview_Tabs_Robot_View.txt |
| UI-05 | Vista Process (sintesi) | Contesto di process modeling: tipi di prodotto, flow group, step di processo, transizioni e link di trasporto tra nodi. | Products (editor tipi prodotto/flow group), Processes, Flow (editor di flusso), Filter Transport Links, controller di trasporto umano/robot, nodi di trasporto | Getting_Started_UI_Overview_Tabs_Process.txt |
| UI-06 | Scheda Help | Accesso a help, riferimenti API (Python, .NET, Python 2.7) e link a risorse online/social. | Help, Python API, .NET API, Python 2.7; Academy, Forum, Community, Blog, Search, Extranet (solo OLP) | Getting_Started_UI_Overview_Tabs_Help_tab.txt |
| UI-07 | Guided Workflow (esecuzione) | Procedure guidate passo-passo ("Task") con istruzioni sequenziali, comandi cliccabili negli step, avanzamento/skip/indietro. | Filter Public/Local, Category, Task, Refresh, Reset (torna al primo step), Done, Skip, Back; workflow locali salvati in My Templates/Workflow Templates | Getting_Started_UI_Overview_Tabs_Help_tab.txt |
| UI-08 | Workflow Configurator (authoring) | Editor per creare/modificare i workflow guidati: step, istruzioni con link, comandi incorporati. | Category, Task (nome univoco per categoria), Step title, Instructions, IsSkippable, link URL, add/delete/move step, add/delete command, copia task, save/delete | Getting_Started_UI_Overview_Panels_Workflow_Configurator.txt |
| UI-09 | Pannello Properties | Edita le proprietà dell'oggetto selezionato; posizione XYZ e orientamento RxRyRz nel sistema di coordinate attivo; editing multiplo con valori comuni. | Position/Orientation (click sull'etichetta = azzera), tab di proprietà, sezioni, nome oggetto, Coordinate System (World/Parent/Object), Lock Editing, marcatore `<multiple values>`, TAB/SHIFT+TAB tra campi | Getting_Started_UI_Overview_Panels_Properties_panel.txt |
| UI-10 | Proprietà comuni del componente | Ogni componente nasce con un set fisso di proprietà base. | Name, Material (ereditato da nodi/feature senza materiale), Visible, PDF ExportLevel, BOM, BOMname, BOMdescription, Category (= metadato Type), Simulation Level (Default/Detailed/Balanced/Fast), BackfaceMode | Getting_Started_UI_Overview_Panels_Properties_panel.txt |
| UI-11 | Docking dei pannelli | I pannelli si agganciano in aree indicate da un docking manager; trascinando l'header si sposta il singolo pannello o l'intero gruppo. | Pannello pinned/floating; drag da title bar o tab header | Getting_Started_UI_Overview_Panels_Docking_a_panel.txt |
| UI-12 | Raggruppamento pannelli a schede | Pannelli agganciati nella stessa area formano un gruppo a tab. | — | Getting_Started_UI_Overview_Panels_Grouping_panels.txt |
| UI-13 | Auto-hide pannelli | Puntina sul pannello: fissato o a scomparsa laterale quando perde il focus. | Pushpin giù/laterale | Getting_Started_UI_Overview_Panels_Hiding_panels.txt |
| UI-14 | Mostra pannelli | Comando "Show" che elenca i pannelli disponibili nel contesto e li mostra/nasconde; alcuni pannelli compaiono automaticamente in base a scheda o comando attivo. | Lista pannelli per contesto | Getting_Started_UI_Overview_Panels_Viewing_panels.txt |
| UI-15 | Layout pannelli indipendente per contesto | Opzione per cui dimensione/posizione di un pannello può differire tra le schede (es. Properties in Home vs Program). | Enable Independence (General options; default false dal 5.0) | Getting_Started_UI_Overview_Panels_Editing_panels_by_context.txt; Release_Notes_release_notes_5.0.txt |
| UI-16 | Restore Windows | Ripristina la disposizione di default dei pannelli, globalmente o solo per la scheda attiva. | Enable Global Restore (default attivo) | Getting_Started_UI_Overview_Panels_Restoring_windows.txt |
| UI-17 | Multiprogram Editor | Fissa più istanze del Program Editor (una per robot) per programmare più robot contemporaneamente; pannello e robot evidenziati. | Pin active Program Editor, Close, titolo = nome robot | Getting_Started_UI_Overview_Panels_Multiprogram_Editor.txt |
| UI-18 | Quick Access Toolbar | Barra rapida personalizzabile: aggiunta/rimozione di comandi o gruppi dal ribbon, posizione sopra/sotto il ribbon. | Add/Remove to QAT (rimozione per gruppo intero), Show Above/Below Ribbon, Undo/Redo | Getting_Started_UI_Overview_QAT_*.txt |
| UI-19 | Copia/Incolla | Copia la selezione negli appunti e incolla con offset rispetto all'originale. | CTRL+C / CTRL+V, gruppo Clipboard | Getting_Started_UI_Overview_Clipboard_Copying_and_pasting_a_selection.txt |
| UI-20 | Clone | Duplicazione rapida dal Mini toolbar; il clone può auto-collegarsi all'originale se le interfacce sono compatibili, altrimenti viene posto con offset. | Clone; CTRL+doppio click = duplica | Getting_Started_UI_Overview_Clipboard_Cloning_selected_objects.txt; Setup_Shortcuts.txt |
| UI-21 | Elimina | Rimozione permanente della selezione. | DELETE, Clipboard > Delete | Getting_Started_UI_Overview_Clipboard_Deleting_a_selection.txt |
| UI-22 | Navigazione camera | Orbita (RMB), pan (LMB+RMB), zoom (rotella o SHIFT+RMB), centro di interesse (CTRL o SHIFT + click destro). Nel mondo 2D dei disegni niente rotazione. | Zoom factor e zoom mode (attraverso il COI o no) in General options | Getting_Started_Navigation.txt; Getting_Started_Visualization.txt |
| UI-23 | View Selector e origine flottante | Cubo di viste standard cliccabile (6 facce, spigoli, angoli) e terna World in angolo; Top/Down sulla stessa faccia (click/doppio click), in Top ogni click ruota di 45°. | Viste standard, viste di spigolo/angolo | Getting_Started_Navigation.txt |
| UI-24 | Supporto 3D mouse | Navigazione con SpaceMouse, configurabile; punto blu sul centro di interesse. | Abilitazione e taratura in General options | Getting_Started_Navigation.txt |
| UI-25 | Fill View | Inquadra tutti i componenti o solo la selezione. | CTRL+F, CTRL+ALT+F | Getting_Started_Visualization.txt |
| UI-26 | Headlight | Luce direzionale solidale con la camera per evidenziare facce e spigoli (utile nel teach). | On/off | Getting_Started_Visualization.txt |
| UI-27 | Proiezione camera | Commutazione prospettica/ortografica. | Perspective (default) / Orthographic | Getting_Started_Visualization.txt |
| UI-28 | Modalità di rendering | Stili di visualizzazione della scena. | Wireframe, All edges shaded, Face edges shaded, X-ray shaded, Shaded, Material shaded, Realistic shaded, Realistic edge shaded; qualità | Getting_Started_Visualization.txt |
| UI-29 | Visibilità frame | Mostra/nasconde per tipo i frame: basi robot, tool, robot world, posizioni robot (con etichette, assi, punti centrali), frame feature generici. | Toggle per tipo | Getting_Started_Visualization.txt |
| UI-30 | Collider toggle (toolbar) | Pulsante sulla toolbar 3D per i collider (geometrie di collisione semplificate). | On/off | Getting_Started_Visualization.txt; Release_Notes_release_notes_5.0.txt |
| UI-31 | View Editor | Crea, rinomina, seleziona, aggiorna, elimina viste camera salvate nel layout. | Create/Rename/Use/Update/Delete/Save View | Getting_Started_Visualization.txt |
| UI-32 | Full-screen | Vista 3D a schermo intero. | F11 | Getting_Started_Visualization.txt |
| UI-33 | Focus mode | Isola una selezione: il resto diventa trasparente e non selezionabile/agganciabile; non salvato nel layout. | ALT+Q; SHIFT+ALT+Q include figli gerarchici; Unfocused opacity 0..1; opzioni componenti dinamici: Same as parent / Always focused / Always unfocused; vale anche per sub-layout e nodi in Modeling | Getting_Started_focus_mode.txt |
| UI-34 | Selezione diretta | Click seleziona, CTRL+click aggiunge/toglie, click a vuoto deseleziona; evidenziazione silhouette+fill; Mini toolbar contestuale. | Colori Silhouette/Fill | Getting_Started_Selecting_components.txt; Getting_Started_Selection.txt |
| UI-35 | Selezione ad area | Selezione rettangolare o a mano libera (lazo); l'oggetto deve occupare una parte significativa dell'area; Select All, Invert selection. | Rectangular, Free-form, CTRL+A, Invert | Getting_Started_Selecting_components_in_an_area.txt |
| UI-36 | Selezione contestuale per vista | L'oggetto selezionabile dipende dalla scheda: componenti (Layout), posizioni robot (Program), feature/nodi (Modeling, doppio click = nodo), layout item (Drawing); selezione anche da pannelli (Cell Graph, Component Graph, Program Editor, BOM). | — | Getting_Started_Selection.txt |
| UI-37 | Aggiunta componenti dal catalogo | Doppio click = inserimento all'origine con auto-plug se c'è un'interfaccia compatibile con la selezione; Open per più elementi; drag = posizione sotto il puntatore. Il componente aggiunto resta selezionato con PnP attivo. | Filtro Components; doppio click / Open / drag | Getting_Started_Adding_components.txt |
| UI-38 | Spostamento/rotazione | Posizionamento numerico (XYZ/RxRyRz) o con manipolatore: frecce (traslazione), anelli (rotazione), piani (2 assi), toro/origine (snap su superficie). | Coordinate System World/Parent/Object; riferimento = ultimo selezionato; Grid snap (Size, Automatic Size, Always Snap) | Getting_Started_Moving_and_rotating_components.txt |
| UI-39 | Plug & Play (PnP) | Trascinando un componente vicino ad altri, le interfacce fisiche libere (frecce gialle) indicano le compatibili (frecce verdi, la più grande = più vicina); a distanza/angolo sufficiente scatta l'aggancio; allontanando si sgancia. | Interfacce su Frame feature; verde = connessa; angolo di approccio accettabile | Getting_Started_Plugging_in_components.txt |
| UI-40 | Interact (jog parti) | Muove a mano le parti mobili del componente entro DOF e limiti; i giunti accoppiati si muovono insieme. | Undo/Redo, Reset, Save State | Getting_Started_Interacting_with_parts_of_components.txt |
| UI-41 | Nuovo layout / Clear All | Svuota la scena; all'avvio è sempre presente un layout vuoto. | CTRL+N, File > Clear All | Getting_Started_Opening_a_new_layout.txt |
| UI-42 | Apri layout | Apertura file VCMX (archivio) dal catalogo per drag o da file/recenti; il contenuto viene estratto/scaricato. | CTRL+O, filtro Layouts | Getting_Started_Opening_a_layout.txt |
| UI-43 | Chiudi layout | Chiudere equivale ad aprire un nuovo layout vuoto, con prompt Save/Don't Save. | File > New | Getting_Started_Closing_a_layout.txt |
| UI-44 | Salva layout | Salva posizioni, connessioni, valori proprietà, vista e layout item; l'inquadratura corrente diventa l'anteprima nel catalogo. | Save As, CTRL+S; Include Components (default true dal 5.0); Include Component original path (percorso nel pannello Output) | Getting_Started_Saving_a_layout.txt |
| UI-45 | FactoryLens (add-on) | Viewport alternativo fotorealistico RTX (NVIDIA Omniverse) sincronizzato col layout; export immagine, video (avvia la simulazione e registra), USD. | Licenza add-on + riavvio; Image: Resolution/Width/Height/Aspect/Format; Video: Resolution/Frame rate/Codec, Stop and save; USD: File Format | Getting_Started_FactoryLens.txt; Getting_Started_UI_Overview_Tabs_Layout_View.txt |

### Note per l'implementazione
- **Modello di scena**: un unico "mondo 3D" con nodo radice di simulazione; ogni componente ha un *parent* (radice o nodo di altro componente). Le coordinate Parent coincidono con World se il genitore è la radice. Esiste anche un "mondo disegno" 2D separato (Drawing).
- **Sistemi di coordinate** del manipolatore e del pannello Properties: World, Parent, Object (relativo allo stato corrente). Con selezione multipla il riferimento è l'ultimo oggetto aggiunto.
- **PnP**: le interfacce fisiche risiedono su Frame feature; il matching richiede compatibilità di tipo e un angolo di approccio entro tolleranza; lo snap avviene sotto una distanza soglia (valori non documentati). Plug ≠ attach: due componenti collegati possono essere o no in gerarchia padre-figlio.
- **Focus mode** è stato di sessione, non persistito; i componenti creati durante la simulazione ("dinamici") seguono una policy configurabile.
- **Simulation Level** per componente (Default delega al globale): Detailed = moto completo, Balanced = salta moti di giunto inutili, Fast = salti alla configurazione finale.
- **Persistenza layout**: riferimento al componente via VCID; geometria non inclusa salvo "Include Components"; geometria CAD importata invece è sempre salvata nel layout.
- **Snap da tastiera** (vedi SET-07): SHIFT sul toro = snap a centri geometrici; CTRL sul toro = movimento libero senza snap; CTRL durante drag di una freccia = allinea una coordinata a una geometria; SHIFT+drag su geometria = allineamento a 1-3 vettori (soglia 30 px).
- La UI è dipendente dal contesto (ribbon, pannelli, oggetti selezionabili cambiano con la scheda): serve un modello "contesto attivo" che filtra comandi e tipi selezionabili.

---

## 2. Setup — opzioni, unità, scorciatoie (SET)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| SET-01 | Sistema di unità (unit family) | Sceglie il sistema di misura attivo per la visualizzazione; internamente tutto resta metrico e viene convertito a video. | Unit family → unit group → unit; quantità scalari o vettoriali; una sola famiglia attiva | Setup_Defining_units_and_quantities.txt |
| SET-02 | Opzioni General | Lingua UI (riavvio), unità, navigazione (zoom, SpaceMouse), editor script Python, streaming animazione, backup, Enable Independence/Global Restore. | Limit backups (per data modifica), Delete old backups all'avvio, cartella backup o My Models; estensione numerica più alta = backup più recente | Setup_Options_General.txt |
| SET-03 | Opzioni Display | Preferenze visualizzazione 3D, precisione decimali di quote/annotazioni, qualità generazione disegni 2D, preferenza GPU, LOD point cloud, colore sfondo (per PDF). | Graphics preference: Let Windows decide / Power saving / High performance (riavvio); Point Cloud LOD (senza riavvio); Display accuracy | Setup_Options_Display.txt; 3D_Operations_Measuring_components.txt |
| SET-04 | Gestione add-on | Elenca add-on installati con stato e descrizione; attivazione/disattivazione con toggle (alcuni richiedono riavvio). | Toggle per add-on | Setup_Options_Add-on.txt |
| SET-05 | Info (backstage) | Metadati del layout corrente, licenza (registrazione, disattivazione chiavi, chiavi add-on), versione, area legale con impostazioni analytics. | Layout / License / Version / Legal (EULA, 3rd party, Analytics Settings) | Setup_Info.txt |
| SET-06 | Software Rendering | Rendering via CPU come fallback per sessioni Remote Desktop; disponibile solo con licenza di rete. | Display > Software Rendering + riavvio | Setup_Software_Rendering.txt |
| SET-07 | Scorciatoie da tastiera | Tabelle di shortcut per comandi base, navigazione, manipolazione, picking, teach robot, editing geometria, editor Python. | CTRL+N/O/S/C/V, SPACE = play/pausa, CTRL+ALT+J = versione in Output; teach: SHIFT=trim curva, CTRL=rimuovi segmento, ALT=inverti normale; editor: F9/CTRL+S compila, CTRL+J snippet, CTRL+SPACE autocompletamento, CTRL+D duplica riga, CTRL+L elimina riga | Setup_Shortcuts.txt |
| SET-08 | Editor Python esterno | Usa un editor di sistema al posto di quello interno; il debug runtime resta solo nell'editor interno. | General > Configuration > Python script editor: Internal/External | Setup_Using_external_Python_editor.txt |
| SET-09 | Streaming animazione (opzione) | Abilita un server di streaming della scena su porta TCP configurabile (vedi EXP-10). | Enabled, Port (default 7000), riavvio | Export_Streaming_animation.txt |

### Note per l'implementazione
- **Unità**: unità canoniche interne mm, s, deg, g, N, Nmm, K, mm/s (vedi EXPR-07). La preferenza utente è solo un livello di presentazione; serve un registro unità con famiglie, gruppi e fattori di conversione.
- Molte opzioni richiedono riavvio (lingua, GPU, add-on, streaming, software rendering): il clone può distinguere opzioni "hot" da opzioni "di avvio".
- Il drag di una feature nel Component Graph con SHIFT mantiene la posa mondo, senza SHIFT mantiene la posa relativa al nuovo genitore: regola utile per la re-parentela in generale.

---

## 3. eCatalog — catalogo componenti e layout (CAT)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| CAT-01 | Pannello eCatalog / vista Collections | Browser di sorgenti di file (componenti, layout, CAD) con area di visualizzazione e filtri. | Collections view, Display area, check box Components/Layouts/Files | eCatalog_Items_Filtering_items.txt |
| CAT-02 | Sorgente locale | Collega una cartella locale come sorgente indicizzata. | Edit Sources > Add New Source > Select Local Folder | eCatalog_Sources_Adding_a_local_source.txt |
| CAT-03 | Sorgente remota | Collega una libreria online tramite URL (documento HTML/XML che descrive gli elementi); il pannello si comporta come un browser HTTPS. | URL; soggetta a proxy/firewall | eCatalog_Sources_Adding_a_remote_source.txt |
| CAT-04 | Abilita/disabilita sorgente | Check box accanto al nome per usare o no una sorgente. | Source Name checkbox | eCatalog_Sources_Using_sources.txt |
| CAT-05 | Visibilità sorgente | Nasconde/mostra la sorgente nel pannello. | Colonna Visible | eCatalog_Sources_Showing_sources.txt |
| CAT-06 | Rimozione sorgente | Scollega una sorgente. | Remove | eCatalog_Sources_Removing_sources.txt |
| CAT-07 | Refresh sorgente | Forza la risincronizzazione/indicizzazione (es. dopo errori di rete). | Click destro > Refresh | eCatalog_Sources_Forcing_the_update_of_sources.txt |
| CAT-08 | Copia locale di sorgente remota | Mantiene una cache locale sincronizzata automaticamente quando c'è connessione. | Keep local copy (togliere/rimettere = forza update) | eCatalog_Local_Copies_Forcing_the_update_of_local_copies.txt |
| CAT-09 | Mappatura cartella della copia locale | Cambia la cartella di destinazione della cache. | Options > Change folder | eCatalog_Local_Copies_Mapping_local_folder_to_remote_source.txt |
| CAT-10 | Eliminazione copia locale | Svuota la cartella di cache di una sorgente remota. | Options > Delete | eCatalog_Local_Copies_Deleting_a_local_copy.txt |
| CAT-11 | Collezioni manuali | Raccolte create dall'utente in cui trascinare elementi; rimozione elementi via menu. | Add Collection, drag items, Remove, Delete | eCatalog_Collections_*.txt |
| CAT-12 | Smart collection | Collezione dinamica definita da condizioni di filtro su metadati combinate con AND/OR, con raggruppamento e sottogruppi. | Condizioni "campo Is valore" (es. Collection, Type), operatori logici, Group by (es. Manufacturer), subgroup; editing con doppio click | eCatalog_Smart_Collections_*.txt |
| CAT-13 | Collection group | Cartella che raggruppa collezioni e sorgenti; eliminarla separa i membri. | Add Collection Group, drag in/out, Delete | eCatalog_Collection_Groups_*.txt |
| CAT-14 | Ricerca | Ricerca per parole chiave sui metadati (es. tag). | Search box | eCatalog_Items_Searching_for_items.txt |
| CAT-15 | Ordinamento | Ordina gli elementi secondo opzioni disponibili. | Sort | eCatalog_Items_Sorting_items.txt |
| CAT-16 | Filtro per tipo | Filtra file nativi (Components, Layouts) o file supportati (Files, es. CAD). | Check box | eCatalog_Items_Filtering_items.txt |
| CAT-17 | Modalità di visualizzazione | Slider tra modalità (icone → dettagli); in Details è una tabella con colonne di metadati riordinabili, ordinabili, configurabili; copia valore cella. | View slider, Edit Columns, Copy Value | eCatalog_Items_Viewing_items.txt |
| CAT-18 | Metadati elemento | Visualizzazione (sorgenti remote/file) o modifica (sorgenti locali native) di campi chiave-valore e tag. | Main Fields (add/edit/remove property), Tags (add/edit/remove) | eCatalog_Items_Viewing_and_editing_metadata.txt |
| CAT-19 | Diagnostica e log | Log di sincronizzazione (file aggiunti/aggiornati/rimossi/invariati), indice su database SQLite, pacchetto offline su richiesta. | Porta 443; file log `...eCatalogue.UpdateWebCat`; "eCat package" offline | eCatalog_Troubleshoot_the_eCatalog_panel.txt |

### Note per l'implementazione
- **Modello dati**: `Source` (locale = cartella; remota = URI con descrittore XML; flag enabled, visible, keepLocalCopy, localFolder) → `Item` (file nativo .vcmx componente/layout o file CAD supportato) con metadati chiave-valore + tag + anteprima/icona; `Collection` (lista esplicita di item), `SmartCollection` (query su metadati + raggruppamento gerarchico), `CollectionGroup` (contenitore di collezioni e sorgenti).
- **Indicizzazione**: tutte le sorgenti vengono indicizzate in un DB (SQLite) all'avvio e su refresh; l'aggiornamento è un diff (added/updated/removed/unchanged).
- **Risoluzione dei componenti dei layout**: un layout referenzia i componenti tramite VCID; all'apertura si cerca la geometria nelle sorgenti del catalogo → il catalogo è anche un *resolver* di identificativi.
- Metadati modificabili solo per item nativi di sorgenti locali.
- Proprietà del componente `Category` alimenta il metadato `Type`, usabile nei filtri smart (es. "Type Is Conveyors").

---

## 4. Import — CAD e dati matematici (IMP)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| IMP-01 | Import geometria come nuovo componente | Importa un file CAD creando uno o più componenti; anteprima del conteggio dati prima dell'import; anche drag & drop di file CAD. | Home/Modeling > Geometry; Analyze; Import; conversione interna via formato PRC | Import_Importing_geometry_as_new_components.txt; Layout_Configuration_Building_a_layout.txt |
| IMP-02 | Struttura di import | Mappa l'albero di assieme CAD su feature, nodi o componenti separati. | Structure: Feature / Node / Layout (Component) | Import_Import_Model_Settings.txt |
| IMP-03 | Qualità di tessellazione | Livello di dettaglio della triangolazione delle superfici curve. | Da Extra low (~80 poligoni) a Extra high (~528); consiglio < 100.000 triangoli per componente statico | Import_Import_Model_Settings.txt |
| IMP-04 | Elementi da includere | Selezione dei contenuti aggiuntivi da importare. | Hidden, Markups, Points (→ Frame feature/point cloud), Materials, Textures, All configurations, Read block attributes | Import_Import_Model_Settings.txt |
| IMP-05 | Regola di creazione materiali | Come mappare i materiali CAD sulle librerie materiali. | Material Creation Rule | Import_Import_Model_Settings.txt |
| IMP-06 | Feature Tree | Gerarchia delle feature risultanti. | Full (replica assieme; geometry o transform feature), Optimized (piatto), Collapsed (una sola feature) | Import_Import_Model_Settings.txt |
| IMP-07 | Organizzazione geometry set | Come raggruppare le mesh in geometry set (un materiale per set). | By faces, By material, Collapsed (nuovo set oltre 10.000 triangoli / 16.000 punti), Mathematical data | Import_Import_Model_Settings.txt |
| IMP-08 | Pulizia geometria in import | Asse "up", rimozione fori piccoli, scarto di parti piccole, healing di gap. | Up axis, Minimum hole diameter, Minimum geometry diameter (bounding box), Healing tolerance (0 = off) | Import_Import_Model_Settings.txt |
| IMP-09 | Unità del file | Conversione di scala per formati senza unità esplicite. | Units (U3D, CGR, STL, VRML); Default = unità corrente | Import_Import_Model_Settings.txt |
| IMP-10 | Import dati matematici (BREP) | Conserva la BREP per faccia insieme alla mesh; permette ritessellazione reversibile e accesso alla topologia via API. | Organize geometry = Mathematical data; Tessellation quality per feature (Extra Low..Extra High, Chord height ratio, Angle tolerance), Tessellate, Unload mathematical data (irreversibile) | Import_Importing_mathematical_data.txt |
| IMP-11 | Topologia via API | Da script si leggono curve della topologia (lunghezza, posizione a distanza) per generare frame/punti/percorsi. | Geometry set → Topology → lunghezza curva, posizione su curva; frame set | Import_Importing_mathematical_data.txt |
| IMP-12 | Formati CAD supportati | Ampia matrice di formati in import e un sottoinsieme in export (vedi tabella sotto). | — | Import_Supported_CAD_files.txt |

**Matrice formati** (I = import, E = export):

| Formato | Estensioni | I | E |
|---|---|---|---|
| 3D Manufacturing Format 1.2.3 | .3mf | ✔ | ✔ |
| 3D Studio | .3ds | ✔ | ✔ |
| ACIS (fino a 2025) | .sat, .sab | ✔ | — |
| Point cloud ASCII | .xyz, .pts, .xyzrgb | ✔ | — |
| Point cloud binario | .bxyz | ✔ | — |
| E57 1.0 | .e57 | ✔ | — |
| Autodesk FBX (ASCII 7100–7500, binario tutte) | .fbx | ✔ | ✔ |
| Autodesk Inventor (fino a 2026) | .ipt, .iam | ✔ | — |
| Navisworks 2012–2026 | .nwd | ✔ | — |
| RealDWG (fino a AutoCAD 2024) | .dwg, .dxf | ✔ | ✔ |
| CATIA V4 / V5 (fino a V5-6R2025) / V6 (3DXML 4.3) | .session .dlv .exp / .CATPart .CATProduct .CATDrawing .CATShape .cgr / .3dxml | ✔ | — |
| COLLADA | .dae | ✔ | — |
| Creo / Pro-E (19.0 → 12.4) | .asm .neu .prt .xas .xpr | ✔ | — |
| DGN 7, 8 | .dgn | ✔ | — |
| glTF 2.0 | .gltf, .glb | ✔ | — |
| I-deas | .mf1 .arc .unv .pkg | ✔ | — |
| IFC2 (≤2.3.0.1), IFC4 (≤4.3) | .ifc, .iczip | ✔ | — |
| IGES 5.1–5.3 | .igs, .iges | ✔ | — |
| Igrip/Quest/VNC | .pdb | ✔ | ✔ |
| JT (≤10.9) | .jt | ✔ | ✔ (senza texture) |
| Parasolid (≤v38) | .x_b .x_t .xmt .xmt_txt | ✔ | — |
| PRC | .prc | ✔ | ✔ |
| Revit 2015–2026 | .rvt | ✔ | — |
| Robface | .rf | ✔ | — |
| Rhino 4–8 | .3dm | ✔ | — |
| Solid Edge | .asm .par .pwd .psm | ✔ | — |
| SolidWorks 97–2026 | .sldasm, .sldprt | ✔ | — |
| STEP AP203/214/242 | .stp, .step | ✔ | ✔ (AP242 ed3, solo tessellato) |
| STEP/XML | .stpx, .stpxz | ✔ | — |
| STL ASCII/binario | .stl | ✔ | ✔ |
| U3D (ECMA-363 ed. 1–3) | .u3d | ✔ | ✔ |
| Siemens NX/UG | .prt | ✔ | ✔ |
| VDA-FS 1.0/2.0 | .vda | ✔ | — |
| VRML 1.0/2.0 | .wrl, .vrml | ✔ | ✔ (senza materiali) |
| Wavefront | .obj | ✔ | ✔ (con texture) |

### Note per l'implementazione
- Pipeline: file CAD → formato intermedio (PRC, tramite libreria commerciale HOOPS Exchange secondo le release notes) → albero di assieme → mappatura su nodi/feature/componenti → tessellazione → raggruppamento in geometry set → pulizia. Per un clone open source va scelto un kernel alternativo (es. OpenCASCADE per STEP/IGES/BREP, Assimp per mesh): l'help non lo dice, è una nostra ipotesi.
- Vincolo dati: **un materiale per geometry set**; i set collassati vengono spezzati oltre 10.000 triangoli o 16.000 punti.
- Export DWG: un layer per oggetto (nome componente o proprietà `LayerName`), colore dal materiale; materiale nullo/nero → colore del layer. In import DWG le geometrie senza colore diventano nere.
- Point cloud .xyz: separatore spazio o TAB.
- I dati BREP possono convivere con la mesh e la ritessellazione è reversibile finché non si scaricano.

---

## 5. Layout Configuration — layout, sub-layout, connessioni (LAY)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| LAY-01 | Costruzione layout | Layout vuoto con pavimento e griglia all'avvio; si popola con componenti da catalogo o CAD; editing singolo o multiplo delle proprietà comuni. | Nomi univoci; proprietà condivisibili (es. PDFExportLevel) | Layout_Configuration_Building_a_layout.txt |
| LAY-02 | File layout VCMX | Archivio nativo che contiene almeno dati di scena, metadati, anteprima e icona per il catalogo. | `layout.rsc`, `component.dat`, `layout_preview_icon.tga`, `layout_icon.tga` | Layout_Configuration_Layouts.txt |
| LAY-03 | Riferimenti VCID | Il layout salva per ogni componente VCID, posa, connessioni e valori di proprietà; geometria opzionalmente inclusa. | Include Components | Layout_Configuration_Layouts.txt; Layout_Configuration_Building_a_layout.txt |
| LAY-04 | Layout item | Oggetti visivi non-componente (quote, annotazioni, disegni, grafici, programma camera) salvati e ricreati col layout. | — | Layout_Configuration_Layouts.txt |
| LAY-05 | Impostazioni di scena salvate | Alcune impostazioni (es. vista 3D corrente, viste nominate) sono persistite nel layout. | — | Layout_Configuration_Layouts.txt; Getting_Started_Visualization.txt |
| LAY-06 | Merge di layout | Aprire/aggiungere un layout in un altro lo inserisce come nuovo sub-layout. | — | Layout_Configuration_Sub-layouts.txt |
| LAY-07 | Sub-layout | Suddivisione gerarchica del layout (sotto ROOT) per organizzare, copiare, spostare, nascondere sezioni; annidabili per drag & drop; rinominabili. | Create sub-layout, Move to Layout > Create new sub-layout; proprietà: nome, aspetto bounding box (colore, alpha in Advanced) | Layout_Configuration_Sub-layouts.txt |
| LAY-08 | Cell Graph – scheda Components | Elenco componenti del layout raggruppati per categoria con ricerca, filtri, espandi/comprimi, mostra invisibili; azioni per gruppo o singolo. | Lock/Unlock editing, Show/Hide, Delete | Layout_Configuration_Components_tab.txt |
| LAY-09 | Cell Graph – scheda Layouts | Albero dei sub-layout con lista componenti del sub-layout selezionato, ricerca e ordinamento/categorizzazione. | Create sub-layout, Show/Hide/Delete layout; lock/show/delete componente | Layout_Configuration_Layouts_tab.txt |
| LAY-10 | Connect Interfaces (connessioni remote) | Collega componenti tramite interfacce astratte (senza contatto fisico) con editor grafici in 3D (nodi e fili) o task pane. | Component, Interface, Connections (giallo = disponibile, verde = connesso), Display Network Map, Show Connection Lines, Show Non-abstract Interfaces; click sul componente = scelta automatica del candidato migliore; soglia visibilità `interfacesNodeVisibilityThreshold` | Layout_Configuration_Connect_Interfaces.txt |
| LAY-11 | Virtualizzazione interfaccia | Un'interfaccia non astratta senza posizione fisica può essere collegata a una astratta compatibile e diventa astratta al momento della connessione. | Show Non-abstract Interfaces | Layout_Configuration_Connect_Interfaces.txt |
| LAY-12 | Connect Signals | Collega segnali booleani tra componenti e I/O dei robot via editor 3D (fili) o tabella; ordinamento/filtri; toggle dei segnali durante la simulazione. | Solo Boolean; robot: In/Out N, porte modificabili, porte 1–99 riservate alle azioni (consigliato ≥100), azioni su uscite; Disconnect all, Clear all actions; Enable Signal Toggling (verde = true); ordinamento I/O asc/desc/per direzione | Layout_Configuration_Connect_Signals.txt; Getting_Started_UI_Overview_Tabs_Layout_View.txt |
| LAY-13 | Pattern di componenti | Clona la selezione in serie lineare o angolare con anteprima a bounding box; opzionale connessione automatica delle interfacce; i cloni ereditano il genitore. | Mode Linear/Angular; Coordinate System World/Object/Parent; Pattern Along + Linear Step; Pattern Around + Offset Along/Distance/Vector + Angular Step; Count (originale incluso); Connect Interfaces; Apply | Layout_Configuration_Creating_a_pattern_of_components.txt; Getting_Started_UI_Overview_Tabs_Layout_View.txt |

### Note per l'implementazione
- **Due tipi di connessione**: (a) fisica/PnP, legata a posizione e orientamento di frame; (b) astratta/remota, solo logica (es. robot ↔ controller, tool ↔ robot per segnali). Entrambe sono modellate come *interfacce* con sezioni compatibili; l'help non definisce la regola di compatibilità (tipi/sezioni/campi) e non spiega i criteri del "best candidate".
- **Segnali**: grafo di connessioni tra segnali booleani (uno-a-molti implicito); per i robot esistono mappe di segnali IN/OUT indicizzate per porta, con azioni predefinite (grasp/release ecc.) sulle prime porte.
- **Gerarchia del layout**: ROOT → sub-layout annidati → componenti; indipendente dalla gerarchia di attach (padre-figlio cinematica).
- Persistenza: VCID + stato (posa, proprietà, connessioni) + layout item + viste. Limite storico: layout > ~2,15 GB non salvabili (bug corretto nel 5.0, indica offset a 32 bit).

---

## 6. 3D Operations — snap, allineamento, misure, gerarchia, gruppi (3D)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| 3D-01 | Snap componenti | Posiziona la selezione su un punto scelto, sul punto medio di due punti o sul centro dell'arco per tre punti; anteprima prima del click. | Mode: 1 Point / 2 Point / 3 Point–Arc Center; Set Position, Set Orientation, Align Axis (asse e verso), Preview; Snap Type: Frame, Edge, Face, Edge & Face, Origin, Bound (angoli bounding box), cambiabile per ogni punto; ESC/Cancel, Undo/Redo | 3D_Operations_Snapping_components.txt |
| 3D-02 | Align | Allinea posizione e/o orientamento della selezione usando due punti picked. | Settings: posizione/orientamento/asse; Snap Type | 3D_Operations_Aligning_components.txt |
| 3D-03 | Attach / Detach | Rende un componente figlio di un nodo di un altro (si muove con esso); freccia blu indica il genitore; box rosso indica il nodo puntato. | Attach non aggancia né posiziona; eliminare il padre elimina i figli | 3D_Operations_Attaching_components.txt |
| 3D-04 | Measure | Misura distanza e/o angolo tra due punti con anteprima; risultato in 3D e nel pannello Output. | Mode distance/angle/both; Settings (visualizzazione, sistema di coordinate); Snap Type; CTRL = centro geometrico; Display accuracy (decimali) | 3D_Operations_Measuring_components.txt |
| 3D-05 | Quota 3D | Quota persistente (layout item) lineare o angolare tra punti/spigoli nel mondo 3D. | Linear, Angular; due punti + punto di riferimento; Snap Type | 3D_Operations_Creating_a_3D_dimension.txt; Getting_Started_UI_Overview_Tabs_Layout_View.txt |
| 3D-06 | Gruppi di componenti | Raggruppamento per selezione rapida; un componente può stare in più gruppi; gruppi annidabili; selezione del singolo dentro il gruppo attivo. | Group, Ungroup (rimuove solo l'ultimo gruppo aggiunto; elimina gruppi, non componenti) | 3D_Operations_Grouping_components.txt |
| 3D-07 | Grid snap del manipolatore | Spostamento/rotazione a passi discreti. | Size, Automatic Size, Always Snap | Getting_Started_UI_Overview_Tabs_Layout_View.txt |
| 3D-08 | Editing origine del componente | Sposta o aggancia l'origine (frame di riferimento) del componente senza muovere la geometria. | Origin Snap (1/2/3 punti), Origin Move, Apply | Getting_Started_UI_Overview_Tabs_Layout_View.txt |

### Note per l'implementazione
- Primitivi di picking condivisi da Snap/Align/Measure/Dimension/Annotation: **Frame, Edge, Face, Edge & Face, Origin, Bound** + modificatori per centri geometrici (mezzeria spigolo, centro faccia). Conviene un unico servizio "snap picker".
- 3 punti → centro della circonferenza passante; 2 punti → punto medio.
- La gerarchia di gruppo è ordinata (stack): Ungroup toglie l'ultimo livello.
- Attach è ortogonale a PnP: alcune interfacce, però, fanno attach automatico (tool su flangia, sensori su conveyor).

---

## 7. Simulation — esecuzione, tempo, statistiche, output (SIM)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| SIM-01 | Controlli di simulazione | Barra in alto nella vista 3D per avvio/pausa, reset, orologio, velocità, registrazioni. | Settings, Reset, Play/Pause (SPACE), Clock, Speed −/slider/+, Record 3D PDF/Video/Animation | Simulation_Simulation_controls.txt |
| SIM-02 | Esecuzione | Avviando, i componenti eseguono i propri compiti (programmi robot, trasporto su conveyor, ecc.). | — | Simulation_Running_a_simulation.txt |
| SIM-03 | Reset | Riporta la scena allo stato salvato a inizio simulazione. | Vedi note | Simulation_Simulation_controls.txt |
| SIM-04 | Save State | Fotografa posa e configurazione di tutti i componenti come stato iniziale; fatto automaticamente all'avvio. | Settings > Save State | Simulation_Simulation_controls.txt |
| SIM-05 | Modalità tempo | Tempo reale (scalabile) o tempo virtuale (massima velocità con passo di rendering). | Simulation Mode: Real Time (fattore di scala) / Virtual Time (step size, es. 0,3 s = un frame ogni 0,3 s simulati) | Simulation_Simulation_controls.txt |
| SIM-06 | Velocità | Regolazione della velocità con slider e pulsanti. | Speed | Simulation_Simulation_controls.txt |
| SIM-07 | Durata (runtime) | Simulazione infinita o limitata a un tempo impostato. | Simulation Run Time: ∞ / Custom; Clock Display Mode (formato orologio) | Simulation_Using_a_runtime.txt |
| SIM-08 | Repeat | Ripete in loop una simulazione con durata/fine definita. | Repeat | Simulation_Repeating_a_simulation.txt |
| SIM-09 | Warm-up | Istante da cui la simulazione "conta" (tipicamente per iniziare a raccogliere statistiche a regime). | Warm Up Time; Reset = 0 | Simulation_Using_a_warm_up_time.txt |
| SIM-10 | Simulation Level globale | Accuratezza globale dei movimenti. | Default (delega al componente) / Detailed / Balanced / Fast | Simulation_Simulation_controls.txt |
| SIM-11 | Dashboard statistiche | Cruscotto a schede con layout a griglia di grafici; grafici nuovi o da template (template richiede componente selezionato con behavior Statistics). | Tab, Choose/Change Layout (non elimina grafici salvo meno posti) | Simulation_Reporting_statisticsv2.txt |
| SIM-12 | Proprietà grafico | Ogni grafico ha nome, legenda, titolo, intervallo di campionamento proprio e serie di dati. | Name, Legend Visibility, Title Visibility, Sampling Interval (−1 = globale), Type; Data Series: Index, Name, Components (lista), Property (Real/Integer), Thickness | Simulation_Reporting_statisticsv2.txt |
| SIM-13 | Intervallo di campionamento globale | Frequenza globale di report delle proprietà dei behavior Statistics (cumulative, a intervalli, basate su stato). | Interval (default 60 s) | Simulation_Reporting_statisticsv2.txt |
| SIM-14 | Lettura grafici | Tooltip sui valori, zoom con rotella, pan con scrollbar, selezione di un intervallo, reset vista. | — | Simulation_Reporting_statisticsv2.txt |
| SIM-15 | Stampa grafici | Stampa del grafico selezionato o di tutti quelli di una scheda; export XPS. | Print Chart(s), File > Print > Statistics Chart | Simulation_Reporting_statisticsv2.txt |
| SIM-16 | Export dati statistici | Export in Excel (un foglio per grafico) o CSV (un file per grafico). | Export Graph > Excel/CSV | Simulation_Reporting_statisticsv2.txt |
| SIM-17 | Pannello Output | Messaggi di simulazione/applicazione per categoria, con filtri, espansione, ricerca avanzata, impostazioni, pulizia; messaggi copiabili. | Error/Warning/Info; Clear All; API Python 3/.NET per estenderlo | Simulation_Output_panel.txt; Release_Notes_release_notes_5.0.txt |
| SIM-18 | Stop su collisione / limiti | Ferma la simulazione a collisione rilevata o a superamento limiti (dalla scheda Program). | Stop on collision, Stop at limits, Detectors Active, Message Panel Output | Getting_Started_UI_Overview_Tabs_Robot_View.txt |
| SIM-19 | Toggle segnali in run | Forzatura manuale dei segnali connessi durante la simulazione. | Enable Signal Toggling | Layout_Configuration_Connect_Signals.txt |

### Note per l'implementazione
- **Ciclo di vita**: `Stopped(stato iniziale salvato)` → Play → `Running` ⇄ `Paused` → Reset → ripristino dello stato salvato. Save State viene fatto implicitamente al primo Play dopo un reset.
- **Cosa ripristina il Reset**: componenti dinamici (creati in simulazione) eliminati; componenti statici a posa, gerarchia e connessioni iniziali; valori di giunti/assi; frame base e tool; segnali e variabili robot ai *default* (non fanno parte dello stato salvato). La vista camera NON viene ripristinata. Tutti i grafici statistici vengono azzerati al nuovo run.
- **Tempo**: tempo di simulazione in secondi (unità canonica). Real Time = tempo simulato agganciato al wall clock × fattore; Virtual Time = esecuzione "as fast as possible" con step di rendering fisso. Da questo segue l'architettura: un motore a eventi discreti + interpolazione cinematica, con un renderer che campiona a step.
- **Warm-up**: la simulazione parte (o le statistiche iniziano) dopo un tempo T_wu; Repeat = riavvio automatico (reset + play) alla fine del runtime.
- **Statistiche**: i grafici sono layout item (salvati) ma i dati no. Ogni serie = (lista componenti, nome proprietà numerica) campionata a intervallo locale o globale. I tipi di statistica (cumulative, interval, state-based) provengono dal behavior Statistics del componente (documentato in altra sezione dell'help).
- Le funzioni pseudo-casuali delle espressioni (EXPR-05) e gli stream con seed sono il fondamento della stocasticità; bug 48000 (5.1) indica che il generatore deve essere deterministico a parità di seed.
- Motore fisico: PhysX 5.5 (5.1) con richiesta di determinismo (bug 47441).

---

## 8. Drawings — disegni 2D, template, BOM (DRW)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| DRW-01 | Creazione disegno | Genera una vista a linee del layout 3D, manuale (selezione di un'area) o automatica da vista ortografica standard. | Create View: Select / viste standard; proprietà Scale | Drawings_Create_a_drawing.txt |
| DRW-02 | Quote nel disegno | Quote lineari o angolari tra punti/spigoli con punto di riferimento; editabili direttamente o da Properties; stile ereditabile dal template. | Linear, Angular, Snap Type; altezza, spessore frecce | Drawings_Creating_a_dimension.txt |
| DRW-03 | Annotazioni | Richiami rettangolari, a palloncino o testo libero; possono mostrare un'immagine invece del testo; stile modificabile anche in multiselezione. | Rectangle, Balloon, Text; ancoraggio + posizione etichetta; Is image, Texture Path, Texture Width/Height | Drawings_Creating_an_annotation.txt |
| DRW-04 | Distinta base (BOM) automatica | Genera palloncini e tabella parts list nel template a partire dai disegni e dalle proprietà BOM dei componenti; selezione dei disegni dalla tabella. | BOM > Create; proprietà componente BOM, BOMname, BOMdescription | Drawings_Creating_an_annotation.txt; Drawings_Create_a_drawing.txt |
| DRW-05 | Import template di disegno | Carica un foglio (template) che dà scala, titolo, cartiglio, BOM. | Load Template; Title, Scale | Drawings_Importing_a_drawing_template.txt |
| DRW-06 | Creazione template | Un template è esso stesso un componente: geometria del foglio (es. A4 ~297×209 mm), feature di testo, script Python per scala/campi, interfaccia per la parts list; salvato nella cartella Drawing Templates. | Feature Geometry, PythonScript, PartListInterface; metadati nuovi; export DXF per ridisegno | Drawings_Creating_a_drawing_template.txt; Drawings_Finding_a_drawing_template.txt |
| DRW-07 | Export disegno | Esporta disegni selezionati o tutti in un formato file supportato. | Components to export: Selected/All; File Format | Drawings_Exporting_a_drawing.txt |
| DRW-08 | Stampa disegno | Anteprima di stampa con adattamento al foglio o selezione area, stampante, orientamento, formato carta, filigrana; export XPS. | Area selection, Printer, Page Orientation, Paper Size; watermark: immagine, scala, opacità | Drawings_Printing_a_drawing.txt |

### Note per l'implementazione
- Tutto ciò che sta nel mondo disegno è un **layout item** salvato solo dentro il layout (un disegno non è salvabile separatamente) e rigenerato al caricamento.
- Template = componente parametrico con script: il clone può modellarlo come "componente foglio" con campi testuali e uno slot che raccoglie i disegni agganciati per costruire la BOM.
- Il formato di export dei disegni non è elencato in questa pagina (probabilmente DWG/DXF, da IMP-12; ipotesi).
- La qualità della generazione delle linee 2D è regolata da Display options (SET-03).

---

## 9. Export — geometria, immagini, video, PDF 3D, animazioni, streaming (EXP)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| EXP-01 | Export geometria componenti | Esporta la geometria dei componenti selezionati in un formato CAD supportato. | Components to export: Selected; File Format | Export_Exporting_geometry_of_components.txt |
| EXP-02 | Export geometria layout | Esporta l'intero layout. | Components to export: All | Export_Exporting_geometry_of_layouts.txt |
| EXP-03 | Export immagine | Cattura l'area dentro un riquadro rosso nella vista 3D in file o negli appunti; esclusi overlay UI (view selector, controlli, evidenziazioni). | Resolution (orientamento, dimensione), File Format / Clipboard, Render Mode | Export_Exporting_simulation_as_image.txt |
| EXP-04 | Stampa vista 3D | Stampa vettoriale della vista 3D o del disegno corrente; export XPS. | 3D View / Current Drawing, Printer, Page Orientation, Paper Size | Export_Printing_simulation_as_image.txt |
| EXP-05 | Registrazione video | Registra la simulazione nell'area del riquadro; la camera si può muovere durante la registrazione (anche con componente "Fly Camera"). | Resolution, Frame rate, Video format (MP4/H.264; MOV deprecato), Quality | Export_Recording_simulation_as_video.txt; Release_Notes_5.0_api_deprecations_and_removals.txt |
| EXP-06 | Registrazione 3D PDF | Registra animazioni e movimenti in un PDF 3D con template; esclusione componenti tramite proprietà. | Template, Frame rate, Title; Background color; PDF ExportLevel (No Export ecc.), BOM=false; nomi esclusi → `_unknown_` | Export_Recording_simulation_as_3D_PDF.txt |
| EXP-07 | Registrazione animazione VCAX | Registra un'animazione riproducibile in viewer mobile o VR. | Frame rate; esclusione con invisibilità o PDF ExportLevel | Export_Recording_simulation_as_animation.txt |
| EXP-08 | FBX – geometria | Esporta geometria e cinematica diretta (scheletro, limiti giunti) in FBX. | Export all/selected; Use alphanumeric names; Include Materials/Textures/Skeleton/Limits; Up Axis; Units | Export_FBX_Exporter.txt |
| EXP-09 | FBX – animazione | Registra la simulazione come FBX animato. | Come EXP-08 senza Limits + Frame rate; Start Recording / Stop and save | Export_FBX_Exporter.txt |
| EXP-10 | Streaming live | Trasmette la scena in tempo reale a un client viewer (desktop o mobile, desktop/VR) via IP:porta; Play diventa blu con client connesso. | Enabled, Port 7000; client: host IP, porta, modalità Desktop/VR, sorgente "Stream - host:port" | Export_Streaming_animation.txt |
| EXP-11 | Camera Animator | Programma di movimenti camera durante la simulazione (layout item): sequenza di viste con tempo e interpolazione. | Simulation Time (assoluto) / Relative Time; View (esistente o nuova); Time; Arc (arco vs lineare); +/cestino; Camera animation enabled; API vcViewAnimation | Export_Animating_camera_movements.txt |
| EXP-12 | Export FactoryLens | Immagine, video, USD dal viewport RTX (vedi UI-45). | — | Getting_Started_FactoryLens.txt |

### Note per l'implementazione
- **Registrazioni** (PDF, video, VCAX, FBX animato) condividono lo schema: impostazioni → Start Recording (scelta file) → simulazione in corso → stop da controlli o da task pane. Si possono avviare/fermare in qualunque momento della simulazione. Serve un "recorder" agganciato al clock di simulazione con frame rate proprio.
- **Esclusione dall'export**: proprietà per componente `PDF ExportLevel` (valori non elencati qui oltre "No Export"), `Visible`, `BOM`.
- FBX: geometrie supportate GeometrySet, LineSet, materiali, texture, testo 3D come mesh; limiti: follower esportati come giunti indipendenti, giunti custom solo se a un asse, niente point cloud, modelli non skinnati.
- Camera Animator: keyframe (view, t, arc flag) con tempo assoluto o relativo al keyframe precedente; interpolazione lineare o ad arco.
- Streaming: protocollo proprietario non documentato (solo host/porta). Per un clone: WebSocket/gRPC con stream di trasformazioni.

---

## 10. Expressions — linguaggio di espressioni (EXPR)

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| EXPR-01 | Tipi | Sistema di tipi del linguaggio con nomi di cast. | Boolean `bool`, Integer `int` (32 bit con segno), Real `double` (64 bit IEEE), Vector `rVector` (4 reali), Matrix `rMatrix` (4×4), String `rString`, Reference `object`, List (tipizzata, senza cast) | Expressions_Types_and_Operators.txt |
| EXPR-02 | Operatori | Logici con short-circuit, aritmetici (divisione intera tra interi), confronto, assegnazione composta, incremento/decremento, bitwise e shift sugli interi. | `! && || + - * / % ++ -- < <= > >= == != = += -= *= /= %= <<= >>= &= ^= |= ~ & ^ | << >>` | Expressions_Types_and_Operators.txt |
| EXPR-03 | Precedenza e associatività | 15 livelli: postfissi/membro/indice/chiamata/unità → unari/cast → moltiplicativi → additivi → shift → relazionali → uguaglianza → & → ^ → | → && → || → assegnazioni (destra-sinistra) → condizionale → sequenza. | Vedi colonna | Expressions_Operator_Precedence.txt |
| EXPR-04 | Raggruppamento | Parentesi, operatore condizionale `?:`, operatore sequenza `,` (restituisce l'ultimo). | — | Expressions_Grouping_Operators.txt |
| EXPR-05 | Funzioni matematiche | Trigonometriche (radianti; suffisso `d` per gradi), triangoli e quadrilateri generici, conversione angoli, potenze/log, iperboliche, varie. | `sin cos tan asin acos atan atan2`, `sasa sass sssa`, `sasasa sasass sasssa1 sasssa2`, `degree radian`, `pow sqrt exp log log10`, `sinh cosh tanh`, `abs ceil floor inrange max min mod ramp range round` | Expressions_Functions.txt |
| EXPR-06 | Distribuzioni pseudo-casuali | Campionamento da distribuzioni statistiche, con o senza stream esplicito (solo gli stream valgono negli statement di process modeling). | `normal lognormal uniform exponential gamma triangular weibull` (+ variante con primo argomento `stream`) | Expressions_Functions.txt |
| EXPR-07 | Costruttori vettore/matrice | Funzioni che restituiscono matrici di trasformazione/vettori; i metodi omonimi sono concatenabili (es. traslazione poi rotazione). | `Identity Pos Tx Ty Tz Rx Ry Rz Sx Sy Sz Euler WPR IJK AxisAngle Quaternion Set(16 valori, per colonne N,O,A,P) Vector(x,y,z[,w])` (angoli matrice in gradi) | Expressions_Functions.txt; Expressions_Types_and_Operators.txt |
| EXPR-08 | Membri e metodi dei tipi complessi | Vettore: X,Y,Z,W, Length, Square, Set; operatori: negazione, scala, prodotto scalare `*`, vettoriale `^`, somma. Matrice: inversa (`-M` o `~`), uguaglianza, M*V, M*M, M/M, `*=` pre-moltiplicazione. Stringa: `length`, confronto lessicografico, concatenazione. Reference: accesso a proprietà, confronto con stringa (sul nome), cast a bool/string. List: indice, `length()`, riferimenti a oggetti cancellati → Null. | — | Expressions_Types_and_Operators.txt |
| EXPR-09 | Letterali | Interi decimali/ottali (0 iniziale)/esadecimali (0x), reali con esponente, stringhe tra doppi apici con escape; niente letterali booleani (usare 1/0). | — | Expressions_Types_and_Operators.txt |
| EXPR-10 | Conversione di tipo | int ↔ double automatica; cast esplicito con troncamento (`(int)2.5` → 2); bool ↔ numerico (0 = falso); cast a stringa non supportato (tranne Reference). | — | Expressions_Type_Conversion.txt; Expressions_Types_and_Operators.txt |
| EXPR-11 | Specifica di unità | Suffisso `valore{unità}` per dichiarare l'unità di un letterale o sottoespressione; risultato reale in unità canonica; unità miste consentite. | `10{cm}` → 100.0; nomi lunghi o sigle; ~80 unità built-in + custom | Expressions_Unit_Specification.txt |
| EXPR-12 | Espressioni negli statement di processo | Accesso per punto a proprietà di prodotto, tipo prodotto, componente, gruppo, flow info, assembly/slot/step/pattern; indice su liste, `length()`, `SimTime()`. | Validi in Assign Variable, Delay, If, Print, Switch Case, Wait Property Condition, While | Expressions_Process_Statement_Expressions.txt |
| EXPR-13 | Espressioni negli statement robot | Riferimento a porte delle mappe di segnali booleani: `IN[n]`, `OUT[n]` o `<NomeMappa>[n]`. | — | Expressions_Robot_Statements.txt |

### Note per l'implementazione
- Linguaggio in stile C, **case sensitive**, con unità fisiche come operatore postfisso ad alta precedenza. Le unità *non convertono* in visualizzazione: normalizzano il valore all'unità canonica (mm, s, deg, g, N, Nmm, K, mm/s, deg/s, kg·m/s ...).
- Ambiguità da gestire: `~` è complemento bitwise sugli interi e inversa sulle matrici; `^` è XOR sugli interi e prodotto vettoriale sui vettori; `-Matrix` è inversa (non negazione).
- Trigonometria in radianti ma le funzioni matrice in gradi: incoerenza voluta, documentata.
- `atan2(a,b)` è definita come atan(b/a) con a≠0 (ordine argomenti invertito rispetto a C, da verificare).
- `round` arrotonda per eccesso a metà; `ceil/floor/round` restituiscono reali.
- `range(a,b,c)` = clamp di b in [a,c]; `inrange(a,b,c)` = test; `ramp` = interpolazione lineare tra due punti.
- Elenco unità built-in completo nel file sorgente (lunghezza, area, volume, volume liquido, massa, tempo, angolo, velocità/accelerazione lineari e angolari, forza, coppia, quantità di moto, momento angolare, temperatura). Due refusi nella fonte: "newton metres" con sigla Nmm, "short tons" e "tonnes" entrambi "t".

---

## 11. What's New e Release Notes (REL) — solo elenco funzionalità

| ID | Funzionalità | Cosa fa | Parametri/proprietà chiave | Fonte |
|---|---|---|---|---|
| REL-01 | FactoryLens (5.1.1) | Viewport ad alta fedeltà Omniverse con export immagine/video/USD; niente aggiornamenti geometrici dinamici né point geometry. | — | Release_Notes_release_notes_5.1.1.txt; WhatsNew.txt |
| REL-02 | Nuovo motore fisico (5.1) | PhysX 5.5 con più stabilità e prevedibilità. | — | Release_Notes_release_notes_5.1.txt |
| REL-03 | Behavior di navigazione (5.1) | Pathfinding consapevole dell'ambiente con navmesh automatiche, agenti umani/AGV/AMR, evitamento, multi-agente; API Python 3. | Navigation Controller, Pathways, Resources, Transport Controllers | Release_Notes_release_notes_5.1.txt; WhatsNew.txt |
| REL-04 | Raycast sensor con filtro contenitore (5.1) | Limita i componenti rilevati per migliorare le prestazioni. | Container Filter | Release_Notes_release_notes_5.1.txt |
| REL-05 | Localizzazioni (5.1) | UI anche in italiano, portoghese, vietnamita. | — | Release_Notes_release_notes_5.1.txt |
| REL-06 | Connettori robot/PLC | 5.0: Denso, Techman, MELFA, Yamaha, MQTT client (SUB/PUB, TLS, QoS, template, trasformazioni in/out), LS Electric PLC, Fronius; 5.1: Allen-Bradley, Nachi, Epson; aggiornamento OPC UA; Beckhoff ADS citato. Disponibilità per edizione (Premium = tutti). | — | Release_Notes_release_notes_5.0.txt; _5.1.txt; WhatsNew.txt |
| REL-07 | API Python 3 (5.0) | Nuova API su Python 3.12 con async/await, pip (NumPy, Pandas, SciPy), `help()`, comandi auto-caricati con prefisso `cmd`; Python 2 opzionale. | Cartelle Python 3 e My Commands | Release_Notes_release_notes_5.0.txt |
| REL-08 | Python Statement / Custom Python Statement (5.0) | Statement di processo che esegue script Python 3 in simulazione; API per statement custom. | Statement Gallery | Release_Notes_release_notes_5.0.txt |
| REL-09 | Nuovo Motion Controller (early access) | Pipeline di moto robot con interpolazione più fluida e cinematiche migliorate; nuovi solver (Articulated 6 ZYYYZY, ZYYZYZ 46Y, Delta revolute). | — | Release_Notes_release_notes_5.0.txt; WhatsNew.txt |
| REL-10 | Path Solver (OLP) | Pianificazione di percorsi senza collisioni, sostituisce "Check Path". | — | Release_Notes_release_notes_5.0.txt |
| REL-11 | Collider (5.0) | Geometrie di collisione semplificate (Box, Sphere, Convex) in un behavior contenitore; tool di decomposizione convessa; detector possono usarle o ignorarle. | Qualità collider | Release_Notes_release_notes_5.0.txt |
| REL-12 | Output panel migliorato (5.0) | Categorie, filtri, copia, impostazioni, API. | — | Release_Notes_release_notes_5.0.txt |
| REL-13 | Licenze | Credenziali/VC Accounts (early access), Network License Server 3.0 per licenze flottanti, telemetria con impostazioni analytics. | — | Release_Notes_release_notes_5.0.txt; Release_Notes_network_license_server_3.0.txt |
| REL-14 | Estensione VS Code | VC Python Tools: IntelliSense per l'API. | — | Release_Notes_release_notes_5.0.txt |
| REL-15 | Aggiornamenti formati CAD | HOOPS Exchange 2025.6 → 2026.2; nuovo DGN; STPX. | — | Release_Notes_release_notes_5.0.1.txt; _5.0.txt |
| REL-16 | Funzioni OLP (welding ecc.) | Numerosi miglioramenti di post-processor (ABB, Fanuc, Yaskawa, Comau, ...), import XML, ricerche/saldature, calibrazione TCP: fuori ambito di questa mappa. | — | Release_Notes_release_notes_5.0*.txt; _5.1*.txt |
| REL-17 | Deprecazioni/rimozioni API 5.0 | Elenco classi Python 2 e interfacce .NET rimosse/rinominate (es. vcBehaviour→vcBehavior, path → MotionPath, world manager), formato MOV deprecato. | — | Release_Notes_5.0_api_deprecations_and_removals.txt |
| REL-18 | Varie | Creazione/eliminazione frame dal pannello Jog; Material Properties e nuovi materiali; refresh eCat sync; percorso default eCatalog configurabile. | — | Release_Notes_release_notes_5.0.txt; WhatsNew.txt |

### Note per l'implementazione
- Le release notes rivelano requisiti non funzionali: determinismo del generatore casuale e della fisica, file di layout > 2 GB, prestazioni UI tra schede, memoria nei pannelli dockabili.
- Nomi di behavior emersi (utili per l'inventario del modello a componenti): MotionPath one/two-way, Statistics, Raycast sensor, Collider container, Servo controller, PythonScript, ComponentCreator, Executor robot, Boolean Signal Map.

---

## Lacune (cosa l'help, nelle pagine lette, non spiega)

1. **Regole di compatibilità delle interfacce** (PnP e astratte): struttura di sezioni/campi, tolleranze di distanza e angolo per lo snap, criterio del "best candidate", quando una connessione fisica produce anche un attach.
2. **Semantica del motore di simulazione**: tipo di scheduler (eventi discreti vs. passo fisso), ordine di aggiornamento dei componenti, propagazione dei segnali nello stesso istante, rapporto con il passo della fisica.
3. **Formato interno dei file**: contenuto di `layout.rsc` e `component.dat`, struttura del VCID, formato VCAX, formato descrittore XML delle sorgenti remote del catalogo, schema del DB SQLite.
4. **Protocollo di streaming** verso Visual Components Experience (solo host/porta).
5. **Valori ammessi di PDF ExportLevel** e di BackfaceMode, e regole di Material Creation Rule.
6. **Modello statistiche**: definizione precisa di statistiche cumulative, a intervallo e basate su stato, stati misurati, tipi di grafico disponibili e template (rimandano alla pagina del behavior Statistics, fuori ambito).
7. **Warm-up e Repeat**: se durante il warm-up la simulazione gira normalmente senza registrare dati o se "salta" il tempo; cosa succede ai grafici a ogni ciclo di Repeat.
8. **Real Time**: comportamento quando il calcolo non tiene il passo del tempo reale; range dello slider di velocità.
9. **Area di selezione**: soglia di "occupazione significativa" per includere un componente.
10. **Formati di export dei disegni** e delle immagini (elenchi non presenti nelle pagine).
11. **Espressioni**: precedenza dei due operatori sovraccaricati (`^`, `~`) sui tipi complessi, comportamento in overflow/divisione per zero, semantica esatta dei parametri di `sasa/sass/sssa/sasasa...` (nomi lati/angoli non definiti), funzioni definibili dall'utente (assenti?), unità custom (come si definiscono).
12. **Smart collection**: elenco dei campi e operatori di confronto disponibili (solo "Is" negli esempi).
13. **Undo/Redo**: granularità e quali operazioni sono annullabili (citato solo per Snap/Align/Interact).
14. **Gruppi vs sub-layout**: rapporto tra i due meccanismi di raggruppamento (un bug cita "component group" lasciato da un sub-layout cancellato).
15. **Guided Workflow**: formato dei file workflow e catalogo dei "comandi" eseguibili negli step.
16. **Simulation Level**: algoritmo concreto di Balanced/Fast.
