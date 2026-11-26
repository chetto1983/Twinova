# 21 — Open Industry Project: tutte le funzionalità, e il confronto con Twinova v0.4.0

Data: 2026-11-26. Richieste dell'utente: "guarda bene come funziona questa UI, è fatta bene e chiara"
(`https://www.openindustryproject.org/app/?scene=demo`) e "studia tutte le funzionalità che ha OIP sul repo di
GitHub". Le ricerche [07](07-open-industry-project.md), [12](12-m4-componenti-editor-opensource.md),
[13](13-oip-app-comportamenti-live.md), [14](14-oip-formato-progetto.md), [15](15-fisica-nastri-oip-rapier.md) e
[19](19-ladder-oip.md) guardavano OIP per un tema alla volta, al commit `0572868` (2026-11-07). Questa lo inventaria
tutto, a un commit di diciannove giorni dopo.

## Metodo e perimetro

- **Codice:** `Open-Industry-Project/Open-Industry-Project`, licenza MIT, commit `49fd7c7` del 2026-11-26, clonato con
  `--depth 1` fuori dal repository. Quattro letture parallele: interfaccia dell'editor; componenti e simulazione; PLC e
  connettività; piattaforma (desktop, progetti, collaborazione, agenti, qualità). I percorsi citati sono del loro
  repository.
- **App dal vivo:** la demo aperta in Chromium headless a 1440 × 900, temi chiaro e scuro: caricamento, selezione di
  una parte, schede del dock. Le schermate restano nella scratchpad della sessione, non nel repository (come per la 13).
- **Twinova:** l'AppImage v0.4.0 della release, avviata sotto Xvfb con rendering software e pilotata via MCP
  (`twinova-agent`), a 1440 × 900 e 1366 × 768. Vedi `docs/superpowers/verification/2026-11-26-ux-audit.md`.
- **Cosa non dimostra:**
  - di OIP non ho compilato né eseguito nulla dal sorgente: le funzioni sono lette in codice e documenti;
  - la marcia della demo non è osservata: su questo container (4 core, WebGL software) la pagina in marcia non
    risponde nemmeno a uno screenshot in 240 s;
  - i collegamenti ai PLC esterni sono quelli che il codice e le guide dichiarano; S7 è provato dagli autori contro
    snap7, non contro una CPU reale (`src/plc/DEVICE-IO.md`);
  - i numeri di test sono conteggi di file, non esecuzioni.

## 1. L'interfaccia dell'editor

**Layout** (`src/editor/dockLayout.ts`, `App.tsx`). Barra in alto da 48 px; colonna sinistra da 260 px (Scene, con
History dietro); vista 3D; colonna destra da 320 px (Inspector); dock in basso con Parts, PLC, Problems, Watch.
I pannelli si spostano tra 9 posti, le colonne si piegano a barra con un clic sulla scheda attiva, il layout si salva.
Solo il pannello PLC si stacca in una finestra o si ingrandisce. **Non c'è una barra di stato** e non c'è una modalità
"marcia" separata: in marcia si continua a modificare, e Play disabilita solo le operazioni sul documento (apri,
importa, assegna I/O), spiegando "Stop the simulation first".

**Barra in alto** (`components/Toolbar.tsx`): logo, menu progetto (nome, badge `EXAMPLE` nella demo); al centro Play
verde, Pausa, Stop e velocità (0,5–8×, ambra con la velocità reale se la macchina non tiene il passo), il chip del PLC
esterno, il chip "N forced"; a destra Share, prima persona, VR, modalità senza distrazioni, Impostazioni, Aiuto.

**Cosa la rende chiara** (dal codice e dalle schermate):
- **Ogni controllo disabilitato spiega perché** (`disabledReason` nei comandi, `commands.ts`): per esempio Reload con
  il programma in errore dice che il PLC esegue ancora quello precedente.
- **Tasti mostrati ovunque e sempre attuali:** i tooltip leggono la keymap viva (`keyHint.ts`); 89 combinazioni,
  ridefinibili dalle impostazioni.
- **Palette dei comandi** (Ctrl+Shift+P, `CommandPalette.tsx`): 80 comandi, le parti della scena e quelle del
  catalogo in una lista; i comandi disabilitati restano visibili con il motivo; sinonimi (cercando "S7" o "Modbus"
  trova Controller Properties).
- **Tutorial guidato dalle azioni vere** (`src/editor/tutorial/steps.ts`), 16 passi: guardarsi intorno, posare una
  rulliera, una scatola e una fotocellula, Play, Stop, scrivere il tag `Conveyor` nell'Inspector, il programma con
  l'interblocco, Play, vedere l'interblocco, modificare e ricaricare. Ogni passo avanza solo quando l'azione è fatta.
