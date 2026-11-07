# R1: architettura dei motori di simulazione discreta e ibrida

Ricerca del 2026-11-07, per il progetto del tool 3D di simulazione di fabbrica e virtual commissioning
(nastri, sensori, robot, link al PLC).

- **Verificato** = letto nel sorgente o nella specifica (URL accanto a ogni affermazione).
- **[inferenza]** = conclusione mia, non scritta nella fonte.
- **[secondaria]** = fonte di terzi (forum, tesi, motore di ricerca), non letta nella fonte primaria.

## 1. Il nucleo: la lista degli eventi futuri (FEL)

### SimPy 4: il ciclo, letto nel sorgente

Fonti: [https://gitlab.com/team-simpy/simpy/-/blob/master/src/simpy/core.py] e
[https://gitlab.com/team-simpy/simpy/-/blob/master/src/simpy/events.py].

- **La coda.** La FEL è un binary heap (`heapq`) di tuple `(tempo, priorità, eid, evento)`.
  `eid = next(count())` è un contatore monotono. A parità di tempo e priorità l'ordine è quindi
  FIFO per inserimento, ed è deterministico.
- **Le priorità.** Ce ne sono due:
  - `URGENT = 0`, usata da `Initialize` (l'avvio di un processo) e da `Interruption`;
  - `NORMAL = 1`, il default.

  Anche `run(until=t)` schedula il proprio evento di stop come URGENT, "before all regular
  timeouts".
- **`step()`**, in quattro mosse:
  1. `heappop`, poi `now = t`;
  2. prende `event.callbacks` e lo mette a `None`, che segna l'evento come *processed* e impedisce
     modifiche concorrenti;
  3. chiama in ordine ogni callback;
  4. se l'evento è fallito e nessuno lo ha "defused", rilancia l'eccezione e la simulazione si
     ferma.
- **Il tempo zero.** `succeed()` / `trigger()` fanno `schedule(self)` con `delay=0`. Un evento
  "immediato" passa comunque dalla heap: viene eseguito dopo gli eventi già in coda allo stesso
  istante con priorità ≤ e `eid` minore.
- **I processi.** Un processo è un generatore. `Process._resume(event)` fa `generator.send(value)`,
  oppure `throw(exc)` se l'evento è fallito, e ottiene il prossimo evento su cui il processo si
  sospende.
  - Se quell'evento non è ancora processato, `_resume` aggiunge se stesso ai suoi `callbacks` e
    ritorna.
  - Se è già processato, continua subito nel loop, senza passare dalla heap.
  - Quando il generatore termina (`StopIteration`), anche il `Process`, che è un evento, viene
    schedulato.
- **In sintesi** [inferenza]: SimPy è un *event scheduling* puro (heap più callback), con sopra i
  processi realizzati come callback che riprendono un generatore. Non ha né la scansione delle
  condizioni né il "wait until": le condizioni sono eventi `AllOf` / `AnyOf` che si valutano nel
  callback `_check` di ciascun evento figlio [events.py, `class Condition`].

### salabim 26.0.9: letto nel sorgente da PyPI

Fonte: `salabim/salabim.py` in [https://pypi.org/project/salabim/].

- **La coda.** È una heap di tuple `(t, priority, seq, component, return_value)`, con `seq`
  incrementato a ogni push.
- **`urgent=True`** usa `seq` negativo, quindi il componente passa davanti agli altri allo stesso
  `(t, priority)`, in ordine LIFO tra gli urgenti (`Component._push`).
- **I processi** sono di due tipi:
  - generatori (`yield self.hold(...)`);
  - con il modo *yieldless*, greenlet (`greenlet.greenlet(self._process)` e poi `switch()`): il
    codice del processo è normale codice sequenziale, senza `yield`.
- **`standby`.** I componenti in standby vengono reinseriti dopo *ogni* evento (`_standbylist` →
  `_pendingstandbylist` in `step`). La documentazione avverte che questo "can have significant
  impact on the performance" [https://www.salabim.org/manual/Modelling.html].

### Strutture dati alternative alla heap

- **Calendar queue** (R. Brown, CACM 31(10) 1988, pp. 1220–1227).
  - Lista multipla a "giorni" (bucket) di larghezza fissa, con ridimensionamento.
  - O(1) dimostrato *sperimentalmente*, non provato.
  - Tempi di hold 3 volte inferiori agli splay tree a 10.000 eventi
    [https://www.maths.tcd.ie/~eoin/index/brown_calendar88.html].
  - Degrada quando la distribuzione degli incrementi cambia, per "excessive resizing"
    [https://sigsim.acm.org/conf/mskr/Courseware/Fujimoto/Slides/FujimotoSlides-03-FutureEventList.pdf].
- **Ladder queue** (Tang, Goh, Thng, ACM TOMACS 15(3) 2005, pp. 175–204,
  doi:10.1145/1103323.1103324).
  - O(1) ammortizzato se il salto medio è finito, anche con distribuzioni a varianza infinita.
  - Provata da 100 a 10M di eventi [https://api.crossref.org/works/10.1145%2F1103323.1103324].
  - Variante adattiva: Furfaro e Sacco, PADS 2018 [https://github.com/fx74/ALQ].
- **Red-black tree** con nodi raggruppati per `(tick, priorità)`: lo usa JaamSim (`EventTree`,
  "custom red-black tree implementation intended to be used as the priority queue")
  [https://github.com/jaamsim/jaamsim/blob/master/src/main/java/com/jaamsim/events/EventTree.java].
- **Code piccole.** Sotto circa 10 elementi una lista ordinata batte sia la calendar queue sia gli
  alberi [https://clynelish.hgu.mrc.ac.uk/emap/analysis_tools_resources/software/woolz/html_Core/AlcCPQueue_8c.html]
  [secondaria].
- **Scelta per il nostro tool** [inferenza]: con poche migliaia di eventi pendenti (una fabbrica di
  linea, non una rete) la binary heap basta. Calendar e ladder servono a 10^5–10^7 eventi pendenti.

### Spareggio e determinismo negli altri motori

- **OMNeT++** ordina per arrival time, poi per *scheduling priority*, poi per "scheduling order
  (a.k.a. FES insertion order)". La priorità non va modificata mentre l'evento è nella FES
  [https://doc.omnetpp.org/omnetpp/api/classomnetpp_1_1cEvent.html].
- **JaamSim** ordina per tempo, poi per priorità (1 = la più alta, default 5), poi in modo "FIFO or
  LIFO" scelto dall'utente per ogni wait.
  - Il tempo è un `long` in tick: `secondsToNearestTick`, cioè il double viene scalato all'intero
    più vicino.
  - In questo modo non ci sono confronti tra float per decidere la simultaneità
    [https://github.com/jaamsim/jaamsim/blob/master/src/main/java/com/jaamsim/events/EventManager.java].
- **FlexSim.** Uno sviluppatore ha attribuito un bug di nastri bloccati a "floating point precision
  error at large model times, where … events happening at the same time get out of order"
  [https://forums.autodesk.com/t5/flexsim-forum/conveyor-items-getting-stuck/td-p/13533579]
  [secondaria]. È la prova pratica del perché il tempo intero in JaamSim conviene [inferenza].
- **AnyLogic.** Gli eventi simultanei sono "current events": l'engine ne sceglie uno, lo esegue e
  ripete finché non ce ne sono più (più *event step* consecutivi a tempo fermo)
  [https://anylogic.help/anylogic/statecharts/event-processing.html].
  - L'ordine può essere FIFO, LIFO o casuale. Il default sarebbe LIFO dalla 8.6.0
    [https://coderapp.vercel.app/answer/79660404] [secondaria, da verificare nella guida].
  - Il modo casuale serve a scoprire i modelli che dipendono da un ordine arbitrario
    [https://www.scribd.com/document/576141064/8-events] [secondaria].
- **DEVS classico** usa una funzione `select` di spareggio tra i modelli imminenti. Se un evento
  interno e uno esterno coincidono, quello interno viene ignorato.
  - **Parallel DEVS** introduce la *confluent transition* (δ_conf) e i "bag" di input, e rende
    `select` superfluo [https://arxiv.org/pdf/1701.07697].
  - **PythonPDEVS** documenta più scheduler intercambiabili ("Custom scheduler", "Activity
    tracking", …) [https://msdl.uantwerpen.be/documentation/PythonPDEVS/].
  - **adevs** (J. Nutaro, ORNL) implementa Parallel DEVS e Dynamic DEVS, con un framework ibrido
    dalla versione 2.3 [https://web.ornl.gov/~nutarojj/adevs/]. Il suo modello di calcolo coincide
    con la variante a *superdense time* del Parallel DEVS (libro *Building Software for
    Simulation*, Wiley 2010) [https://bdevs.sourceforge.io/] [secondaria].

### Problemi tipici allo stesso istante

- **Più eventi allo stesso tempo e ordine di trasferimento.** Schriber, Brunner e Smith descrivono
  due fasi:
  - la *Clock Update Phase*, che sposta le entità dalla Future alla Current Events List;
  - la *Entity Movement Phase*.

  In caso di parità di tempo "some tools will transfer all the time-tied entities … during a single
  CUP, whereas other tools take a 'only one entity transfer per CUP' approach", e il risultato del
  modello cambia
  [https://www.informs-sim.org/wsc17papers/includes/files/054.pdf, WSC "Inside discrete-event simulation software"].
- **Eventi a tempo zero a catena** (A schedula B a +0). In SimPy B va in coda dopo gli altri
  eventi NORMAL già presenti allo stesso istante, grazie all'eid. Nell'AnyLogic in modo LIFO va in
  testa [inferenza dai due meccanismi sopra].
- **Regola di progetto** [inferenza]:
  - chiave `(tick_intero, priorità, seq)`;
  - priorità esplicite per fase: input PLC → logica → fisica → output → sensori;
  - una modalità "shuffle dei pari" per i test di robustezza, come il modo random di AnyLogic.

## 2. Le visioni del mondo: event scheduling, activity scanning, process interaction

- **Event scheduling** (SimPy core, OMNeT++ `handleMessage`).
  - Pro: è il nucleo più semplice e veloce.
  - Contro: la logica di un'entità è spezzata in tanti handler.
- **Activity scanning / polled waiting.** A ogni avanzamento si rivalutano le condizioni.
  - Pro: le condizioni complesse ("a berth is empty and a tugboat is idle") si esprimono in modo
    diretto.
  - Contro: il costo è O(numero di condizioni) a ogni evento
    [https://www.informs-sim.org/wsc17papers/includes/files/054.pdf].
  - Schriber et al. distinguono la *related waiting*, cioè le entità in delay list legate
    all'evento che può sbloccarle ("the prevalent approach"), dalla *polled waiting*, cioè il
    controllo di routine [ibid.].
- **Process interaction** (coroutine, generatori, greenlet, thread).
  - Pro: il codice segue il ciclo di vita dell'entità (`hold`, `request`, `wait`, `passivate`).
  - Contro: serve un meccanismo di sospensione e il debugging dello stato è più difficile.
  - SimPy 3+ accetta qualunque generatore come process function, e i processi sono essi stessi
    eventi su cui si può fare `yield`
    [https://simpy.readthedocs.io/en/latest/about/defense_of_design.html].
  - JaamSim usa thread Java coordinati da un lock globale (`EventManager`, `evtLock`,
    `runningProc`) [EventManager.java sopra].
  - salabim usa generatori o greenlet (§1).

### "Wait until" e la rivalutazione delle condizioni

- **salabim `State`, related waiting.**
  - `self.wait(state)`, `self.wait((light,'green'), night)` (OR), `all=True` (AND); la condizione
    può essere un'espressione con `$` o una funzione `lambda x, comp, state`.
  - I componenti in attesa stanno nella coda `state.waiters()`.
  - Quando `state.set(v)` cambia valore, "the waiters queue will be checked", cioè
    `State._trywait` scorre i waiters e chiama `c._trywait()`.
  - `trigger(max=1)` sveglia un solo waiter.
  - `fail_at` / `fail_delay` danno il timeout
    [https://www.salabim.org/manual/State.html], [salabim.py `State.set`, `State._trywait`].
  - Il risultato: la condizione si rivaluta solo quando cambia uno stato da cui dipende, che è lo
    schema da adottare per i sensori e i segnali PLC [inferenza].
- **JaamSim `waitUntil(Conditional)`, polled waiting a fine istante.** Le condizioni vengono
  valutate (`evaluateConditions`) solo quando il prossimo evento richiederebbe di avanzare il
  tempo.
  - Se una condizione risulta vera, l'engine inserisce un evento al tick corrente con priorità 0 e
    riparte dall'inizio di quel tick.
  - Questa è una vera "event iteration" discreta allo stesso istante [EventManager.java, righe
    circa 290–360].
- **AnyLogic, condition event.**
  - Nei modelli discreti puri la condizione è testata "when something changes in the agent": a
    ogni evento dell'agente, oppure quando si chiama esplicitamente `onChange()`.
  - Se l'agente ha variabili continue, "the numeric engine constantly monitors the condition, i.e.
    the condition is checked on every step of the numeric equation solver"
    [https://anylogic.help/anylogic/statecharts/events.html].
  - Il pericolo è dimenticare `onChange()` dopo una modifica fatta da codice: la condizione non
    viene più vista [inferenza].
- **SimPy** non ha `wait until`: si crea un `Event` e lo si fa scattare a mano nel punto in cui la
  condizione può cambiare (related waiting manuale)
  [https://simpy.readthedocs.io/en/latest/topical_guides/events.html].

## 3. Ibrido discreto/continuo

### Due tipi di evento (FMI 3.0.1)

Fonte: [https://fmi-standard.org/docs/3.0.1/].

- I **time event** sono noti in anticipo: l'FMU li annuncia con `nextEventTime` di
  `fmi3UpdateDiscreteStates`, oppure con `fmi3GetShift` / `fmi3GetInterval`.
- Gli **state event** sono impliciti e vengono segnalati da *event indicator* `z_j`.
  - L'evento è definito come cambio di **dominio** (da `z>0` a `z≤0` o viceversa), non come
    "zero-crossing". La specifica giudica il termine "misleading".
  - L'FMU deve garantire `z_j ≠ 0` all'uscita dall'Event Mode, per esempio spostando `z` di un
    valore piccolo.
  - Gli indicatori vanno scalati con il loro valore nominale (§3.1.1, "Localization of State
    Events").
- Gli **step event** vengono chiesti dall'FMU a fine passo dell'integratore
  (`fmi3CompletedIntegratorStep`, `enterEventMode`).
- **State event detection** [sintesi della §3.1.1]:
  1. dopo ogni passo l'integratore confronta il segno degli indicatori;
  2. se un indicatore ha cambiato dominio, localizza l'istante con una ricerca della radice
     (bisezione, regula falsi o simili) nell'intervallo del passo;
  3. integra fino a quell'istante, entra in Event Mode, gestisce l'evento e riavvia l'integratore.
- **Chattering.** Serve isteresi sugli indicatori, ma "non risolve del tutto" i sistemi ibridi
  [https://core.ac.uk/download/49378469.pdf] [secondaria].

### Superdense time ed event iteration

- **FMI** definisce il tempo come coppia `(t_R, t_I)`: `t_I` conta ed enumera gli eventi allo
  stesso istante reale `t_R`, ed è zero durante l'integrazione. È "also called 'super-dense time'
  in literature", con riferimento a Lee e Zheng 2007.
  - Esempio della specifica: un evento a 2.1 s senza iterazione occupa l'istante `(2.1, 0)` e
    l'integrazione riparte da `(2.1, 1)`.
  - Nell'Event Mode di più FMU l'importatore cicla `do { … fmi3UpdateDiscreteStates … }` finché
    nessuna FMU chiede `discreteStatesNeedUpdate`, e ogni giro introduce "new instant of
    super-dense time" [https://fmi-standard.org/docs/3.0.1/].
- **Modelica 3.6, Appendice B.**
  - A un istante di evento: `loop solve(...) con pre(z), pre(m) fissi; if z == pre(z) and
    m == pre(m) then break; pre(z):=z; pre(m):=m; end loop`. È il punto fisso delle variabili
    discrete.
  - Le relazioni (`x > 2`, `floor(x)`) hanno un buffer: durante l'integrazione restano costanti al
    valore dell'ultimo evento, e quando il valore valutato è incoerente con il buffer scatta un
    evento (§8.5)
    [https://specification.modelica.org/maint/3.6/modelica-dae-representation.html]
    [https://specification.modelica.org/maint/3.6/equations.html].
  - `noEvent()` sopprime la generazione di eventi.
  - `reinit` assegna alla fine del passo di event iteration. Esempio: la pallina che rimbalza con
    `when h < 0 then reinit(v, -e*pre(v))` [ibid.].
- **AnyLogic.** Alterna *time step* (avanza l'orologio, il solver integra le equazioni, si testano
  le "expected change events") ed *event step* (tempo fermo, azioni discrete, ripetuto finché ci
  sono eventi correnti).
  - Durante un time step il solver continuo può segnalare una condition event a un istante
    intermedio: l'orologio avanza fino a quell'istante e si esegue l'event step
    [https://anylogic.help/anylogic/statecharts/event-processing.html].
  - `getEngine().setEventAwareSolver(false)` disattiva la localizzazione: gli eventi vedono valori
    vecchi fino a `htol`
    [risultato di ricerca su anylogic.help, non letto direttamente].
  - L'unico modo con cui la parte SD influenza quella discreta è una condizione su variabili SD,
    o una decisione che le usa [https://anylogic.help/anylogic/multimethod/index.html]
    [secondaria].
- **DEVS e QSS** (Kofman e Junco, 2001; QSS2 di Kofman, 2002).
  - Al posto della discretizzazione del tempo si quantizza lo stato: ogni variabile genera un
    evento quando cambia di un quantum ΔQ. L'integrazione diventa a sua volta un modello DEVS.
  - QSS1 è del primo ordine e non adatto ai sistemi stiff.
  - Gli state event sono naturali, perché l'istante di attraversamento di una soglia si calcola
    analiticamente sul polinomio della traiettoria
    [https://icc.fcen.uba.ar/en/discrete-event-simulation-of-continuous-time-systems-evolution-and-state-of-the-art-of-quantized-state-system-methods/]
    [https://build.openmodelica.org/Documentation/ModelicaDEVS.UsersGuide.QSS.html].
- **Per noi** [inferenza]: i moti dei nastri sono a tratti lineari (velocità costante tra un
  evento e l'altro), e quelli degli assi a tratti polinomiali (rampe). L'istante in cui un pezzo
  raggiunge una fotocellula si ottiene allora in forma chiusa (lineare o quadratica). Non serve
  né un ODE solver né la ricerca della radice: è la stessa idea di QSS, con traiettorie
  polinomiali esatte.
  - Restano necessari il tempo superdenso o la event iteration per propagare a tempo zero le
    catene sensore → PLC → attuatore → nuova traiettoria.
  - Un limite di iterazioni rileva i loop algebrici discreti [inferenza].

## 4. Nastri e flusso materiale senza step per frame

### JaamSim EntityConveyor (algoritmo pubblico, Apache 2.0, letto nel sorgente)

Fonte:
[https://github.com/jaamsim/jaamsim/blob/master/src/main/java/com/jaamsim/ProcessFlow/EntityConveyor.java].

- **Lo stato.** Per ogni entità si tengono `ConveyorEntry{entity, length, position}`, con
  `position` frazione `[0,1]` della lunghezza, più un unico `presentTravelTime` per il nastro.
- **`updateProgress(dt)`** è *lazy*: viene chiamato solo agli eventi, non a ogni frame.
  - Calcola `frac = dt / travelTime`.
  - Avanza ogni entità di `frac`, ma con il limite `maxPos = prev.position - entry.length/L`.
    Questo realizza l'accumulo: un'entità non supera quella davanti meno la sua lunghezza di
    accumulo.
- **`startProcessing`** calcola in forma chiusa i *prossimi* istanti candidati, in tick:
  - `nextEntTicks`, quando c'è di nuovo spazio in ingresso (`entitySpace`);
  - `exitTicks`, quando la prima entità arriva a `position = 1`;
  - `accumTicks`, quando l'ultima si compatta contro la coda (solo se accumulante);
  - `nextTriggerTicks`, i punti di trigger per gli osservatori, cioè i sensori.

  Schedula un solo evento, al minimo dei quattro, e alza flag per il tipo di evento.
- **Cambio di velocità.** `updateTravelTime`: se il travel time cambia si chiama `resetProcess()`,
  cioè si cancella l'evento pendente, si aggiornano le posizioni con il vecchio tempo e si
  ricalcola l'istante.
  - Anche `addEntity` fa prima `updateProgress()`, poi `updateTravelTime` e infine
    `performUnscheduledUpdate()`.
- **Nastro non accumulante.** Si ferma quando la prima entità non può uscire.
- **Nastro accumulante.** Continua finché è pieno (descrizione degli input `Accumulating`,
  `AccumulationLength`).
- **Complessità** [inferenza]: un solo evento pendente per nastro e O(n entità) a ogni evento.
  Nessun costo per frame. La grafica interpola le posizioni a `simTime` (`updateGraphics`).
- **Limite noto.** Il nastro non cambia velocità finché un'entità non entra o esce, se la
  velocità è un'espressione non osservata
  [https://groups.google.com/g/jaamsim-users/c/20A0yM0eh0U] [secondaria]. È lo stesso problema di
  `onChange()` in AnyLogic [inferenza].

### Prodotti commerciali (sola documentazione pubblica, nessun algoritmo interno trovato)

- **AnyLogic Material Handling.** Tre tipi di nastro:
  - *Roller*, accumulante: "the rollers will keep on rolling, bringing all the preceding items to
    the one that stopped"; `Gap` è la distanza minima;
  - *Belt*, non accumulante: un oggetto fermo ferma tutto il nastro;
  - *Fixed cell*, celle a passo fisso con un pezzo per cella, non accumulante
    [https://anylogic.help/markup/conveyor.html].

  Il vecchio blocco PML `Conveyor` (velocità e accumulo modificabili a runtime, "preserving a
  minimum space between them") è deprecato dalla 8.9.5
  [https://anylogic.help/library-reference-guides/process-modeling-library/conveyor.html].
- **FlexSim.**
  - Lo stato *Blocked* esiste solo nei nastri non accumulanti
    [https://flexsim.cloud.answerhub.com/articles/23476/conveyor-module.html].
  - Accelerazione e decelerazione si applicano solo ai cambi di velocità del nastro, non al moto
    del singolo item bloccato
    [https://flexsim.cloud.answerhub.com/articles/23506/conveyor-type-settings.html]
    [secondaria, dalla sintesi del motore di ricerca].
  - FloWorks modella il flusso continuo: il *Flow Conveyor* è non accumulante, con capacità oltre
    la quale fa `stop()`; il *Mass Flow Conveyor* è accumulante, con densità massima
    [https://docs.flexsim.com/en/24.0/modules/FloWorks/manual/Reference/Objects/LegacyConveyors.html].
- **Siemens Plant Simulation.** È un DES con Event Controller. `Conveyor` (ex `Line` fino alla
  14.0) è un oggetto "length-oriented" che usa la lunghezza reale dell'MU
  [https://docs.plm.automation.siemens.com/content/plant_sim_help/15/plant_sim_all_in_one_html/en_US/tecnomatix_plant_simulation_help/objects_reference_help/mobile_objects/mobile_objects_mus.html].
  Non ho trovato una descrizione pubblica dell'algoritmo di accumulo.
- **Simio.** Il testo ufficiale distingue i nastri accumulanti da quelli non accumulanti, che si
  fermano finché l'entità in testa non viene rimossa [https://textbook.simio.com/Workbook6/materials-handling.html].
  Non ho trovato l'algoritmo.
- **Linee ad alta velocità** (centinaia o migliaia di pezzi al minuto). Arena HiSpeed$im (Sturrock
  e Drake, WSC 1996) rifiuta sia le entità discrete ("overwhelm traditional simulation languages")
  sia l'aggregazione a lotti, che non permette "sensors on a conveyor line to determine when there
  is a backup … 10 feet from its end". Propone un motore algoritmico dedicato
  [https://www.informs-sim.org/wsc96papers/059.pdf].
- **Formula base** usata dai tool: tempo = (lunghezza del nastro − lunghezza del pezzo) / velocità
  [https://promodel.com/onlinehelp/ProModel/80/C-05 - Conveyors.htm].

### Schema "segmento" consigliato [inferenza, derivato da JaamSim più QSS]

- **Stato per segmento:**
  - `t0`, l'ultimo aggiornamento;
  - `v(t)`, polinomiale a tratti per le rampe;
  - la lista ordinata di pezzi `(x0_i, len_i)`;
  - i "trenini" accumulati, trattati come un blocco rigido.
- **A ogni evento:**
  1. si porta lo stato a `now` in forma chiusa;
  2. si cancella l'unico evento pendente;
  3. si calcola il minimo fra: prossimo contatto con il pezzo davanti (inizio dell'accumulo),
     arrivo a fine segmento, fronte di salita o discesa di ogni sensore (posizione del sensore ±
     lunghezza del pezzo), spazio libero in ingresso;
  4. si schedula quel minimo.
- **Cambio di velocità** (dal PLC o dall'inverter): è un evento esterno che riparte dal punto 1.
- **3D:** il render interpola a `t_render` senza toccare lo stato.

## 5. Determinismo e riproducibilità

- **Un flusso RNG per componente, mai uno globale.** Se si aggiunge un oggetto cambiano i numeri
  di tutti, e si rompono i common random numbers [inferenza, motivazione standard].
- **OMNeT++.**
  - `num-rngs=N` crea N RNG fisici; i moduli si mappano su RNG diversi.
  - Il default è `cMersenneTwister`, alternativa `cLCG32`.
  - Il seme automatico è N = `run# * num-rngs + rng-index`
    [https://omnetpp.org/news/2004/10/31/new-random-number-architecture.html] (post del 2004:
    la configurazione attuale `seed-set` non è stata verificata)
    [https://doc.omnetpp.org/omnetpp/api/classomnetpp_1_1cRNG.html].
- **RngStreams** (L'Ecuyer, Simard, Chen e Kelton, Operations Research 50(6) 2002, pp. 1073–1075).
  - Basato su MRG32k3a, con tanti stream lunghi divisi in substream (stream distanziati di 2^127,
    substream di 2^76).
  - Il substream serve per le replicazioni con CRN: la replica k usa il substream k di ogni
    stream.
  - È l'unico modo per *garantire* che gli stream non si sovrappongano (un seme casuale non lo
    garantisce) [https://github.com/umontreal-simul/RngStreams]
    [https://www.cs.odu.edu/~mweigle/research/netsim/rng/rng.html].
  - ns-2 è passato a MRG32k3a [https://isi.edu/nsnam/ns/doc/node267.html].
- **NumPy `SeedSequence.spawn`.**
  - Deriva figli "very probably independent" per hashing della posizione nell'albero e del seme
    radice, senza coordinazione.
  - La probabilità di collisione è circa n²·2⁻¹²⁸.
  - Per riprodurre una run servono il seme radice e la stessa struttura di spawn
    [https://numpy.org/doc/stable/reference/random/parallel.html]. In pratica conviene spawnare
    per *ID stabile* del componente, non per ordine di creazione [inferenza].
- **Il tempo intero in tick** (JaamSim `secondsToNearestTick`) elimina le divergenze dovute agli
  arrotondamenti float tra macchine o build diverse
  [https://github.com/jaamsim/jaamsim/blob/master/src/main/java/com/jaamsim/events/EventManager.java].
  Il bug di FlexSim citato al §1 mostra l'alternativa [secondaria].
- **Spareggio deterministico.** `seq` monotono (SimPy, OMNeT++, salabim) e niente ordinamenti su
  hash o indirizzi di memoria [inferenza]. La modalità random di AnyLogic va usata solo come test
  di sensibilità all'ordine [secondaria, §1].
- **Replicazioni.** Replica r ⇒ substream r di ogni stream di componente, oppure
  `SeedSequence(root).spawn(...)` con chiave `(r, component_id)`. A parità di seme si ottiene una
  run bit-identica, a condizione che anche gli input dal PLC siano registrati o riprodotti.
  - Con un PLC reale o PLCSIM nel loop la riproducibilità vale solo per la parte simulata, perché
    il tempo di ciclo del PLC introduce jitter non controllato [inferenza].
  - Va dichiarato nei risultati di misura (regola del CLAUDE.md "cosa non prova").

## Cosa questo documento non prova

- Non ho letto il codice né i manuali interni di FlexSim, Simio e Plant Simulation: per loro
  descrivo solo il comportamento esterno documentato, non l'algoritmo.
- Il default LIFO di AnyLogic e `setEventAwareSolver` vengono da fonti secondarie o dagli estratti
  del motore di ricerca.
- La sintassi attuale `seed-set` di OMNeT++ non è verificata (l'URL del manuale restituiva 404 il
  2026-11-07).
- Gli articoli su calendar queue e ladder queue: letti abstract e metadati, non il testo completo.
- I sorgenti letti sono SimPy (master su GitLab), salabim 26.0.9 (sdist PyPI) e JaamSim (master su
  GitHub), alla data del 2026-11-07.
