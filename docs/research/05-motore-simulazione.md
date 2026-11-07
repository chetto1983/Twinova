# Motore di simulazione: eventi discreti o passo fisso?

Stato: **studio, 2026-11-07**. Risponde alla domanda "come è organizzato il motore di Visual Components
e come lo fanno gli altri". La §6 è una **proposta da decidere**, non un design approvato.

Fonti:
- l'API Python 3 di Visual Components 5.1 (pagine citate sotto);
- tre ricerche con un URL per ogni affermazione:
  - [r1](ricerca/r1-eventi-discreti.md): motori a eventi discreti e ibridi;
  - [r2](ricerca/r2-passo-fisso-fisica.md): passo fisso, fisica, ECS, Rust;
  - [r3](ricerca/r3-virtual-commissioning-plc.md): virtual commissioning e PLCSIM Advanced.

Ho ricontrollato i punti chiave sulla fonte:
- il ciclo di SimPy, in `core.py`;
- i modi `TimespanSynchronized_*`, `StartProcessing(t)` e `OnSyncPointReached` nel Function Manual
  PLCSIM Advanced 11/2019;
- le pagine API di Visual Components citate in §1.

## 1. Visual Components: ibrido con un nucleo a eventi discreti

Fonte: `https://help.visualcomponents.com/5.1/Premium/en/Python_3_API/...`. Il testo dice quello che
segue; l'interpretazione è mia.

| Indizio | Pagina | Cosa ne segue |
|---|---|---|
| "vcCore uses asyncio": la logica è `async def OnRun()`, il tempo si consuma con `await vc.delay(s)`, `await signal.waitFor(v)`, `await vc.condition(f)` | `How to use/how_to_use_consume_simulation_time.html`, `Modules/vcCore.html` | **process interaction**: ogni componente è una coroutine sospesa su un evento futuro |
| `condition()` viene rivalutata "sui segnali" o a mano con `evaluateConditions()` | `Modules/vcCore.html` | attesa *correlata*: si rivaluta solo al cambio di un segnale, senza polling a ogni passo |
| `autoHalt()` ferma la simulazione se nessuno script genera più eventi | `Classes/vcCore_vcSimulation.html` | c'è una coda di eventi futuri: quando è vuota, il tempo non ha motivo di avanzare |
| `FastScheduling` sui path: se attivo il nastro "limita gli eventi generati"; se spento muove i pezzi "a incrementi più corti, con più eventi" | `Classes/vcBehaviors_vcMotionPath.html` | il nastro è un generatore di eventi, analitico oppure a micro-passi |
| `UseHeartbeat` e `HeartbeatTime` sui Servo e Robot Controller | `Classes/vcRobotics_vcServoController.html`, guida Servo Controller | sopra il nucleo DES c'è un impulso periodico **opzionale** |
| Sensori con `UseSampling` e `SampleTime`, oppure a impulso su un segnale | guida Volume e Raycast Sensor | campionamento solo dove serve |
| `vcSimulationLevel` FAST, BALANCED, DETAILED | `Classes/vcCore_vcSimulationLevel.html` | fedeltà del moto regolabile: si può saltare da punto a punto oppure simulare l'intera traiettoria |
| `RealTimeMode` e `SimSpeed` | `Classes/vcCore_vcSimulation.html` | lo stesso clock virtuale va al ritmo del clock reale (con fattore di scala) oppure il più veloce possibile |
| Fisica NVIDIA PhysX (aggiornata a 5.5.0 nella 5.1); la 5.1.1 corregge risultati non deterministici della 5.1 | guida Physics world, release notes 5.1 e 5.1.1 | la fisica è un'isola a passo fisso, separata dal nucleo |
| Solo il controller virtuale Stäubli "usa il clock della simulazione per i passi"; i PLC si scambiano dati in modo ciclico o a evento | guida Stäubli Add-on, Connectivity | **nessun lockstep con il PLC** |

**Cosa non prova:**
- la struttura dati della coda;
- le regole di spareggio allo stesso istante;
- se il tempo è in virgola mobile o in tick;
- il passo di PhysX.

L'help non le documenta. Per saperlo servirebbe il prodotto.

## 2. Le tre famiglie e chi usa cosa

