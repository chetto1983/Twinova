// End-to-end run of the editor (spec 2026-11-11-editor-basics-design.md, acceptance 4 and 5), in the real app. From an
// empty project, written here independently of the app's codec:
// - a click in the 3D view selects what is under it and a click on empty floor clears the selection;
// - a drag of the gizmo moves a belt and is one undo step;
// - a line is built by hand: a source, two belts, a sink and a photocell from the catalogue, laid end to end, connected,
//   mounted and bound in the inspector, its program written; Build, then Run delivers to the sink;
// - undo back to the empty scene and redo forward give the same scene; Save and reopen give the same project.
// Screen points come from the camera Viewport.svelte sets up for an empty project: target at the origin, 40° field of
// view, seen from (0.35, 0.6, 1), framed to a half-width of 1 m.
//
//   npx tauri build --debug --no-bundle && node e2e/editor.mjs [screenshot-dir]

import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import * as THREE from "three";
import { app, browser, button, check, click, eventually, expectEqual, bit, selectSpeed, sessions } from "./helpers.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

// ---------- an empty project, written by hand ----------

const json = (value) => JSON.stringify(value, null, 2) + "\n";
const sha256 = (text) => createHash("sha256").update(text).digest("hex");

function emptyProject() {
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
    name: "Hand-built line",
    program: ["plc/Main.st"],
    scene: "scene.json",
    version: 1,
  };
  writeFileSync(join(folder, "twinova.json"), json(manifest));
  return folder;
}

// ---------- the UI ----------

const inspector = `//section[contains(@class,"inspector")]`;
const canvas = `//section[contains(@class,"stage")]//canvas`;
const treeNames = () =>
  browser.execute(() =>
    [...document.querySelectorAll("section.tree button > span:first-child")].map((span) => span.textContent),
  );
const selectedName = async () => {
  const field = await browser.$(`${inspector}//input[@aria-label="Name"]`);
  return (await field.isExisting()) ? field.getValue() : null;
};

async function place(label) {
  await click(`//section[contains(@class,"catalogue")]//button[normalize-space()="${label}"]`);
}

async function select(name) {
  await click(`//section[contains(@class,"tree")]//button[span[1][normalize-space()="${name}"]]`);
  await eventually(async () => expectEqual(await selectedName(), name, "selected"), "select", 5_000);
}

/** Types into an inspector field and leaves it, which commits it. */
async function setField(label, text) {
  const field = await browser.$(`${inspector}//input[@aria-label="${label}"]`);
  await field.setValue(text);
  await browser.keys("Tab");
}

async function choose(label, option) {
  await (await browser.$(`${inspector}//select[@aria-label="${label}"]`)).selectByVisibleText(option);
}

const fieldValue = async (label) => (await browser.$(`${inspector}//input[@aria-label="${label}"]`)).getValue();

/** Where a world point lands on the canvas, as an offset from its centre, with Viewport.svelte's camera on an empty
 *  project. */
async function onScreen(x, y, z) {
  const element = await browser.$(canvas);
  const { width, height } = await element.getSize();
  const camera = new THREE.PerspectiveCamera(40, width / height, 0.1, 500);
  const tanV = Math.tan((40 * Math.PI) / 360);
  const distance = Math.max(1 / (tanV * camera.aspect), 1 / tanV);
  camera.position.copy(new THREE.Vector3(0.35, 0.6, 1).normalize().multiplyScalar(distance));
  camera.lookAt(0, 0, 0);
  camera.updateMatrixWorld();
  const p = new THREE.Vector3(x, y, z).project(camera);
  return { origin: element, x: Math.round((p.x * width) / 2), y: Math.round((-p.y * height) / 2) };
}

async function clickAt(point) {
  await browser
    .action("pointer")
    .move({ duration: 0, ...point })
    .down()
    .up()
    .perform();
}

async function drag(from, to) {
  await browser
    .action("pointer")
    .move({ duration: 0, ...from })
    .down()
    .move({ duration: 300, ...to })
    .up()
    .perform();
}

const PROGRAM = `PROGRAM Main
VAR
    run_1 AT %QX0.0 : BOOL;
    run_2 AT %QX0.1 : BOOL;
    running_1 AT %IX1.0 : BOOL;
    running_2 AT %IX1.1 : BOOL;
    seen AT %IX0.0 : BOOL;
    delivered AT %ID4 : DINT;
END_VAR
run_1 := TRUE;
run_2 := TRUE;
END_PROGRAM
`;

const setProgram = (text) =>
  browser.execute((t) => {
    const area = document.querySelector("textarea");
    area.value = t;
    area.dispatchEvent(new Event("input", { bubbles: true }));
  }, text);

const folder = emptyProject();

