# Open Industry Project: comportamenti osservati nell'app

Data: 2026-11-08. URL: https://www.openindustryproject.org/app/.
Build indicata in Settings: `0572868 · 2026-11-07`, coerente con il checkout della ricerca 12.
Progetto locale di prova: `Twinova M4 behavior study`. Viewport principale: 1600 × 1000.

## Metodo e significato delle evidenze

- **Provato**: azione eseguita e cambiamento osservato.
- **Esposto**: controllo, proprietà, tooltip o shortcut osservato; semantica runtime non ancora dimostrata.
- **Non verificato**: dipendenza esterna o combinazione non esercitata; non è un test superato.

L'esplorazione usa l'app online, senza invocare direttamente gli store interni. I sorgenti già letti sono una
fonte separata. Il materiale esterno è conservato in `D:\tmp\Twinova-references\oip-live-2026-11-08`.
Le catture grezze non sono asset Twinova e non vengono incorporate nel prodotto.

## Primo avvio, progetto ed editor

| ID | Comportamento | Evidenza | Stato |
|---|---|---|---|
| UI-01 | Primo avvio con tutorial di 16 passi e proposta di traduzione in italiano | `oip-01-start.png`; Keep English e Start azionati | Provato per i primi due passi |
| UI-02 | Layout con albero/cronologia, viewport, inspector e dock Parts/PLC/Problems/Watch | `oip-02-workspace.png` | Provato |
| UI-03 | New Project apre dialogo, accetta nome e cambia progetto/titolo | Progetto di prova dedicato creato | Provato |
| UI-04 | Catalogo → scelta nastro → click nel viewport inserisce e seleziona | `oip-03-belt-inspector.png` | Provato |
| UI-05 | Escape termina il posizionamento ripetuto | Stesso nastro selezionabile dall'albero | Provato |
| UI-06 | Nome, lunghezza e larghezza modificano l'istanza | Nome `Audit belt A`, lunghezza 4→3 m, larghezza 1.524→0.6 m | Provato |
| UI-07 | Cronologia descrive le singole modifiche e permette il ritorno a uno stato | `oip-05-history.png`; ritorno a Edit Segments ripristina Width a 1.52 m visualizzati | Provato |
| UI-08 | Parametri raggruppati, filtro proprietà, sezioni espandibili | Inspector nastro | Esposto |
| UI-09 | Preset nastro e tabella segmenti; edit manuale produce Custom | Inspector dopo modifica della lunghezza | Provato |
| UI-10 | Visible distingue visibilità da partecipazione a collisioni/simulazione | Tooltip Visible | Esposto; collisioni a oggetto nascosto non provate |

## Agganci e navigazione

`oip-06-snap-options.png`: menu aperto; traslazione impostata a 1 m, rotazione a 15°, face snap On,
vertex snap con base Vertex/Origin e sorgente Collision/Mesh. Il tooltip descrive esclusioni per piccoli carichi.
La barra distingue coordinate Local/Global, griglia, part snap, floor snap e frecce di flusso.
È stata esercitata la rotella zoom; il matching geometrico automatico non è dimostrato dall'apertura del menu.

`oip-09-keyboard.png`: inventario dei comandi esposti. Ctrl+Z/redo, duplicazione/copia/taglio/incolla,
lock/unlock, hide, select-as-one, reparent-to-new-node, focus, ortografica/prospettiva, viste cardinali,
free fly, pilot mode, comando palette e passo di fisica. Elenco acquisito, non esecuzione di ogni scorciatoia.

## Catalogo caricato: 37 famiglie

| Categoria | Famiglie |
|---|---|
| Equipment (16) | Six-Axis Robot, Diverter, Blade Stop, Belt Conveyor, Roller Conveyor, Flex Conveyor, Turntable Conveyor, Curved Belt Conveyor, Curved Roller Conveyor, Chain Transfer, Spur Belt Conveyor, Spur Roller Conveyor, Pallet Dispenser, Stretch Wrapper, Gantry, AGV |
| Devices (7) | Stack Light, Photo Eye, Color Sensor, Laser Sensor, Push Button, Barcode Scanner, Light Curtain |
| Products (5) | Spawner, Despawner, Box, Pallet, Unit Load |
| Structure (9) | Safety Gate, Rack, Guard Rail, Platform, Stairs, Floor Marking, Fence, Safety Bollards, Shape |

Il conteggio considera le famiglie con il comando **Edit … in the Part Editor**. Building e Annotation sono
altri elementi di scena, non inclusi nelle 37 famiglie. Le librerie remote aggiuntive possono ampliare il catalogo.
Le schermate individuali sono in `catalog/`, denominate con il nome della famiglia.

## Proprietà e contratti industriali emersi

