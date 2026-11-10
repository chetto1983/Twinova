// End-to-end run of the 4.4a line (spec 2026-11-10-diverter-line-design.md, acceptance 1): the line fixture's belt A,
// roller conveyor with OIP's chain transfer, and belt B, opened in the real app. Every third carton is raised and
// carried to the reject sink; the I/O panel shows the transfer's signals and the two sinks' counts.
//
//   npx tauri build --debug --no-bundle && node e2e/line.mjs [screenshot-dir]

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  app,
  bit,
  browser,
  button,
  check,
  eventually,
  expectEqual,
  click,
  root,
  selectSpeed,
  sessions,
} from "./helpers.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;
const folder = join(root, "src-tauri/project/tests/fixtures/valid/two_belts_diverter");

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

const count = async (name) => Number(await bit(name));

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The 4.4a line: a roller conveyor with OIP's chain transfer, every third carton to reject");
    await check("the line builds", () =>
      eventually(
        async () =>
          expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "status"),
        "build",
        10_000,
      ),
    );
    await check("the I/O panel lists the chain transfer's popup and raised_fb", async () => {
      expectEqual(await bit("chain_transfer.popup"), "FALSE", "popup");
      expectEqual(await bit("chain_transfer.raised_fb"), "FALSE", "raised_fb");
    });

    await click(button("Run"));
    await check("the third carton raises the chains, and raised_fb reports the top", () =>
      // It reaches the transfer at about 11.4 s of virtual time; Run goes at the wall clock.
      eventually(async () => expectEqual(await bit("chain_transfer.raised_fb"), "TRUE", "raised_fb"), "raise", 20_000),
    );
    await screenshot("chain-transfer-raised");
    await click(button("Pause"));

    await selectSpeed("Max");
    await click(button("Run"));
    await check("both sinks count, two to one", () =>
      eventually(
        async () => {
          const [ok, reject] = [await count("sink_ok.count"), await count("sink_reject.count")];
          if (reject < 3) throw new Error(`${ok} / ${reject} so far`);
          // A carton on belt B is two thirds of the way to its sink when the reject count moves.
          if (Math.abs(ok - 2 * reject) > 2) throw new Error(`${ok} to sink_ok, ${reject} to reject`);
        },
        "counts",
        // The debug build on the windows-2022 runner reaches two rejects in about a minute at Max (CI run 42801667508).
        120_000,
      ),
    );
    await click(button("Pause"));
    await screenshot("line-after-run");
  }),
);