- **Demo che non si rompe:** è di sola lettura finché non la si modifica; la prima modifica ne fa un progetto proprio,
  con un avviso. L'Inspector senza selezione spiega il passo fisico ("16.7 ms per step… the soonest a PLC output can
  follow an input").
- **Stati vuoti scritti** (albero, Inspector, History, Problems, Watch: "Press Play to see live values and scan times").
- **Albero con gruppi** nominati per zona di linea (nella demo: "Layer-forming line", "Infeed", "Trailer unloading"),
  pulsanti ed E-stop come oggetti della scena.
- **Inspector:** parametri con unità, barra che mostra dove cade il valore nel suo intervallo e ripristino del default
  per campo; i campi accettano espressioni ("+= 600mm"); i tag hanno un'etichetta di direzione e tipo (`READ BOOL`) e un
  verdetto di collegamento (Connected / Not driven / not in program / wrong type); in marcia mostrano il valore live con
  Force/Release (`inspector/tagForce.tsx`), e un parametro comandato dal programma si colora e diventa di sola lettura.

**Design** (`src/editor/styles/base.css`): ogni colore è una coppia chiaro/scuro; testo #d7dade su #17181b in scuro
(12,7:1), testo secondario 4,96:1; corpo 13 px, molte etichette a 11–12 px. Nessuna libreria di icone: SVG disegnati a
mano. Nessun documento di design system: le regole sono di processo (comandi solo in `commands.ts`, tasti solo in
`src/core/keymap.ts`, tutti i testi via i18n con un test di copertura, screenshot prima/dopo in ogni PR visibile).

