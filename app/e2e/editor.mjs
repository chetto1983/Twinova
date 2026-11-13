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

import { mkdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  app,
  bit,
  browser,
  button,
  check,
  click,
  eventually,
  expectEqual,
  selectSpeed,
  sessions,
  setEditorText,
  textOf,
} from "./helpers.mjs";
import {
  choose,
  clickAt,
  drag,
  emptyProject,
  fieldValue,
  onScreen,
  place,
  select,
  selectedName,
  setField,
  treeNames,
} from "./editing.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
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

const folder = emptyProject("Hand-built line");

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
    await setEditorText(PROGRAM);
    await click(button("Build"));
    await check("the line builds", () =>
      eventually(
        async () =>
          expectEqual(
            await browser
              .$("//footer[contains(@class,'statusbar')]/span[normalize-space()='Built: ready to run']")
              .isExisting(),
            true,
            "built",
          ),
        "build",
        10_000,
      ),
    );
    await click(`//button[@aria-label="Frame all"]`);
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
      // The save is done when the app says so: the manifest goes last and the save marker after it, so a file on disk
      // is not the end of it, and an app closed before then rolls the save back at the next open (storage.rs).
      await eventually(
        async () => {
          const notice = await textOf(`//p[@role="status"]`);
          if (!notice.includes(`Saved to ${folder}`)) throw new Error(notice);
        },
        "saved",
        15_000,
      );
      const scene = JSON.parse(readFileSync(join(folder, "scene.json"), "utf8"));
      expectEqual(scene.instances.length, 5, "instances");
      expectEqual(scene.connections.length, 4, "three flows and a mount");
    });
  });
  await app("debug", [`--project=${folder}`], async () => {
    await check("the saved line reopens the same, with nothing rolled back, and builds", async () => {
      await eventually(async () => expectEqual((await treeNames()).length, 5, "tree"), "reopened", 10_000);
      const notice = await textOf(`//p[@role="status"]`);
      if (notice.includes("rolled back")) throw new Error(notice);
      await eventually(
        async () =>
          expectEqual(
            await browser
              .$("//footer[contains(@class,'statusbar')]/span[normalize-space()='Built: ready to run']")
              .isExisting(),
            true,
            "built",
          ),
        "build",
        10_000,
      );
    });
  });
});
