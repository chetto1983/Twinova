# Verification: the component model (ROADMAP 4.1, 2026-11-08)

Spec: `2026-11-08-component-model-design.md`, approved by the user on 2026-11-08 after asking for every open
question to be covered first. Acceptance of 4.1 is "spec approved"; nothing of the model is implemented yet.

## Decisions taken by the user

| Question | Decision |
|---|---|
| Who commands a diverter or turntable in v1 | the PLC only, by signal; a component-internal routing rule is later work |
| How a component signal reaches the ST program | an IEC direct address from the start (`%IX`, `%QX`, `%ID`, …) |
| The document's up axis | Y up, as glTF and three.js |

## What was measured

Three throwaway integration tests against truST `6405915`, through the `TestHarness` Twinova uses, in worktrees
outside the repository. The full results are in the spec's table, under "Binding to the controller".

| Area | Result |
|---|---|
| Sizes and types at an address | BOOL, BYTE/SINT, WORD/INT/UINT, DWORD/DINT/REAL, LWORD/LINT/LREAL, ARRAY, STRUCT, `STRING[n]`, `%M` all work; the image is bit-string typed and little-endian |
| Several instances of one component | an address fixed inside a function block is shared by every instance; `AT %I*` with `VAR_CONFIG P1.d1.x AT %IX2.0` gives each instance its own; an unmapped wildcard fails the build |
| Per-instance parameters | `VAR_CONFIG` initial values for REAL, DINT and TIME reach each instance; a wrong type (`E201`) or an unknown name (`E202`) fails the build |
| What truST leaves unchecked | overlapping addresses, two writers on one output bit, a type whose size does not match its address; Twinova validates these |
| A write by name to an `AT` input | accepted, then overwritten by the next input sampling |

The first run of the task-scheduled case read as a failed mapping because the task's interval had not elapsed.
It was repeated with `advance_time`, as Twinova's scan does, and passed.

## What this does not prove

- **Nothing of the format, the editor or a multi-component run exists.** Those are 4.2–4.5, each with its own
  acceptance.
- **The probes the spec assigns to later phases:**
  - product spanning across two conveyors and same-time ordering with several components (4.3);
  - the atomic multi-file save (4.2);
  - catalogue scale, performance and ergonomics (4.4–4.5).
- **`VAR_CONFIG` into nested function blocks or arrays of instances, and the build time of a large generated
  configuration.**
- **Any external I/O driver.** truST's Modbus, MQTT and EtherCAT drivers are an M5 inventory item.
