// What the editor's end-to-end runs share (editor.mjs, snapping.mjs): an empty project written by hand, independently
// of the app's codec; the scene tree, the inspector and the catalogue; and where a world point lands on the canvas with
// the camera Viewport.svelte sets up, from (0.35, 0.6, 1) towards a target, 40° field of view, framed to a half-width.

import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import * as THREE from "three";
import { browser, click, dock, eventually, expectEqual } from "./helpers.mjs";

const json = (value) => JSON.stringify(value, null, 2) + "\n";
const sha256 = (text) => createHash("sha256").update(text).digest("hex");

/** An empty project in a new folder: no instances, an empty program. */
export function emptyProject(name) {
  const folder = join(mkdtempSync(join(tmpdir(), "twinova-editor-")), "line");
  const files = {
    "scene.json": json({ connections: [], controller: { panel: [] }, groups: [], instances: [] }),
    "plc/Main.st": "PROGRAM Main\nEND_PROGRAM\n",
  };
  mkdirSync(join(folder, "plc"), { recursive: true });
  for (const [path, text] of Object.entries(files)) writeFileSync(join(folder, path), text);
  const manifest = {
    app: "0.1.0",
    components: [],
    files: Object.fromEntries(Object.entries(files).map(([path, text]) => [path, sha256(text)])),
    format: "twinova-project",
    name,
    program: ["plc/Main.st"],
    scene: "scene.json",
    version: 1,
  };
  writeFileSync(join(folder, "twinova.json"), json(manifest));
  return folder;
}

export const inspector = `//section[contains(@class,"inspector")]`;
export const canvas = `//section[contains(@class,"stage")]//canvas`;
const catalogueButton = (label) =>
  `//section[contains(@class,"catalogue")]//button[.//span[contains(@class,"name") and normalize-space()="${label}"]]`;

export const treeNames = () =>
  browser.execute(() => [...document.querySelectorAll("section.tree .item-name")].map((span) => span.textContent));

export const selectedName = async () => {
  const field = await browser.$(`${inspector}//input[@aria-label="Name"]`);
  return (await field.isExisting()) ? field.getValue() : null;
};

/** A catalogue family placed by a click, under the view's framing centre. */
async function catalogueCategory(label) {
  await dock("parts");
  const category = /source|sink/.test(label) ? "Products" : /sensor|scanner/.test(label) ? "Devices" : "Equipment";
  await click(`//div[contains(@class,"categories")]//button[normalize-space()="${category}"]`);
}
export const place = async (label) => {
  await catalogueCategory(label);
  await click(catalogueButton(label));
};

export async function select(name) {
  await click(`//section[contains(@class,"tree")]//button[span[1][normalize-space()="${name}"]]`);
  await eventually(async () => expectEqual(await selectedName(), name, "selected"), "select", 5_000);
}

/** Types into an inspector field and leaves it, which commits it. */
export async function setField(label, text) {
  const field = await browser.$(`${inspector}//input[@aria-label="${label}"]`);
  await field.setValue(text);
  await browser.keys("Tab");
}

export async function choose(label, option) {
  await (await browser.$(`${inspector}//select[@aria-label="${label}"]`)).selectByVisibleText(option);
}

export const fieldValue = async (label) => (await browser.$(`${inspector}//input[@aria-label="${label}"]`)).getValue();

/** The selected instance's position, x and z, as the inspector shows it. */
export const position = async () => [
  Number(await fieldValue("Position (m) x")),
  Number(await fieldValue("Position (m) z")),
];

/** The camera of an empty project: its target at the origin, a half-width of 1 m. */
export const EMPTY_VIEW = { target: [0, 0, 0], halfWidth: 1 };

/** The view "Frame all" gives instances at `positions` ([x, z] each): project.ts `frame`. */
export function framed(positions) {
  const xs = positions.map((p) => p[0]);
  const zs = positions.map((p) => p[1]);
  const [x0, x1, z0, z1] = [Math.min(...xs), Math.max(...xs), Math.min(...zs), Math.max(...zs)];
  return { target: [(x0 + x1) / 2, 0.8, (z0 + z1) / 2], halfWidth: Math.max(x1 - x0, z1 - z0) / 2 + 1 };
}

