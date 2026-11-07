# G3: simulatori leggeri/didattici e ecosistema open-source

Data della ricerca: 2026-11-07. Testi riformulati (clean-room), nessuna copia di testo o immagini.
Legenda: **[URL]** = fonte primaria letta; **[secondaria]** = forum, rivenditore o snippet di ricerca;
**[inferenza]** = deduzione mia, non scritta nelle fonti.

Fonti primarie lette per intero o per capitolo:
- Factory I/O: docs.factoryio.com (sitemap completa, letti manuale driver, tag, edit/run, failure injection,
  console, SDK, Web API, scene, parti, emitter, opzioni, licenze, release notes) + factoryio.com/editions e /buy.
- CIROS: manuale Festo "CIROS CP-System User Manual v7.1" (PDF 236 pagine, ftp.festo.com) + pagina prodotto
  CIROS Studio dello sviluppatore (VEROSIM Solutions, ciros-engineering.com).
- FluidSIM: manuale utente FluidSIM 6 (PDF 604 pagine, art-systems.de) + pagina prodotto Art Systems.
- festo.com risponde **403 (Akamai)** alle richieste automatiche: non aggirato. Ciò che viene da festo.com è
  marcato [secondaria] (snippet di ricerca).

---

## 1. Factory I/O (Real Games, Portogallo)

