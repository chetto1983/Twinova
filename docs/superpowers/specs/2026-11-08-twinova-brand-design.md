# Twinova Brand Design

## Intent

Replace Twinova's provisional linked-rings mark with a distinctive industrial-digital-twin identity, and install VectorCraft as a reusable Codex MCP tool so the source artwork remains editable.

## Goals

- Preserve the existing Aura palette: graphite `#101214`, blue `#8AB4F8`, and green `#81C995`.
- Express two synchronized representations of one production system through two interlocking modular paths.
- Suggest conveyor motion and 3D industrial layout without drawing a literal factory or using fine detail.
- Remain recognizable at 16, 24, and 32 pixels, in both light and dark application themes.
- Keep an editable `.vectorcraft` master, a clean SVG master, and the complete Tauri icon set.
- Use the same mark in the window header and packaged desktop application.
- Register VectorCraft's headless MCP server with read/write access limited to `D:\Repo\Twinova`.

## Non-goals

- Redesign the Twinova wordmark, typography, or wider Aura visual system.
- Add a slogan to the icon.
- Introduce gradients, shadows, photorealism, gears, circuit traces, or small decorative details.
- Depend on VectorCraft at application runtime or during normal CI builds.

## Anti-goals

- A generic chain-link, infinity, or Venn-diagram symbol.
- A mark whose meaning or silhouette disappears when reduced to a Windows taskbar icon.
- Different geometry between the in-app mark and packaged icons.
- Granting the MCP server access beyond the Twinova repository.

## Constraints

- VectorCraft is installed from its official signed Windows release; its CLI is registered by absolute path.
- The MCP command is `vectorcraft-cli.exe mcp --headless` with matching `--automation-read-root` and `--automation-write-root` values of `D:\Repo\Twinova`.
- The master icon is a square 1024×1024 composition with a transparent outer canvas; platform masks provide final corner treatment.
- The symbol uses flat fills and must still read when converted to a single-color silhouette.
- Generated platform assets remain under `app/src-tauri/icons/` using Tauri's existing filenames.

## Approaches

### Selected: interlocking industrial twins

Two offset, modular paths form a compact paired-loop silhouette. Their overlaps communicate synchronization; clipped or chamfered segments evoke conveyor modules and the existing 3D editor. This preserves the useful “twin” idea while making it specific to industrial simulation.

### Considered: geometric TN monogram

PLC-like traces can form `T` and `N`, but the result depends on reading letters and becomes less international and less legible at small sizes.

### Considered: isometric factory cell

A miniature conveyor or factory cell explains the product directly, but requires too much detail for a durable application icon.

## Visual construction

- Center the mark optically inside a 1024×1024 artboard with generous safe area.
- Build two equal-width closed modular paths, offset horizontally, with chamfered transitions rather than perfect circles.
- Use blue for the left/front twin and green for the right/rear twin, with deliberate over/under crossings that remain clear without outlines.
- Use graphite only for optional preview backgrounds; exported SVG artwork remains transparent.
- Keep `Twinova` as live UI text in Fraunces; the new asset is the mark, not a baked wordmark.

## Deliverables

- Editable source: `app/src-tauri/icons/twinova-logo.vectorcraft`.
- Vector master: `app/src-tauri/icons/logo.svg`.
- UI asset: `app/src/assets/twinova-mark.svg` generated from the same geometry.
- Existing PNG, ICO, ICNS, StoreLogo, and Square* files regenerated through the Tauri icon command.
- A repeatable asset verifier that checks required files, dimensions/signatures, SVG palette, and UI usage.

