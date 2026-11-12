# 17 — L'IDE di un PLC: editor ST/SCL, debug e monitoraggio in TIA Portal e CODESYS

Data: 2026-11-12. Richiesta dell'utente, durante la 4.7: breakpoint, compilazione ed esecuzione, completamento "come TIA
Portal e gli IDE PLC professionali", tutto nell'app senza VS Code; «Leggi i manuali dell'IDE di TIA e CODESYS così sai
di cosa sto parlando».

Fonti solo pubbliche, lette il 2026-11-12:

- **TIA:** la guida in linea di STEP 7 V20 (`https://docs.tia.siemens.cloud/r/en-us/v20/`, abbreviata **DT**) e il
  manuale di PLCSIM Advanced V6.0.
- **CODESYS:** la guida di CODESYS V3.5
  (`https://content.helpme-codesys.com/en/CODESYS%20Development%20System/`, abbreviata **CD**).

Nessun login aggirato; le pagine che non si sono aperte sono elencate in fondo. Le funzioni sono descritte con parole
nostre. «Non verificato» vuol dire che nessuna fonte Siemens o CODESYS lo conferma. La lettura delle pagine è stata fatta
da due agenti di ricerca; le righe su truST le ho verificate io, sui sorgenti installati.

## 1. Assistenza alla scrittura

| Funzione | TIA Portal (SCL) | CODESYS (ST) |
|---|---|---|
| Completamento | Si apre da solo dopo le prime lettere, oppure con Ctrl+Space/Ctrl+I. Propone solo ciò che è valido al cursore: tag locali e globali, DB, multi-istanze, istruzioni. Il prefisso restringe: `#` tag locali dell'interfaccia, `"` globali, `%` indirizzi assoluti. Il `.` apre i membri di struct, DB e istanze (DT `program-editor/using-autocompletion/...`) | "List components": il `.` apre le uscite e gli ingressi di un'istanza e i membri di una struct; un'opzione la apre già mentre si digita. Ctrl+Space filtra fra tutto, parole chiave, globali e locali (CD `_cds_using_input_assistants.html`) |
| Chiamata di un blocco | Un'istruzione inserita dal catalogo arriva con i segnaposto degli operandi: giallo chiaro gli opzionali, giallo scuro gli obbligatori; Tab passa al successivo. Un FB scritto a mano seguito da `(` apre il dialogo "Call options": istanza singola (crea il DB), multi-istanza o istanza come parametro. Ctrl+Shift+Space mostra o riduce i parametri opzionali (DT `creating-scl-programs/entering-scl-instructions/...`) | "Insert with arguments" nell'Input Assistant (F2) inserisce la chiamata con tutti i parametri formali, `:=` per gli ingressi e `=>` per le uscite. Dopo `(` un tooltip mostra i parametri (Ctrl+Shift+Space) (CD `_cds_cmd_input_assistant.html`) |
| Variabili non dichiarate | Non verificato | Auto Declare (Shift+F2), anche automatico, con ambito, tipo, valore iniziale, indirizzo e commento. Dichiarazione abbreviata: `bStart b` + Ctrl+Enter diventa `bStart : BOOL` (CD `_cds_cmd_declare_variable.html`) |
| Interfaccia cambiata | "Update inconsistent block calls" aggiunge i parametri nuovi alle chiamate | Refactoring "Add/Remove/Reorder variables" con anteprima delle modifiche (CD `_cds_refactoring.html`) |
| Errori | Rossi nel codice, elenco in Inspector > Info; la vista delle regioni segnala quelle con errori. Se il controllo giri durante la digitazione: non verificato | "Precompile": errori durante la digitazione, sottolineati e nella vista Messages (CD `_cds_edt_st_editor.html`) |
| Commenti del tag | Il commento della tabella dei tag mostrato accanto al codice (DT `displaying-or-hiding-tag-information`) | — |
| Navigazione | Go to definition (Ctrl+clic), uso successivo e precedente, accessi in lettura e scrittura, riferimenti incrociati, segnalibri | Go to Definition (Ctrl+Shift+D), Cross Reference List (Ctrl+Shift+X), evidenziazione delle occorrenze |
| Struttura | `REGION … END_REGION` con un pannello delle regioni; "Surround with" IF/CASE/FOR/REGION; formattazione (Ctrl+Shift+W) | Folding, END_xxx inserito da solo, rinomina con anteprima |

