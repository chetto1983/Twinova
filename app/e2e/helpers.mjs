// Shared tauri-driver session and UI helpers for the E2E (run.mjs) and the soak (soak.mjs).
// The structure follows chetto1983/ModbusTool e2e/.

import { remote } from "webdriverio";
import { spawn } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, openSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

export const root = resolve(import.meta.dirname, "..");

/** A copy of valid fixture `name` in a new temporary folder. Opening a folder writes VS Code's files into it
 *  (src-tauri/src/lsp_files.rs), and the corpus checks that a fixture holds only its project's files. */
export function fixtureCopy(name) {
  const folder = join(mkdtempSync(join(tmpdir(), "twinova-fixture-")), name);
  cpSync(join(root, "src-tauri/project/tests/fixtures/valid", name), folder, { recursive: true });
  return folder;
}
const APP = process.platform === "win32" ? "twinova.exe" : "twinova";
/** Failure screenshots and the driver's log; E2E_OUT names a folder CI uploads when a run fails. */
export const out = process.env.E2E_OUT
  ? (mkdirSync(process.env.E2E_OUT, { recursive: true }), process.env.E2E_OUT)
  : mkdtempSync(join(tmpdir(), "twinova-e2e-"));
const driverLog = join(out, "tauri-driver.log");
const driverPort = Number(process.env.E2E_PORT ?? 4444);
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

export const button = (text) => `//button[@aria-label="${text}" or normalize-space()="${text}"]`;
export const dock = async (tab) => {
  const target = await browser.$(`#dock-${tab}`);
  if ((await target.getAttribute("aria-selected")) !== "true") await target.click();
};
export const click = async (selector) => {
  const element = await browser.$(selector);
  if ((await element.isExisting()) && !(await element.isDisplayed())) {
    const panel = await browser.execute((node) => node.closest('[role="tabpanel"]')?.id, element);
    if (panel) await dock(panel.replace("panel-", ""));
  }
  await element.click();
};
/** The PLC editor's open tab, read and replaced through the Monaco editor its host carries (StEditor.svelte; plan
 *  2026-11-12-st-editor.md, decision 1): WebKitWebDriver drops typed spaces, so the E2E does not type. */
export async function editorText() {
  await dock("plc");
  return browser.execute(() => document.querySelector(".monaco-host").editor.getValue());
}
export async function setEditorText(text) {
  await dock("plc");
  await browser.execute((t) => document.querySelector(".monaco-host").editor.setValue(t), text);
}
/** The text an element shows: its rendered text (`innerText`, trimmed, as WebDriver's Get Element Text) while it is
 *  displayed, else "". WebKitWebDriver's getText returns "" for the text of any `overflow: hidden` element, though it is
 *  on screen (measured 2026-11-12: the status bar's notice and the project button's name read empty, and read whole
 *  once the element is `overflow: visible` or `clip`); msedgedriver reads them. */
export async function textOf(selector) {
  const element = await browser.$(selector);
  if (!(await element.isDisplayed())) return "";
  return browser.execute((e) => e.innerText.trim(), element);
}
/** The value cell of an I/O row, by its exact name: a scene signal (`belt_a.run`, the same in every language) or M3's
 * jam. Exact, because `belt_a.run` is also the start of `belt_a.running`. */
export const bit = async (name) => {
  await dock("watch");
  return textOf(`//li[contains(@class,"row")][.//span[normalize-space()="${name}"]]/span[last()]`);
};
/** Several signals' values, read one after the other: WebKitWebDriver segfaulted twice in five runs of code_sort.mjs
 *  while three reads went in parallel at Max speed, each opening the Watch pane first (dmesg, 2026-11-12). A
 *  session's commands are meant to come one at a time. */
export async function bits(names) {
  const values = [];
  for (const name of names) values.push(await bit(name));
  return values;
}
export const value = async (term) => {
  await dock("watch");
  return textOf(`//dt[normalize-space()="${term}"]/following-sibling::dd[1]`);
};
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
    await selectLanguage("English");
  } catch (e) {
    await browser.saveScreenshot(join(out, "not-started.png")).catch(() => {});
    console.error("tauri-driver output (last 4 KB):\n" + readFileSync(driverLog, "utf8").slice(-4096));
    throw e;
  }
}

/** Starts tauri-driver, runs `body`, which starts the app with `app` as often as it needs, and exits 1 on a failed
 * check. */
export async function sessions(body) {
  const driver = spawn("tauri-driver", ["--port", String(driverPort), "--native-port", String(driverPort + 1)], {
    stdio: ["ignore", openSync(driverLog, "w"), openSync(driverLog, "a")],
  });
  await sleep(2000);
  try {
    if (driver.exitCode !== null) throw new Error(`tauri-driver exited: ${readFileSync(driverLog, "utf8")}`);
    await body();
  } finally {
    driver.kill();
  }
  const failed = results.filter((r) => !r.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length ? 1 : 0);
}

/** Starts the app built in `profile` (debug or release) with the command-line `args`, runs `body` and closes it. */
export async function app(profile, args, body) {
  const target = process.env.CARGO_TARGET_DIR ? resolve(process.env.CARGO_TARGET_DIR) : join(root, "src-tauri/target");
  browser = await remote({
    hostname: "127.0.0.1",
    port: driverPort,
    logLevel: "error",
    capabilities: { "tauri:options": { application: join(target, profile, APP), args } },
  });
  try {
    await whenStarted();
    await body();
  } finally {
    await browser.deleteSession().catch(() => {});
  }
}

/** One run of the app without arguments. */
export const session = (profile, body) => sessions(() => app(profile, [], body));

// ---------- monitoring (monitor.mjs, plc_window.mjs) ----------

export const BAR = `//div[@role="group"][@aria-label="Value to write or force"]`;

/** Clicks row `name`'s value, which opens the write and force bar. */
export const pickRow = (name) =>
  browser.execute((n) => {
    const rows = [...document.querySelectorAll('[aria-label="Variables"] tbody tr')];
    const row = rows.find((r) => r.querySelector('input[aria-label="Name"]')?.value === n);
    row.querySelector(".value button").click();
  }, name);

/** Sets an input and tells Svelte, as typing would: `input` for a bound value, `change` for a committed one. */
export const setInput = (selector, text, event) =>
  browser.execute(
    (s, t, e) => {
      const input = document.evaluate(s, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
      input.value = t;
      input.dispatchEvent(new Event(e, { bubbles: true }));
    },
    selector,
    text,
    event,
  );

/** The values drawn in the code: each as `line class text`. */
export const drawn = () =>
  browser.execute(() =>
    document
      .querySelector(".monaco-host")
      .editor.getModel()
      .getAllDecorations()
      .filter((d) => d.options.after?.content)
      .map((d) => `${d.range.startLineNumber} ${d.options.after.inlineClassName} ${d.options.after.content.trim()}`),
  );
