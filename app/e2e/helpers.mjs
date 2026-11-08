// Shared tauri-driver session and UI helpers for the E2E (run.mjs) and the soak (soak.mjs).
// The structure follows chetto1983/ModbusTool e2e/.

import { remote } from "webdriverio";
import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, openSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

export const root = resolve(import.meta.dirname, "..");
const APP = process.platform === "win32" ? "twinova.exe" : "twinova";
/** Failure screenshots and the driver's log; E2E_OUT names a folder CI uploads when a run fails. */
export const out = process.env.E2E_OUT
  ? (mkdirSync(process.env.E2E_OUT, { recursive: true }), process.env.E2E_OUT)
  : mkdtempSync(join(tmpdir(), "twinova-e2e-"));
const driverLog = join(out, "tauri-driver.log");
const results = [];
export let browser;

export async function check(name, fn) {
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

export async function eventually(fn, what, timeoutMs) {
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

export function expectEqual(actual, expected, what) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

// ---------- the UI ----------

export const button = (text) => `//header//button[normalize-space()="${text}"]`;
export const click = async (selector) => (await browser.$(selector)).click();
const text = async (selector) => (await browser.$(selector)).getText();
/** The TRUE/FALSE cell of an I/O row, by its variable name (the rows start with it in every language). */
export const bit = (name) =>
  text(`//li[contains(@class,"row")][.//span[starts-with(normalize-space(),"${name}")]]/span[last()]`);
export const value = (term) => text(`//dt[normalize-space()="${term}"]/following-sibling::dd[1]`);
/** Virtual time "m:ss.cc" in seconds. Text that is not a time throws: a NaN would make every comparison false and
 * let a check pass on nothing (run 42524833751, where the cell read as empty). */
export const virtualSeconds = async (term) => {
  const shown = await value(term);
  const [minutes, seconds] = shown.split(":");
  const total = Number(minutes) * 60 + Number(seconds);
  if (!/^\d+:\d\d\.\d\d$/.test(shown) || !Number.isFinite(total)) throw new Error(`"${shown}" is not a virtual time`);
  return total;
};
/** A header select by its class, which does not change with the language (its aria-label does). */
const choose = async (cls, label) =>
  (await browser.$(`//header//select[contains(concat(" ", @class, " "), " ${cls} ")]`)).selectByVisibleText(label);
export const selectSpeed = (label) => choose("speed", label);
export const selectLanguage = (label) => choose("lang", label);

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

/** Starts the app built in `profile` (debug or release) under tauri-driver, runs `body`, and exits 1 on a failed check. */
export async function session(profile, body) {
  const driver = spawn("tauri-driver", [], { stdio: ["ignore", openSync(driverLog, "w"), openSync(driverLog, "a")] });
  await sleep(2000);
  browser = await remote({
    hostname: "127.0.0.1",
    port: 4444,
    logLevel: "error",
    capabilities: { "tauri:options": { application: join(root, "src-tauri/target", profile, APP) } },
  });
  try {
    await whenStarted();
    await body();
  } finally {
    await browser.deleteSession().catch(() => {});
    driver.kill();
  }
  const failed = results.filter((r) => !r.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length ? 1 : 0);
}
