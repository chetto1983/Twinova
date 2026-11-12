// End-to-end run of the 4.4d line (spec 2026-11-11-sensors-products-design.md, acceptance 7): red, green and blue
// cartons with codes 1, 2 and 3, opened in the real app. The colour sensor tells the turntable to turn the green ones
// onto belt C, the barcode scanner tells the pusher to reject code 3, and red runs straight on to sink B: each sink
// counts, and the I/O panel shows the scanner's code and count, the colour read and the laser's distance.
//
//   npx tauri build --debug --no-bundle && node e2e/code_sort.mjs [screenshot-dir]

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
const folder = fixtureCopy("code_sort_line");

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The 4.4d line: cartons sorted by their colour and their code");
    await check("the line builds", () =>
      eventually(
        async () =>
          expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "status"),
        "build",
        10_000,
      ),
    );
    await screenshot("code-sort-built");
    await selectSpeed("Max");
    await click(button("Run"));
    await check("red runs to sink B, green is turned to sink C, blue is rejected", () =>
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
    await check("the scanner has counted the cartons it read, the last code one it reads", async () => {
      const [count, code] = await Promise.all([bit("scan_b.count"), bit("scan_b.code")]);
      if (Number(count) < 2) throw new Error(`scanner count ${count}`);
      if (!["1", "3"].includes(code)) throw new Error(`scanner code ${code}`);
    });
    await check("the laser reads a distance within its range", async () => {
      const distance = Number(await bit("s_laser.distance"));
      if (!(distance > 0 && distance <= 4)) throw new Error(`laser distance ${distance}`);
    });
    await screenshot("code-sort-after-run");
  }),
);
