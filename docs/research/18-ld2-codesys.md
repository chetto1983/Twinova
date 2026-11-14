# 18 — CODESYS LD2 ("CODESYS Ladder"): studio per l'editor Ladder di Twinova

Data: 2026-11-13. Richiesta dell'utente: "Manda un agente a studiare ld2 codesys".

## Fonti e metodo

- **Cosa ho letto:** solo pagine pubbliche.
  - Le 38 pagine dell'help "CODESYS Ladder" V1.3.0.0, in inglese.
  - Le release notes 1.3.0.0.
  - L'help di CODESYS 4 LD e di LD FBD, l'editor classico.
  - Un thread del forum Forge.
  - TIA V20 su docs.tia.siemens.cloud.
- **Limite:** le immagini non le ho esaminate.
- **Notazione:**
  - [V] scritto su una pagina;
  - [I] inferenza.
- **Clean-room:** le descrizioni sono riformulate, nessun testo è copiato.
- **Testo estratto dall'help:** si trova nella scratchpad della sessione, non nel repository.

Sigle delle fonti. Le pagine `S*` hanno tutte il prefisso `https://content.helpme-codesys.com/en/CODESYS%20Ladder/`.

| Sigla | Fonte |
|---|---|
| S1 | `_ladder_start_page.html` |
| S2 | `_ld_overview.html` |
| S3 | `_ld_programming_in_ld_editor.html` |
| S4 | `_ld_branch.html` |
| S5 | `_ld_monitoring_debugging.html` |
| S6 | `_ladder_navigation.html` |
| S7 | `_ld_dlg_options.html` |
| S8 | `_ld_dlg_customizable_toolbox.html` |
| S9 | le pagine `_ld_cmd_*.html`, citate per nome del comando |
| S10 | `_pvg_create_ld2_visu.html` |
| R13 | release notes 1.3.0.0: `https://api-it.codesys.com/fileadmin/user_upload/CODESYS_Group/Ecosystem/Up-to-Date/Releases-Lifecycle/Release-Updates/Release-Notes/Release-Notes-CODESYS-Ladder-1300.html` |
| C4 | help di CODESYS 4 LD: `content.helpme-codesys.com/en/CODESYS%204/_c4ld_basics.html`, `_c4ld_shortcuts_ld_editor.html`, `_c4ld_elem_block.html` |
| LDFBD | help dell'editor classico, `content.helpme-codesys.com/en/CODESYS%20LD%20FBD/` |
| F1 | forum Forge: `https://forge.codesys.com/forge/talk/Engineering/thread/f3fa2a239f/` |
| T1–T4 | TIA V20, `docs.tia.siemens.cloud/r/en-us/v20/creating-lad-programs/` (rami e regole degli elementi) |

## 1. Che cos'è LD2

- **Un componente separato.** "CODESYS Ladder" è un componente aggiuntivo a sé, presentato come successore di "CODESYS LD FBD", che resta disponibile [V, S1, S2].
  - Il linguaggio si chiama "Ladder (LD2)" [V, S2].
  - Il LD di CODESYS 4 gli somiglia [V, C4].
- **Perché esiste: un editing più semplice** [V, S1].
  - Nel classico ogni variante di un elemento è una voce a sé della toolbox (contatto negato, contatto in parallelo, bobina di set…).
  - In LD2 la variante nasce dalla posizione di inserimento o da un modificatore applicato all'elemento.
- **Versioni.**
  - La 1.3.0.0 richiede CODESYS 3.5.19.0 o successivo [V, R13].
  - Le date precedenti vengono dai risultati di ricerca e non sono verificate: 1.0.0.0 a ottobre 2024, 1.3.0.0 il 19/08/2026.
