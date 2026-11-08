# M4 discovery and live OIP study — 2026-11-08

## Delivered

- [Live behaviour report](../../research/13-oip-app-comportamenti-live.md), separating exercised behaviour,
  exposed controls/tooltips and unverified cases.
- [M4.1 draft](../specs/2026-11-08-component-model-design.md) and
  [M4.2 planning draft](../plans/2026-11-08-component-format.md).
- Local evidence at `D:\tmp\Twinova-references\oip-live-2026-11-08\index.html`:
  67 PNGs (30 flow captures, 37 catalogue captures), 37 family DOM inventories, additional session snapshots,
  exported `.oip` project and a browsable gallery.
- [SHA-256 manifest](2026-11-08-oip-live-artifacts.sha256) records 129 local evidence files.

The external images and project export stay under the user-designated reference directory. They are research
material, not Twinova's own artwork. The repository contains the report, design/planning drafts and hash manifest.

## Observed build and verification

Opened `https://www.openindustryproject.org/app/` in Playwright MCP, observed build `0572868 · 2026-11-07`.
Used a dedicated local project. No shared room was published; no external PLC or OPC UA connection was opened.

Completed checks:

- Insert/rename/configure a belt; History rollback restores width; duplicate then Undo removes the duplicate.
- Open the built-in palletising demo; Start advances its visible case counter; Pause enables Step, Step invoked,
  speed control switched 1×→2×, Stop invoked. No claim of precise wall-clock scaling from this probe.
- Export `.oip`, import as a new project, verify belt name/length/width; Save and page reload preserve the project.
- Inject ST syntax error, observe line-specific Problems entry and navigate to the line; repair the program.
- Bind belt Run Tag to `Motor`; `Motor := Start` follows Start in Watch; Force holds Motor TRUE when Start is FALSE;
  Release returns Motor to FALSE. Both binding and forced/effective values are visible.
- Create a named Ladder program and add a second rung. Timer/math palettes inspected, not fully executed.
- Open each of 37 family editors; 36 snapshots include part properties, AGV captures its node hierarchy/body inspector.
- Inspect Settings/Keyboard, library metadata, command palette (89 entries in captured state), Share options,
  desktop-only PLC boundary and context menu. Enter/exit distraction-free and pilot modes.

Evidence checks: all 37 inventory entries resolve to PNG and YAML files; all gallery-relative targets exist;
all 67 PNG signatures are valid. Representative screenshots for each main flow were opened and visually checked.
Raw capture 11 is loading and excluded from accepted scene evidence; use 12/13. Misleading intermediate filenames
04, 17, 19 and 20 are explained in the report and excluded from the gallery's main walkthrough.

`git diff --check` and relative-document-link checks pass. No application code or dependency was changed, so
no new Twinova runtime test result is claimed. Documentation-only paths do not trigger CI under the current workflow.

## Builder review

A read-only reviewer identified three planning ambiguities. Resolutions are recorded as proposals in the spec:

1. 4.2 acceptance is native open/display/save; arbitrary multi-component runtime belongs to 4.3–4.5.
2. Initial parameterisation uses versioned native family generators, avoiding an unmeasured new expression language.
3. Reusable templates expand into independent groups with fresh IDs; no automatic live inheritance.

These resolutions keep the first implementation bounded. They are not evidence that a serializer or editor has
been implemented. ROADMAP M4 is now in progress; M4.1 approval and later functional acceptance remain open.

## What this does not prove

The user requested broad behaviour capture; finite UI exploration cannot validate every composition, every
physical interaction or every program. The report explicitly leaves unverified: full automatic snap accuracy,
all sensor modes, all robot/AGV motions, every Ladder block, external PLCs/OPC UA, multi-user collaboration,
crash recovery and production performance. The live demo's displayed CPU/scan values are observations, not an
independently controlled performance benchmark.

No screenshots or code from OIP were introduced into the Twinova application. No design draft is marked approved
on the basis of this exploration alone.