| Famiglia | Come avanza il tempo | Chi la usa (fonte in r1, r2, r3) | Forte in | Debole in |
|---|---|---|---|---|
| **Eventi discreti (DES)** | salta al prossimo evento della coda | SimPy, salabim, JaamSim, OMNeT++, Plant Simulation, FlexSim, nucleo di VC | flusso materiali, migliaia di pezzi, simulare ore in secondi | moto continuo e contatti, che vanno resi eventi |
| **Passo fisso** | `t += dt`, con accumulatore e interpolazione nel render | Unity `FixedUpdate`, Bevy `FixedUpdate`, Godot, PhysX, Rapier, Jolt, realvirtual, NX MCD, SIMIT (multi-rate, 8 slice ≥ 1 ms), ISG-virtuos, Factory I/O (1/120 s), CIROS (40 ms) | fisica, contatti, ciclo PLC, codice semplice | costo a ogni passo anche a macchina ferma; precisione limitata a `dt` |
| **Ibrido** | DES più tratti continui con eventi di tempo e di stato (zero-crossing), iterati allo stesso istante | AnyLogic, Modelica, FMI 3.0, DEVS/QSS | il meglio dei due | è la più complessa da scrivere bene |

Gli strumenti di virtual commissioning si accoppiano al PLC in due modi (r3):
- **asincrono**, a tempo reale: VC, OPC UA;
- **sincrono**, a tempo virtuale: SIMIT isochronous, Emulate3D "synchronized", ISG-virtuos sotto 1 ms in
  tempo reale duro.

## 3. Cosa fanno, concretamente, gli sviluppatori esperti

1. **Coda di eventi deterministica.** Uno heap binario di `(tempo, priorità, seq)`, dove `seq` è un
   contatore crescente: a parità di tempo e priorità gli eventi escono FIFO, sempre nello stesso ordine.
   Così fanno SimPy (verificato: `heappush(self._queue, (now + delay, priority, next(self._eid), event))`,
   URGENT=0, NORMAL=1) e OMNeT++.
   - Calendar queue e ladder queue convengono solo oltre circa 10⁵ eventi pendenti (r1 §1).
2. **Tempo in interi, non in `double`.** JaamSim lavora in tick interi. Uno sviluppatore FlexSim ha
   attribuito un nastro bloccato a eventi "simultanei" ordinati male dal floating point (r1 §1).
3. **Process interaction con coroutine**, come VC, SimPy e salabim. Le attese "finché" si collegano allo
   stato da cui dipendono e si rivalutano solo quando quello cambia, mai con polling a ogni evento: in
   salabim `State.wait` è correlato, mentre `standby` è un polling ed è lento (r1 §2).
4. **Nastri senza passi per frame** (JaamSim `EntityConveyor`, letto nel sorgente, r1 §4):
   - le posizioni si aggiornano solo agli eventi, in forma chiusa;
   - si calcola il minimo fra spazio libero in ingresso, uscita, accumulo e fronte di un sensore, e lo si
     mette in coda come unico evento pendente del nastro;
   - un cambio di velocità cancella quell'evento e lo ricalcola;
   - il 3D interpola senza toccare lo stato.
   Il nastro "fisico" (velocità di superficie in PhysX o Jolt) esiste, ma realvirtual documenta che è
   instabile con nastri comandati a cicli dal PLC, e lo affianca con un trasporto cinematico su spline
   (r2 §2).
5. **Stesso istante, più passi.** In FMI e Modelica si itera finché nessuno cambia più
   (`t_R`, `t_I`: superdense time). JaamSim ripete il tick se una condizione è diventata vera.
   Servono per catene come sensore → segnale → logica → attuatore a tempo zero (r1 §3).
6. **Simulazione separata dal render.**
   - Accumulatore a passo fisso con un tetto ai passi per frame: 0,25 s in Fiedler, 250 ms in Bevy,
     8 passi in Godot. Oltre il tetto il tempo simulato rallenta.
   - Il render interpola con `alpha`.
   - Il render legge una copia dello stato (Bevy "Extract", PhysX `fetchResults`) (r2 §1, §4).
7. **Politica di overrun dichiarata.** I giochi rallentano o scartano il tempo in eccesso. nexosim (Rust)
   rileva il ritardo e fallisce con `OutOfSync` oltre una tolleranza, e questo è più onesto per una prova
   contro il PLC (r2 §4–5).
8. **Ordine esplicito dei sistemi.** Nell'ECS di Bevy l'ordine non dichiarato è indefinito (r2 §3). La
   catena PLC → logica → cinematica → sensori va fissata a mano.
9. **Riproducibilità.** Uno stream casuale per componente, legato a un ID stabile (MRG32k3a, NumPy
   `SeedSequence.spawn`), più repliche e numeri casuali comuni (r1 §5). Con un PLC nel loop il
   determinismo vale solo in modo sincrono.

## 4. Il punto PLC: asincrono o sincrono

Nell'accoppiamento asincrono (VC con OPC UA o S7) simulazione e PLC girano ognuno col proprio orologio:
il jitter del PC finisce nella prova. Per una macchina veloce conta: a 2 m/s **1 ms vale 2 mm**.