async function camera(view) {
  const element = await browser.$(canvas);
  const { width, height } = await element.getSize();
  const cam = new THREE.PerspectiveCamera(40, width / height, 0.1, 500);
  const tanV = Math.tan((40 * Math.PI) / 360);
  const distance = Math.max(view.halfWidth / (tanV * cam.aspect), 1.6 / tanV);
  const target = new THREE.Vector3(...view.target);
  cam.position.copy(new THREE.Vector3(0.35, 0.6, 1).normalize().multiplyScalar(distance).add(target));
  cam.lookAt(target);
  cam.updateMatrixWorld();
  return { element, width, height, cam };
}

/** Where a world point lands on the canvas, as an offset from its centre. */
export async function onScreen(x, y, z, view = EMPTY_VIEW) {
  const { element, width, height, cam } = await camera(view);
  const p = new THREE.Vector3(x, y, z).project(cam);
  return { origin: element, x: Math.round((p.x * width) / 2), y: Math.round((-p.y * height) / 2) };
}

/** Whether a world point lands on the canvas, clear of its edge. */
export async function inView(x, y, z, view) {
  const { width, height, cam } = await camera(view);
  const p = new THREE.Vector3(x, y, z).project(cam);
  return Math.abs((p.x * width) / 2) < width / 2 - 10 && Math.abs((p.y * height) / 2) < height / 2 - 10;
}

export async function clickAt(point) {
  await browser
    .action("pointer")
    .move({ duration: 0, ...point })
    .down()
    .up()
    .perform();
}

export async function drag(from, to) {
  await browser
    .action("pointer")
    .move({ duration: 0, ...from })
    .down()
    .move({ duration: 300, ...to })
    .up()
    .perform();
}

/** A family dragged from the catalogue and released over the floor at (x, z). */
export async function dropAt(label, x, z, view) {
  await catalogueCategory(label);
  if (!(await inView(x, 0, z, view))) throw new Error(`Drop point for ${label} is outside the viewport: ${x}, ${z}`);
  const before = (await treeNames()).length;
  const from = await browser.$(catalogueButton(label));
  await from.scrollIntoView({ block: "nearest", inline: "center" });
  await browser
    .action("pointer")
    .move({ duration: 0, origin: from })
    .down()
    .move({ duration: 300, ...(await onScreen(x, 0, z, view)) })
    .up()
    .perform();
  await eventually(
    async () => expectEqual((await treeNames()).length, before + 1, "placed instance count"),
    "drop",
    5000,
  );
}

/** Where the selection's gizmo is grabbed to drag it along world `axis` ("x" or "z") from `at` ([x, z], where the
 *  selection stands), and where it is released to move it `by` metres. TransformControls scales its handles to the
 *  camera's distance times min(1.9 tan(fov / 2), 7), and an axis has a picker each way along it, out to 0.15 of that
 *  (three 0.187, `pickerTranslate`). The one behind the move is grabbed, 0.08 of the size out, so the release stays
 *  near the part. */
async function gizmoPoints(at, axis, by, view) {
  const { cam } = await camera(view);
  const origin = new THREE.Vector3(at[0], 0, at[1]);
  const size = origin.distanceTo(cam.position) * Math.min(1.9 * Math.tan((40 * Math.PI) / 360), 7);
  const dir = axis === "x" ? new THREE.Vector3(1, 0, 0) : new THREE.Vector3(0, 0, 1);
  const grab = origin.clone().addScaledVector(dir, (by > 0 ? -0.08 : 0.08) * size);
  return { grab, release: grab.clone().addScaledVector(dir, by) };
}

export async function dragGizmo(at, axis, by, view) {
  const { grab, release } = await gizmoPoints(at, axis, by, view);
  await drag(await onScreen(grab.x, 0, grab.z, view), await onScreen(release.x, 0, release.z, view));
}

/** The longest part of a `by` metre gizmo drag, halved until its release lands on the canvas. */
export async function reachable(at, axis, by, view) {
  let step = by;
  for (;;) {
    const { release } = await gizmoPoints(at, axis, step, view);
    if ((await inView(release.x, 0, release.z, view)) || Math.abs(step) < 0.05) return step;
    step /= 2;
  }
}
