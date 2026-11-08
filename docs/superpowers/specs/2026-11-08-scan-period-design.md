# Spec: the scan period comes from the program (ROADMAP 3.4)

Date: 2026-11-08. Status: **implemented**; the user asked to go on with ROADMAP 3.4.

ROADMAP 3.4 asks for the PLC scan as a periodic event, with the scan time per program; acceptance: "same results as
M1 at 10 ms; scan time configurable". The scan has been a periodic nexosim event since 3.2. What was missing is a
period other than M1's fixed 10 ms.

## The decision

The period is declared where a PLC programmer declares it, in the program, with IEC 61131-3 tasks:

```
CONFIGURATION Cell
  RESOURCE CPU ON PLC
    TASK Fast(INTERVAL := T#5ms, PRIORITY := 1);
    PROGRAM P WITH Fast : Main;
  END_RESOURCE
END_CONFIGURATION
```

- **The scan period.** The bench scans at the greatest common divisor of the intervals of the periodic tasks, so that
  every task falls on a scan. At each scan, truST runs only the tasks whose interval has elapsed.
- **The default.** A program without a periodic task, such as the default program, is scanned every 10 ms, as in M1.
- **Event tasks are refused** at build, with a message: these are tasks started by `SINGLE`. The bench has no way to
  trigger them yet.
- **Time in nanoseconds.** The bench and `Engine` keep time in ns, so a period such as `T#500us` is exact. The snapshot
  still reports milliseconds to the UI.

## Measured on the pinned truST (2026-11-08)

The probe was a throwaway test against `trust-runtime` at `6405915`: two programs, `Fast` at `T#5ms` and `Lazy` at
`T#20ms`, with `advance_time(5 ms)` and `cycle()` eight times.

| Question | Result |
|---|---|
| The tasks of a program | `runtime().tasks()` gives each task's name, interval and programs: `Fast` 5 ms `[P]`, `Lazy` 20 ms `[Q]` |
| Contract variables with a configuration | still read and written by their plain names (`motor`, `sensor`, `start`) |
| Which tasks a cycle runs | only the due ones: `Fast` at every 5 ms step, `Lazy` at 20 ms and 40 ms |
| The default program | `tasks()` is empty |

## Acceptance

- **Same results as M1 at 10 ms.** The pinned per-scan fingerprints of the default program are unchanged.
- **Scan time configurable.** Tests in `plc.rs` and in the engine:
  - `T#5ms` gives 5 ms;
  - `T#4ms` with `T#6ms` gives 2 ms;
  - `SINGLE` is refused;
  - no configuration gives 10 ms;
  - with `T#5ms`, each Step advances 5 ms and the program runs at every step.

## What this does not prove

- That two tasks with different priorities preempt each other as on a CPU. The hypothesis, not measured, is that truST
  runs the due tasks in one cycle, in priority order (its `TaskConfig` documents "lower values run before higher
  values"), with no preemption inside a scan.
- Event (`SINGLE`) tasks and their timing.
- That the UI shows the period: it is not displayed yet.
