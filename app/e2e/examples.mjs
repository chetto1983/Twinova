// The acceptance A3 of spec 2026-11-26-ux-consolidation-design.md (plan 2026-11-26-u1-noise.md, Task 8), through the
// real app: every example the project menu lists opens, builds, and shows no problem at the default level. Its hints
// stay one click away. A picture of each example's window goes to the output folder.
//
//   npx tauri build --debug --no-bundle && node e2e/examples.mjs
//
// Needs what e2e/run.mjs needs.

import { join } from "node:path";
import { browser, button, check, click, dock, eventually, expectEqual, out, session, textOf } from "./helpers.mjs";

const projectButton = `//header//button[@aria-haspopup="menu"]`;
const examples = () =>
  browser.execute(() =>
    [...document.querySelectorAll('ul[role="menu"] li[role="separator"] ~ li button')].map((b) => b.textContent.trim()),
  );

await session("debug", async () => {
  await click(projectButton);
  const names = await examples();
  await click(projectButton);
  await check("the project menu lists the seven examples", async () => expectEqual(names.length, 7, "examples"));
  for (const [i, name] of names.entries()) {
    await check(`${name}: builds, and Problems shows no problem and no count`, async () => {
      await click(projectButton);
      await click(`//ul[@role="menu"]//button[normalize-space()=${JSON.stringify(name)}]`);
      await eventually(
        async () => expectEqual(await textOf(`${projectButton}/span[1]`), name, "opened"),
        "opened",
        10_000,
      );
      await click(button("Build"));
      await eventually(
        async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
        "built",
        30_000,
      );
      await dock("problems");
      await eventually(
        async () => {
          const shown = await browser.execute(() => ({
            rows: document.querySelectorAll("#panel-problems .problem").length,
            badge: Boolean(document.querySelector("#dock-problems .badge")),
            clear: Boolean(document.querySelector("#panel-problems .all-clear")),
          }));
          expectEqual(shown, { rows: 0, badge: false, clear: true }, "problems");
        },
        "analysed",
        10_000,
      );
      await browser.saveScreenshot(join(out, `example-${i + 1}.png`));
    });
  }
});