PLCSIM Advanced ha la modalità che serve per l'accoppiamento sincrono (Function Manual 11/2019 §6.3, verificato; r3 §2):
- in `TimespanSynchronized_C`, `_P` o `_CP`, la CPU si ferma in *freeze* (tempo virtuale fermo, OB e
  timer fermi, I/O consistente) a ogni sync point e lancia `OnSyncPointReached`;
- il master chiama `StartProcessing(t_ns)`: la CPU gira **almeno** `t` di tempo virtuale e si ferma al
  sync point successivo, riportando il tempo usato (`TimeSinceSameSyncPoint_ns`).
- Il passo reale è quindi il ciclo del PLC: tra `t` e circa `t` più un ciclo di OB *[inferenza dal
  testo]*.

Cosa è misurato e cosa no:
- **Misurato nel progetto BoxLine** (2026-10-29, il repository `chetto1983/apr`): una S7-1200 G2 gira in PLCSIM Advanced **V8** e l'API
  .NET V8 funziona (`plc_test/PlcSimAdv.ps1` in quel repository).
- **Non misurato:** che i modi `SingleStep` e `TimespanSynchronized` funzionino su un'istanza G2.
- **Limite:** PLCSIM non simula gli HSC onboard (r3 §2, PLCSIM V19 §5.11): encoder e ingressi di cattura
  vanno emulati dal simulatore, ed è un punto aperto.

## 5. Risposta alla domanda

- Visual Components è **ibrido**:
  - nucleo **a eventi discreti** con coroutine;
  - nastri analitici o a micro-passi;
  - campionamento opzionale (heartbeat, sensori);
  - fisica PhysX a passo fisso, separata;
  - clock reale o virtuale;
  - nessun lockstep con i PLC.
- I simulatori di flusso (Plant Simulation, FlexSim, JaamSim) sono DES puri.
- I tool di virtual commissioning con fisica (NX MCD, realvirtual) sono a passo fisso.
- Chi fa VC "seria" (SIMIT, Emulate3D) aggiunge un modo **sincrono**, con il PLC in tempo virtuale.

## 6. Proposta per il clone (da decidere)

Combina quello che fanno i migliori. È un'ipotesi da misurare prima di diventare design.

1. **Nucleo DES:**
   - tempo intero in **ns** (`i64`, circa 292 anni);
   - heap `(t, prio, seq)`;
   - iterazione allo stesso istante fino a quiete, con un tetto e un errore se viene superato.
2. **Logica dei componenti come processi** (coroutine o macchine a stati). Le attese sono collegate ai
   segnali e i segnali notificano solo quando il valore cambia.
3. **Nastri e assi a segmenti analitici** (schema JaamSim con velocità a tratti):
   - un solo evento pendente per nastro;
   - i fronti dei sensori sono calcolati, non campionati, quindi l'istante del fronte è esatto al ns.
4. **Fisica opzionale, a passo fisso**, solo per le isole che ne hanno bisogno (cadute, ribaltamenti).
   Il passo è comandato dal nucleo come evento periodico. Nastri cinematici di default, non fisici (r2 §2).
5. **Due modi di marcia:**
   - **Libero**: clock virtuale agganciato al reale con fattore di scala. Se il ritardo supera la
     tolleranza si segnala `OutOfSync`, non si recupera di nascosto.
   - **Lockstep con PLCSIM Advanced**: il simulatore è il sync master. Il ciclo è:
     1. `StartProcessing(Δ)`;
     2. `OnSyncPointReached` (tempo virtuale `t_plc`);
     3. il DES avanza fino a `t_plc`;
     4. si scrivono gli ingressi e si leggono le uscite;
     5. si ricomincia.

     In questo modo una prova è ripetibile.
6. **Render separato**: snapshot a 30–60 Hz verso la vista 3D, interpolati sul client. I
   comandi dell'interfaccia arrivano come eventi esterni in coda.

Il Rust ha un candidato da valutare come base del nucleo, invece di scriverlo da zero (regola
"inventario prima dell'invenzione"): **nexosim** 1.0 (DES ad attori async, ordine deterministico,
`SystemClock` con `OutOfSync`, save/restore, r2 §5). Non è ancora stata fatta una scelta di linguaggio.

## 7. Da misurare prima di decidere

1. PLCSIM Advanced V8 in `TimespanSynchronized_CP` (S7-1500 e S7-1200 G2):
   - funziona?
   - quale passo minimo?
   - quanto costa un giro `StartProcessing` → `OnSyncPointReached` in tempo reale?
2. Velocità del lockstep rispetto al tempo reale con un programma PLC tipico.
3. Emulazione di encoder e HSC: cosa è scrivibile dall'API.
4. Jitter del modo libero su Windows sotto carico (le fonti non danno numeri, r2 "Cosa queste fonti NON provano").
