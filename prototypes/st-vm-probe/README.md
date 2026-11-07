# st-vm-probe

Probe for `docs/research/10-st-senza-compilazione.md`: run IEC 61131-3 Structured Text on truST's former bytecode
VM (trust-platform `6405915973a7`, the last commit read before the engine was deleted upstream on 2026-10-24),
driven in lockstep with virtual time.

```sh
cargo run --release                                      # native: behaviour check + benchmark
cargo build --release --target wasm32-unknown-unknown    # fails today, see below
```

`src/conveyor.st` is a conveyor function block with start/stop, a part counter on the end sensor, and a latched
2 s jam alarm (`TON`).

Native result (2026-11-07, 4-core cloud container):
- ST → bytecode in process: 6.1–6.6 ms;
- behaviour correct;
- 137 421 scans/s, which is 7.28 µs per scan.

On `wasm32-unknown-unknown` it does not build: `trust-runtime`'s harness is only behind its `full` feature, which
pulls tokio and mio (zenoh, rumqttc, mdns-sd).
