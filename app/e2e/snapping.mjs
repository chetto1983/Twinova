// End-to-end run of snapping (spec 2026-11-11-snapping-design.md, acceptance 5 and 6), in the real app.
// - The line with a diverter, two_belts_diverter, built by hand from an empty project with drags and snaps only:
//   - families dragged from the catalogue and released over the floor;
//   - the gizmo dragged along x, then z, until a release lands an end, a rail or a side exit within reach.
//   The inspector sets parameters, signals and a gap; the program is the fixture's. It builds, runs until both sinks
//   count, and, saved, matches the fixture: the conveyors within 1 mm of it relative to the first belt (the view rounds
//   a drop to the millimetre), the same flows, the mounts on the same hosts within 20 mm (a drop's point: a pixel is about 10 mm
//   when the whole line is framed).
// - The merge line: its spur dragged off the flank and back merges again, and its cartons reach the sink.
//
//   npx tauri build --debug --no-bundle && node e2e/snapping.mjs [screenshot-dir]

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
  fixtureCopy,
  root,
  selectSpeed,
  sessions,
} from "./helpers.mjs";
import {
  EMPTY_VIEW,
  dragGizmo,
  dropAt,
  emptyProject,
  fieldValue,
  framed,
  inspector,
  reachable,
  position,
  select,
  setField,
  setProgram,
  treeNames,
} from "./editing.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;
const fixtures = join(root, "src-tauri/project/tests/fixtures/valid");
const fixture = (name) => join(fixtures, name);

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

const near = (a, b, tolerance, what) => {
  if (a.some((v, k) => Math.abs(v - b[k]) > tolerance)) throw new Error(`${what}: ${a} is not ${b} ±${tolerance}`);
};

/** Where an axis is in an [x, z] pair. */
const AXIS = { x: 0, z: 1 };

/** Clicks "Frame all" and returns the view it gives `known` instances. */
async function frameAll(known) {
  await click(`//button[@aria-label="Frame all"]`);
  return framed(Object.values(known));
}

/** Drags the selection along `axis` towards `to` in hops, framing all before each: a hop goes as far as its release
 *  stays on the canvas, and the part, moving, widens the framing. The hops go on until the part stands within
 *  `close` of `to`, so a release that lands short (the pointer is less exact on some platforms) is followed by
 *  another; a release that snaps the part ends them once it is within `close`. Records where it stands. */
async function dragAlong(known, id, axis, to, close) {
  const k = AXIS[axis];
  for (let hop = 0; hop < 20; hop++) {
    const at = known[id];
    if (Math.abs(to - at[k]) <= close) return;
    const view = await frameAll(known);
    await dragGizmo(at, axis, await reachable(at, axis, to - at[k], view), view);
    known[id] = await eventually(
      async () => {
        const now = await position();
        if (now.every((v, j) => v === at[j])) throw new Error(`${id} has not moved from ${at}`);
        return now;
      },
      `${id} moved along ${axis}`,
      5_000,
    );
  }
  throw new Error(`${id} did not reach ${axis} ${to}: it stands at ${known[id]}`);
}

/** Drags the selection to `to`, along x then z, or `zFirst`: the first axis to within 2 cm, the last until the part
 *  is within a snap's reach (snapping.ts `REACH`, 0.3 m), where the release that brought it there has snapped it. */
async function dragTo(known, id, to, zFirst = false) {
  const [first, last] = zFirst ? ["z", "x"] : ["x", "z"];
  await dragAlong(known, id, first, to[AXIS[first]], 0.02);
  await dragAlong(known, id, last, to[AXIS[last]], 0.3);
}

/** Waits until the selection stands at `at`, within `tolerance`, and records it. */
async function standsAt(known, id, at, tolerance) {
  known[id] = await eventually(
    async () => {
      const p = await position();
      near(p, at, tolerance, id);
      return p;
    },
    `${id} snapped`,
    5_000,
  );
}

const original = JSON.parse(readFileSync(join(fixture("two_belts_diverter"), "scene.json"), "utf8"));
const PROGRAM = readFileSync(join(fixture("two_belts_diverter"), "plc/Main.st"), "utf8");
/** The fixture's ids, as the catalogue numbers the same families placed in this order. */
const IDS = {
  belt_a: "belt_1",
  roller_r: "roller_1",
  belt_b: "belt_2",
  chain_transfer: "chain_transfer_1",
  sensor_a: "sensor_diffuse_1",
  sensor_b: "sensor_diffuse_2",
  source: "source_1",
  sink_ok: "sink_1",
  sink_reject: "sink_2",
};

