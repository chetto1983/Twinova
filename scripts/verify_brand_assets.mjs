import assert from "node:assert/strict";
import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const iconsDir = join(repoRoot, "app", "src-tauri", "icons");

const pngDimensions = new Map([
  ["32x32.png", 32],
  ["64x64.png", 64],
  ["128x128.png", 128],
  ["128x128@2x.png", 256],
  ["icon.png", 512],
  ["StoreLogo.png", 50],
  ["Square30x30Logo.png", 30],
  ["Square44x44Logo.png", 44],
  ["Square71x71Logo.png", 71],
  ["Square89x89Logo.png", 89],
  ["Square107x107Logo.png", 107],
  ["Square142x142Logo.png", 142],
  ["Square150x150Logo.png", 150],
  ["Square284x284Logo.png", 284],
  ["Square310x310Logo.png", 310],
]);

function required(path) {
  assert.ok(existsSync(path), `missing required brand asset: ${path}`);
  return path;
}

function readPng(path, expectedSize) {
  const bytes = readFileSync(required(path));
  assert.equal(
    bytes.subarray(0, 8).toString("hex"),
    "89504e470d0a1a0a",
    `${path} is not a PNG`,
  );
  assert.equal(
    bytes.subarray(12, 16).toString("ascii"),
    "IHDR",
    `${path} has no IHDR header`,
  );
  assert.equal(bytes.readUInt32BE(16), expectedSize, `${path} width`);
  assert.equal(bytes.readUInt32BE(20), expectedSize, `${path} height`);
  assert.ok(
    [4, 6].includes(bytes[25]),
    `${path} must preserve an alpha channel`,
  );
}

for (const [filename, size] of pngDimensions) {
  readPng(join(iconsDir, filename), size);
}

const ico = readFileSync(required(join(iconsDir, "icon.ico")));
assert.equal(
  ico.subarray(0, 4).toString("hex"),
  "00000100",
  "icon.ico has an invalid ICO header",
);

const icns = readFileSync(required(join(iconsDir, "icon.icns")));
assert.equal(
  icns.subarray(0, 4).toString("ascii"),
  "icns",
  "icon.icns has an invalid ICNS header",
);

const vectorcraftPath = required(join(iconsDir, "twinova-logo.vectorcraft"));
assert.ok(
  statSync(vectorcraftPath).size > 500,
  "editable VectorCraft master is unexpectedly small",
);

const svg = readFileSync(required(join(iconsDir, "logo.svg")), "utf8");
assert.match(
  svg,
  /viewBox=["']0 0 1024 1024["']/,
  "logo.svg must use the 1024×1024 master artboard",
);
assert.doesNotMatch(
  svg,
  /<(?:text|image|foreignObject)\b/i,
  "logo.svg must contain vector paths only",
);
assert.doesNotMatch(
  svg,
  /(?:font-family|data:image|background(?:-color)?\s*:)/i,
  "logo.svg must not embed fonts, rasters, or a background",
);
assert.doesNotMatch(
  svg,
  /<rect\b[^>]*(?:width=["']1024["']|width=["']100%["'])/i,
  "logo.svg canvas must stay transparent",
);

const colors = new Set(
  [...svg.matchAll(/#[0-9a-f]{6}/gi)].map(([color]) => color.toUpperCase()),
);
const allowedColors = new Set(["#8AB4F8", "#81C995"]);
assert.ok(colors.has("#8AB4F8"), "logo.svg is missing Aura blue");
assert.ok(colors.has("#81C995"), "logo.svg is missing Aura green");
assert.deepEqual(
  [...colors].filter((color) => !allowedColors.has(color)),
  [],
  "logo.svg uses colors outside the approved palette",
);

function normalizedGeometry(source) {
  const geometryAttributes = [
    "points",
    "x1",
    "y1",
    "x2",
    "y2",
    "fill",
    "stroke",
    "stroke-width",
    "stroke-linejoin",
    "stroke-linecap",
  ];
  return [...source.matchAll(/<(polygon|line)\b([^>]*)\/?\s*>/gi)].map(
    ([, tag, rawAttributes]) => {
      const attributes = Object.fromEntries(
        geometryAttributes.flatMap((name) => {
          const match = rawAttributes.match(
            new RegExp(`\\b${name}=["']([^"']*)["']`, "i"),
          );
          return match
            ? [[name, name === "stroke" ? match[1].toLowerCase() : match[1]]]
            : [];
        }),
      );
      return { tag: tag.toLowerCase(), attributes };
    },
  );
}

const uiSvg = readFileSync(
  required(join(repoRoot, "app", "src", "assets", "twinova-mark.svg")),
  "utf8",
);
assert.deepEqual(
  normalizedGeometry(uiSvg),
  normalizedGeometry(svg),
  "packaged and in-app logos must use identical geometry and paint",
);

const readme = readFileSync(required(join(repoRoot, "README.md")), "utf8");
assert.match(
  readme,
  /<div align=["']center["']>[\s\S]*<img src=["']app\/src-tauri\/icons\/logo\.svg["'][^>]*alt=["']Twinova logo["']/i,
  "README must open with a centered Twinova brand hero",
);
assert.match(
  readme,
  /open-source 3D factory simulator and virtual-commissioning tool/i,
  "README must state Twinova's concise product position",
);
assert.match(
  readme,
  /actions\/workflows\/ci\.yml\/badge\.svg/,
  "README must show the CI badge",
);
assert.match(
  readme,
  /shields\.io\/badge\/License-/i,
  "README must show the license badge",
);
for (const anchor of [
  "what-is-twinova",
  "current-prototype",
  "architecture",
  "quick-start",
  "documentation",
  "development",
]) {
  assert.ok(
    readme.includes(`](#${anchor})`),
    `README navigation is missing #${anchor}`,
  );
}
for (const target of [
  "ROADMAP.md",
  "docs/research/00-indice.md",
  "CHANGELOG.md",
  "CLAUDE.md",
]) {
  assert.match(
    readme,
    new RegExp(
      `\\]\\(${target.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\$&")}\\)`,
      "i",
    ),
    `README is missing link to ${target}`,
  );
}
for (const [, rawTarget] of readme.matchAll(
  /\]\((?!https?:\/\/|mailto:|#)([^)]+)\)/g,
)) {
  const target = rawTarget.replace(/^<|>$/g, "").split("#", 1)[0];
  assert.ok(
    existsSync(join(repoRoot, target)),
    `README link target does not exist: ${rawTarget}`,
  );
}

const verificationDir = join(repoRoot, "docs", "superpowers", "verification");
const verification = readFileSync(
  required(join(verificationDir, "2026-11-08-twinova-brand.md")),
  "utf8",
);
for (const filename of [
  "2026-11-08-twinova-brand-dark.png",
  "2026-11-08-twinova-brand-light.png",
]) {
  const bytes = readFileSync(required(join(verificationDir, "img", filename)));
  assert.equal(
    bytes.subarray(0, 8).toString("hex"),
    "89504e470d0a1a0a",
    `${filename} is not a PNG`,
  );
}
for (const evidence of [
  "578A92BA4020D8935F64E0437F3FB3B5C5DCA7AF3D8FC70EFFB9C7FD9BF9D283",
  "codex mcp get vectorcraft",
  "26 × 26",
  "npm.cmd run coverage",
  "npm.cmd run build",
  "What this does not prove",
]) {
  assert.ok(
    verification.includes(evidence),
    `brand verification record is missing evidence: ${evidence}`,
  );
}

console.log(
  `Brand verified: ${pngDimensions.size} PNGs, ICO, ICNS, SVG, editable VectorCraft master, UI asset, and README.`,
);
