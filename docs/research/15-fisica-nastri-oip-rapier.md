# 15 — Nastri con la fisica: come fa OIP, e Rapier misurato

Data: 2026-11-09. Contesto: 4.3, regola di trasferimento fra due nastri. L'utente ha chiesto di guardare come fa Open
Industry Project e poi ha deciso: «Fai uguale a OIP con la fisica».

## Open Industry Project (letto il sorgente, commit `1e7488c`, MIT)

Il clone sta fuori dal repository e non è stato copiato codice.

| Cosa | Dove | Come |
|---|---|---|
| Moto dei prodotti | `docs/architecture.md`, `src/sim/` | corpi rigidi dinamici in Jolt (WASM); i nastri sono corpi statici |
| Il nastro trascina | `src/sim/conveyorContacts.ts` | ogni faccia del nastro dà una velocità superficiale ai contatti (`setSurfaceVelocity` nel `PartContext`) |
| Trasferimento fra nastri | — | **nessuna logica esplicita**: niente proprietario, accettazione o coda. Il passaggio emerge da contatti e attrito |
| Pezzo a cavallo | `src/core/conveyor/conveyorFriction.ts` | attrito di Coulomb su entrambi i nastri: nastro 0,7, rulli 0,35, cartone 0,5, combinati come √(a·b) da Jolt |
| Accumulo | — | collisioni fra scatole |
| Giunzioni | `src/sim/conveyorContacts.ts` | due filtri sui contatti «fantasma»: speculativo (separazione > 1 mm, normale oltre circa 11° dalla verticale) e «shallow-ram» (profondità ≤ 2 mm, normale entro 45° dall'orizzontale), più una protezione dei fianchi (70°). Senza questi filtri la scatola sobbalza o si incastra sullo spigolo del nastro successivo |
| Attrito accoppiato | stesso file, `FRICTION_COUPLING` | un fork di Jolt accoppia l'attrito tangenziale e torsionale, così una scatola ferma contro uno spigolo può ruotare e proseguire |
| Taratura | stesso file, `SURFACE_GATE_COS` | 65° tarati sulla demo: funziona a 60° e 70°, fallisce a 45° e 80° |
| Passo | `src/core/types/base.ts`, `src/sim/world.ts` | fisso, `DEFAULT_SIM_HZ = 60`, configurabile fra 60 e 1000 Hz; il PLC scandisce sul tempo della fisica (`docs/sim-clock.md`) |
| Diagnostica | `src/core/flow/flowFacts.ts` | «flow facts»: caduto, ribaltato, fermo, sfarfallio di una fotocellula, perso. Sono fatti, mai verdetti |

## Rapier 0.36.1 (già nell'albero per truST; Apache-2.0)

Il meccanismo documentato per i nastri è `SolverContact::tangent_velocity`: «Set to a non-zero value to simulate, e.g.,
conveyor belts». Si imposta in `PhysicsHooks::modify_solver_contacts`, attivo sui collider con
`ActiveHooks::MODIFY_SOLVER_CONTACTS`.

### La sonda (2026-11-09)

Un progetto Cargo a sé, nello scratchpad, con il solo `rapier3d = "=0.36.1"`, in release. Il nastro A è lungo 3,0 m e va
a 0,5 m/s; il nastro B è lungo 2,0 m, posto testa a testa con A. Le scatole sono cubi da 0,3 m. Attrito: nastro 0,7,
scatola 0,5, combinati con `Average`, perché Rapier non ha la media geometrica di Jolt. Il passo è di 1 ms.

| Prova | Senza filtro | Con il filtro di OIP (nastro: normale entro 45° dall'orizzontale e profondità ≤ 2 mm → contatto scartato) |
|---|---|---|
| Su un nastro solo | 0,500 m/s a regime; 21 mm indietro dopo 2 s per l'accelerazione da ferma | uguale |
| Sobbalzo sulla giunzione | 4,1–4,7 mm | 0,00 mm |
| A 0,5 → B 0,5, velocità a cavallo (10/50/90 %) | 0,159 / 0,500 / 0,500 | 0,500 / 0,500 / 0,500 |
| A 0,5 → B 1,0 | −0,135 (rimbalzo) / 0,500 / 0,632 | 0,500 / 0,501 / 0,663 |
| A 0,5 → B 0,25 | 0,258 / 0,403 / 0,250 | 0,500 / 0,298 / 0,250 |
| A 0,5 → B fermo | si ferma a cavallo | si ferma con circa 6 cm su B |
| 5 scatole contro B fermo | in coda, spazi 0–2 mm | in coda, spazi 0–1 mm |
| poi B riparte | 2 scatole restano incastrate sullo spigolo di B | ripartono tutte |
| Stesso run due volte nello stesso processo | identico bit per bit | identico bit per bit |
| Costo per passo da 1 ms, 1 / 20 / 100 scatole | 3,5 / 22 / 85 µs | 3,6 / 20 / 79 µs |
| Lo stesso con `enhanced-determinism` | — | 3,7 / 22 / 98 µs |

A 60 Hz la scatola passa la giunzione come a 1 kHz, ma ogni evento è quantizzato a 16,7 ms.

## Cosa non dimostra

- **Il determinismo fra Linux e Windows.** È misurato solo nello stesso processo; `enhanced-determinism` lo promette,
  e lo dirà la CI.
- **Rulli, curve, deviatori, scatole di forme e masse diverse, ribaltamenti voluti.**
- **La taratura del filtro su scene vere.** OIP ha tarato a mano i suoi angoli; i 45° e i 2 mm qui sono i suoi, provati
  su una sola giunzione.
- **Le fotocellule.** Nella sonda non c'erano. Con la fisica un fronte cade su un passo, non al nanosecondo come nel
  nastro analitico di M3.

## Decisioni dell'utente (2026-11-09)

1. Nastri con la fisica, come OIP: Rapier con velocità tangenziale ai contatti e il filtro sulle giunzioni.
2. Passo fisso di 1 ms, uguale al ciclo del runtime di campo.
3. Il nastro analitico di M3 (`belt.rs`) è sostituito dalla fisica; la sua spec resta come storia.
4. `enhanced-determinism` attiva, con un test in CI che confronta l'impronta su Linux e su Windows.

## Jolt via jolt-rust, la stessa sonda (2026-11-09)

L'utente ha indicato `SecondHalfGames/jolt-rust` (MIT o Apache-2.0; Jolt 5.6.0 tramite JoltC; il README avverte
«early work in progress»). La sonda è stata clonata fuori dal repository ed è stata scritta come crate del suo
workspace, `rolt` e `joltc-sys` da git, con la stessa geometria e lo stesso passo di 1 ms. Il nastro usa
`ContactSettings::RelativeLinearSurfaceVelocity` in OnContactAdded/Persisted, come il campione ufficiale
`ConveyorBeltTest.cpp`. Il filtro sulle giunzioni sta in `OnContactValidate`, perché Jolt non permette di modificare un
singolo punto di contatto.

| | Rapier 0.36.1 | Jolt 5.6.0 |
|---|---|---|
| Build | Rust puro, già nell'albero | CMake + C++ + bindgen (libclang); 56 s; 746 MB di target |
| Senza filtro | sobbalzo 4–5 mm, 2 scatole su 5 incastrate | sobbalzo 2,7–4,4 mm, 4 scatole su 5 incastrate |
| Con il filtro | funziona (tabella sopra) | il nastro a valle smette di trascinare: B a 0,25 e B fermo danno gli stessi numeri (0,500 / 0,458 / 0,359), benché il trace mostri i contatti e la velocità relativa applicati. Causa non trovata; OIP filtra con un listener nativo nel suo fork di Jolt |
| Determinismo | identico nello stesso processo | identico in tre processi diversi (`CROSS_PLATFORM_DETERMINISTIC` attivo nel build script) |
| Costo, 1 / 20 / 100 scatole | 3,7 / 22 / 98 µs per passo | 23 / 69 / 160 µs con 1 thread; 114 / 103 / 166 µs con il pool di default |
| Più mondi nello stesso processo | — | risultati incoerenti finché ogni scenario non è stato eseguito in un processo a sé |

**Decisione dell'utente (2026-11-09): Rapier.** Fa «come OIP» nel metodo, cioè velocità superficiale ai contatti e
filtro sulle giunzioni, non nel motore.
