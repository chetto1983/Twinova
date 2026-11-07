# R2: loop di simulazione a passo fisso e loop di rendering (fonti primarie)

Data ricerca: 2026-11-07. Ogni affermazione ha la sua fonte tra [ ]. [inferenza] = deduzione mia, non scritta nella fonte.
Le fonti sono state lette direttamente: pagine docs, sorgenti su GitHub raw e README. Dove una pagina non diceva qualcosa, lo segnalo.

---

## 1. Game/physics loop: separare la frequenza della simulazione da quella del rendering

### Glenn Fiedler, "Fix Your Timestep!"
- **Accumulatore.** Il tempo reale del frame si somma a un buffer; la simulazione consuma passi `dt` fissi finché il buffer contiene ≥ `dt`. Il resto "is passed on to the next frame via the accumulator variable and is not thrown away" [https://gafferongames.com/post/fix_your_timestep/]
- **Clamp del frame time.** Nel codice finale il tempo di frame in ingresso è limitato a 0.25 s, così un singolo stallo non genera una raffica di passi di recupero [https://gafferongames.com/post/fix_your_timestep/]
- **Spiral of death.** Se un passo costa più tempo reale di quanto ne simula, si resta indietro e servono sempre più passi per recuperare: "being behind causes your update to simulate more steps to catch up". Rimedi: margine di CPU, oppure un limite ai passi per frame [https://gafferongames.com/post/fix_your_timestep/]
- **Semi-fixed timestep.** Si divide il frame in passi ≤ `dt`. Questo dà un limite superiore al dt, ma l'ultimo passo parziale rende i run leggermente diversi, quindi non deterministici [https://gafferongames.com/post/fix_your_timestep/]
- **Interpolazione.** `alpha = accumulator / dt` serve a fondere lo stato precedente con quello corrente al rendering (slerp per i quaternioni). Il prezzo è un ritardo visivo fino a 1 passo [https://gafferongames.com/post/fix_your_timestep/]. La quantificazione del ritardo è un'[inferenza].

### Unity (FixedUpdate / Update)
- `Update` gira una volta per frame; `FixedUpdate` a intervallo fisso "important for the accuracy and consistency of physics simulations" [https://docs.unity.com/en-us/engine/current/manual/scripting/managing-time-and-frame-rate/fixed-updates]
- Se il frame rate supera il fixed rate, si ha 0 o 1 fixed update per frame. Se è inferiore, si ha un "backlog" e più fixed update per frame per recuperare. Esempio: dt 0.01 a 40 FPS dà 2-3 fixed update per frame [stessa URL]
- Esiste un limite oltre il quale Unity smette di recuperare [stessa URL]. È `Time.maximumDeltaTime`: limita `Time.deltaTime` dopo un frame lento e quindi il numero di `FixedUpdate` per frame a `maximumDeltaTime / fixedDeltaTime` [https://docs.unity.com/en-us/engine/current/script-reference/unityengine/time/maximumdeltatime]
- I default (fixedDeltaTime 0.02 s e maximumDeltaTime 1/3 s) non compaiono nelle pagine lette. Sono noti, ma qui **non verificati**.

### Bevy (FixedUpdate, Time<Fixed>, Time<Virtual>)
- `FixedUpdate` fa parte di `FixedMain` e gira prima di `Update`. Il default è **64 Hz** (15625 µs), scelto anche per evitare uno stutter a 60 Hz con certi refresh [https://docs.rs/bevy/latest/bevy/time/struct.Fixed.html] [https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html]
- Il ciclo è letteralmente `while world.resource_mut::<Time<Fixed>>().expend() { run FixedMain }`. Esiste `overstep_fraction()`, cioè l'alpha di Fiedler [https://raw.githubusercontent.com/bevyengine/bevy/main/crates/bevy_time/src/fixed.rs]
- `Time<Fixed>` segue `Time<Virtual>`: risente di `pause()`, `set_relative_speed()` e `set_max_delta()`. Con il clock virtuale in pausa, `FixedUpdate` non gira [https://docs.rs/bevy/latest/bevy/time/struct.Fixed.html]
- `Time<Virtual>` ha `max_delta` di default **250 ms**. Il tempo in eccesso si scarta ("better to lose the extra time and pretend a shorter duration of time passed"). `set_relative_speed(2.0)` fa correre il tempo virtuale al doppio di quello reale [https://raw.githubusercontent.com/bevyengine/bevy/main/crates/bevy_time/src/virt.rs] [https://docs.rs/bevy/latest/bevy/time/struct.Virtual.html]
- Le esecuzioni fixed non sono equispaziate nel tempo reale, quindi non vanno usate per audio o networking [https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html]

### Godot (_process / _physics_process)
- `_physics_process` gira a frequenza fissa, **60/s** di default, "independent of your game's actual framerate"; `_process` gira a ogni frame disegnato con un delta variabile [https://raw.githubusercontent.com/godotengine/godot-docs/master/tutorials/scripting/idle_and_physics_processing.rst]
- `max_physics_steps_per_frame` vale **8** di default ed è il guard contro la "physics spiral of death". Sotto `ticks/8` FPS il gioco "appear to slow down", cioè il tempo simulato rallenta rispetto al reale [https://raw.githubusercontent.com/godotengine/godot/master/doc/classes/ProjectSettings.xml]
- `physics_interpolation` (default false) interpola le trasformazioni tra gli ultimi due tick [stessa URL]

### PhysX: "Well of Despair" (la stessa spirale, vista dal motore)
- Esempio di NVIDIA: la fisica costa 9 ms per simulare 1/60 s. Dopo uno spike di 50 ms servono 3 substep, cioè ~27 ms, e il ritardo si autoalimenta. Il substepping è preferibile all'aumento del dt "because changing the size of time-steps … can significantly change perceived behavior". Variare leggermente il dt "can introduce non-determinism" [https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/BestPractices.html]

**Sintesi 1 [inferenza]:** il pattern comune ai quattro motori è accumulatore, dt fisso, tetto ai passi per frame (o al delta: Godot 8 passi, Bevy 250 ms, Unity maximumDeltaTime, Fiedler 0.25 s) e interpolazione al render. Oltre il tetto **il tempo simulato rallenta** rispetto al reale, invece di perdere passi. Per un virtual commissioning questo è il comportamento giusto: il tempo simulato non salta mai, al massimo va più lento del reale, e lo si deve segnalare.

---

## 2. Motori fisici nella simulazione industriale

### NVIDIA PhysX
- `simulate()` avanza il mondo; con i worker thread ritorna prima di aver finito. I risultati non sono disponibili finché `fetchResults()` non ritorna. Mentre la simulazione gira "it is not allowed to add, remove or modify scene objects" [https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/Simulation.html]
- Indicazioni: almeno 50 passi/s per scene complesse; il solver "is sensitive to both very small and large time steps"; spesso conviene simulare a una frequenza più alta di quella dell'applicazione, cioè fare substep [stessa URL]
- **Determinismo.** I risultati sono identici tra run solo con la stessa scena, attori inseriti nello stesso ordine, stesso schema di time-step, stessa release e stessa piattaforma; il numero di worker thread non conta. L'aggiunta di attori anche non interagenti può cambiare il risultato. `PxSceneFlag::eENABLE_ENHANCED_DETERMINISM` rende ogni isola indipendente dalle altre (con ordine di inserimento deterministico), a costo di prestazioni [https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/RigidBodyDynamics.html]
- **Kinematic.** `setKinematicTarget()` va chiamato **a ogni passo**; l'attore "will appear to have infinite mass and will push regular dynamic actors out of the way" [stessa URL]
- **Conveyor.** Le "Kinematic Surface Velocities" (tramite contact modification) emulano una superficie in moto senza muovere la posa: "can be used to create conveyor belts and rotating surfaces" [stessa URL]
- Il substepping prevede di disabilitare l'aggiornamento delle scene query nei substep intermedi [https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/BestPractices.html]

### Rapier (Rust)
- Determinismo **locale** di default: stessa macchina, stessa versione e stesso compilatore danno risultati identici [https://rapier.rs/docs/user_guides/rust/determinism/]
- Determinismo **cross-platform**: richiede la feature `enhanced-determinism` (incompatibile con `simd8`, compatibile con `parallel`: il solver parallelo dà gli stessi risultati di quello sequenziale), piattaforme IEEE 754-2008, funzioni trigonometriche da nalgebra (`ComplexField`/`RealField`), stesse condizioni iniziali e stesso ordine di inserimento e rimozione [stessa URL]
- La pagina non cita il dt come condizione, né il costo prestazionale. Che serva lo stesso dt è un'[inferenza] ovvia.

### Jolt Physics
- È deterministico se le API che modificano la simulazione sono chiamate nello stesso ordine e il binario è lo stesso. `CROSS_PLATFORM_DETERMINISTIC` costa "approximately 8% slower". Le broadphase query e l'ordine delle callback dei listener **non** sono deterministici (multithread), quindi i dati vanno ordinati. Esistono `SaveState` e il rollback [https://jrouwe.github.io/JoltPhysics/]
- Lo step è divisibile in "collision steps" (60 Hz con 2 step = 2×1/120 s); "stable when running at 60 Hz with 1 collision step" [stessa URL]
- Conveyor: `ContactSettings::mRelativeLinearSurfaceVelocity` "can be used to create a conveyor belt effect" [https://raw.githubusercontent.com/jrouwe/JoltPhysics/master/Jolt/Physics/Collision/ContactListener.h]
- I body kinematic si muovono con `MoveKinematic` ("moved by velocities only") [https://jrouwe.github.io/JoltPhysics/]

### Conveyor fisici in un tool di virtual commissioning (realvirtual.io, su Unity)
- `TransportSurface` usa "pure Unity physics functions for transporting the MUs". Per sistemi controllati in posizione da PLC raccomanda `GuidedTransport` o `KinematicMU` se diventa instabile. CAD complessi causano "sluggish physics" [https://realvirtual.io/doc/unity/components-and-scripts/motion/transportsurface]
- `KinematicMU` nasce perché con nastri aggiornati a cicli da PLC "traditional physics-based transport would violate physical constraints due to rapid or unpredictable position updates" e "can lead to instability or incorrect behavior"; disabilita le collisioni durante il moto guidato e mantiene i sensori [https://realvirtual.io/doc/unity/components-and-scripts/motion/kinematicmu-pro]
- Libreria Conveyor: la modalità guidata su spline fa sì che "Unity physics do not affect the transported object", con moto "precise and consistent". Con Unity 6 il trasporto fisico in decelerazione "isn't stable" (bug noto del vendor) [https://realvirtual.io/doc/unity/extensions/page-4/conveyor-library]
- Un riassunto di ricerca (non verificato su fonte primaria) attribuisce a realvirtual la tesi opposta sulle prestazioni, cioè fisico più veloce del path-based. **Non l'ho trovato nelle pagine lette: non usarlo.**

**Sintesi 2 [inferenza]:** il nastro fisico è "lento e non deterministico" per tre motivi documentati: (a) il contatto per attrito dipende dal solver iterativo e dal dt, e il solver è sensibile al dt (PhysX); (b) il determinismo vale solo a parità di ordine, dt, binario e piattaforma (PhysX, Rapier, Jolt), e le callback multithread hanno ordine non deterministico (Jolt); (c) aggiornamenti a scatti da PLC violano i vincoli fisici (realvirtual). Il pattern industriale è **cinematico per default** (collo = posizione lungo un percorso, s += v·dt da encoder o drive) e **fisica solo dove serve** (cadute, urti, ribaltamenti), magari con surface velocity. Per un'applicatrice tipo BoxLine con encoder unico, la posizione del collo come ascissa curvilinea esatta è un modello più fedele del PLC reale rispetto a un attrito simulato [inferenza].

---

## 3. Architettura ECS per la simulazione

- **Bevy.** Un sistema senza ordine esplicito ha un ordine scelto dallo scheduler; per sistemi in conflitto "the order is *nondeterministic* by default" e può cambiare a ogni frame. L'ordine si rende esplicito con `.before()`, `.after()`, `.chain()` e i system set [https://bevy-cheatbook.github.io/programming/system-order.html]. Lo scheduler multi-thread esegue in parallelo i sistemi non in conflitto, rileva le ambiguità (`AmbiguousSystemConflictsWarning`, `LogLevel` configurabile) e i system set servono a raggruppare [https://docs.rs/bevy/latest/bevy/ecs/schedule/index.html]
- **Change detection in Bevy.** `Changed<T>` scatta quando il componente è aggiunto **o anche solo dereferenziato mutabilmente**: "Bevy does not compare components to their previous values". Non è un filtro di archetipo, quindi itera comunque tutte le entità. Le modifiche via `Commands` sono visibili solo dopo l'apply. `Ref<T>` è l'alternativa senza filtro [https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Changed.html]
- **Unity DOTS/Entities.** Ci sono tre root group (Initialization, Simulation, Presentation). `UpdateInGroup`, `UpdateBefore/After` valgono solo tra figli diretti dello stesso gruppo, `OrderFirst/Last` ha la precedenza e l'ordinamento è gerarchico [https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/systems-update-order.html]. `FixedStepSimulationSystemGroup` ha un default di 1/60 s, gira "as many times as necessary at the fixed timestep" e sovrascrive temporaneamente `Time.DeltaTime` [https://docs.unity3d.com/Packages/com.unity.entities@1.3/api/Unity.Entities.FixedStepSimulationSystemGroup.html]
- **Fit per una factory sim [inferenza]:**
  - Pro: migliaia di colli omogenei (posizione, velocità, segmento) si prestano bene al layout ECS e alle query; la separazione tra schedule fixed e frame viene gratis (Bevy).
  - Contro: l'ordine implicito non è deterministico, quindi in un VC ogni sistema della catena I/O PLC → logica → cinematica → sensori → I/O PLC va **incatenato esplicitamente** (`chain()`). Le macchine a stati complesse (drive G120, teste) stanno meglio come componenti/attori espliciti che come sistemi sparsi.
  - Il change detection basato su DerefMut genera falsi positivi: per inviare al PLC solo i fronti, conviene confrontare i valori a mano.

---

## 4. Tempo reale e tempo "as fast as possible": pacing, overrun, thread

- **Clock virtuale con fattore di scala.** Il modello è Bevy `Time<Virtual>`: `relative_speed`, `pause`, `max_delta` (overrun → il tempo in eccesso è scartato) [https://docs.rs/bevy/latest/bevy/time/struct.Virtual.html]. In nexosim lo stesso concetto passa dal trait `Clock`: "abstracts over different types of clocks, such as as-fast-as-possible and real-time clocks"; i clock scalati si ottengono implementando `Clock` [https://raw.githubusercontent.com/asynchronics/nexosim/main/nexosim/src/time/clock.rs] [https://raw.githubusercontent.com/asynchronics/nexosim/main/nexosim/src/simulation.rs]
- **Due politiche di overrun documentate:**
  1. *Rallentare o scartare* (game engine: tetto ai passi per frame o al delta, §1). Il tempo simulato perde sincronia con il reale.
  2. *Rilevare e fallire* (nexosim): `synchronize()` restituisce `SyncStatus::OutOfSync(lag)`; con `SimInit::with_clock_tolerance(d)` un lag > d diventa `ExecutionError::OutOfSync`; `with_timeout` limita il tempo reale di ogni step (`ExecutionError::Timeout`) [https://raw.githubusercontent.com/asynchronics/nexosim/main/nexosim/src/time/clock.rs] [https://raw.githubusercontent.com/asynchronics/nexosim/main/nexosim/src/simulation/sim_init.rs]
  - [inferenza] Per un VC accoppiato a un PLC reale o PLCSIM la politica 2 è più onesta: un test che ha perso il real time va invalidato, non nascosto. La politica 1 va bene per la sola vista 3D.
- **Thread di simulazione separato da render/UI.**
  - Bevy pipelined rendering: il render del frame N gira "at the same time as the N + 1 frame's simulation". Lo step *Extract* "is the step where data is copied from the main world to the render world" (snapshot), sul thread principale, e poi i due mondi procedono indipendenti [https://docs.rs/bevy/latest/bevy/render/pipelined_rendering/struct.PipelinedRenderingPlugin.html]
  - PhysX: durante `simulate()` la scena non è modificabile; le scritture vanno bufferizzate e i risultati si leggono dopo `fetchResults()`, che fa da punto di sincronizzazione o double buffer. L'API "active actors" restituisce solo gli attori mossi, per aggiornare le mesh di render [https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/Simulation.html] [https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/RigidBodyDynamics.html]
  - [inferenza] Per un'HMI web  il pattern equivalente è: thread o processo di simulazione a dt fisso → snapshot immutabile (pos/rot per entità + timestamp sim) pubblicato a 30-60 Hz → client che interpola tra gli ultimi due snapshot (alpha di Fiedler). I comandi UI → sim viaggiano come messaggi in coda (cfr. l'`Injector` di nexosim, §5) e non come scritture dirette sullo stato.

---

## 5. Rust: crate di simulazione e scheduling

### nexosim (ex asynchronix), il più rilevante
- Stato: `asynchronix` è deprecato ("Asynchronix is now NeXosim", ultima versione 0.2.4 del 2024-11). `nexosim` **1.0.0** è uscito il 2026-02-03, con circa 23k download [https://crates.io/api/v1/crates/asynchronix] [https://crates.io/api/v1/crates/nexosim] [https://raw.githubusercontent.com/asynchronics/nexosim/main/CHANGELOG.md]
- **Natura:** "discrete-event simulation framework", nato "for simulators able to handle large cyberphysical systems", con esperienza da "spacecraft real-time simulators". Licenza MIT/Apache [https://raw.githubusercontent.com/asynchronics/nexosim/main/README.md]
- **Modello a componenti/attori.** Un modello è un'"isolated entity with a fixed set of typed inputs and outputs", collegata a bench-assembly time, in stile flow-based programming. Le porte sono Output (eventi broadcast), Requestor/UniRequestor (richiesta/risposta), input e replier come metodi `&mut self` sync o async. Ogni modello ha una `Mailbox` (MPSC async bounded) [https://docs.rs/nexosim/latest/nexosim/] [README]
- **Runtime:** "custom multi-threaded executor", con auto-parallelizzazione dei modelli su un pool fisso di thread. Non usa Tokio perché la fine di un istante simulato "is technically a deadlock" (tutti i modelli bloccati su mailbox vuote) e al posto del reactor c'è una **priority queue di eventi**; avanzamento "next event increment" [README]
- **Ordinamento deterministico:** si eseguono i task in ordine cronologico. Garanzie: (1) A invia M1 e poi M2 a B, quindi B li processa in quell'ordine; (2) transitività attraverso modelli intermedi; valgono anche per eventi contemporanei dallo scheduler [https://docs.rs/nexosim/latest/nexosim/]
- **Controllo del tempo:** `step()` porta al prossimo evento, `step_until(deadline)`, `run()` fino all'esaurimento degli eventi, `process_event` / `send_query` senza avanzare il tempo. I modelli schedulano eventi futuri cancellabili via `Context` (`#[nexosim(schedulable)]`) [https://docs.rs/nexosim/latest/nexosim/]
- **Pacing in tempo reale (letto nel sorgente):**
  - default `NoClock`, che "effectively makes the simulation run as fast as possible", in modalità tickless [clock.rs] [simulation.rs]
  - `SystemClock::from_instant(t0, Instant)` / `from_system_time`: offset fisso tra tempo simulato e monotonic clock. `synchronize(deadline)` fa `spin_sleep` fino alla deadline e restituisce `Synchronized`; se è già passata, restituisce `OutOfSync(now - deadline)` **senza aspettare né saltare**. `AutoSystemClock` fissa il riferimento alla prima chiamata [https://raw.githubusercontent.com/asynchronics/nexosim/main/nexosim/src/time/clock.rs]
  - Un test interno verifica una precisione di ±0.5 ms su 200 ms [stessa URL]. Questo **non** prova il jitter sotto carico o su Windows.
  - **Ticker** (`PeriodicTicker`, con `SimInit::with_clock(clock, ticker)`): sveglia il clock a intervalli regolari anche senza eventi, così la simulazione resta reattiva a eventi iniettati e `halt`. In tickless la simulazione dorme fino al prossimo evento e "prohibits events scheduling within the current idle window". Con un ticker, `run()` attende eventi iniettati fino a `Scheduler::halt` [https://raw.githubusercontent.com/asynchronics/nexosim/main/nexosim/src/simulation/sim_init.rs] [simulation.rs]
  - `Injector` / `EventInjector`: una coda per eventi esterni (rete, UI, PLC) introdotta in 1.0 insieme al Ticker [sim_init.rs] [CHANGELOG.md]
- **Extra 1.0:** save/restore del bench (i modelli devono derivare Serialize/Deserialize), `ClockReader` clonabile, server gRPC opzionale (feature `server`) per controllo e monitoraggio remoto, con client Python annunciato, tracing [CHANGELOG.md] [https://docs.rs/nexosim/latest/nexosim/]
- **Rilevamento deadlock:** mailbox piene o cicli di requestor → `ExecutionError::Deadlock` con i modelli coinvolti [simulation.rs]
- [inferenza] Per un VC con fisica cinematica continua, nexosim si usa con un **evento periodico auto-schedulato** (es. 1 ms) che integra la cinematica, insieme a eventi discreti per sensori e attuatori. Il fixed step diventa quindi un caso particolare del DES. Il ponte verso il PLC (S7CommPlus/OPC UA) passerebbe dall'Injector, e il rendering leggerebbe snapshot via event sink o gRPC. Da verificare con un prototipo: costo per evento a 1 kHz × N modelli, jitter di `spin_sleep` su Windows.

### Altri crate (meno adatti)
- **sim** (ndebuhr, "SimRS"): DES con modelli pronti, input modeling stocastico, output analysis, target WASM/npm; v0.13.1 del 2025-04, "stability-experimental". Il README non cita il pacing real time [https://raw.githubusercontent.com/ndebuhr/sim/master/README.md] [https://crates.io/api/v1/crates/sim]
- **desim**: "inspired by Simpy"; i processi sono coroutine che restituiscono `Effect`; richiede **nightly** (feature coroutines), GPLv3, v0.4.0 del 2023 [https://raw.githubusercontent.com/garro95/desim/master/README.md] [https://raw.githubusercontent.com/garro95/desim/master/src/lib.rs]
- **simrs** (elshize): "experimental", con State/Scheduler/Components e code tipizzate; v0.2.0 del 2022 [https://raw.githubusercontent.com/elshize/simrs/master/src/lib.rs] [https://crates.io/api/v1/crates/simrs]
- In nessuno dei tre ho trovato supporto al tempo reale nel README o nella doc del modulo radice [inferenza da assenza: grep "real-time|wall" senza risultati utili].

---

## Cosa queste fonti NON provano
- Nessun benchmark numerico fisica vs cinematica per i conveyor: le affermazioni di realvirtual sono qualitative.
- Il determinismo di PhysX, Rapier e Jolt è documentato dai vendor e non l'ho verificato qui.
- Il jitter reale di nexosim `SystemClock` su Windows o sotto carico non è misurato (c'è solo un unit test a ±0.5 ms).
- I default Unity (0.02 s / 0.333 s) non sono confermati dalle pagine lette.