| Famiglia | Controlli/semantica esposti, non tutti provati in dinamica |
|---|---|
| Belt Conveyor | Segmenti lunghezza/inclinazione, quota, velocità anche negativa, rampe, attrito, sponde e aperture, supporti, velocità REAL, marcia BOOL, feedback BOOL |
| Diverter | Tempo e distanza di deviazione, stato iniziale, comando Extend BOOL |
| Blade Stop | Larghezza, altezza di sollevamento, stato Active, comando Raise BOOL |
| Turntable | Velocità del trasporto, angolo, velocità di rotazione, target REAL, Done BOOL, Enable BOOL |
| Photo Eye | Portata, modalità di rilevamento, fascio, normalmente chiuso, filtro bersagli, presenza BOOL |
| Spawner | Portata pezzi/min, cadenza fissa, tipo e dimensioni del prodotto, massa, colori/attributi, pausa e modalità rilascio |
| Unit Load | Identità del carico, numero strati, colore casse, film e relativa opacità |
| AGV (nodi) | Mesh e frame, offset parametrici, ruote con giunti revolute, sollevamento e pistone prismatici |

Due regole particolarmente delicate sono dichiarate nei tooltip del nastro: uno Speed Tag collegato ma non
pilotato vale zero; un Run Tag collegato entra in funzione quando il programma lo pilota. Questa differenza è
registrata come **esposta**, non come validata. Per Twinova serve un contratto proprio e test espliciti.

## Definizioni e istanze

`oip-10-part-editor.png`: l'editor segnala il contesto DEFINITION e il numero di istanze. Nome/categoria dei
built-in sono disabilitati; Save as New Part offre una copia. Il testo distingue le modifiche alla definizione
dai parametri che in quel contesto servono solo all'anteprima e vengono scartati tornando alla scena.
Nell'AGV l'inspector può aprire direttamente un nodo; selezionare una famiglia non garantisce quindi la stessa
schermata iniziale per tutti i componenti. Non sono stati modificati i comportamenti condivisi dei built-in.

## PLC, preferenze e limiti

`oip-07-plc.png`: tab PLC provato; organizer con PLC1 simulato, MainTask a 10 ms, Main e Devices.
Sono presenti Reload, menu File, copia ST e apertura in finestra separata. Le verifiche dinamiche sono riportate
nel seguito del registro, distinguendo il PLC interno dalle connessioni a un controllore esterno.

`oip-08-settings.png` e `oip-settings.yml`: Preferences e Keyboard; temi, lingua, reset dock, navigazione,
collaborazione, librerie e grafica. Renderer corrente indicato come WebGPU, alternativa WebGL; qualità e ombre.
Collaborazione e server remoto non sono stati attivati.

## Prove dinamiche e round-trip completati

| ID | Passaggi ed esito osservato | Evidenza | Stato |
|---|---|---|---|
| RUN-01 | Open Demo → Start Simulation: la demo di pallettizzazione esegue, i contatori Cases crescono da 2 a 3 e poi 5 | `oip-12-watch.png`, `oip-13-paused-speed.png` | Provato; non è una misura di throughput |
| RUN-02 | Pause rende Step disponibile; Step eseguito; il pulsante 1× passa a 2× con un clic | `oip-13-paused-speed.png` | Provato; fattore temporale non misurato contro cronometro |
| RUN-03 | Stop termina il run; la demo mostra proprietà protette e richiede una copia per modificarle | Snapshot della demo e relativo menu | Provato |
| FILE-01 | Export Project scarica `.oip`; Import propone un nuovo progetto e conserva il precedente; import confermato | `oip-audit-project.oip`, `oip-14-import-restored.png` | Provato |
| FILE-02 | Dopo import: nome Audit belt A, lunghezza 3 m e larghezza 1.52 m visualizzata conservati (la larghezza era stata annullata con History) | Lettura dei campi e screenshot 14 | Provato, non confronto byte-per-byte di tutti gli asset |
| FILE-03 | Save dal menu, reload della pagina: stesso progetto, nastro e programma Ladder ritrovati | `oip-30-reload-saved.png`, `oip-final-state.yml` | Provato |
| PLC-01 | `Motor := ;` produce errore alla riga 7: `unexpected ';' in expression`; clic sul problema apre la riga nell'editor | `oip-15-st-error.png` | Provato |
| PLC-02 | Con ST non compilabile, Play rende comunque Pause/Stop disponibili; Problems dichiara non eseguiti i controlli che richiedono le dichiarazioni | Screenshot 15 e DOM | Provato per gli stati UI; non provata la logica PLC in quello stato |
| PLC-03 | Ripristinato `Motor := Start;`, Run Tag del nastro collegato a Motor, avviata simulazione; Watch mostra BOOL e Wired to Audit belt A | `oip-16-watch-binding.png` | Provato |
| PLC-04 | Scrittura di Start FALSE→TRUE: Motor diventa TRUE | Tabella Watch prima/dopo | Provato |
| PLC-05 | Force su Motor: Start riportato FALSE, Motor resta TRUE FORCED; Release nella riga Motor lo riporta FALSE | `oip-force-active.yml`, screenshot 16 e 18 | Provato |
| LD-01 | New → Ladder program, nome AuditLadder: creati programma/istanza e rung iniziale; +Rung aggiunge la seconda rete | `oip-21-ladder-editor.png`, `oip-22-ladder-two-rungs.png` | Provato per authoring; logica Ladder non collaudata |
| LD-02 | Schede contatti/bobine, timer/contatori e matematica confronti espongono blocchi dedicati | DOM delle tre schede | Esposto: non ogni blocco eseguito |
| LIB-01 | Get parts mostra Containers con Drum, IBC tote e Tote; versione, editore, firma e licenza codice MIT/modelli CC-BY-4.0 | `oip-23-library.png` | Navigazione provata; pacchetti non installati |
| EDIT-01 | Duplicate crea Audit belt A 2; Ctrl+Z rimuove la copia | Menu `oip-27-context-menu.png` e albero prima/dopo | Provato |
| VIEW-01 | Distraction Free nasconde dock; Escape ripristina. Pilot mostra navigazione WASD/E/Esc; ingresso/uscita provati | `oip-25-distraction-free.png`, `oip-26-pilot.png` | Provato per cambio modo; interazione fisica in pilot non collaudata |
| CMD-01 | Ctrl+Shift+P apre la palette: 89 opzioni nella cattura, inclusi comandi, parti e oggetti della scena | `oip-28-command-palette.png`, `oip-command-palette.yml` | Inventario esposto, non 89 test eseguiti |
| NET-01 | PLC Controllers esplicita che PLC esterni e OPC UA server richiedono desktop; distingue impostazioni del progetto da indirizzi locali | `oip-29-plc-controllers.png` | Blocco browser osservato; nessuna connessione esterna |
| SHARE-01 | Share distingue edit/view/display e avverte del nuovo storico Undo | `oip-24-share-options.png` | Solo menu; nessun link creato/coperto da test multiutente |

