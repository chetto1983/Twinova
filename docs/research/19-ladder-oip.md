# 19 — Il Ladder di Open Industry Project, confrontato con trust-visual

Data: 2026-11-13. Richiesta dell'utente: "Guarda anche OIP".

## Fonte

- **Codice:** Open Industry Project, commit `0572868` del 2026-11-07, letto in una copia locale fuori dal repository.
- **Licenza:** MIT, "Automation Standard LLC and Contributors".
- **Come è stato letto:** solo in lettura. I percorsi citati sono relativi alla radice di OIP.

## 1. Modello: un albero serie/parallelo scritto come testo ST

- **Dove sta il Ladder.** Una sezione `LADDER … END_LADDER` occupa il posto di uno statement. Ogni `;` chiude un rung (`src/plc/ladder/ladder.ts:3-15`, `docs/ladder.md:21-47`). Esempi:

  ```
  [|Start|, |Motor|] |/Stop| (Motor);
  |Eye| |P Button| (S Jam);
  |Run| T1(IN, PT := T#2s) (Late);
  EQ(IN1 := Step, IN2 := 10, OUT) |/Busy| MOVE(IN := 20, OUT => Step);
  ```

- **Serie e parallelo.** La serie è la giustapposizione degli elementi. Il parallelo `[p1, p2, …]` ha almeno due percorsi, e un percorso vuoto è un filo.
- **Nessuna posizione salvata:** il layout lo calcola la vista.
- **Annidamento senza limite.** Il parser è ricorsivo (`parserLadder.ts:117-170`); un test copre i rami annidati (`ladder.test.ts:268`).
- **Elementi.**
  - **Contatti:** NO, NC, P, N.
  - **Bobine:** normale, negata, S, R, P, N.
  - **Box:** qualsiasi FB standard (TON, TOF, TP, CTU, CTD, CTUD, R_TRIG, F_TRIG, SR, RS…), anche dell'utente, con i pin letti dalle dichiarazioni. Qualsiasi funzione, anche dell'utente, comprese le conversioni. Gli operatori AND, OR, XOR, NOT e MOD come box (`ladder.ts:200-261`, `parserLadder.ts:285-299`).
  - **Pin dei box:**
    - il pin "nudo" sta sul rung;
    - EN/ENO;
    - un pin non collegato può restare vuoto;
    - `?` è un operando vuoto.
  - **Confronti:** si usano come box con OUT nudo, alla barra sinistra.
  - **Salti e ritorno:** etichetta `L:`, salto `>>L` solo in avanti, `RETURN`.
  - **Commento** sopra il rung.
- **Non trovato:**
  - un titolo del rung distinto dal commento;
  - bridge e connettori;
  - salti all'indietro;
  - un confronto in serie dopo un contatto.

## 2. PLCopen TC6 `<LD>` (`ladderXml.ts`, 669 righe)

- **Scrittura.**
  - Un rail per rung; un ramo diventa fan-out più un OR cablato.
  - Le bobine in serie in coda vengono disegnate in parallelo.
  - Le posizioni stanno su una griglia fissa.
  - Il namespace è tc6_0201, validato in CI con lo XSD di Beremiz.
- **Lettura.**
  - Union-find sui punti di connessione, poi riduzione serie-parallelo, poi l'albero.
  - Le posizioni del file vengono ignorate.
  - Rifiuta esplicitamente quello che non sa rappresentare (bridge, connettori, box→box, pin negati o a fronte…) ed esclude la POU con una nota.
  - Riconosce gli alias dei pin degli altri strumenti.
- **Round-trip semantico, non grafico.** Un test riesporta e reimporta tutti i casi Ladder e verifica che si comportino allo stesso modo, scan per scan. I test sono 14.

## 3. Esecuzione: lowering a ST, nessun interprete

- **Come si esegue.**
  - Ogni rung diventa ST, che viene riparsato.
  - Le posizioni sono rimappate sugli operandi dei rung (`mapBack`), così errori e marcature tornano sugli elementi.
  - `expandLadder` produce ST puro più un blocco VAR di temporanei nascosti: un formato che truST potrebbe eseguire.
- **Semantica.**
  - Ogni elemento è valutato una volta, da sinistra a destra, e vede le scritture di quelli che lo precedono.
  - Se una scrittura successiva tocca una variabile che la condizione legge, la condizione viene prima catturata in un temporaneo (`beforeWrite`).
  - Ordine dei rami: prima quelli con contatti o box, poi quelli con sole bobine, ciascun gruppo dall'alto in basso (`branchOrder`).
  - I fronti P/N usano i temporanei `LD_E`/`LD_M`; i salti sono flag con `IF NOT(flag)`.
