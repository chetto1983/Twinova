// End-to-end run of the 4.4b curve line (spec 2026-11-10-curves-belt-options-design.md, acceptance 1): belt A, a left
// belt curve, belt B, a right roller curve and belt C, opened in the real app. Every carton turns through both curves
// to the sink; the I/O panel shows curve A's drive feedback, its velocity on OIP's scale and its travel.
//
//   npx tauri build --debug --no-bundle && node e2e/curve.mjs [screenshot-dir]

import { mkdirSync } from "node:fs";
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
  root,
  selectSpeed,
  sessions,
} from "./helpers.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;
const folder = join(root, "src-tauri/project/tests/fixtures/valid/curve_line");

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The 4.4b curve line: two curves, every carton through both to the sink");
    await check("the line builds", () =>
      eventually(
        async () =>
          expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "status"),
        "build",
        10_000,
      ),
    );
    await screenshot("curve-line-built");
    await click(button("Run"));
    await check("curve A's drive reads 30 at its rated speed, and counts its travel", () =>
      eventually(
        async () => {
          expectEqual(await bit("curve_a.velocity"), "30", "velocity");
          const travel = Number(await bit("curve_a.encoder"));
          if (!(travel > 0)) throw new Error(`travel ${travel}`);
        },
        "drive feedback",
        10_000,
      ),
    );
    await click(button("Pause"));
    await selectSpeed("Max");
    await click(button("Run"));
    await check("cartons turn through both curves to the sink", () =>
      // The first carton covers 10.2 m at 0.5 m/s: about 20.4 s of virtual time; a carton every 2 s after it.
      eventually(
        async () => {
          const delivered = Number(await bit("sink.count"));
          if (delivered < 3) throw new Error(`${delivered} delivered so far`);
        },
        "deliveries",
        120_000,
      ),
    );
    await click(button("Pause"));
    await screenshot("curve-line-after-run");
  }),
);