## Copertura delle catture e riserve

- 37 famiglie aperte nell'editor, ciascuna con screenshot e snapshot DOM in `catalog/`.
- Per 36 famiglie è stato acquisito l'inspector della parte; per AGV la cattura si apre sull'inspector del nodo
  `body` e mostra la gerarchia/giunti. Non è un inventario completo dei parametri di guida dell'AGV.
- 30 catture di percorso conservate, incluse quelle scartate sotto; totale 67 PNG. Galleria locale `index.html`.
- Scartata come prova di scena caricata: `oip-11-demo-running.png` mostra ancora compilazione shader; usare 12 e 13.
- `oip-04-history.png` mostra il nastro deselezionato: usare 05 per lo storico.
- `oip-17-force-released.png` precede il rilascio effettivo: usare 18. Il comando Release è presente sia
  nell'inspector sia nel Watch, quindi l'azione deve identificare il pannello.
- 19 mostra il menu File ST e 20 l'inserimento del nome: usare 21/22 come prova dell'editor Ladder aperto.
- Screenshot del catalogo acquisiti sistematicamente; controllo visivo approfondito su nastro, deviatore,
  arresto, tavola rotante, fotocellula, sorgente e unit load. Le altre catture sono inventario da riesaminare.
- Snap automatico tra componenti, gruppo completo, tutti i sensori, robot/AGV in ogni configurazione, export
  PLCopen, desktop I/O, collaborazione, crash recovery e tempi reali restano non verificati.

## Osservazioni UX/accessibilità circoscritte

L'inspector con unità, limiti e direzioni READ/WRITE è utile, e la cronologia rende leggibili le modifiche.
Watch espone il dispositivo collegato e rende esplicita la forzatura. L'errore ST porta alla riga interessata.
Molti comandi hanno nomi accessibili e scorciatoie; la navigazione completa con lettore di schermo non è provata.
I campi numerici della posa hanno nomi accessibili incompleti e l'inspector lungo richiede scorrimento.
Un programma non compilabile con controlli Run attivi può confondere lo stato della scena con quello del PLC.
Il segnaposto shader resta nel DOM con `aria-hidden` dopo il caricamento: le prove visive devono osservare il canvas,
non dedurre una schermata bloccata dalla sola stringa di stato.

## Implicazioni per M4

1. Tenere distinti definizione, istanza e stato runtime: l'editor deve rendere evidente quale dei tre si modifica.
2. Metadati dei parametri sufficienti a costruire inspector, unità, limiti, suggerimenti e segnali READ/WRITE.
3. Separare proprietà geometriche, segnali comandati e feedback; nascondere un oggetto non deve spegnere la macchina.
4. Distinguere gruppo di selezione, gerarchia e modello riutilizzabile; Undo deve ripristinare collegamenti e proprietà.
5. Per Twinova mantenere il DES e le prove di M3: il passo fisico di OIP non è automaticamente il nostro modello temporale.
6. La UI deve spiegare binding incompleti e valori non pilotati prima del Run.

Questa è una mappa dei comportamenti raggiunti e delle superfici esposte. Non dimostra tutte le combinazioni
di componenti, ogni script di parte, precisione fisica, prestazioni industriali o connessioni PLC reali.