- **Conversioni.**
  - "Convert to New Ladder" porta il LD classico in LD2 [V, S9 convert_to_new_ladder].
  - EN/ENO e i fronti si convertono bene solo da LDFBD 4.7.0.0 in poi [V, R13].
  - Non ho trovato conversioni verso ST né quella inversa da LD2 al classico. È un'assenza nella documentazione, non una prova che manchino [I].
  - Un utente del forum dice di convertire a mano le sue POU in ST [V, F1].

## 2. Layout e modello di disegno

- **Reti.** Una lista di reti numerate.
  - Ogni rete ha titolo, commento ed etichetta di salto, editabili con un doppio clic [V, S9 insert_network; S3].
  - Una rete commentata (Ctrl+O) diventa grigia e corsiva e non viene eseguita [V, S9 outcommented].
- **Niente griglia, niente posizionamento libero, niente fili disegnati a mano.**
  - I collegamenti nascono quando si inseriscono e si spostano gli elementi [V, S3].
  - Il layout è automatico e topologico, cioè un albero serie/parallelo [I].
- **Rami.**
  - Vengono eseguiti uno dopo l'altro, dall'alto in basso, e si annidano senza un limite dichiarato [V, S4].
  - Il punto di diramazione ha un marcatore selezionabile [V, S4].
  - Un ramo può essere aperto (non si ricongiunge) o chiuso (si ricongiunge) [V, S3; S9 open/close_parallel_branch].
- **Box.** Qualsiasi FB, funzione, POU di libreria o operatore [V, S9 insert_block].
  - Viene inserito con EN/ENO per default. Ctrl+Shift+E e Ctrl+Shift+N li tolgono o li rimettono; ENO esiste solo se c'è EN, e con EN=FALSE il box non esegue [V, S9].
  - Dalla 1.3.0.0, EN senza ENO è ammesso:
    - sempre, per i comparatori;
    - per ADD, AND, MUL, OR, SEL, SUB e XOR solo in certe posizioni [V, R13].
  - "Set Output Connection" (Ctrl+W) sceglie quale uscita del box prosegue il rung.
  - I box estensibili accettano pin di ingresso in più, e i pin non usati si possono rimuovere.
  - "Update Parameters" riallinea il box alla dichiarazione dopo una modifica dell'FB [V, S9].
- **Elementi Input e Output.**
  - Input porta il valore di una variabile o di una costante dentro il flusso, per esempio su un pin di un box.
  - Output assegna a una variabile di qualunque tipo, non solo BOOL [V, S9].
- **Link Via Variable** collega un'uscita e un ingresso attraverso una variabile di appoggio [V, S9].
- **Più bobine sullo stesso rung:** "Insert Coil (parallel)" [V, S9].
- **Salti e ritorni.**
  - Jump verso l'etichetta di una rete; Return termina la POU [V, S9].
- **Opzioni di visualizzazione** [V, S7]:
  - titolo e commento della rete;
  - commento dell'operando, commento e indirizzo del simbolo;
  - righe e caratteri dei nomi;
  - cifre e lunghezze dei valori in monitoraggio.

## 3. Elementi

- **Contatto** (Ctrl+K): normale, negato (Ctrl+N), fronte di salita o di discesa (Ctrl+E). Il fronte vale anche sugli ingressi dei box [V, S9].
- **Bobina** (Ctrl+A): normale, negata, Set, Reset (Ctrl+M alterna Set, Reset e nessuno). Set e Reset valgono anche sulle uscite non primarie dei box [V, S9]. Una bobina a fronte non l'ho trovata.
- **Box** (Ctrl+B): qualsiasi FB, funzione o operatore. Al posto di `???` si scrive il tipo, con l'aiuto dell'Input Assistant [V, S3].
- **ST Block** (Ctrl+Shift+S): un blocco di codice ST dentro la rete [V, S9]. CODESYS 4 non ce l'ha [V, C4].
- **Operandi.** Al posto di `???` si scrive il nome; se manca la dichiarazione si apre Auto Declare [V, S3].
- **Toolbox personalizzabile** (1.3.0.0):
  - un elemento della toolbox si crea da una selezione nel diagramma;
  - "Restore Default" genera chiamate già pronte;
  - si inserisce con trascinamento o doppio clic [V, S8, R13].