**Limiti dichiarabili dal codice:** solo inglese (le altre lingue dal traduttore del browser); nessun elenco allarmi
né conteggio errori sulla scheda Problems; Run Project Tests, Check Layout e Present solo dalla palette; scarsa
accessibilità (nessun `role="tree"` nell'albero, pochi stili di focus); circa 67 `alert()`/`confirm()` nativi.

## 2. Componenti e simulazione

**36 parti incorporate**, tutte scritte con lo stesso SDK delle parti utente (`examples/parts/<dir>/`):
- *Equipment:* nastro, rulliera, nastro flessibile a 16 sezioni con accumulo, tavola rotante, curve a nastro e a rulli,
  chain transfer, spur a nastro e a rulli, diverter, blade stop, robot a 6 assi (job Move/Vacuum/Wait/Set), gantry a 3
  assi su servo, AGV con forche, dispenser di pallet, avvolgitrice.
- *Devices:* fotocellula, sensore colore, laser, lettore barcode, barriera fotoelettrica, pulsante ed E-stop con
  lampada, colonna luminosa.
- *Products:* spawner, despawner (conteggio, rate, tempo ciclo), scatola, pallet (6 standard), unità di carico.
- *Structure:* piattaforma, scale, scaffale, recinzione, cancello di sicurezza, parapetto, dissuasori, segnaletica a
  terra, forme. In più: gruppi, annotazioni con valore PLC, edificio con porte.

**Segnali dei nastri:** `speedTag` (REAL m/s), `runTag` (BOOL, agisce solo se un programma lo pilota), `runningTag`,
velocità, impulso ed encoder. Ogni tag dichiara direzione e tipo.

**Prodotti:** scatole, pallet, tote o parti con corpo proprio; attributi come "mark" (codice, colore, avvolto,
assemblato) letti da fotocellule e lettori; assemblaggi a slot (pezzi che si fissano alla base, fixture).

**Snap:** un solo modello a plug e socket per nastro-nastro (testa-coda, fianco, curve, spur, inclinati), sensori e
diverter sulla sponda, blade stop e chain transfer sulla griglia dei rulli, recinzioni a catena, scale adattate
all'altezza della piattaforma, scaffali affiancati. L'anteprima di posizionamento è la parte vera, con la freccia di
flusso; le dimensioni cambiano solo dalle maniglie dichiarate.

**Parti proprie:** SDK (livello 20), manifest più comportamento JS in sandbox SES; giunti con servo; geometria
procedurale; import di GLB, STEP (OpenCascade in un worker), USD e URDF; Part Editor con **controllo PLC senza codice**
(due posizioni: comando, finecorsa A e B, tempo di corsa; oppure rotazione: marcia, velocità, a regime); librerie
firmate e scaffale condiviso tra progetti.

**Motore:** Jolt (fork) in un Web Worker, 60–1000 Hz per progetto (default 60), nastri come corpi statici con velocità
di superficie, un solo clock con lo scan PLC intercalato. Headless bit-riproducibile; l'editor dal vivo no.

**Stato in 3D:** nastri e rulli animati alla velocità reale e fermi da spenti; frecce di flusso verdi (tasto "."); raggi
dei sensori **verdi liberi e rossi interrotti**, il sensore colore mostra il colore letto; lampade di pulsanti, colonne,
diverter (rosso/verde), dispenser; targhe con letture; **annotazioni con il valore PLC live** ancorate nella scena
("--" se il tag non si legge, ON/OFF per i BOOL).

## 3. PLC e connettività

**PLC interno:** sottoinsieme IEC 61131-3 di ST e LD (il Ladder è tradotto in ST, un solo runtime compilato in
WebAssembly, con l'interprete AST come riferimento). Tipi completi, OOP della terza edizione, `AT %I/%Q/%M`, TON/TOF/TP,
trigger, contatori, SR/RS, funzioni standard; task con intervallo e priorità (default 10 ms); fino a 32 controllori per
scena (`Cell2::tag`); motion PLCopen ad asse singolo (MC_Power, MoveAbsolute/Relative/Velocity, Halt, Stop, Reset,
Home, Read*). RETAIN accettato ma senza effetto. 18 divergenze dall'IEC dichiarate (`docs/st-conformance.md`).

**Editor PLC:** organizer per controllore → task → istanze, più l'elenco **Devices** (le parti con tag, comms on/off,
forzature); ST in CodeMirror (lint, completamento, F12, F2 che rinomina anche i tag della scena, Add to Watch dal menu);
Ladder con barra elementi a schede, trascinamenti, Cross Reference con avviso di doppia bobina, quick fix "Declare";
monitoraggio con valori inline e power flow; forzature da Watch, Inspector, menu e dialogo del dispositivo, con badge
"F" e "Release All Forces". **Non ha** breakpoint né online change: Reload riavvia il programma.

**Problems** raggruppa Program, Wiring, Parts, Layout: il cablaggio dà errori (conteso, tipo, indirizzo) e avvisi (non
pilotato, non letto, non dichiarato, non collegato); **gli avvisi sono nascosti di default**. **Watch** ha la colonna
**Wired to** (le parti che leggono o scrivono la variabile, con clic che seleziona la parte) e i tempi di scan per task.
Nessun trend.

**I/O:** dialogo **PLC connections…** (macchine e campi a sinistra, variabili del PLC interno o lette dal PLC esterno a
destra, collegamento con due clic e controllo di tipo e direzione); **Assign I/O Addresses** (indirizzi deterministici);
**Export/Import I/O List** in CSV (Tag, Address, Type, Direction, Modbus point, Device).

**PLC esterni** (solo app desktop, `apps/desktop/*.cjs`): Modbus TCP (simulatore server o client), OPC UA client, S7
PUT/GET, EtherNet/IP (libplctag), SLMP (MELSEC), ADS (TwinCAT, solo Windows). **Free-running:** polling ogni 20 ms,
timeout 500 ms, watchdog di 1 s che tiene OFF le uscite; modalità "Watch only". Guide passo passo per S7, Modbus, SLMP,
ADS, OPC UA. **Server:** Modbus TCP, OPC UA (porta 4843) e MQTT, per HMI e SCADA.

**Test di progetto** (`docs/project-tests.md`): JSON con ingressi, controlli `at`/`always`/`eventually`, sonde del mondo
e **guasti iniettati** (attuatore morto, sensore bloccato, inceppamento, perdita di comunicazione); in app e da CLI.

## 4. Piattaforma

- **Due host, un core:** browser (primario, anche su telefono) ed Electron. Il desktop aggiunge capacità, non
  prestazioni: socket per i PLC, cartelle, menu nativo, più finestre, aggiornamento automatico.
- **Progetti:** una scena, un file per POU in `plc/`, definizioni delle parti copiate, test; `.oip` zip
  byte-riproducibile; migrazioni dalla versione 1.
- **Collaborazione** senza account (Yjs): link di modifica, di sola visione e "per un display"; presenza; seguire la
  marcia di un altro.
- **MCP** (solo desktop, spento di default): 61 strumenti; ogni modifica dell'agente è un passo di undo e la persona
  può rivederla, tenerla o annullarla.
- **Qualità:** circa 6800 unit test e 443 script E2E, budget di draw, texture, DOM e interazione.
- **Distribuzione:** web (PWA), installer Windows firmato, deb e AppImage non firmati; niente macOS.

## 5. Cosa manca anche a OIP (rispetto a Visual Components e SIMIT)

| Funzione | OIP | Riferimento |
|---|---|---|
| Sincronia con il PLC | free-running, 20 ms | SIMIT bus-sincrono, PLCSIM Advanced su tempo virtuale ([16](16-simulatori-commerciali-trasporto-sincronia.md)) |
| Accesso simbolico S7-1500 (DB ottimizzati) | no, solo PUT/GET con accesso assoluto | ROADMAP 5.4 |
| Guasti in marcia | solo nei test di progetto | SIMIT, Emulate3D |
| Trend e analizzatore logico | no | SIMIT (trend e messaggi) |
| Allarmi ed elenco eventi | no | VC Output panel, SIMIT messaggi |
| Statistiche di flusso | contatori sui singoli spawner e despawner | VC dashboard, FlexSim |
| Debug PLC | Step, nessun breakpoint né online change | TIA Portal, CODESYS ([17](17-ide-plc-tia-codesys.md)) |
| Motore per flussi lunghi | passo fisso per tutto | DES di VC, FlexSim, JaamSim ([07](07-open-industry-project.md) §3) |

## 6. Twinova v0.4.0 a confronto (audit dal vivo, 2026-11-26)

Il layout di Twinova è stato preso da OIP (ricerca 13): stessi pannelli e stesso dock. Le differenze sono nel contenuto.

| Tema | OIP | Twinova v0.4.0 |
|---|---|---|
| Stato in 3D | raggi verde/rosso, rulli animati, frecce, lampade, annotazioni live | tessuto dei nastri che scorre e lampade del pusher (`ProjectScene.svelte`); sensori senza raggio né stato, nessuna freccia, nessun valore nella scena; contatori in un angolo |
| Segnale nell'Inspector | direzione e tipo, verdetto, valore live con Force | indirizzo come testo libero, etichette `PLC`/`TAG` poco chiare, nessun valore live |
| Elenco I/O | Watch con "Wired to" che seleziona la parte | Watch con nome, indirizzo, valore e forzatura, senza legame con la scena |
| Modello dell'impianto e PLC | la logica delle parti sta nella parte (Part Editor) | i comportamenti dei componenti compaiono come "Function blocks" del PLC |
| Problems | avvisi nascosti di default, gruppi per area | 23–25 avvisi sugli esempi forniti, tra cui "unused program 'Main'" e variabili di I/O "unused" |
| Dock PLC a 1366 × 768 | 45 % dell'altezza di default | 1 riga di codice e 1,5 righe di tabella |
| Pannelli vuoti | stati vuoti scritti | "Operator panel" con intestazione e niente sotto (gli esempi non hanno pulsanti); Inspector vuoto largo circa 290 px |
| Guida al primo uso | tutorial di 16 passi | nessuno |
| PLC esterni | sei protocolli | nessuno (M5 non iniziata) |
| Debug PLC | Step | breakpoint in HOLD, online change con RETAIN, VS Code |

Correzione rispetto a quanto detto in chat: il catalogo di Twinova ha le anteprime vere (l'icona-cubo era in uno
screenshot di CI del 23/11).

## 7. Cosa significa per Twinova

1. **In ampiezza OIP è molto avanti**, e il divario cresce: commit quotidiani, sei protocolli, server OPC UA e MQTT,
   36 parti, editor di parti, test con guasti, tutorial. Twinova ne copia layout e modelli.
2. **I vantaggi di Twinova oggi dimostrati** sono da IDE PLC (breakpoint, online change, VS Code), non da simulatore.
   Il motore a eventi discreti pesa meno di quanto previsto in 05/07: dalla 4.3 anche i colli di Twinova vanno con la
   fisica Rapier a passo fisso.
3. **L'unico vuoto che giustifica un progetto separato** è quello di §5: sincronia su tempo virtuale con PLCSIM
   Advanced, accesso simbolico S7, e gli strumenti da messa in servizio (guasti in marcia, trend, allarmi). In Twinova
   nessuno di questi esiste ancora.
4. **Misura proposta prima di decidere** (sul PC Windows dell'utente): OIP desktop con PLCSIM Advanced via PUT/GET, una
   linea semplice e la logica di messa in servizio reale. Se il free-running a 20 ms basta, la strada A di 07
   (usare OIP e contribuire) vince. Se non basta, il lockstep va prima proposto al maintainer di OIP; solo se rifiutato
   Twinova si restringe a quel vuoto, riusando modelli e interfaccia di OIP (MIT).