## 2. Debug

| | TIA Portal | CODESYS |
|---|---|---|
| Dove | S7-300/400 e S7-1500 dal firmware V2.5, solo per STL e SCL. Il numero di breakpoint attivi dipende dalla CPU. Non si salvano nel progetto (DT `testing-with-breakpoints-.../introduction-...`) | Tutti gli editor IEC; serve l'applicazione online in modalità Debug (CD `_cds_using_breakpoints.html`) |
| Condizioni | Su S7-300/400 solo per ambiente di chiamata (istanza o percorso di chiamata); condizioni su valori non documentate | Per task, per numero di passaggi (=, multiplo, ≥ N), per una condizione booleana, per una singola istanza di FB. "Execution point": esegue codice o scrive un log senza fermarsi. Data breakpoint solo su Control Win (CD `_cds_dlg_new_breakpoint.html`) |
| Passo | Run e Run to cursor su tutte. Step over, into e out solo su S7-300/400: **S7-1500 non li ha** | Step Into F8, Over F10, Out Shift+F10, Run to Cursor, Set Next Statement, Single Cycle (Ctrl+F5), Call Stack (CD `_cds_stepping.html`) |
| Cosa fa la CPU quando si ferma | Va in **HOLD**: il programma si ferma, nessun evento parte, **timer, watchdog e controllo del tempo di ciclo si fermano**; l'ora di sistema continua. Su S7-1500 le uscite restano attive con l'ultimo valore e gli ingressi non vengono più letti; la pagina generica di HOLD parla invece di valori sostitutivi (DT `hold-operating-mode-s7-1500`, `safety-information-for-testing-with-breakpoints-...`) | Si ferma solo il task in debug, gli altri continuano; l'I/O non viene aggiornato. Sulla barra di stato compare `HALT ON BP` (CD `_cds_using_breakpoints.html`). Il watchdog durante l'arresto non è verificato |
| Valori all'arresto | Al passaggio del mouse, gli operandi globali; si possono modificare una volta | Valori in linea e Call Stack |
| Alternativa che non ferma | — | **Flow control**: mostra i valori di ogni riga nel momento in cui è stata eseguita; righe eseguite in verde. Allunga il ciclo; non si combina con i breakpoint (CD `_cds_flowcontrol.html`) |
| Simulatore | Il manuale di PLCSIM Advanced V6.0 non nomina i breakpoint (non verificato); il suo orologio virtuale guida gli OB ciclici e il controllo del ciclo, ed è scalabile (pp. 73–74) | Modalità Simulation, con il menu Debug |

## 3. Monitoraggio, modifica e forzatura

- **TIA, "Monitoring on/off" (Ctrl+T):**
  - una colonna accanto al codice, una riga per riga di codice: il primo tag e il suo valore; la si espande per gli
    altri;
  - IF, WHILE e REPEAT mostrano TRUE o FALSE;
  - le righe non eseguite sono in grigio;
  - i loop non sono monitorati finché non si attiva "Monitor loops";
  - è monitorata solo la parte visibile, e il monitoraggio allunga il ciclo;
  - si sceglie l'istanza da guardare ("call environment");
  - modifica dal codice: Ctrl+Shift+1/9/2;
  - fonte: DT `program-status-display-for-scl-programs`.
- **TIA, watch table:** modifica con trigger "permanente", "una volta a inizio ciclo", "a fine ciclo", "al passaggio in
  STOP". Gli ingressi vanno modificati a inizio ciclo, le uscite a fine ciclo.
- **TIA, force table:** forzatura permanente **solo della periferia** (I/O). Resta attiva anche disconnettendosi, fino a
  "Stop forcing" (DT `testing-with-the-force-table/...`).
- **CODESYS, monitoraggio:** un campo valore dopo ogni variabile nel codice ST, e una colonna Valore nella dichiarazione,
  per l'istanza aperta (CD `_cds_using_monitoring_in_pous.html`).
- **CODESYS, preparare, scrivere e forzare:**
  - prima si prepara un valore, poi lo si scrive (Ctrl+F7, una volta, il programma lo può sovrascrivere) o lo si forza
    (F7, finché non si toglie con Alt+F7 o con il logout);
  - una forzatura si applica prima della prima chiamata del ciclo e di nuovo dopo l'ultima;
  - quattro watch list e "Watch All Forces";
  - fonte: CD `_cds_forcing_values.html`.