const ALONG = "Along it (m, ° on a curve)";
/** A fixture's endpoint under this run's ids. */
const ours = (e) => [IDS[e[0]], e[1]].join(".");
/** A scene's flows, `from to`, each endpoint named by `name`. */
const flowsOf = (scene, name) =>
  scene.connections
    .filter((c) => c.kind === "flow")
    .map((c) => `${name(c.from)} ${name(c.to)}`)
    .toSorted();

const folder = emptyProject("Line with a diverter, by hand");
const merging = fixtureCopy("merge_line");

sessions(async () => {
  await app("debug", [`--project=${folder}`], async () => {
    const known = {};
    let p0 = [0, 0];
    let transferAt = 1;
    console.log("The line with a diverter, by drag and snap");
    await check("a belt dragged from the catalogue lands where it is released", async () => {
      await dropAt("belt", 0, 0, EMPTY_VIEW);
      p0 = await eventually(
        async () => {
          const at = await position();
          near(at, [0, 0], 0.05, "belt_1");
          return at;
        },
        "dropped",
        5_000,
      );
      known.belt_1 = p0;
    });
    const rel = (x, z = 0) => [p0[0] + x, p0[1] + z];
    await check("a roller conveyor dragged to the belt's exit snaps onto it", async () => {
      await dropAt("roller", ...rel(0.5, -0.8), await frameAll(known));
      known.roller_1 = await eventually(position, "roller placed", 5_000);
      await dragTo(known, "roller_1", rel(3.1, 0.04));
      await standsAt(known, "roller_1", rel(3.085), 0.001);
      await (await browser.$(`${inspector}//input[@aria-label="left_guard"]`)).click();
      await setField("run", "%QX0.3");
      await setField("running", "%IX0.5");
    });
    await check(
      "belt B dragged to the roller conveyor's exit snaps onto it, and its gap goes to 20 mm and back",
      async () => {
        await dropAt("belt", ...rel(3.5, -1.2), await frameAll(known));
        known.belt_2 = await eventually(position, "belt placed", 5_000);
        await dragTo(known, "belt_2", rel(5.2, 0.03));
        await standsAt(known, "belt_2", rel(5.17), 0.001);
        await setField("length", "2");
        await setField("in Gap (mm)", "20");
        await standsAt(known, "belt_2", rel(5.19), 0.001);
        await setField("in Gap (mm)", "0");
        await standsAt(known, "belt_2", rel(5.17), 0.001);
        await setField("run", "%QX0.1");
        await setField("running", "%IX0.3");
        await screenshot("snapping-belts");
      },
    );
    await check(
      "the chain transfer and the first photocell mount on the roller conveyor where they are dropped",
      async () => {
        const view = await frameAll(known);
        await dropAt("chain transfer", ...rel(4.085), view);
        transferAt = await eventually(
          async () => {
            const along = Number(await fieldValue(ALONG));
            near([along], [1], 0.02, "transfer");
            return along;
          },
          "transfer",
          5_000,
        );
        known.chain_transfer_1 = rel(4.085);
        await setField("popup", "%QX0.2");
        await setField("raised_fb", "%IX0.4");
        await dropAt("sensor diffuse", ...rel(4.085), view);
        await eventually(async () => near([Number(await fieldValue(ALONG))], [1], 0.02, "eye A"), "eye A", 5_000);
        known.sensor_diffuse_1 = rel(4.085);
        await setField("detected", "%IX0.0");
      },
    );
    await check(
      "the sink dragged to belt B's exit snaps onto it, then the second photocell mounts on belt B",
      async () => {
        // Keep the staging point on the wider viewport before dragging towards the far exit.
        await dropAt("sink", ...rel(5.5, 0.5), await frameAll(known));
        known.sink_1 = await eventually(position, "sink placed", 5_000);
        // Across first, away from the camera, where the far end stays in view.
        await dragTo(known, "sink_1", rel(7.4, 0.05), true);
        await standsAt(known, "sink_1", rel(7.255), 0.001);
        await setField("count", "%ID4");
        await dropAt("sensor diffuse", ...rel(6.67), await frameAll(known));
        await eventually(async () => near([Number(await fieldValue(ALONG))], [1.5], 0.02, "eye B"), "eye B", 5_000);
        known.sensor_diffuse_2 = rel(6.67);
        await setField("detected", "%IX0.1");
      },
    );
    await check(
      "the source dropped by belt A's entry and the reject sink by the transfer's side snap onto them",
      async () => {
        const view = await frameAll(known);
        await dropAt("source", ...rel(-0.25), view);
        await standsAt(known, "source_1", rel(-0.085), 0.001);
        await dropAt("sink", ...rel(4.085, -0.5), view);
        // At the transfer's side exit: its lane's end, where the transfer was mounted along the roller conveyor.
        await standsAt(known, "sink_2", rel(3.085 + transferAt, -0.3), 0.001);
        await setField("count", "%ID8");
        await select("belt_1");
        await setField("run", "%QX0.0");
        await setField("running", "%IX0.2");
      },
    );
    await click(`//button[@role="tab"][normalize-space()="plc/Main.st"]`);
    await setProgram(PROGRAM);
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
    await frameAll(known);
    await screenshot("snapping-line");
    await selectSpeed("Max");
    await click(button("Run"));
    await check("it runs until both sinks count", () =>
      eventually(
        async () => {
          const [ok, reject] = [Number(await bit("sink_1.count")), Number(await bit("sink_2.count"))];
          if (!(ok >= 2 && reject >= 1)) throw new Error(`delivered ${ok} and ${reject}`);
        },
        "delivered",
        90_000,
      ),
    );
    await click(button("Pause"));
    await check("saved, it matches the fixture", async () => {
      await click(`//header//button[@aria-haspopup="menu"]`);
      await click(`//button[@role="menuitem"][normalize-space()="Save"]`);
      await eventually(
        async () => {
          const notice = await browser.$(`//p[@role="status"]`).getText();
          if (!notice.includes(`Saved to ${folder}`)) throw new Error(notice);
        },
        "saved",
        15_000,
      );
      const scene = JSON.parse(readFileSync(join(folder, "scene.json"), "utf8"));
      const mine = Object.fromEntries(scene.instances.map((i) => [i.id, i]));
      const base = mine.belt_1.position;
      for (const id of ["belt_a", "roller_r", "belt_b"]) {
        const want = original.instances.find((i) => i.id === id);
        const got = mine[IDS[id]];
        near(
          got.position.map((v, k) => v - base[k]),
          want.position,
          0.001,
          id,
        );
        // q and -q are one rotation: the angle between them is 2 acos |q1 · q2|.
        const dot = Math.abs(got.rotation.reduce((sum, v, k) => sum + v * want.rotation[k], 0));
        near([2 * Math.acos(Math.min(dot, 1))], [0], 1e-6, `${id} turned`);
      }
      expectEqual(
        flowsOf(scene, (e) => e.join(".")),
        flowsOf(original, ours),
        "flows",
      );
      for (const want of original.connections.filter((c) => c.kind === "mount")) {
        const got = scene.connections.find((m) => m.kind === "mount" && m.device[0] === IDS[want.device[0]]);
        expectEqual(got?.host[0], IDS[want.host[0]], `${want.device[0]}'s host`);
        near([got.position[0]], [want.position[0]], 0.02, `${want.device[0]} along`);
      }
      expectEqual((await treeNames()).length, original.instances.length, "instances");
    });
  });
  await app("debug", [`--project=${merging}`], async () => {
    console.log("The merge line: its spur dragged off and back");
    const known = { source: [2, 2.385], belt_a: [0, 0], sink: [4.085, 0], belt_s: [2, 2.385] };
    await check("the spur dragged off the flank and back merges again, and its cartons reach the sink", async () => {
      await select("Spur");
      // Give the front-facing gizmo room for the return grab after moving the spur towards the camera.
      await click(button("Collapse panel"));
      const view = await frameAll(known);
      await dragGizmo(known.belt_s, "z", 0.5, view);
      await eventually(async () => near(await position(), [2, 2.885], 0.05, "spur off"), "off", 5_000);
      await dragGizmo([2, 2.885], "z", -0.46, view);
      await eventually(async () => near(await position(), [2, 2.385], 0.001, "spur merged"), "merged", 5_000);
      await click(button("Build"));
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
      await selectSpeed("Max");
      await click(button("Start"));
      await click(button("Run"));
      await eventually(
        async () => {
          const delivered = Number(await bit("sink.count"));
          if (!(delivered >= 1)) throw new Error(`delivered ${delivered}`);
        },
        "delivered",
        60_000,
      );
      await screenshot("snapping-merge");
      await click(button("Pause"));
    });
  });
});
