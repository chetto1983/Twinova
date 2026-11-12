// End-to-end run of the 4.4c sort line (spec 2026-11-11-stopper-pusher-turntable-design.md, acceptance 4): a stopper,
// a turntable and a pusher, opened in the real app. The program holds each carton at the stopper until the turntable
// is home and empty, turns every second one to belt C, and pushes every second straight one off belt B: each of the
// three sinks counts, and the I/O panel shows the devices' feedback.
//
//   npx tauri build --debug --no-bundle && node e2e/sort.mjs [screenshot-dir]

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
  fixtureCopy,
  selectSpeed,
  sessions,
} from "./helpers.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;
const folder = fixtureCopy("sort_line");

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The 4.4c sort line: a stopper, a turntable and a pusher, every route counted");
    await check("the line builds", () =>
      eventually(
        async () =>
          expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "status"),
        "build",
        10_000,
      ),
    );
    await screenshot("sort-line-built");
    await click(button("Run"));
    await check("the turntable reports itself in position and the pusher home", async () => {
      await eventually(
        async () => {
          expectEqual(await bit("turntable.in_position"), "TRUE", "in position");
          expectEqual(await bit("pusher.retracted_fb"), "TRUE", "home");
        },
        "feedback",
        10_000,
      );
    });
    await click(button("Pause"));
    await selectSpeed("Max");
    await click(button("Run"));
    await check("cartons run straight, turn at 90° and are pushed off, each into its sink", () =>
      // The first straight carton reaches sink B after about 16 s of virtual time, the first turned one sink C
      // after about 20 s, the first pushed one the reject sink after about 30 s.
      eventually(
        async () => {
          const counts = await Promise.all(["sink_b", "sink_c", "sink_reject"].map((s) => bit(`${s}.count`)));
          if (counts.some((n) => Number(n) < 1)) throw new Error(`sinks B, C and reject at ${counts.join(", ")}`);
        },
        "deliveries",
        180_000,
      ),
    );
    await click(button("Pause"));
    await screenshot("sort-line-after-run");
  }),
);