## 4. Compilazione e caricamento

- **TIA:**
  - compilazione delle sole modifiche o di tutto, con un controllo di consistenza delle interfacce;
  - caricamento in RUN: al più 56 oggetti per ciclo;
  - "senza reinizializzazione": una riserva di 100 byte per blocco ottimizzato mantiene i valori attuali;
  - confronto online/offline riga per riga (DT `compiling-and-downloading-plc-programs/...`, `comparing-plc-programs/...`).
- **CODESYS:**
  - Build controlla e basta, Generate Code (F11) genera;
  - Login (Alt+F8) offre online change, download completo o nessun cambiamento;
  - l'online change carica solo le parti cambiate e non riesegue l'inizializzazione; diverse modifiche lo impediscono
    (CD `_cds_cmd_login.html`, `_cds_cmd_online_change.html`).

## 5. Che cosa ha già truST (verificato sui sorgenti installati, 2026-11-12)

- **Analisi, `trust-wasm-analysis` 0.39.0** (già nell'app con la 4.7): diagnostiche, hover e completamento, e in più
  definizione, riferimenti, rinomina ed evidenziazione delle occorrenze
  (`crates/trust-wasm-analysis/src/lib.rs`: `definition`, `references`, `rename`, `document_highlight`).
- **Il resto del frontend, `trust-ide` 0.39.0:** ha anche signature help, inlay hints, inline values, call hierarchy e
  semantic tokens (`crates/trust-ide/src/`). `trust-wasm-analysis` non li espone: servirebbe dipendere da `trust-ide`
  direttamente.
- **Debug, `trust-runtime` al commit fissato `6405915`:** `DebugControl` (`src/host/debug/control/api/`) offre:
  - breakpoint per file;
  - pausa, continue, step, step over e step out;
  - valutazione di espressioni e lettura e scrittura di variabili (`runtime/installed_debug.rs`);
  - forzature (`writes_forcing.rs`);
  - risoluzione di una riga nella posizione di breakpoint più vicina.

  Si attiva con `Runtime::enable_debug()`, raggiungibile dal `TestHarness` che Twinova già usa (`runtime_mut()`).
- **Non ancora misurato:** cosa fa `cycle()` quando incontra un breakpoint dentro il motore deterministico di Twinova
  (blocca il thread o ritorna?). È la domanda che decide come realizzare l'HOLD.
- **Non ancora misurato:** se il completamento di truST inserisce una chiamata di FB con tutti i parametri. Nei sorgenti
  ci sono snippet di parole chiave (`completion/keywords.rs`), ma non ho trovato un modello di chiamata con `:=` e `=>`.

## 6. Che cosa ne ricavo per Twinova (ipotesi da decidere, non decisioni)

- **Simulatore = HOLD di TIA.** Al breakpoint si fermano il programma, il suo tempo, la fisica e il tempo simulato: è
  quello che fa una S7-1500 (timer e watchdog fermi). Nel motore a eventi è naturale (il tempo virtuale non avanza) e il
  replay resta deterministico. Va dimostrato con una misura.
- **I monitoraggi, più delle funzioni di VS Code.** Entrambi gli IDE mettono il valore accanto al codice (TIA una colonna
  per riga con TRUE/FALSE sulle condizioni, CODESYS un campo per variabile) e permettono di modificare dal codice. È ciò
  che un PLC-ista usa ogni giorno, più dei breakpoint.
- **Il completamento "come TIA":**
  - la chiamata di FB con tutti i parametri, `:=` e `=>`, e gli opzionali riducibili;
  - i prefissi `#` e `"` non sono IEC 61131-3 (sono SCL di Siemens): da non copiare;
  - Auto Declare di CODESYS, utile e semplice.
- **Step over/into/out** li ha CODESYS e non S7-1500: un di più, non il minimo.

## 7. Pagine che non si sono aperte

- **TIA:**
  - `support.industry.siemens.com` (PDF di PLCSIM e manuale di sistema S7-1500): Access Denied, anti-bot;
  - un post del forum Siemens: 403.

  Un numero di breakpoint per PLCSIM ("8–20") compare solo negli snippet di ricerca: non verificato.
- **CODESYS:** `_cds_cmd_format_document.html`, `_cds_cmd_go_to_definition.html` e
  `_cds_cmd_toggle_flow_control_mode.html` hanno dato 403; l'indice `_cds_f_commands_debug.html` era vuoto.
