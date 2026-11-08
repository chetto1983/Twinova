# Twinova Brand Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Install and confine VectorCraft as a Codex MCP server, then use it to create and integrate Twinova's production-ready industrial-twin logo.

**Architecture:** VectorCraft is a local design-time dependency installed from its official Windows release and registered as a headless stdio MCP server by absolute executable path. One editable VectorCraft document produces a transparent SVG master; Tauri generates platform icons from that master, and the Svelte shell displays an identical SVG copy.

**Tech Stack:** VectorCraft MCP, SVG, Tauri 2 icon generator, Svelte 5, Node.js verification script

**Spec:** `docs/superpowers/specs/2026-11-08-twinova-brand-design.md`

## Global Constraints

- Use graphite `#101214`, blue `#8AB4F8`, and green `#81C995`; do not introduce additional logo colors.
- Run VectorCraft headlessly and confine both automation roots to `D:\Repo\Twinova`.
- Keep VectorCraft design-time only: no application or CI runtime dependency.
- Use one geometry for `.vectorcraft`, SVG, the in-app header, and every generated platform icon.
- Preserve user changes and do not rewrite unrelated application styling or assets.

## Review Focus

- The MCP process must start from a fresh Codex session without relying on interactive-shell `PATH`; Task 1 verifies the absolute command.
- File confinement must reject paths outside `D:\Repo\Twinova`; Task 1 performs a representative rejected write.
- The logo must remain distinguishable at 16×16 and 32×32; Task 2 renders and visually checks both sizes.
- Transparent pixels must surround the master artwork, while ICO/ICNS/PNG outputs remain valid; Task 2 checks format signatures and alpha-bearing PNGs.
- Light and dark themes must display the same geometry with adequate contrast and no layout shift; Task 3 covers both theme snapshots.

---

### Task 1: Install and register the confined VectorCraft MCP server

**Files:**
- Modify outside repository: `C:\Users\chett\.codex\config.toml`
- Verify: VectorCraft installation directory selected by the official Windows installer

**Interfaces:**
- Consumes: official VectorCraft GitHub latest release metadata and signed Windows x64 installer
- Produces: Codex MCP server `vectorcraft` invoking an absolute `vectorcraft-cli.exe` path with headless repository confinement

- [ ] **Step 1: Resolve and install the official Windows x64 release**

Read the latest GitHub release metadata, download the `windows-x64.msi` asset to a temporary directory, verify its published SHA-256 when the release provides `SHA256SUMS.txt`, and install it for the current user. Record the resolved `vectorcraft-cli.exe` absolute path.

- [ ] **Step 2: Verify the installed CLI directly**

Run: `& '<absolute-path>\vectorcraft-cli.exe' --help`

Expected: exit 0 with an `mcp` subcommand.

- [ ] **Step 3: Register the stdio MCP server**

Run with `HOME=C:\Users\chett` and `CODEX_HOME=C:\Users\chett\.codex`:

```powershell
codex mcp add vectorcraft -- '<absolute-path>\vectorcraft-cli.exe' mcp --headless --automation-read-root 'D:\Repo\Twinova' --automation-write-root 'D:\Repo\Twinova'
```

- [ ] **Step 4: Verify registration and protocol startup**

Run: `codex mcp get vectorcraft`

Expected: enabled stdio server, absolute executable path, `mcp --headless`, and both roots exactly `D:\Repo\Twinova`.

Initialize the server over stdio and call `tools/list`.

Expected: protocol initialization succeeds and drawing, inspection, save, export, and preview tools are listed.

- [ ] **Step 5: Verify confinement**

Through MCP, create an in-memory document and attempt to save a disposable file inside the repository and another under the system temporary directory.

Expected: the repository write succeeds and is removed; the outside write returns an automation-path rejection and creates no file.

### Task 2: Create the editable logo and generate the platform assets

**Files:**
- Create: `app/src-tauri/icons/twinova-logo.vectorcraft`
- Modify: `app/src-tauri/icons/logo.svg`
- Modify: `app/src-tauri/icons/icon.png`
- Modify: `app/src-tauri/icons/icon.ico`
- Modify: `app/src-tauri/icons/icon.icns`
- Modify: `app/src-tauri/icons/32x32.png`
- Modify: `app/src-tauri/icons/64x64.png`
- Modify: `app/src-tauri/icons/128x128.png`
- Modify: `app/src-tauri/icons/128x128@2x.png`
- Modify: `app/src-tauri/icons/Square*.png`
- Modify: `app/src-tauri/icons/StoreLogo.png`
- Create: `scripts/verify_brand_assets.mjs`

**Interfaces:**
- Consumes: MCP server `vectorcraft`, approved visual construction, Aura palette
- Produces: editable master, canonical 1024×1024 transparent SVG, complete valid Tauri icon set, deterministic asset checks

- [ ] **Step 1: Write the failing asset verifier**

Implement assertions in `scripts/verify_brand_assets.mjs` for required filenames, PNG dimensions from IHDR, ICO and ICNS magic bytes, SVG `viewBox="0 0 1024 1024"`, allowed palette values, transparent canvas, and absence of text/fonts/embedded raster images.

