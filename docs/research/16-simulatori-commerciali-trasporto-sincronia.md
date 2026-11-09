# 16 — Simulatori commerciali: come trasportano e come si sincronizzano con il PLC

Data: 2026-11-09. Domanda dell'utente, al momento di confermare la fisica a passo fisso: «Guarda la soluzione più
industriale possibile come fanno i commerciali da 30K».

Le fonti sono solo pubbliche: manuali, help ufficiali, articoli, forum. Non sono stati aggirati login né paywall, e
testo e codice non sono stati copiati. «S» indica quanto una fonte afferma, «I» la nostra inferenza.

## Trasporto dei prodotti

| Prodotto | Modello | Fonte |
|---|---|---|
| Siemens NX MCD | S: fisica NVIDIA PhysX (PhysX 5 dal 2024); un oggetto senza «Rigid Body» è statico; la «Transport Surface» ha una velocità parallela alla superficie. I: il nastro è una faccia statica che dà velocità tangenziale ai contatti; il passaggio fra nastri è emergente | [annuncio 2010](https://www.digitalengineering247.com/article/siemens-plm-software-announces-mechatronics-concept-designer/products), [NX giugno 2024](https://blogs.sw.siemens.com/nx-design/whats-new-in-nx-june-2024-multi-discipline-design/), [NX tips](https://blogs.sw.siemens.com/nx-design/nx-tips-and-tricks-mechatronics-concept-designer) |
| Emulate3D (Rockwell) | S: i carichi si muovono per attrito col nastro sotto gravità, un moto «emergent, as opposed to deterministic»; la gamma Sim3D usa modelli analitici «friction-free», deterministici e più veloci. Motori selezionabili (snippet): ODE, Bullet, PhysX, AGX | [McGregor, WSC 2012](https://www.informs-sim.org/wsc12papers/includes/files/vdrp104.pdf) |
| Visual Components | S: di default il percorso è cinematico e a eventi (`vcMotionPath`: capacità, accumulo, occupazione dello spazio); il passaggio è esplicito (One to One Interface); la fisica è opzionale per componente (Physics Path spinge gli oggetti con collider; Physics Entity) | [vcMotionPath](https://help.visualcomponents.com/5.0/Premium/en/Python_3_API/Classes\vcBehaviors_vcMotionPath.html), [Physics Path](https://help.visualcomponents.com/4.8/Premium/zh-cn/English/Component%20Modeling/Behaviors/Physics/Physics_Path.htm), [One to One](https://help.visualcomponents.com/4.10/Premium/de/English/Component%20Modeling/Behaviors/Interfaces/One_to_One_Interface.htm) |
| Siemens SIMIT (CONTEC) | S: niente fisica; oggetti su percorsi lineari, con le posizioni calcolate «at equidistant times»; passaggio esplicito per connettori, stopper per l'accumulo; sensori come posizioni sul segmento; modello di accumulo «simplified to a great extent» | [SIMIT V10 Operating Manual](https://cache.industry.siemens.com/dl/files/317/109759317/att_956837/v1/SIMIT_enUS_en-US.pdf) §8.3.2.1 |
| Process Simulate | S: nastri che portano parti o skid; un «simulation time interval» scelto dall'utente; un motore fisico separato per gravità e collisioni. I: trasporto prevalentemente cinematico | [Tecnomatix 14](https://blogs.sw.siemens.com/tecnomatix/tecnomatix-14-whats-new/) |
| Factory I/O | S: fisica a corpi rigidi a passo fisso, `physics.timestep` 1/120 s, 8 sotto-passi, 20 iterazioni del solutore | [console](https://docs.factoryio.com/manual/console/), [forum](https://community.factoryio.com/t/are-there-further-information-about-the-console-physics-parameters/1233) |
| Simumatik | S: modello URDF/SDF; un link può avere una «Motion Surface» con direzione e velocità | [link](https://docs.simumatik.com/datamodel/link/) |
| Plant Simulation, FlexSim | S/I: a eventi discreti, su percorso, con connessioni esplicite (in parte solo da snippet) | — |

**Lettura.** Per la messa in servizio virtuale la fascia alta usa la fisica con superfici di trasporto: MCD ed
Emulate3D in emulazione. Per l'analisi di capacità usa modelli cinematici o analitici: Sim3D, Visual Components di
default, SIMIT CONTEC, Plant Simulation. Emulate3D lo dice esplicitamente (WSC 2012).

## Accoppiamento con il PLC

| Prodotto | Modo | Fonte |
|---|---|---|
| SIMIT + PLCSIM Advanced + MCD | S: tre modi. **Asincrono**: guidato dal tempo, con cicli persi. **Sincrono**: ordine fisso, nessun ciclo perso. **Bus synchronous** (solo per PLCSIM Advanced e MCD): «the same synchronized simulation progress», con la slice uguale al ciclo del bus (esempio 2 ms, minimo 1 ms); il ciclo successivo parte solo quando tutti hanno finito | SIMIT V10 manual, p. 45, §2.12 |
| PLCSIM Advanced | S: un orologio virtuale con fattore di scala; modi SingleStep (C/CT/P/CP/CPT) e TimespanSynchronized, con `RunToNextSyncPoint()` | [Function Manual V6.0](https://cache.industry.siemens.com/dl/files/578/109955578/att_1267815/v1/s7-plcsim_advanced_function_manual_en-US.pdf), [slide 2018](https://assets.new.siemens.com/siemens/assets/api/uuid:3d8be3e0-00bf-49dc-b2c9-ed3c53f993ae/plcsimadvanced.pdf) |
| Process Simulate | S (brevetto): non è un lockstep; la velocità di PLCSIM viene riscalata ciclo per ciclo, s(n+1) = s(n)·Δt_PS/Δt_PLCSIM | [US12117790](https://patents.google.com/patent/US12117790) |
| Emulate3D | S (snippet 2025): un nuovo protocollo per PLCSIM Advanced con sincronizzazione temporale ai punti di sincronizzazione | [ROKLive 2025](https://www.rockwellautomation.com/content/dam/rockwell-automation/documents/pdf/events/roklive-emea-2025/digital-engineering-summit/augment-your-digital-model-with-cosimulation.pdf) |
| Visual Components | S (risposta del fornitore sul forum): scambio sempre asincrono, legato al tempo reale | [forum](https://forum.visualcomponents.com/t/opc-plugin-update-time/787) |
| Factory I/O | I: nessun lockstep documentato | — |

Determinismo: nessuno dei prodotti lo promette. MCD ha un'opzione `RandomizeSolvingOrder`.

## Conseguenza per Twinova (I)

La combinazione più industriale per la messa in servizio virtuale è quella di Siemens, MCD + SIMIT bus synchronous +
PLCSIM Advanced: fisica con superfici di trasporto e un lockstep a slice fissa su tempo virtuale. Twinova la adotta:
- Rapier con velocità tangenziale (ricerca 15);
- un ciclo a passo fisso di 1 ms: fisica, modelli nativi, runtime di campo, poi il PLC quando la sua scansione è dovuta.

In più Twinova garantisce il determinismo, che i commerciali non promettono. In M5 lo stesso ciclo si aggancia a
PLCSIM Advanced tramite i suoi punti di sincronizzazione (TimespanSynchronized / `RunToNextSyncPoint`).

Un modello cinematico «fast», come Sim3D o i percorsi di Visual Components, resta un'opzione per l'analisi di
capacità, da mettere in ROADMAP e non in 4.3.

## Cosa non dimostra

- **Il default del passo di MCD e di Process Simulate.** Non è confermato; il valore di 1 ms per MCD viene da una
  tabella API poco chiara.
- **Il default del motore di Emulate3D, e il metodo dei sensori di Factory I/O e Simumatik.** Non trovati.
- **Parte delle fonti è nota solo da snippet di ricerca** (Emulate3D help, Plant Simulation, FlexSim): le pagine
  erano dietro login o costruite in JavaScript.