## 4. Interazione

- **Marcatori di inserimento**, mostrati durante il trascinamento [V, S2, S3, S9 show_insert_positions]:
  - **rettangolo:** inserisce dopo, cioè a destra;
  - **rombo:** inserisce prima, cioè a sinistra, in serie;
  - **triangolo** in giù o in su: inserisce in parallelo, sotto o sopra.
- **Come funzionano.**
  - La posizione valida si illumina e il cursore mostra un "+".
  - Lo stesso marcatore serve a sostituire un elemento (lo si rilascia sopra) e a spostarlo [V, S3].
  - Dalla 1.3.0.0 i marcatori compaiono anche al passaggio del mouse, e Ctrl+Shift+M li mostra tutti [V, R13].
- **Serie e parallelo con la stessa scorciatoia.** Su un triangolo, Ctrl+K diventa "Insert Contact (parallel)" [V, S9].
- **Rami da tastiera.**
  - Chiusura: Ctrl+clic sui due rettangoli finali, poi Ctrl+Shift+P, oppure si trascina un marcatore sull'altro.
  - Apertura: Ctrl+P, oppure Canc sulla posizione prima del ricongiungimento [V, S3, S9].
- **Selezione.**
  - Una rete si seleziona cliccando una zona vuota; Shift seleziona un intervallo, Ctrl+clic ne toglie una.
  - Gli elementi si selezionano con clic, Ctrl+clic per più di uno.
  - Taglia, copia e cancella valgono su più elementi.
  - La selezione è rossa con contorno [V, S2, S3, R13].
- **Tastiera** [V, S6]:
  - frecce sinistra e destra: posizione di inserimento successiva, anche oltre la rete;
  - frecce su e giù: dentro un gruppo, per esempio i pin di un box;
  - Ctrl+Home e Ctrl+End: prima e ultima rete.
- **Altre scorciatoie:** Ctrl+I rete, Ctrl+L salto, Ctrl+R ritorno, Ctrl+Shift+I input, Ctrl+Shift+O output [V, S9].
- **CODESYS 4** usa tasti singoli per le stesse azioni (K, A, B, N, S, R, E), F2 per editare, Ctrl+Z e Ctrl+Y per annullare e ripetere [V, C4].
- **Non trovato nelle pagine LD2:** zoom, undo, completamento degli operandi. Sono funzioni generiche della piattaforma [I].

## 5. Modo online

- **Linee** [V, S5]:
  - TRUE: blu e spessa;
  - FALSE: nera e spessa;
  - valore sconosciuto o analogico: sottile.
- **Lo stato delle linee è calcolato nell'IDE** dai valori delle variabili, non letto dal PLC: l'help lo dice esplicitamente ("non è un vero controllo di flusso") [V, S5].
  - Le uscite di un box con EN senza ENO non si monitorano [V, R13].
- **Il classico ha invece un vero Flow Control:**
  - barre verdi sulle reti eseguite;
  - linee percorse in verde;
  - linee tratteggiate per le combinazioni di stati [V, LDFBD].
- **Valori.**
  - Accanto a ogni variabile compare il valore attuale; le costanti hanno una "C" verde.
  - Una variabile forzata ha un'icona prima del valore.
  - Un valore preparato per la scrittura compare tra parentesi angolari [V, S5].
- **Breakpoint** [V, S5]:
  - si mettono su assegnazioni, diramazioni, chiamate e su EN/ENO, non sull'intera rete;
  - un cerchio grigio vuoto indica una posizione possibile, un cerchio rosso pieno un breakpoint attivo.
  - Lo step non è descritto.

## 6. Ordine di esecuzione

