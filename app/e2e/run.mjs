// End-to-end run of the real app through tauri-driver, on Linux (WebKitWebDriver) and Windows
// (msedgedriver matching WebView2). The structure and helpers follow chetto1983/ModbusTool e2e/.
// It replays the M1 acceptance that was first done by hand with xdotool
// (docs/superpowers/verification/2026-11-07-tauri-shell.md).
//
//   npx tauri build --debug --no-bundle && node e2e/run.mjs [screenshot-dir]
//
// Needs: tauri-driver and the platform's WebDriver; on Linux an X display (xvfb-run).

import { remote } from "webdriverio";
import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, openSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const root = resolve(import.meta.dirname, "..");
const APP = process.platform === "win32" ? "twinova.exe" : "twinova";
const shots = process.argv[2] ? resolve(process.argv[2]) : null;
/** Failure screenshots and the driver's log; E2E_OUT names a folder CI uploads when a run fails. */
const out = process.env.E2E_OUT
  ? (mkdirSync(process.env.E2E_OUT, { recursive: true }), process.env.E2E_OUT)
  : mkdtempSync(join(tmpdir(), "twinova-e2e-"));
const driverLog = join(out, "tauri-driver.log");
const results = [];
let browser;

async function check(name, fn) {
  try {
    await fn();
    results.push({ name, ok: true });
    console.log(`  ✓ ${name}`);
  } catch (e) {
    results.push({ name, ok: false });
    const shot = join(out, `failure-${results.length}.png`);
    console.log(`  ✗ ${name}\n      ${e.message}\n      screenshot: ${shot}`);
    await browser.saveScreenshot(shot).catch(() => {});
  }
}

async function eventually(fn, what, timeoutMs) {
  const end = Date.now() + timeoutMs;
  let last;
  while (Date.now() < end) {
    try {
      return await fn();
    } catch (e) {
      last = e;
      await sleep(150);
    }
  }
  throw new Error(`${what}: ${last?.message}`);
}

function expectEqual(actual, expected, what) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

async function screenshot(name) {
  if (!shots) return;
  mkdirSync(shots, { recursive: true });
  await browser.saveScreenshot(join(shots, `${name}.png`));
}

// ---------- the UI ----------

const button = (text) => `//header//button[normalize-space()="${text}"]`;
const click = async (selector) => (await browser.$(selector)).click();
const text = async (selector) => (await browser.$(selector)).getText();
/** The TRUE/FALSE cell of an I/O row, by its variable name (the rows start with it in every language). */
const bit = (name) =>
  text(`//li[contains(@class,"row")][.//span[starts-with(normalize-space(),"${name}")]]/span[last()]`);
const value = (term) => text(`//dt[normalize-space()="${term}"]/following-sibling::dd[1]`);
/** Virtual time "m:ss.cc" in seconds. */
const virtualSeconds = async (term) => {
  const [minutes, seconds] = (await value(term)).split(":");
  return Number(minutes) * 60 + Number(seconds);
};

async function whenStarted() {
  try {
    await browser.waitUntil(async () => Boolean(await browser.execute(() => document.documentElement.dataset.theme)), {
      timeout: 30_000,
    });
  } catch (e) {
    await browser.saveScreenshot(join(out, "not-started.png")).catch(() => {});
    console.error("tauri-driver output (last 4 KB):\n" + readFileSync(driverLog, "utf8").slice(-4096));
    throw e;
  }
}

async function lineScenario() {
  console.log("The conveyor driven by the default program");
  await check("the default program builds on start", () =>
    eventually(
      async () =>
        expectEqual(await browser.$("//*[normalize-space()='Built: ready to run']").isExisting(), true, "status"),
      "build",
      10_000,
    ),
  );

  await click(button("Run"));
  await click(button("Start"));
  await check("Start latches the motor", () =>
    eventually(async () => expectEqual(await bit("motor"), "TRUE", "motor"), "motor", 5000),
  );
  await check("parts ride the belt and are delivered", () =>
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
    const [v0, t0] = [await virtualSeconds("Virtual time"), Date.now()];
    await sleep(5000);
    const [v1, t1] = [await virtualSeconds("Virtual time"), Date.now()];
    const ratio = (v1 - v0) / ((t1 - t0) / 1000);
    console.log(`      ${(v1 - v0).toFixed(2)} s virtual in ${((t1 - t0) / 1000).toFixed(2)} s wall`);
    if (ratio < 0.9 || ratio > 1.1) throw new Error(`virtual/wall ratio ${ratio.toFixed(3)}`);
  });

  await click(`//label[normalize-space()="Outlet blocked"]/input`);
  await check("a part held at a blocked outlet covers the photocell, trips the jam and stops the motor", () =>
    // The leading part stops under the photocell; the timeout leaves room for a part still entering.
    eventually(
      async () => {
        expectEqual([await bit("sensor"), await bit("jam"), await bit("motor")], ["TRUE", "TRUE", "FALSE"], "I/O");
      },
      "jam",
      40_000,
    ),
  );
  await screenshot("jam");
}

async function languageScenario() {
  console.log("Interface language");
  await check("Italian applies at once and the run goes on (no reload)", async () => {
    const before = await virtualSeconds("Virtual time");
    await (await browser.$("//header//select")).selectByVisibleText("Italiano");
    await browser.$(`//h2[normalize-space()="Ingressi e uscite"]`).waitForExist({ timeout: 3000 });
    expectEqual(await browser.execute(() => document.documentElement.lang), "it", "lang");
    const after = await virtualSeconds("Tempo virtuale");
    if (after < before) throw new Error(`virtual time went back: ${before} -> ${after}`);
  });
  await screenshot("italian");
  await (await browser.$("//header//select")).selectByVisibleText("English");
}

async function main() {
  const driver = spawn("tauri-driver", [], { stdio: ["ignore", openSync(driverLog, "w"), openSync(driverLog, "a")] });
  await sleep(2000);
  browser = await remote({
    hostname: "127.0.0.1",
    port: 4444,
    logLevel: "error",
    capabilities: { "tauri:options": { application: join(root, "src-tauri/target/debug", APP) } },
  });
  try {
    await whenStarted();
    await lineScenario();
    await languageScenario();
  } finally {
    await browser.deleteSession().catch(() => {});
    driver.kill();
  }
  const failed = results.filter((r) => !r.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length ? 1 : 0);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
