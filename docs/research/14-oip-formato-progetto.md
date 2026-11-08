# Open Industry Project: il formato del progetto, per la 4.2

Data: 2026-11-08. Fonte: https://github.com/Open-Industry-Project/Open-Industry-Project al commit
`05728680455137294bb8e39fb686dcf5c3670c50` (lo stesso della ricerca 12), licenza MIT (`LICENSE`).
File letti: `docs/projects.md`, `src/core/project/project.ts` (righe 85–130 e 427–470), gli esempi
`examples/projects/*/project.json` e `scene.json`, l'elenco di `src/core/project/` e di `test-corpus/`.
Si riprendono idee, descritte con parole nostre; nessun file è copiato in Twinova.

## Cosa fa OIP

| ID | Aspetto | Come lo fa OIP | Fonte |
|---|---|---|---|
| OF-01 | Un progetto, una scena | Un progetto è una macchina: una scena, il programma PLC, le definizioni dei componenti e i test. Le versioni 1–3 con più scene si leggono ancora e vengono divise | `docs/projects.md` §Projects |
| OF-02 | Versione unica | Un intero `version` per tutto il progetto (oggi 7). Si leggono le versioni 1–6 e si migrano al caricamento; si scrive solo la 7. Una versione più nuova è rifiutata con un messaggio che dice quale versione legge la build | `project.ts` 437–444 |
| OF-03 | Riparazioni dichiarate | Il caricamento restituisce `errors` (il documento non si apre) oppure il documento con `repairs`: l'elenco di ciò che la migrazione ha cambiato, mostrato all'utente | `project.ts` 44–46, 450–456 |
| OF-04 | Definizioni copiate nel progetto | Ogni definizione usata è conservata una volta nel progetto, built-in compresi. Una copia di un built-in porta l'impronta della versione dell'app da cui è stata copiata; l'app segnala quando la sua versione cambia e non aggiorna mai il progetto da sola | `docs/projects.md` scelta 1 |
| OF-05 | La scena non contiene copie | Le istanze nominano la definizione per id (`packageId`) e portano solo posa e parametri modificati | `examples/projects/jam-detect/scene.json` |
| OF-06 | Cartella leggibile | `project.json` è un manifest sottile (versione, nome, riferimenti ai file); poi `scene.json`, `plc/<POU>.st` un file per blocco nell'ordine del manifest, `parts/<id>/manifest.json` e `behavior.js`, `tests/<nome>.test.json` | `docs/projects.md` §The project folder |
| OF-07 | Asset per hash | Gli asset incorporati sono file `parts/<id>/assets/<sha256>.glb`, nominati con l'hash dei loro byte | idem, §Serialization |
| OF-08 | Archivio = cartella zippata | Il file `.oip` è la cartella compressa, come un `.docx`; la codifica è riproducibile byte per byte (mtime fissa) | idem |
| OF-09 | Lettura guidata dal manifest | Si leggono solo i file elencati: un file in più o rimasto da prima è ignorato, non è un errore. Il riassemblaggio passa sempre dallo stesso validatore, così una cartella modificata a mano riceve gli stessi controlli di un file importato | idem, §The program as files |
| OF-10 | Scrittura | Un file alla volta, attraverso un file temporaneo e una rinomina, così un lettore non vede mai mezzo file. Ogni scrittura è condizionata: il file si sostituisce solo se ha ancora l'hash letto l'ultima volta, altrimenti è un conflitto con chi l'ha cambiato (un agente, un editor, `git checkout`) | idem, §The folder is the project |
| OF-11 | Niente atomicità su più file | Un salvataggio su più file non è atomico. Il lettore tollera lo stato intermedio: una cartella che non si riassembla viene tenuta in attesa, e il giro riprende quando la cartella cambia di nuovo | idem |
| OF-12 | Sincronizzazione bidirezionale | La cartella è il progetto: le modifiche dell'app vengono scritte poco dopo, e le modifiche esterne vengono ricaricate | idem |
| OF-13 | Marcatore di compatibilità in avanti | `stDialect`: un valore sconosciuto conserva il programma intatto in ogni salvataggio, ma l'esecuzione lo rifiuta con un messaggio | idem, scelta 2 |
| OF-14 | Collegamento ai tag per nome | I parametri delle istanze nominano i tag PLC (`"runTag": "BeltRun"`) | `scene.json` dell'esempio |
| OF-15 | Corpus di regressione | `test-corpus/` contiene 179 file: istanze salvate per famiglia, con l'impronta del comportamento con cui sono state registrate (`behaviour.json`) | `test-corpus/` |
| OF-16 | Validatore a riga di comando | `npm run validate-project -- <cartella o .oip o scene.json>` usa lo stesso validatore dell'app | idem, §Serialization |

## Cosa ne prende Twinova (ipotesi per il piano della 4.2)

- **OF-02 e OF-03 così come sono:** una versione per progetto, migrazioni in lettura, scrittura solo dell'ultima,
  rifiuto esplicito di una versione più nuova, elenco delle riparazioni mostrato all'utente. È già quello che dice la
  spec 4.1; OIP mostra che regge sette versioni.
- **OF-04 e OF-05:** coincidono con le definizioni "copiate e fissate" della spec 4.1. In più c'è l'impronta della
  versione dell'app sulle copie dei built-in, che serve a segnalare un aggiornamento disponibile.
- **OF-06, OF-07 e OF-09 per la cartella:** manifest sottile, `scene.json`, `.st` un file per POU (utile alla 4.6,
  VS Code), asset nominati per hash, lettura guidata dal manifest e un solo validatore per tutte le strade.
- **OF-15 per l'accettazione della 4.2** ("si apre un corpus di scene salvate"): un corpus di progetti salvati da ogni
  versione del formato, che ogni build deve continuare ad aprire.
- **OF-16:** lo stesso validatore richiamabile fuori dall'app (un comando `cargo run --example` o un binario) per la CI.

## Dove Twinova diverge

- **Il collegamento al PLC è per indirizzo, non per nome** (OF-14). Lo ha deciso l'utente nella 4.1: il documento
  salva gli indirizzi IEC, e il programma del controllore riceve una `VAR_CONFIG` generata.
- **L'atomicità su più file** (OF-11). OIP accetta lo stato intermedio e lo gestisce in lettura. Twinova ha come
  requisito della 4.1 che un salvataggio fallito non rovini quello precedente. Una strada è scrivere tutti i file nuovi
  con nomi nuovi e rinominare il manifest per ultimo: il manifest vecchio continua a puntare ai file vecchi finché la
  rinomina non riesce. Funziona solo se i file di contenuto non vengono sovrascritti sul posto. Da misurare nella 4.2.
- **La sincronizzazione bidirezionale con la cartella** (OF-12) non è nella 4.2: interessa la 4.6, quando VS Code
  modifica i `.st` mentre la scena gira.

## Cosa NON dimostra

- Nessuna di queste soluzioni è stata provata in Twinova: sono letture del codice e della documentazione di OIP.
- Le affermazioni sul comportamento di OIP (scritture condizionate, tolleranza della cartella a metà) vengono dalla sua
  documentazione e dai nomi dei suoi test e2e; i test non sono stati eseguiti.
- Le parti di OIP in TypeScript ed Electron non si possono riusare in un core Rust; da OIP si prendono le scelte, non il
  codice.
