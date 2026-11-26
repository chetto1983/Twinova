# Plan: U7, visual rules (ROADMAP M4b)

## Goal

Text a person can read (13 px body, never below 11 px), controls a person can hit (24 px at least, 28 px in the bars),
disabled controls that say why, and the light theme by default.

## Architecture

- **Type:** four type tokens in `app.css`, used by every `font-size` in the Svelte components and the stylesheet. A
  check script keeps it so in CI.
- **Targets:** CSS of the small controls only.
- **Disabled controls:** `title` attributes in `Transport` and `Toolbar`; their `aria-label` stays, since the E2E
  and screen readers name the controls by it.
- **Light default:** one line in `App.svelte`.

## Tech Stack

Svelte 5 CSS, Paraglide messages, Monaco's options, bash check in CI, WebdriverIO E2E under Xvfb.

## Spec

`docs/superpowers/specs/2026-11-26-ux-consolidation-design.md`: decision 8 (type, targets, disabled controls), answer 3
(light theme), acceptance A1, phase U7. Targets as amended on 2026-11-26.

## Global Constraints

- No file above 600 lines.
- Gate 2 for TypeScript and Svelte: check, lint, format, i18n, knip, jscpd, Vitest, build, and the real app's E2E.
- No colour or size outside the tokens.
- `aria-label`s do not change.

## Review Focus

- **Layouts that the larger type overflows:** the toolbar at 1366 px, the tree's rows, the Watch table, the
  Inspector's fields, the Ladder drawing's labels. Pictures before and after at 1440 × 900 and 1366 × 768.
- **The light theme** in every pane, the Monaco editor and the Ladder drawing included.

## Decisions taken while writing this plan

1. **Type tokens.**

   | Token | Size | Use |
   |---|---|---|
   | `--text-xs` | 11 px | badges, units, meta |
   | `--text-sm` | 12 px | secondary text |
   | `--text-md` | 13 px | body, inputs, rows, values, code |
   | `--text-lg` | 15 px | headings |
   | `--text-xl` | 17 px | the brand |

   Today's sizes map as:
   - 8, 9 and 10 px → `xs`;
   - 11 px → `sm`;
   - 12, 12.5 and 13 px → `md`;
   - 17 px → `xl`.

   The project menu's `0.85em` becomes `xs`. Monaco's `fontSize` 12 → 13.
2. **A1 check:** `scripts/check-font-size.sh` fails on any `font-size` or `font` shorthand with a size that is not a
   `--text-*` token. It runs in CI beside the file-size check.
3. **Targets:**
   - the tree's row actions, the table's delete icons, Watch's force buttons, the debug bar's buttons, the snap gap
     field and the Ladder toggles all get a 24 px minimum;
   - the bars' buttons are 28 px already (`.icon-button`).
4. **Disabled controls say why**, in their tooltip:
   - Build: open a project first;
   - Run, Reset, Step, Save state, Restore state: build the project first;
   - speed: stop the line to change it;
   - Undo, Redo: nothing to undo or redo.
5. **Light theme by default:** a saved choice is kept.
6. **The scene's camera and the warehouse** are out of U7: spec amendment, now U3.

## File structure

| File | Change |
|---|---|
| `app/src/app.css` | tokens; body 13 px; global controls |
| `app/src/**/*.svelte` | every `font-size` on a token; small controls' minimum size |
| `app/src/lib/StEditor.svelte` | Monaco 13 px |
| `app/src/lib/Transport.svelte`, `Toolbar.svelte` | disabled reasons |
| `app/src/App.svelte` | light default |
| `app/messages/en.json`, `it.json` | reasons |
| `scripts/check-font-size.sh`, `.github/workflows/ci.yml` | A1 |
| `app/e2e/sizes.mjs` | the computed sizes on the real app, every dock tab |

## Tasks

### Task 1: type tokens everywhere, A1 in CI
- [ ] `scripts/check-font-size.sh` (fails today: run it).
- [ ] Tokens in `app.css`; every declaration mapped; Monaco 13.
- [ ] The script passes; check, lint, format, build.
- [ ] Commit `feat(ui): one type scale, 13 px body, nothing below 11 px`.

### Task 2: targets
- [ ] E2E `sizes.mjs`:
  - in each dock tab of the demo, no visible text renders below 11 px;
  - no visible button, select or input is under 24 × 24 px, Monaco excluded;
  - the toolbar's controls are at least 28 px high.
- [ ] CSS of the small controls.
- [ ] Commit `feat(ui): every control at least 24 px`.

### Task 3: disabled controls say why
- [ ] Titles and messages; E2E: on a project not yet built, Run's title is the reason.
- [ ] Commit `feat(ui): a disabled control says why`.

### Task 4: light theme by default
- [ ] `App.svelte`; E2E `mcp.mjs`'s theme check still passes.
- [ ] Commit `feat(ui): the light theme by default`.

### Task 5: acceptance
- [ ] Pictures before (dark, v0.4.0 audit) and after (light) at 1440 × 900 and 1366 × 768.
- [ ] E2E suites touched.
- [ ] Verification file, CHANGELOG, ROADMAP.
