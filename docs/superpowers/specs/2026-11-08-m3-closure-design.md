# M3 closure: execution runtime and nexosim upstream

Date: 2026-11-08. Scope: close the two decisions left after phases 3.1–3.5.

## Runtime decision

Retain `trust-runtime` with `default-features = false`, feature `full`, at
`6405915973a77587748ffb061a61d37bb8a368b0` as the M3 execution baseline. This makes the
previous provisional decision explicit for the completed deterministic engine. No dependency revision changes.

Evidence: the runtime-footprint study measured at least 45,700 coupled lines for option A′ (extracting
lowering, encoder and runtime), with little expected binary-size benefit and roughly one third of cold Linux
build work in unused dependencies. The current baseline has since demonstrated virtual-time scans, task-derived
periods, replay snapshots, 548× real-time throughput and native Windows/Linux E2E. There is no measured
runtime defect requiring either replacement to satisfy M3.

Options considered:
- Keep the tested pinned runtime: chosen. Retains cold-build cost and maintenance responsibility for the frozen VM.
- A′, extract a reduced runtime: not selected; the measured coupling does not justify that maintenance work now.
- B, write an interpreter on `trust-hir`: not selected; no implementation or comparative measurement supports it.
  This decision does not assert that B would be slower, larger, or less capable.

Reopen the execution-runtime decision only when a reproducible unsupported IEC construct, a runtime defect,
an editor/runtime compatibility failure, or a measured resource-budget failure requires it. Any replacement
must first match reference ST behaviour, virtual time and deterministic replay and report size, build cost,
throughput and memory against this baseline. The M4.7 frontend upgrade still requires a separate compatibility
review; it does not automatically move the execution pin.

## Upstream contribution

The existing FIFO executor commit `480e72471ec3f9f64833bfbb925a1d67d3711ce6` is submitted in
[asynchronics/nexosim#189](https://github.com/asynchronics/nexosim/pull/189). The branch is one commit ahead
of upstream and zero behind at submission, changing four files (+74/-6), including its regression test.

M3's contribution obligation is submission of the tested change, not a promise of upstream acceptance.
Keep the exact fork pin until an upstream revision containing equivalent behaviour passes Twinova's ordering,
replay and native E2E checks. If upstream declines the guarantee, keep the pinned patch and document its maintenance.
Do not describe an open PR as merged or a published crate as containing the patch.

## Limits and handoff

M3 covers one conveyor and the embedded ST controller. Physical PLC connectivity remains M5. Persistent
snapshots across sessions/program edits are not implemented. Multi-component throughput and the observed
webview memory growth belong to the M4 stress scenario. These are explicit follow-ups, not new M3 deliveries.