sessions(async () => {
  await app("debug", [`--project=${folder}`], async () => {
    console.log("Picking and the gizmo in the 3D view");
    await place("belt");
    await check("a placed family is selected, at the origin", async () => {
      await eventually(async () => expectEqual(await selectedName(), "belt_1", "selected"), "placed", 5_000);
      expectEqual([await fieldValue("Position (m) x"), await fieldValue("Position (m) z")], ["0", "0"], "origin");
    });
    await check("a drag of the gizmo's x arrow moves the belt along x, as one undo step", async () => {
      await drag(await onScreen(0.2, 0, 0), await onScreen(1.2, 0, 0));
      await eventually(
        async () => {
          const x = Number(await fieldValue("Position (m) x"));
          if (!(x > 0.6 && x < 1.4)) throw new Error(`x ${x}`);
          expectEqual(await fieldValue("Position (m) z"), "0", "z");
        },
        "moved",
        5_000,
      );
      await screenshot("editor-gizmo");
      await browser.keys(["Control", "z"]);
      await eventually(async () => expectEqual(await fieldValue("Position (m) x"), "0", "x after undo"), "undo", 5_000);
    });
    await check("a click on empty floor clears the selection, a click on the belt selects it", async () => {
      await clickAt(await onScreen(-0.6, 0, 0.6));
      await eventually(async () => expectEqual(await selectedName(), null, "cleared"), "floor", 5_000);
      await clickAt(await onScreen(0.5, 0.8, 0));
      await eventually(async () => expectEqual(await selectedName(), "belt_1", "picked"), "belt", 5_000);
    });

    console.log("A line built by hand");
    for (const label of ["belt", "source", "sink", "sensor diffuse"]) await place(label);
    await check("five instances from the catalogue", () =>
      eventually(
        async () =>
          expectEqual(
            (await treeNames()).toSorted(),
            ["belt_1", "belt_2", "sensor_diffuse_1", "sink_1", "source_1"],
            "tree",
          ),
        "placed",
        5_000,
      ),
    );
    await check("laid out, connected, mounted and bound in the inspector", async () => {
      await select("belt_2");
      await setField("Position (m) x", "3");
      await choose("in", "belt_1.out");
      await choose("out", "sink_1.in");
      await setField("run", "%QX0.1");
      await setField("running", "%IX1.1");
      await select("belt_1");
      await choose("in", "source_1.out");
      await setField("run", "%QX0.0");
      await setField("running", "%IX1.0");
      await select("sink_1");
      await setField("Position (m) x", "6.2");
      await setField("count", "%ID4");
      await select("source_1");
      await setField("Position (m) x", "-0.5");
      await select("sensor_diffuse_1");
      await choose("Conveyor", "belt_2");
      await setField("Along it (m, ° on a curve)", "1");
      await setField("detected", "%IX0.0");
      await eventually(async () => expectEqual(await fieldValue("detected"), "%IX0.0", "bound"), "bound", 5_000);
    });
    await click(`//button[@role="tab"][normalize-space()="plc/Main.st"]`);
    await setProgram(PROGRAM);
    await click(button("Build"));
    await check("the line builds", () =>
      eventually(
        async () =>
          expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "built"),
        "build",
        10_000,
      ),
    );
    await click(`//button[normalize-space()="Frame all"]`);
    await screenshot("editor-line-built");
    await selectSpeed("Max");
    await click(button("Run"));
    await check("Run delivers a carton to the sink", () =>
      eventually(
        async () => {
          const delivered = Number(await bit("sink_1.count"));
          if (!(delivered >= 1)) throw new Error(`delivered ${delivered}`);
        },
        "delivered",
        60_000,
      ),
    );
    await click(button("Pause"));
    const built = await treeNames();

    console.log("Undo and redo");
    await check("undo walks back to the empty scene, redo forward to the same line", async () => {
      const undo = await browser.$(button("Undo"));
      while (await undo.isEnabled()) await undo.click();
      expectEqual(await treeNames(), [], "empty");
      const redo = await browser.$(button("Redo"));
      while (await redo.isEnabled()) await redo.click();
      expectEqual(await treeNames(), built, "redone");
    });
    await check("Save writes the line to its folder", async () => {
      await click(`//header//button[@aria-haspopup="menu"]`);
      await click(`//button[@role="menuitem"][normalize-space()="Save"]`);
      await eventually(
        async () => {
          const scene = JSON.parse(readFileSync(join(folder, "scene.json"), "utf8"));
          expectEqual(scene.instances.length, 5, "instances");
          expectEqual(scene.connections.length, 4, "three flows and a mount");
        },
        "saved",
        10_000,
      );
    });
  });
  await app("debug", [`--project=${folder}`], async () => {
    await check("the saved line reopens the same, and builds", async () => {
      await eventually(async () => expectEqual((await treeNames()).length, 5, "tree"), "reopened", 10_000);
      await eventually(
        async () =>
          expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "built"),
        "build",
        10_000,
      );
    });
  });
});