- **Le reti:** dall'alto in basso.
- **Dentro una rete**, dalla 1.2.0.0:
  - da sinistra a destra e dall'alto in basso;
  - i rami in successione, dall'alto in basso [V, S4].
- **Reti create con versioni precedenti:**
  - possono avere il vecchio ordine e un triangolo giallo le segnala;
  - "normalizza" le converte e non si può annullare [V, S4].
- **Più bobine su rami paralleli:** si assegnano nell'ordine dei rami. L'ho dedotto dal resto: l'help mostra il caso solo in figura [I].

## 7. TIA Portal LAD V20 (solo dove la fonte è chiara)

- **Rami:**
  - si aprono con "Open branch" dalla task card, verso il basso, e si chiudono verso l'alto;
  - un ramo richiede almeno un elemento, e sparisce con l'ultimo [V, T4, T1].
- **Vincoli** [V, T1, T2]:
  - bobine e box in parallelo solo se il ramo parte dalla barra;
  - dopo una logica, in parallelo vanno solo contatti;
  - la rete finisce con una bobina o un box;
  - un solo salto e una sola etichetta per rete;
  - niente TRUE/FALSE costanti sui contatti.
- **Collegamenti vietati** [V, T3]:
  - flusso da destra a sinistra;
  - corto circuiti;
  - più di un percorso logico completo per rete.
- **Colori di stato:** li ho trovati solo su blog, con versioni discordanti. Non verificati.

## 8. Cosa portare in Twinova

### Priorità 1

1. **Un albero serie/parallelo con layout automatico.**
   - Niente griglia libera, niente fili disegnati a mano.
   - Il flusso inverso e i corto circuiti diventano impossibili per costruzione [T3].
   - Da verificare: trust-visual descrive un grafo con coordinate, non un albero.
2. **Marcatori di inserimento come zone di rilascio:** rettangolo, rombo, triangolo. Servono anche per spostare e sostituire, e c'è un comando per mostrarli tutti.
3. **Le varianti come modificatori.**
   - Contatto: negato, P, N. Bobina: negata, S, R.
   - Tasti che alternano i modificatori; una toolbox piccola.
4. **Box con pin** (timer, contatori, confronti, calcoli), elementi Input/Output per i pin non BOOL, EN/ENO opzionali.
5. **Operandi `???` da riempire**, con completamento e Auto Declare, anche per le istanze (per esempio quella di un TON).
6. **Reti:** numero, titolo, commento, stato commentato, selezione per intervallo, copia e incolla, annulla e ripeti.
7. **Navigazione da tastiera lungo le posizioni di inserimento.**
8. **Monitoraggio con stato del rung e valori sui pin**, con spessore o tratteggio oltre al colore.
   - Twinova potrebbe mostrare lo stato reale dalla VM in-process invece di ricalcolarlo, come fa CODESYS.
   - È un'ipotesi: va verificata leggendo il runtime.

### Priorità 2

- Comandi espliciti per aprire e chiudere i rami.
- Scrittura e forzatura con il valore preparato.
- Breakpoint per elemento.
- Update Parameters.
- Pin aggiuntivi sui box estensibili.
- Elementi preferiti nella toolbox.

### Da lasciare fuori

- ST Block.
- Jump, Label e Return, finché trust-visual non li ha.
- Link Via Variable e Set Output Connection.
- La normalizzazione delle reti vecchie.
- PLCopen.
- La generazione di visualizzazioni.
- La commutazione LD/FBD/IL.
- La libertà di posizionamento in stile CFC.

## 9. Pagine che non si sono aperte

- **HTTP 429:** le release page di codesys.com (en, de, it) e le release notes 1.0.0.0.
- **HTTP 403:** due URL dell'help del Development System, forse sbagliati.
- **Senza contenuto:** le pagine TIA "Insert block calls in LAD" e "Rungs in LAD".
- **Non trovate:** la pagina CODESYS Store di "CODESYS Ladder" e la legenda ufficiale dei colori LAD di Siemens.
