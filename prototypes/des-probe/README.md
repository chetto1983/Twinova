# des-probe

Spike for ROADMAP 3.1: nexosim 1.0 (MIT OR Apache-2.0) against a plain event queue ordered by
`(time in ns, priority, insertion sequence)` (research note 05 §6). Results and decision:
`docs/superpowers/specs/2026-11-07-des-core-design.md`.

```sh
cargo run --release                                     # A determinism, B throughput, D save/restore
cargo build --release --target wasm32-unknown-unknown   # C: builds (running in wasm is not measured)
```

- `src/own.rs`: the plain queue, about 40 lines.
- `src/nexo.rs`: the same periodic emitters as nexosim models, following its documentation (lib.rs) and
  `examples/stepper_motor.rs`, `examples/bench_save_restore.rs`.
