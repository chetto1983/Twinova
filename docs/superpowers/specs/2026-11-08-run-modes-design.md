# Spec: run modes and snapshots (ROADMAP 3.5)

Date: 2026-11-08. Status: **implemented** (user, 2026-11-08: snapshots by replay; a lag beyond the tolerance is
reported and the run goes on).

ROADMAP 3.5:
- Run (wall clock × scale, `OutOfSync` beyond a tolerance);
- as fast as possible;
- Step;
- Reset;
- snapshots.

Acceptance: the lag is reported, never hidden; save and restore a state. Step, Reset and Pause exist since M1, and the
lag indicator too: the UI shows "Behind real time" above 100 ms (`isBehind`, `snapshot.ts`).

## Decisions

1. **Speed.**
   - Run paces virtual time at `scale × wall time`, with `scale` in {0.25, 0.5, 1, 2, 4, 10}.
   - "As fast as possible" runs scans with no pacing, in chunks per tick, and the snapshot is still published at
     about 30 Hz.
   - The speed is chosen in the header, next to Run, and applies from the next Run.
2. **The lag beyond the tolerance is reported, and the run goes on** (user, 2026-11-08).
   - The tolerance stays 100 ms of virtual time behind the scaled wall clock.
   - The UI already shows it. What changes is that the lag is measured against the scaled target, and that "as fast
     as possible" has no lag by definition.
   - nexosim's `OutOfSync` *error* is not used, because the run must not stop.
3. **Snapshots by replay** (user, 2026-11-08).
   - **Why replay.** truST at the pin can save only `RETAIN` variables (`retain_snapshot()`), not the VM's whole
     state: function block instances such as a running TON, and non-retained variables. nexosim's own `save` leaves
     `Model::Env` out (spec 2026-11-07-des-core-design.md, F).
   - **What a snapshot is.** The program source, the log of external inputs with their virtual time (Start, Stop,
     outlet blocked), and the virtual time of the snapshot.
   - **Restoring.** Build the program again, replay each input at its time, and advance to the snapshot's time. This
     is exact because the engine is deterministic, which the 3.2 replay test checks.
   - **The cost.** It grows with the snapshot's time, and is measured below.
   - **One slot per session.** A Save button and a Restore button in the header. Files are left for ROADMAP 6.2.

## Acceptance

- **Restore.** After a restore, every following scan equals the uninterrupted run (fingerprint test), with inputs
  before and after the snapshot.
- **Scale.** Over 5 s of wall time the virtual/wall ratio is within 10 % of the chosen scale, at 2 and at 0.5.
  Engine-level tests; the E2E checks ×1 as today.
- **Fast.** "As fast as possible" advances more than 10 s of virtual time per second of wall time on the default scene
  (in the release build).
- **Replay cost** of a 1 h snapshot, measured and recorded.
- E2E green. en/it texts for the new controls.

## Measured (2026-11-08)

- **Restore is exact.** Test `a_restored_state_continues_exactly_like_the_uninterrupted_run`: the state is saved after
  inputs at scans 0 and 700; inputs follow at 1500, 1900 and 2000. Every scan after the restore equals the
  uninterrupted run (fingerprint over 1 800 scans).
- **Scale.** `catch_up` at ×2 turns 500 ms of wall time into 1 000 ms of virtual time, and at ×0.5 turns 400 ms into
  200 ms. The E2E checks ×1 within 10 % and ×2 within 10 % on the real app (CI).
- **As fast as possible.** One 25 ms tick ran more than 25 ms of virtual time (command test, debug build).
- **Replay cost.** Restoring a snapshot taken at 600 s simulated (60 000 scans) took **4.5 s in a debug build**, about
  7.5 ms per simulated second, so about 27 s per simulated hour in debug.
  - The release build was not measured, to avoid a long local build (user's request): a release run is expected to be
    much faster, but that is a hypothesis.
  - Replay grows linearly with the snapshot's time. Snapshots of long runs therefore call for a faster path later,
    such as saving the VM's state if truST ever offers it.

## What this will not prove

- Snapshots across sessions or program edits: one in-memory slot, for the program it was taken with.
- Real-time behaviour beyond the report: the lag is shown, not prevented.