- [ ] **Step 2: Run the verifier to confirm the provisional assets fail**

Run: `node scripts/verify_brand_assets.mjs`

Expected: FAIL because the approved geometry and editable `.vectorcraft` master are absent.

- [ ] **Step 3: Build the logo through VectorCraft MCP**

Create a 1024×1024 artboard and construct two equal-width, chamfered modular closed paths with deliberate blue/green over-under crossings. Name layers and objects semantically, save `twinova-logo.vectorcraft`, export `logo.svg`, and render previews at 1024, 32, and 16 pixels.

- [ ] **Step 4: Visually inspect the full and reduced previews**

Expected: the mark reads as two synchronized industrial paths; crossings are unambiguous; no segment collapses, muddies, or resembles a generic Venn diagram at 16×16 or 32×32.

- [ ] **Step 5: Generate all Tauri icons from the SVG master**

Run from `app/`: `npx tauri icon src-tauri/icons/logo.svg --output src-tauri/icons`

Expected: Tauri regenerates the existing PNG, ICO, ICNS, StoreLogo, and Square* filenames.

- [ ] **Step 6: Run the asset verifier**

Run: `node scripts/verify_brand_assets.mjs`

Expected: PASS with every required format and dimension validated.

- [ ] **Step 7: Commit the design assets**

```powershell
git add app/src-tauri/icons scripts/verify_brand_assets.mjs
git commit -m "feat: create Twinova industrial twin logo"
```

### Task 3: Integrate the mark in the Svelte shell and validate both themes

**Files:**
- Create: `app/src/assets/twinova-mark.svg`
- Modify: `app/src/App.svelte`
- Modify: `app/src/lib/snapshot.test.ts`
- Modify: `app/package.json`

**Interfaces:**
- Consumes: canonical SVG geometry from Task 2
- Produces: accessible decorative header mark and one `brand:check` command covering packaged and UI assets

- [ ] **Step 1: Add failing snapshot assertions for the branded header**

Assert that `App.svelte` uses `twinova-mark.svg`, supplies an empty alt attribute for the decorative mark, and no longer contains the two-circle inline placeholder.

- [ ] **Step 2: Run the focused test to verify it fails**

Run from `app/`: `npm test -- --run src/lib/snapshot.test.ts`

Expected: FAIL on the missing branded asset reference.

- [ ] **Step 3: Integrate the shared mark**

Copy the canonical mark-only SVG geometry to `app/src/assets/twinova-mark.svg`, import its URL in `App.svelte`, replace the inline circles with `<img src={twinovaMark} alt="" />`, and preserve the current 26×26 header footprint.

- [ ] **Step 4: Add the repeatable brand check**

Add `"brand:check": "node ../scripts/verify_brand_assets.mjs"` to `app/package.json` and extend the verifier to assert byte-equivalent normalized geometry between `logo.svg` and `twinova-mark.svg` while allowing platform-only metadata.

- [ ] **Step 5: Run focused and static checks**

Run from `app/`: `npm test -- --run src/lib/snapshot.test.ts`, `npm run brand:check`, `npm run check`, and `npm run lint`.

Expected: all commands exit 0 with no warnings.

- [ ] **Step 6: Validate light and dark presentation**

Run the application or browser preview, capture the header in both themes, and confirm the mark remains 26×26, aligned with the wordmark, fully visible, and unchanged between themes.

- [ ] **Step 7: Run the complete relevant verification**

Run from `app/`: `npm test` and `npm run build`.

Expected: all tests pass and the production build completes.

- [ ] **Step 8: Commit the integration**

```powershell
git add app/src/assets/twinova-mark.svg app/src/App.svelte app/src/lib/snapshot.test.ts app/package.json app/package-lock.json
git commit -m "feat: integrate Twinova brand mark"
```

### Task 4: Refresh the project README around the new brand

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: final logo asset from Task 2, verified product facts and commands from the repository
- Produces: an Aura-style project landing page with a centered brand hero, badges, navigation, product explanation, capabilities, architecture, quick start, documentation, development, and license sections

- [ ] **Step 1: Capture a failing README contract**

Extend `scripts/verify_brand_assets.mjs` to require the repository-relative logo, concise positioning, CI and license badges, anchor navigation, and links to the roadmap, research index, changelog, and engineering instructions.

- [ ] **Step 2: Run the verifier to confirm the current README fails**

Run: `node scripts/verify_brand_assets.mjs`

Expected: FAIL on the missing centered brand hero and navigation contract.

- [ ] **Step 3: Rewrite the README using Aura's information hierarchy**

Keep Twinova's claims at early-prototype maturity while adding the centered logo/title/tagline, factual badges, anchor navigation, `What is Twinova?`, current capabilities, one-screen architecture, technical stack/status, quick start, documentation, development checks, roadmap, and license.

- [ ] **Step 4: Verify README links and brand contract**

Run: `node scripts/verify_brand_assets.mjs`

Expected: PASS with every repository-relative README link resolving.

- [ ] **Step 5: Commit the README refresh**

```powershell
git add README.md scripts/verify_brand_assets.mjs docs/superpowers/plans/2026-11-08-twinova-brand.md
git commit -m "docs: refresh Twinova project overview"
```

