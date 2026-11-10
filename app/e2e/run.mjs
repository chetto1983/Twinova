// End-to-end run of the real app through tauri-driver, on Linux (WebKitWebDriver) and Windows
// (msedgedriver matching WebView2). The structure and helpers follow chetto1983/ModbusTool e2e/.
// It replays the M1 acceptance that was first done by hand with xdotool
// (docs/superpowers/verification/2026-11-07-tauri-shell.md).
//
//   npx tauri build --debug --no-bundle && node e2e/run.mjs [screenshot-dir]
//
// Needs: tauri-driver and the platform's WebDriver; on Linux an X display (xvfb-run).

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import {
  bit,
  browser,
  button,
  check,
  click,
  eventually,
  expectEqual,
  selectLanguage,
  selectSpeed,
  session,
  value,
  virtualSeconds,
} from "./helpers.mjs";

const shots = process.argv[2] ? resolve(process.argv[2]) : null;

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

async function lineScenario() {
  console.log("The M3 demo project: a belt, its behaviour and the line program");
  await check("the demo builds on start", () =>
    eventually(
      async () =>
        expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "status"),
      "build",
      10_000,
    ),
  );

  await click(button("Run"));
  await click(button("Start"));
  await check("Start latches the belt's run command", () =>
    eventually(async () => expectEqual(await bit("belt_a.run"), "TRUE", "run"), "run", 5000),
  );
  await check("the belt's behaviour reports it running", () =>
    eventually(async () => expectEqual(await bit("belt_a.running"), "TRUE", "running"), "running", 5000),
  );
  await check("cartons ride the belt and are delivered", () =>
    // The first part needs (3.0 - 0.3) / 0.5 = 5.4 s of virtual time to reach the end.
    eventually(
      async () => {
        if (Number(await value("Parts delivered")) < 1) throw new Error("none delivered yet");
      },
      "delivery",
      15_000,
    ),
  );
  await screenshot("running");

  await check("virtual time follows the wall clock in Run", async () => {
    const ratio = await pace(5000);
    if (ratio < 0.9 || ratio > 1.1) throw new Error(`virtual/wall ratio ${ratio.toFixed(3)}`);
  });

  await click(`//label[normalize-space()="sink blocked"]/input`);
  await check("a carton held at a blocked sink covers the photocell, trips the jam and stops the belt", () =>
    // The leading carton stops under the photocell; the timeout leaves room for one still entering.
    eventually(
      async () => {
        expectEqual(
          [await bit("sensor_a.detected"), await bit("jam"), await bit("belt_a.run")],
          ["TRUE", "TRUE", "FALSE"],
          "I/O",
        );
      },
      "jam",
      40_000,
    ),
  );
  await screenshot("jam");
}

/** Virtual seconds per wall second over `ms` of wall time. */
async function pace(ms) {
  const [v0, t0] = [await virtualSeconds("Virtual time"), Date.now()];
  await sleep(ms);
  const [v1, t1] = [await virtualSeconds("Virtual time"), Date.now()];
  console.log(`      ${(v1 - v0).toFixed(2)} s virtual in ${((t1 - t0) / 1000).toFixed(2)} s wall`);
  return (v1 - v0) / ((t1 - t0) / 1000);
}

async function runModesScenario() {
  console.log("Run modes and saved states");
  await click(button("Pause"));
  await check("a saved state comes back on Restore", async () => {
    await click(button("Save state"));
    const saved = await virtualSeconds("Virtual time");
    for (let i = 0; i < 5; i++) await click(button("Step"));
    expectEqual((await virtualSeconds("Virtual time")) > saved, true, "stepped past the saved time");
    await click(button("Restore"));
    await eventually(
      async () => expectEqual(await virtualSeconds("Virtual time"), saved, "virtual time"),
      "restore",
      5000,
    );
  });
  await check("Run at ×2 runs virtual time twice as fast", async () => {
    await selectSpeed("×2");
    await click(button("Run"));
    const ratio = await pace(4000);
    await click(button("Pause"));
    if (ratio < 1.8 || ratio > 2.2) throw new Error(`virtual/wall ratio ${ratio.toFixed(3)} at ×2`);
  });
  await selectSpeed("×1");
}

async function languageScenario() {
  console.log("Interface language");
  await check("Italian applies at once and the run goes on (no reload)", async () => {
    const before = await virtualSeconds("Virtual time");
    await selectLanguage("Italiano");
    await browser.$(`//h2[normalize-space()="Ingressi e uscite"]`).waitForExist({ timeout: 3000 });
    expectEqual(await browser.execute(() => document.documentElement.lang), "it", "lang");
    const after = await virtualSeconds("Tempo virtuale");
    if (after < before) throw new Error(`virtual time went back: ${before} -> ${after}`);
  });
  await screenshot("italian");
  await selectLanguage("English");
}

/** The text of the editor's open tab, and a replacement for it that Svelte sees (an input event). */
const editorText = () => browser.execute(() => document.querySelector("textarea").value);
const setEditorText = (text) =>
  browser.execute((t) => {
    const area = document.querySelector("textarea");
    area.value = t;
    area.dispatchEvent(new Event("input", { bubbles: true }));
  }, text);

async function behaviourScenario() {
  console.log("A component's behaviour edited and rebuilt (ROADMAP 4.3)");
  await click(`//button[@role="tab"][normalize-space()="twinova.belt@1"]`);
  const original = await editorText();
  await check("the belt's behaviour, edited to a 500 ms start delay, reports running 500 ms after run", async () => {
    await setEditorText(original.replace("PT := start_delay", "PT := T#500ms"));
    await click(button("Build"));
    await browser.$("//*[normalize-space()='Built: ready to run']").waitForExist({ timeout: 10_000 });
    await click(button("Start"));
    let run = null;
    let running = null;
    for (let scan = 0; scan < 80 && running === null; scan++) {
      await click(button("Step"));
      const time = await virtualSeconds("Virtual time");
      if (run === null && (await bit("belt_a.run")) === "TRUE") run = time;
      if (running === null && (await bit("belt_a.running")) === "TRUE") running = time;
    }
    // run is scanned at 10 ms; the field sees it at 11 ms, and truST's TON reaches Q one cycle before PT (spec
    // 2026-11-09, "Measured"): running at 510 ms, read at the 510 ms scan.
    expectEqual([run, running], [0.01, 0.51], "run, running (s)");
  });
  await screenshot("behaviour-edited");
  await check("a broken behaviour fails Build with its file and line, and the line keeps running", async () => {
    await click(button("Run"));
    await setEditorText(original.replace("    target := speed_ref;", "    target := ;"));
    const before = await virtualSeconds("Virtual time");
    await click(button("Build"));
    await browser.$("//*[normalize-space()='Build failed']").waitForExist({ timeout: 10_000 });
    const shown = await (await browser.$(`//pre[contains(@class,"error-line")]`)).getText();
    if (!shown.includes("components/twinova.belt@1.st:")) throw new Error(`no location in "${shown}"`);
    await sleep(1000);
    const after = await virtualSeconds("Virtual time");
    if (!(after > before)) throw new Error(`virtual time stopped: ${before} -> ${after}`);
    await click(button("Pause"));
  });
  await setEditorText(original);
  await click(button("Build"));
}

session("debug", async () => {
  await lineScenario();
  await runModesScenario();
  await languageScenario();
  await behaviourScenario();
}).catch((e) => {
  console.error(e);
  process.exit(1);
});