- **Prove.**
  - 30 casi golden di conformità.
  - Il demo Palletizer, in versione ST e LD, è confrontato su 12.000 scan casuali.

## 4. Monitoraggio online

- **Lettura dei valori.** L'immagine delle variabili viene letta in polling ogni 120 ms. È indicizzata per nome e gestisce le istanze, i bit di word e gli indici.
- **Il power flow è ricalcolato nella vista, non dal motore.**
  - È esatto per fronti e confronti, perché legge i loro temporanei.
  - È approssimato per gli stati intermedi dello scan, perché usa l'immagine di fine scan.
- **Interazione dalla vista.**
  - Valori sui pin.
  - Toggle Bit con Ctrl+T e scrittura con doppio clic, tra uno scan e l'altro.
  - Force e Release, segnalati con un badge "F".

## 5. Editor (React, da riscrivere per Svelte)

- **Rendering.**
  - DOM con flexbox e bordi CSS.
  - Layout ad albero con a capo dei rung lunghi, zoom e virtualizzazione.
- **Editing.**
  - La vista legge i rung dal testo, applica trasformazioni pure e riscrive la sezione con un'unica modifica di testo.
  - Annulla e ripeti sono quelli dell'editor di testo.
- **Gesti puri** (`ladderEdit.ts`, nessun import):
  - inserire, spostare e trascinare su slot di rilascio, cancellare;
  - rami: aggiungere un percorso o un livello, ridimensionarli;
  - spostare un rung, incollare.
- **Toolbox e picker.**
  - Toolbox a tab.
  - BoxPicker (Ctrl+B) su un catalogo derivato: prima i blocchi dell'utente, poi la libreria.
  - Un'istanza inserita viene dichiarata da sola; quick fix "Declare X : BOOL".
- **Tastiera e strumenti.**
  - Frecce attraverso rami e rung, selezione con Shift.
  - Trova e sostituisci, riferimenti incrociati, doppie bobine.
  - Completamento ordinato per il tipo atteso.
  - Pannello delle variabili.
  - Pin aperti collegabili con doppio clic.

## 6. Test

- **Modello:** circa 106 test (1.507 righe).
- **UI:** 25 test puri.
- **Integrazione:** 30 golden e 10 script E2E.
- **Tipo di test:** comportamentali, cioè si esegue il programma e si confrontano i valori.

## 7. Accoppiamento e dimensioni

- **Modello:** 3.267 righe, più 324 del parser.
  - Non dipende da React.
  - Dipende dal lexer, dal parser e dal checker ST di OIP: `LadderParser` estende `StmtParser`, e `readLadder` riparsa tutto il programma.
- **Moduli standalone:**
  - `ladderEdit.ts` e `ladderFind.ts`;
  - `ladderComplete.ts`, quasi;
  - `ladderXml.ts`, che usa solo i tipi.
- **UI React:** circa 3.500 righe, più 1.092 di CSS.

## 8. Confronto con trust-visual

| Aspetto | OIP | trust-visual |
|---|---|---|
| Modello | albero serie/parallelo, senza posizioni | grafo di nodi e archi con posizioni |
| Contatti e bobine | NO/NC/P/N; normale/negata/S/R/P/N | NO/NC; normale/negata/S/R |
| Box | qualsiasi FB o funzione, anche dell'utente; EN/ENO | set fisso TON…DIV |
| Salti, RETURN, commenti | sì (salti solo in avanti) | no |
| PLCopen `<LD>` | import ed export, validati | no |
| Monitoraggio | power flow nella vista, pin, toggle, write, force | non collegato |
| Test | circa 130 unitari, 30 golden, 10 E2E | test di lowering ed edit |

## Raccomandazione dell'analisi

- **Modello e semantica di OIP, portati in Rust come estensione nel fork di truST.**
  - Da portare: l'albero, il lowering (`Lowerer`, `branchOrder`, la cattura delle condizioni, i fronti) e PLCopen.
  - truST resta l'unico compilatore e l'unico a fare la diagnostica.
- **I moduli TS puri, copiati con attribuzione nel frontend Svelte:** gesti di editing, ricerca, completamento e la logica del watch.
- **La vista:** riscritta, prendendo le idee di OIP.
- **Un'estensione che OIP non ha:** temporanei di power per ogni elemento, per un monitoraggio esatto invece che ricostruito.
- **Perché non adottare il modello TS così com'è:** si porterebbe dietro il parser ST di OIP, che duplica truST e darebbe diagnostiche diverse.