### Cosa è e per chi
- Simulatore 3D di impianti per la formazione PLC: si sceglie una scena, si collega un PLC (reale, soft-PLC
  o simulatore), si allena programmazione e troubleshooting [https://factoryio.com/].
- Pubblico: scuole, università, centri di formazione, e reparti manutenzione/automazione per l'onboarding
  [https://factoryio.com/].
- Lo staff stesso lo posiziona come strumento didattico, non di virtual commissioning: in forum un utente
  viene rimandato a WinMOD per il commissioning [secondaria, https://community.factoryio.com/t/create-custom-parts/97
  tramite snippet di ricerca].
- Solo Windows (7 SP1+), GPU DX10/11/12 [https://docs.factoryio.com/manual/system-requirements/].
- Ultima versione: 2.5.10 del 2025-10-15 (hotfix OPC UA); 2.5.9 aggiunge PLCSIM V20 [https://docs.factoryio.com/release-notes/].

### Licenza / prezzi / edizioni (pubblici)
| Edizione | Cosa include | Prezzo indicativo | Fonte |
|---|---|---|---|
| Ultimate | tutti i driver + SDK + Web API | da 278 €/anno | https://factoryio.com/editions |
| Siemens | S7-1200/1500, LOGO!, S7-200/300/400, PLCSIM e PLCSIM Advanced | 158 €/anno o 435 € una tantum | https://factoryio.com/editions |
| Allen-Bradley | Logix5000, Micro800, MicroLogix, SLC 5/05 | 158 €/anno o 435 € | https://factoryio.com/editions |
| Modbus & OPC | Modbus TCP client/server, OPC DA/UA client | 158 €/anno o 435 € | https://factoryio.com/editions |
| Starter / MHJ / Automgen | soft-PLC Control I/O, oppure WinSPS-S7/Grafcet-Studio, oppure Automgen | 40 €/anno o 109 € | https://factoryio.com/editions |

- Tutte le edizioni contengono tutte le scene e tutte le parti; cambiano solo i driver [https://factoryio.com/editions].
- Abbonamento mensile da 17 €/mese; l'acquisto una tantum copre tutte le release 2.x [https://factoryio.com/buy].
- Trial completo di 30 giorni [https://docs.factoryio.com/manual/options/].
- Licenze standalone (legate al PC, disattivabili) o floating (pool con "share code", Internet permanente,
  ritorno al pool dopo 10 min se crash) [https://docs.factoryio.com/manual/licensing/].

### Funzionalità principali
| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Libreria scene | ~21 scene pronte, da "From A to B" (un sensore) a magazzino automatico, pallettizzatore, controllo di livello PID; sola lettura, si salvano come copia | https://docs.factoryio.com/manual/scenes/ |
| Scene numerate per argomento PLC | 1 trasporto, 2 set/reset, 3 timer, 4 contatori: la scena insegna un'istruzione | https://docs.factoryio.com/manual/scenes/ |
| Parti | 8 categorie: item, carico pesante (rulli, chain transfer, turntable), carico leggero (nastri, deviatori, pusher, bilancia), sensori, operatori (pulsanti, selettori, potenziometro, display), stazioni (centro di lavoro, elevatore, pick&place, traslo-elevatore, pallettizzatore, serbatoio), segnalatori, passerelle | https://docs.factoryio.com/manual/parts/ |
| Sensori | capacitivo, diffuso, induttivo, barriera a matrice, retroriflettente, RFID, visione, encoder incrementale; ognuno con materiali rilevabili e portata dichiarata (es. induttivo 0-0,1 m, capacitivo 0-0,2 m con uscita 0-10 V di distanza) | https://docs.factoryio.com/manual/parts/sensors/ |
| Portata via gizmo | la portata del sensore si regola trascinando un cerchio nella vista 3D | https://docs.factoryio.com/manual/parts/sensors/ |
| Emitter / Remover | generano/cancellano pezzi; scelta casuale di pezzo e base, intervallo min/max, numero massimo; il mix è anche impostabile dal PLC come maschera di bit | https://docs.factoryio.com/manual/parts/emitter/ |
| Tag | ogni sensore/attuatore ha tag Bool / Float (REAL) / Int (DINT), con nome auto-generato rinominabile | https://docs.factoryio.com/manual/tags/ |
| Forzatura | clic sul tag per forzare sensori o attuatori; forzando gli attuatori l'utente "fa il PLC" a mano | https://docs.factoryio.com/manual/tags/ |
| Tag di simulazione | tag di sistema leggibili dal PLC (Paused, Reset 1 s, Running, Time Scale) e comandabili (Run, Pause, Reset, Camera) | https://docs.factoryio.com/manual/tags/ |
| Driver | Advantech USB-4750/4704, AB Logix5000/Micro800/MicroLogix/SLC 5/05, Automgen, Control I/O, MHJ, Modbus TCP client e server, OPC DA/UA client, LOGO!, S7-200/300/400, S7-1200/1500, S7-PLCSIM | https://docs.factoryio.com/manual/drivers/ |
| PLCSIM | PLCSIM V5, V13-21 e Advanced V3.0-8.0; con PLCSIM "classico" V13-21 serve un **progetto template TIA** con un Network 1 in OB1 da non cancellare (esiste anche un template S7-1200 G2 V20); con Advanced l'istanza deve chiamarsi `factoryio` | https://docs.factoryio.com/manual/drivers/s7-plcsim/ |
| Mappatura S7 | I/Q a partire da offset configurabile: max 256 bool in, 256 bool out, 64 analogici in, 64 out; analogici come WORD (scala ±27648 come un modulo AI reale) o DWORD (REAL/DINT) | https://docs.factoryio.com/manual/drivers/s7-1200-1500/ |
| Limite I/O | lo staff conferma: 2000 punti I/O impossibili; i limiti sono per driver (circa 256/64) | https://community.factoryio.com/t/whats-the-maximum-number-of-i-o-points-2000-io-points/547 |
| Modbus client | sceglie se leggere da Inputs o Coils e da Input o Holding Registers; fattore di scala per mandare float come interi | https://docs.factoryio.com/manual/drivers/modbus-client/ |
| OPC UA client | solo `opc.tcp`; browse dei nodi con limite 256 e filtri "starts with"/"contains"; conversioni automatiche Bool/Int/Float; certificati e identità utente configurabili | https://docs.factoryio.com/manual/drivers/opc/ |
| UX di mappatura | **drag & drop** dei tag sui punti del driver; LED verde di connessione; "auto connect" | https://docs.factoryio.com/getting-started/controlling-with-a-plc/ |
| Export mappatura | esporta la mappatura in CSV, per Siemens in **XML importabile in TIA Portal** (tabella tag pronta) | https://docs.factoryio.com/manual/drivers/ |
| Export CSV per IEC | export dei tag con nomi resi identificatori IEC 61131-3 validi; log periodico dei tag in CSV | https://docs.factoryio.com/manual/console/ |
| Editor scena | drag dal palette, Alt+drag per duplicare, copia/incolla tra scene, selezione rettangolare, gruppi (Ctrl+G); **collisione "smart"**: i pezzi si posano solo in posizioni valide, se intersecano diventano rossi e vengono cancellati; rotazioni a 90° (sensori liberi sull'asse verticale) | https://docs.factoryio.com/manual/edit-and-run/ |
| Modalità Edit / Run | F5 commuta; in Run pausa, slow motion, fast forward (x2/x3/x4) | https://docs.factoryio.com/manual/edit-and-run/ , https://docs.factoryio.com/manual/tags/ |
| Interazione in Run | i pezzi interattivi si trascinano col mouse durante la simulazione | https://docs.factoryio.com/manual/edit-and-run/ |
| Fault injection | su un tag "agganciato": clic sinistro = sempre ON, destro = sempre OFF; sugli analogici ON satura il valore | https://docs.factoryio.com/manual/failure-injection/ |
| Pannello istruttore | "solo Run" (scena non modificabile), blocco dei pezzi esistenti (lo studente completa una scena parziale), nascondere i guasti, protezione con password | https://docs.factoryio.com/manual/options/ |
| SDK | (solo Ultimate) assembly .NET Standard 2.0 "Engine I/O" che legge/scrive tutti i punti I/O tramite **memory-mapped file**, indipendentemente dal driver scelto | https://docs.factoryio.com/manual/sdk/ ; [secondaria] README del repo GitHub realgamessoftware/factoryio-sdk via snippet |
| Web API | (Ultimate) server HTTP REST su porta 7410: elenco tag, filtri per nome con wildcard, tipo, input/output; per ogni tag valore, forzatura, openCircuit/shortCircuit (guasti) | https://docs.factoryio.com/manual/web-api/ |
| Console / auto.cfg | console di comandi e file `auto.cfg` all'avvio per configurare driver, licenza, fisica, camera | https://docs.factoryio.com/manual/console/ |
| Artwork | si può mettere il proprio logo su un cartellone nella scena | https://docs.factoryio.com/manual/setting-up-artwork/ |
| Soft-PLC propria | Control I/O: soft-PLC a blocchi funzione, indipendente dal marchio, pensata per i principianti | https://docs.factoryio.com/manual/drivers/control-io/ |

### Architettura del motore (documentata)
- Fisica a passo fisso: `physics.timestep` default **1/120 s**, `physics.max_substeps` default 8,
  `physics.solver_iterations` default 20; il manuale suggerisce 1/240 per più qualità
  [https://docs.factoryio.com/manual/console/].
- Lo slow motion è impostabile ma "può destabilizzare la fisica"; in fast forward i pezzi veloci possono
  sfuggire ai sensori e i timer del PLC si sfasano [https://docs.factoryio.com/manual/console/ ,
  https://docs.factoryio.com/manual/edit-and-run/].
- Motore grafico: [inferenza] Unity, perché il percorso di installazione contiene `Factory IO_Data\StreamingAssets`
  (struttura tipica di Unity) [https://docs.factoryio.com/manual/setting-up-artwork/]; di conseguenza
  fisica probabilmente PhysX. Non dichiarato.
- Frequenza di scambio I/O col PLC: **non documentata** nelle pagine lette.

### Punti di forza distintivi
- Tempo al primo risultato in minuti: scena pronta → driver → drag & drop dei tag → Run
  [https://docs.factoryio.com/getting-started/controlling-with-a-plc/].
- Editor "a prova di principiante": collisioni intelligenti, snap a 90°, nessun parametro fisico da toccare
  [https://docs.factoryio.com/manual/edit-and-run/].
- Sensori con comportamento realistico per materiale (induttivo vede solo metallo, ecc.)
  [https://docs.factoryio.com/manual/parts/sensors/].
- Analogici con la stessa scala dei moduli Siemens (±27648), così si usano NORM_X/SCALE_X come sul campo
  [https://docs.factoryio.com/manual/drivers/s7-1200-1500/].
- Export della tabella tag in XML per TIA: elimina la doppia digitazione [https://docs.factoryio.com/manual/drivers/].
- Guasti a un clic + modalità istruttore: esercizi di troubleshooting pronti [https://docs.factoryio.com/manual/options/].
- Prezzo basso e licenze floating per un'aula intera [https://factoryio.com/buy].

### Debolezze / critiche
- **Niente parti personalizzate / import CAD**: lo staff conferma nel 2025 che non si possono importare;
  prima arriverà l'assemblaggio di parti native, poi l'import [secondaria,
  https://community.factoryio.com/t/custom-3d-parts-lift-motor-control-any-updates-in-2025/4199].
- Attuatori semplificati: niente controllo di velocità analogico con segno, niente finecorsa reali
  (solo un segnale "in movimento"); il traslo-elevatore è "quasi inusabile" con un solo 0-10 V di posizione,
  secondo un utente; lo staff ammette il limite e promette controllo in velocità ±10 V
  [secondaria, stessa URL].
- Sviluppo lento: thread "Is this a dead product?" (2024-2025), lo staff parla di una riscrittura in corso
  [secondaria, https://community.factoryio.com/t/is-this-a-dead-product/3063].
- Limite di ~256 DI/DO e 64 AI/AO per driver; un PLC alla volta [https://docs.factoryio.com/manual/drivers/s7-1200-1500/ ,
  https://community.factoryio.com/t/whats-the-maximum-number-of-i-o-points-2000-io-points/547].
- Solo Windows; SDK e Web API solo nell'edizione più cara [https://docs.factoryio.com/manual/system-requirements/ ,
  https://docs.factoryio.com/manual/sdk/].
- PLCSIM classico richiede un progetto template proprietario [https://docs.factoryio.com/manual/drivers/s7-plcsim/].

### Cosa prendere per il nuovo simulatore [inferenza]
1. Flusso "scena → driver → drag & drop → Run" in tre schermate, con LED di connessione.
2. Scene-esercizio graduate per istruzione PLC (bobina, set/reset, timer, contatori, PID).
3. Collisione di posa con snap e feedback rosso; rotazioni a 90° come default.
4. Tag con tipo IEC esplicito, forzatura e guasto (stuck-on/off, saturazione) sullo stesso widget.
5. Tag di sistema leggibili dal PLC (running, reset, time scale) e comandi Run/Pause/Reset dal PLC.
6. Export della mappatura nel formato del tool del PLC (XML TIA, CSV per Studio 5000 / CODESYS).
7. Analogici in scala "di campo" (±27648) oltre a REAL.
8. Web API REST + IPC a memoria condivisa aperti a tutti, non solo all'edizione top.
9. Da evitare: limite fisso di I/O, attuatori senza finecorsa, assenza di import CAD.
10. Fast forward con avviso esplicito sul rischio di "tunneling" dei sensori (o ray-cast continuo per evitarlo).

---

## 2. Festo CIROS (CIROS Studio / CIROS Education)

### Cosa è e per chi
- CIROS = "Computer Integrated RObot Simulation": simulazione 3D cinematica in tempo reale, con librerie
  CP Lab / CP Factory (le linee didattiche Festo) e oltre 1000 robot [manuale CIROS CP v7.1 p. 11,
  https://ftp.festo.com/Public/DIDACTIC/FactoryViews/documentation/CIROS-CP_UserManual_EN_v7.1-220908.pdf].
- Lo sviluppatore è VEROSIM Solutions, che vende CIROS Studio anche all'industria come alternativa più
  economica a Process Simulate / RobotExpert / DELMIA [https://www.ciros-engineering.com/].
- Pubblico didattico Festo: scuole tecniche con linee CP Lab/CP Factory e MES4; pubblico industriale tramite
  VEROSIM [manuale p. 11-12; https://www.ciros-engineering.com/].
- Nome "CIROS Automation Suite": usato in materiale Festo più vecchio (circa 2010) per indicare la famiglia
  (es. CIROS Advanced Mechatronics); "Festo Automation Suite" è un prodotto diverso (commissioning di azionamenti)
  [secondaria, snippet di ricerca da manuali Scribd e drivesandcontrols.ca].

### Licenza / edizioni
| Edizione | Differenza | Fonte |
|---|---|---|
| CIROS Studio | versione completa: crea e salva modelli, interfaccia RCI ai controller Mitsubishi | manuale p. 12 |
| CIROS Education | apre, analizza e modifica modelli esistenti, ma **non salva**; niente RCI | manuale p. 12 |
| CIROS Starter | variante citata nel capitolo "Advanced" | manuale p. 230 |
| Licenza | dongle USB CodeMeter o server di rete CodeMeter | manuale p. 14, 20-24 |
| CIROS Studio (VEROSIM) | modulare: pacchetto base + import modelli, import formati proprietari (JT, NX, CATIA, Creo, SolidWorks...), export, meccanismi, ecc. | https://www.ciros-engineering.com/ |

- Prezzi: **non pubblici** (offerta tramite Festo Didactic / VEROSIM). Windows 10, GPU NVIDIA OpenGL 4.5,
  8 GB RAM [manuale p. 14].

### Funzionalità principali
| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Librerie modelli | moduli CP Lab/CP Factory con configurazione per proprietà; sorgenti e pozzi; aggiunta di librerie proprie | manuale cap. 4 (p. 60-90) |
| Libreria robot | > 1900 robot di 19 costruttori (versione VEROSIM) | https://www.ciros-engineering.com/ |
| Sensori | induttivi, capacitivi, distanza ottica, barriere, ultrasuoni, laser scanner di sicurezza con campi, camere, barcode, RFID; visualizzazione 3D del campo di misura | https://www.ciros-engineering.com/ ; manuale p. 108-109 |
| Trasporto | sistemi a binario/WPC, AGV, magazzini; accumulo realistico basato su inviluppi geometrici | https://www.ciros-engineering.com/ |
| Modello elettrico | I/O digitali e analogici, monitor delle connessioni I/O con drag & drop, override temporaneo, breakpoint, sigle BMK | https://www.ciros-engineering.com/ |
| PLCSIM Advanced | ogni PLC virtuale del modello si lega a **una** istanza PLCSIM Advanced (V3.0+), per nome; anche in remoto via Virtual Ethernet Adapter + porta del runtime manager | manuale p. 140-162 |
| UX di mappatura | pagina "Items": Connect, elenco dei tag TIA (tutti o solo "visibili in HMI"), **drag & drop** sui virtual I/O, anche più I/O consecutivi insieme; accesso anche a M, contatori, timer, DB | manuale p. 155-157 |
| Accoppiamento temporale | dopo il collegamento l'istanza PLCSIM gira solo quando la simulazione CIROS gira; un download TIA "si blocca" se CIROS è in pausa | manuale p. 160 |
| OPC UA | CIROS come client OPC UA: oggetto "OPC UA client", endpoint, mappatura nodi → I/O interni | manuale p. 172-173 |
| Altri PLC | S7-PLCSIM, CODESYS SoftPLC, PLC reali via EasyPort + EzOPC | manuale p. 27 ; [secondaria] Festo Product Guide via snippet |
| Controllori reali | Siemens, Beckhoff, 3S, anche su bus di campo reali (VEROSIM) | https://www.ciros-engineering.com/ |
| Robotica | Mitsubishi MELFA-BASIC, online teaching, upload/download programmi; KRL, RAPID; linguaggio IRL | manuale cap. 11 ; https://www.ciros-engineering.com/ |
| Collisioni | rilevamento collisioni e analisi raggiungibilità | manuale p. 32 ; https://www.ciros-engineering.com/ |
| Python | Python 3.6-3.9; modulo `Ciros` con 19 classi e 275 comandi (v7.1); script per creare il modello, come controllore, o come comando di menu; attivazione manuale, su trigger o come "Python controller" | manuale p. 164-170 |
| Generazione modello | import di uno script Python generato da Festo Factory View (moduli già "agganciati" e con ID MES) | manuale p. 120-121 |
| MES4 / Robotino | interfacce a MES4 e al fleet manager | manuale cap. 7 |
| Fault injection | percentuali di pezzi ruotati, difettosi, variazione di misura (parametri dei moduli); "fault simulation in teacher mode" nelle opzioni del modello | manuale p. 32, 78-80 |
| Trace codice | traccia riga per riga dei programmi PLC/robot eseguiti | manuale p. 107 ; https://www.ciros-engineering.com/ |
| Data logging | profili di logging di I/O, variabili, posizioni; grafici nel tempo ed export | manuale p. 110-113 |
| CAD import | STEP, IGES, VRML, STL, FBX, Collada, OBJ...; 19 strumenti di semplificazione geometrica | https://www.ciros-engineering.com/ ; manuale p. 233 |
| Analisi modello | trova errori di modellazione | manuale p. 231 |
| VR, video | occhiali VR, export immagini/video ad alta risoluzione | manuale cap. 12-13 |

### Architettura del motore
- Kernel **a tempo discreto**: incremento di default **40 ms** (25 aggiornamenti/s); il manuale raccomanda
  di non modificarlo [manuale p. 11, 96].
- Versione VEROSIM: passo selezionabile fino a 1 ms, simulazione **deterministica e ripetibile**, quante
  controller si vuole [https://www.ciros-engineering.com/].
- Rapporto tempo modello / tempo reale: real-time (default), saltare frame grafici, saltare attese, più veloce
  del reale (es. visualizzare ogni 5 incrementi = 5x), più lento per il debug [manuale p. 97-99].
- È soprattutto **cinematico**; la fisica è un'opzione ("Physik- und Prozesssimulation")
  [https://www.ciros-engineering.com/]. Il motore fisico non è nominato.
- Attenzione: i timestamp CIROS sono in tempo modello, quelli MES4 in tempo reale; se CIROS va più veloce le
  misure MES4 sono sbagliate [manuale p. 100].

### Punti di forza distintivi
- Gemello digitale 1:1 dell'hardware didattico Festo: lo stesso programma PLC gira sul virtuale e sul reale,
  e molti studenti programmano lo stesso modulo che la scuola possiede una sola volta [manuale p. 138].
- Catena completa PLC + robot + MES + AGV in un unico modello [manuale cap. 7-11].
- Import CAD serio e Python con un'API ampia [manuale p. 168; https://www.ciros-engineering.com/].

### Debolezze / critiche
- Curva d'apprendimento: il manuale stesso dice che l'help non è centrato su CP Lab e consiglia due monitor
  [manuale p. 14, 28].
- Fragilità d'installazione: Python deve stare nel PATH nella versione giusta, altrimenti errore
  `cirosPluginPython.dll not loaded`; la lingua si sceglie solo all'installazione [manuale p. 13, 164-167].
- Passo di 40 ms grossolano per la logica veloce [manuale p. 96] [inferenza: inadatto a camme/encoder].
- Education non salva i modelli [manuale p. 12]; prezzi non pubblici; Windows only.
- Critiche da forum: **non trovate** in questa ricerca.

### Cosa prendere per il nuovo simulatore [inferenza]
1. Binding a PLCSIM Advanced per nome di istanza, anche remoto, con sincronizzazione del tempo virtuale
   (il PLC avanza solo quando avanza la scena).
2. Elenco tag letto dal PLC (non digitato) e filtro "solo tag visibili HMI"; drag & drop multiplo.
3. Python embedded con API documentata e tre modi di esecuzione (manuale, trigger, controller).
4. Fault injection "di processo" (percentuale di pezzi difettosi/ruotati, rumore di misura), non solo stuck-at.
5. Trace del codice e data logging integrati; regole chiare tempo modello vs tempo reale.
6. Gerarchia "gemello del kit didattico": librerie che corrispondono a hardware reale vendibile.

---

## 3. Festo FluidSIM 6 (sviluppato da Art Systems Software GmbH)

### Cosa è e per chi
- Software per disegnare e **simulare schemi** elettropneumatici, elettroidraulici, digitali ed elettronici,
  con descrizioni dei componenti, foto, sezioni animate e video; per lezione, preparazione e autoapprendimento
  [https://www.art-systems.de/www/site/en/fluidsim/].
- Oltre 200 000 installazioni dal 1995; nato all'Università di Paderborn (FluidSIM-P) e Duisburg (FluidSIM-H)
  [https://www.art-systems.de/www/site/en/fluidsim/ ; manuale FluidSIM 6 p. 17-18,
  https://www.art-systems.de/www/files/fluidsim/downloads/v6/fluidsim_manual_enus.pdf].
- È 2D (schema), non una scena 3D di fabbrica. Ultimo aggiornamento: 6.3 del 17-07-2026
  [https://www.art-systems.de/].

### Licenza / edizioni
| Voce | Dettaglio | Fonte |
|---|---|---|
| Tecnologie a licenza separata | Pneumatica (P), Idraulica (H), Elettrotecnica (E, ora con automotive); combinabili in un unico schema | https://www.art-systems.de/www/site/en/fluidsim/ ; manuale p. 16-17 |
| Modalità | locale, di rete, uso a casa; gruppi, password, scadenze dal license manager | https://www.art-systems.de/www/site/en/fluidsim/ |
| FluidSIM 365 | abbonamento a tempo, web/Windows/Festo LX, trial 30 giorni | [secondaria] snippet di festo.com (pagina 403) |
| Vendita | esclusiva tramite Festo Didactic SE; prezzi non pubblicati sulle pagine lette | https://www.art-systems.de/ |

### Funzionalità principali
| Funzionalità | Cosa fa | Fonte URL |
|---|---|---|
| Editor CAD + simulazione | disegno conforme DIN e simulazione dinamica dallo stesso schema: nessuna separazione tra disegno e calcolo | manuale p. 17 |
| Libreria | centinaia di componenti pneumatici, idraulici, elettrici, con parametri di default "sensati" | https://www.art-systems.de/www/site/en/fluidsim/ |
| Interattività | in simulazione si azionano pulsanti e valvole a mano; segnali da hardware o da altri programmi | https://www.art-systems.de/www/site/en/fluidsim/ |
| Strumenti virtuali | multimetri/manometri che non perturbano il circuito, puntali su ogni connettore; oscilloscopio a 4 canali | https://www.art-systems.de/www/site/en/fluidsim/ ; manuale p. 240-243 |
| GRAFCET (EN 60848) | modalità disegno, osservazione e **controllo** (il GRAFCET pilota il circuito); componente "GRAFCET-PLC" | manuale cap. 9, p. 250-263 |
| OPC UA | server **e** client contemporaneamente; quasi ogni parametro è una variabile (le stesse del GRAFCET); in modalità server tutto è esposto automaticamente all'avvio della simulazione, in gerarchia progetto → componente → parametro | manuale p. 211-213 |
| OPC Classic / DDE | componenti dedicati; DDE disattivato di default per sicurezza; porta e formato NodeId configurabili; file con l'elenco dei nodi | manuale p. 208, 476-477 |
| EasyPort | I/O hardware Festo su seriale/USB: FluidSIM comanda una macchina reale o la sostituisce | manuale p. 206-208 |
| Guasti | modelli di guasto predefiniti per componente, raggruppati in "configurazioni di guasto"; lo studente trova il guasto con gli strumenti virtuali e lo ripara con lo strumento "repair"; password per nascondere i guasti; opzione per vietare la modifica del circuito | manuale cap. 12, p. 317-319 |
| Profili utente | profili che riducono libreria, parametri, menu (es. "Pneumatics basics" con soli 30 componenti del kit Festo) | manuale p. 527-528 |
| Didattica | descrizione di ogni componente con simbolo DIN, foto del pezzo reale, sezioni animate, film, presentazioni componibili | manuale cap. 24, p. 534-535 |
| Ambiente | temperatura ambiente, pressione atmosferica, tipo di fluido (densità, viscosità) | manuale p. 478-479 |

### Architettura del motore
- Simulazione basata su descrizioni fisiche dei componenti; linee pneumatiche/idrauliche ideali di default,
  con modelli di perdita opzionali [manuale p. 17, 475-476].
- Fattore di rallentamento/accelerazione (1:1 tempo reale, 1:10 slow motion, 1:0,1 dieci volte più veloce)
  [manuale p. 475].
- Segnali elettrici simulabili fino a 100 kHz [https://www.art-systems.de/www/site/en/fluidsim/]; cicli di
  simulazione fino a 10 kHz [secondaria, snippet di festo.com].
- Solver e passo di integrazione: non documentati nelle parti lette.

### Punti di forza distintivi
- Primo schema funzionante in pochi minuti grazie ai default dei parametri [https://www.art-systems.de/www/site/en/fluidsim/].
- Lo schema **è** il modello: niente modello separato da mantenere [manuale p. 17].
- Didattica integrata (foto, sezioni, film) a un clic dal componente [manuale cap. 24].
- Guasti "nascosti" e ricerca guasti con strumenti virtuali: esercizio realistico [manuale cap. 12].
- OPC UA "senza configurazione" lato server [manuale p. 212].

### Debolezze / critiche
- Solo schema 2D: non c'è cinematica 3D di macchina [inferenza dalla documentazione].
- I modelli di guasto sono fissi: l'utente non può crearne di nuovi [manuale p. 317].
- Problemi di certificati OPC UA da risolvere a mano spostando file tra cartelle PKI [manuale p. 211].
- Critiche da forum: **non cercate a fondo** (tempo); nessuna trovata.

### Cosa prendere per il nuovo simulatore [inferenza]
1. Esposizione OPC UA automatica di tutto il modello con gerarchia leggibile (scena → oggetto → proprietà).
2. Profili di complessità (principiante/esperto) che nascondono librerie e parametri.
3. Guasti come "configurazioni" con nome e password, più uno strumento di diagnosi e riparazione.
4. Strumenti di misura virtuali non intrusivi (sonda su qualunque segnale, oscilloscopio).
5. Un pannello pneumatico/elettrico 2D accoppiato alla scena 3D (cilindro nello schema = cilindro nella scena).
6. Scheda didattica per componente (foto reale, funzione, parametri).

---

## 4. Progetti open-source (e free) da conoscere

Stelle GitHub: **non visibili** (API GitHub bloccata in questa sessione). Versioni prese da PyPI o dal sito.

| Progetto | Licenza | Copre | Maturità | Riuso per il nuovo simulatore |
|---|---|---|---|---|
| **OpenPLC** (Autonomy Logic) | GPL-3.0 [da verificare, non letta] | runtime e editor PLC IEC 61131-3, Modbus | > 1 M download dichiarati [https://autonomylogic.com/] | soft-PLC di riferimento per test senza Siemens; controllore "bundled" alla Control I/O |
| **Gazebo** (gz-sim) | Apache-2.0 [noto, non riletto] | simulatore robotico 3D, fisica a plugin, sensori, ROS | release LTS "Jetty" supportata da set-2025 a mag-2031 [https://gazebosim.org/docs/latest/releases/] | architettura a plugin, formato SDF, trasporto messaggi; poco adatto a nastri/sensori PLC |
| **Webots** (Cyberbotics) | Apache-2.0 da dic-2018 [secondaria, https://en.wikipedia.org/wiki/Webots] | simulatore robotico desktop, controller in C/Python | R2025a l'ultima stabile vista [secondaria, https://formulae.brew.sh/cask/webots] | modello controller/mondo, deterministico, documentazione |
| **MuJoCo** (DeepMind) | Apache-2.0 [https://pypi.org/pypi/mujoco/json] | motore fisico a contatti, veloce e deterministico | 4.0.0 del 2026-11-06 [PyPI] | candidato motore fisico (nastri, pezzi, cinematiche), bindings Python, WASM |
| **Project Chrono** | BSD-3 [https://projectchrono.org/] | multibody, contatti, veicoli, granulari, FEA, co-simulazione | attivo (sito 2026); PyChrono su PyPI 1.1.0 del 2024-10 | alternativa a MuJoCo per meccanismi complessi |
| **Isaac Sim** (NVIDIA) | proprietaria NVIDIA sul pacchetto PyPI [https://pypi.org/pypi/isaacsim/json]; gratuita, **non open** | simulazione robotica fotorealistica su Omniverse/USD, PhysX | 6.2.0.0 del 2026-10-13 [PyPI] | idee (USD come formato scena), non riusabile come base libera |
| **JaamSim** | Apache-2.0 [https://jaamsim.com/] | simulazione a eventi discreti con GUI drag & drop e 3D | stabile; versione Pro commerciale [https://jaamsim.com/] | throughput/OEE della linea, layer DES sopra la scena |
| **SimPy** | MIT [https://pypi.org/pypi/simpy/json] | libreria Python per eventi discreti | 4.1.2 del 2026-05-24 [PyPI] | motore DES leggero, test di logica di flusso |
| **OpenModelica** | OSMC-PL (doppia con GPL/AGPL) [secondaria, https://github.com/OpenModelica/OpenModelica/blob/868b628a698d3de84ca2cfda0aba5f577f4ee144/OSMC-License.txt] | modelli fisici acausali (Modelica), pneumatica/idraulica/elettrico, export FMU | v1.27.1 del 2026-09-08 [https://openmodelica.org/] | equivalente libero di FluidSIM lato fisica; FMU in co-simulazione |
| **FreeCAD** | LGPL [da verificare] | CAD parametrico, STEP, assiemi | attivo [https://www.freecad.org/] | import/conversione STEP → mesh e cinematica; editor di parti custom |
| **ROS-Industrial** | per pacchetto (Apache/BSD) [da verificare] | driver robot industriali, Tesseract (pianificazione), ponte PLCnext-ROS | attivo, notizie giu-2026 [https://rosindustrial.org/] | driver robot e planner, se si includono robot |
| **python-snap7** | MIT [https://pypi.org/pypi/python-snap7/json] | client S7 (PUT/GET) | 3.3.1 del 2026-10-28 [PyPI] | driver S7 su CPU reale (non verso PLCSIM classico [inferenza]) |
| **pymodbus** | BSD-3 [https://pypi.org/pypi/pymodbus/json] | Modbus TCP/RTU client e server | 3.16.2 del 2026-11-03 [PyPI] | driver Modbus pronto |
| **asyncua** | LGPL-3.0+ [https://pypi.org/pypi/asyncua/json] | OPC UA client/server Python | 2.1.0 del 2026-10-06 [PyPI] | server OPC UA "auto-esposizione" alla FluidSIM |

---

## 5. Sintesi trasversale [inferenza]

- Ciò che rende amichevoli questi tool **non è la fisica**, ma: (a) scene/esercizi pronti, (b) mappatura
  I/O drag & drop con elenco tag importato, (c) guasti e modalità istruttore, (d) didattica a un clic.
- Il punto debole comune è la chiusura: niente CAD in Factory I/O, prezzi non pubblici e Windows-only per
  CIROS, guasti non estendibili in FluidSIM. Un progetto aperto può vincere su import CAD, API aperte e
  numero di I/O illimitato.
- Passi di simulazione documentati: Factory I/O 8,3 ms di fisica (1/120 s); CIROS 40 ms di default (fino a
  1 ms nella versione VEROSIM); FluidSIM non dichiarato.
- Per un utente S7: Factory I/O e CIROS usano PLCSIM Advanced per nome di istanza; Factory I/O supporta anche
  PLCSIM classico V13-21 tramite un progetto template.

## Cosa non è stato verificato

- **Frequenza di scambio I/O PLC↔simulatore** di Factory I/O e CIROS: non documentata nelle pagine lette.
- **Motore grafico/fisico di Factory I/O** (Unity/PhysX): è un'inferenza dal percorso `Factory IO_Data`.
- **Pagine festo.com** (FluidSIM, Product Guide CIROS): 403 Akamai, non aggirato. Il "ciclo fino a 10 kHz",
  FluidSIM 365 e la compatibilità CIROS con CODESYS/EasyPort/EzOPC vengono da snippet di ricerca o dal
  diagramma a p. 27 del manuale, non da una lettura completa.
- **Prezzi CIROS e FluidSIM**: non pubblici, non trovati.
- **Repo factoryio-sdk**: GitHub bloccato in questa sessione; il contenuto (memory-mapped file, .NET Standard
  2.0) viene da uno snippet del README.
- **Stelle GitHub, licenze di OpenPLC, Gazebo, FreeCAD, ROS-Industrial**: date a memoria o da fonti
  secondarie, marcate "[da verificare]". La licenza effettiva di Isaac Sim (pacchetto proprietario vs codice
  sorgente su GitHub) non è stata chiarita.
- **Webots**: non è confermato che R2025a sia ancora l'ultima release a novembre 2026.
- **Critiche da forum** su CIROS e FluidSIM: non trovate; quelle su Factory I/O vengono dal forum ufficiale
  (fonte secondaria, campione piccolo).
- Nessun software è stato installato o provato: tutto viene da documentazione, non da misure.
