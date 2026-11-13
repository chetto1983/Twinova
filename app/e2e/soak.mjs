// CPU and memory of the release app, phase by phase, on Linux (reads /proc).
//
//   npx tauri build --no-bundle && xvfb-run node e2e/soak.mjs [seconds-per-phase] [project-folder]
//
// Without a folder it runs the bundled demo and presses its Start; with one, the project opened at start.
//
// It samples the app process and its children (the WebKit web and network processes) every 2 s:
// CPU as a percentage of one core, and the resident set (VmRSS).

import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { app, browser, button, click, dock, selectSpeed, sessions, virtualSeconds } from "./helpers.mjs";

const PHASE_S = Number(process.argv[2] ?? 60);
const PROJECT = process.argv[3] ? [`--project=${resolve(process.argv[3])}`] : [];
/** SOAK_MONITOR=1 turns Monitoring on in the PLC pane before the phases; SOAK_FORCE=<variable> also forces that
 *  BOOL variable to the opposite of its value, which attaches the runtime's debug control (spec
 *  2026-11-13-online-monitoring-design.md, acceptance 5). */
const MONITOR = process.env.SOAK_MONITOR === "1";
const FORCE = process.env.SOAK_FORCE;
/** SOAK_PLC_WINDOW=1 opens the PLC pane in its own window first, where SOAK_MONITOR turns Monitoring on (spec
 *  2026-11-13-plc-window-design.md). */
const PLC_WINDOW = process.env.SOAK_PLC_WINDOW === "1";
const SAMPLE_MS = 2000;
const TICKS_PER_S = 100; // USER_HZ, the unit of utime and stime in /proc/<pid>/stat

function stat(pid) {
  const raw = readFileSync(`/proc/${pid}/stat`, "utf8");
  const fields = raw.slice(raw.lastIndexOf(")") + 2).split(" ");
  // After "pid (comm) ": state is field 0, ppid 1, utime 11, stime 12 (proc(5), fields 3, 4, 14, 15).
  return { ppid: Number(fields[1]), ticks: Number(fields[11]) + Number(fields[12]) };
}

function processes() {
  const all = new Map();
  for (const name of readdirSync("/proc")) {
    if (!/^\d+$/.test(name)) continue;
    try {
      const cmd = readFileSync(`/proc/${name}/cmdline`, "utf8").split("\0")[0];
      all.set(Number(name), { cmd: cmd.split("/").pop(), ...stat(name) });
    } catch {
      // the process ended while it was being read
    }
  }
  const main = [...all].find(([, p]) => p.cmd === "twinova");
  if (!main) throw new Error("the twinova process is not running");
  const tree = new Map([main]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const [pid, p] of all) {
      if (tree.has(pid) || !tree.has(p.ppid)) continue;
      tree.set(pid, p);
      grew = true;
    }
  }
  for (const [pid, p] of tree) {
    const rss = /VmRSS:\s+(\d+) kB/.exec(readFileSync(`/proc/${pid}/status`, "utf8"));
    p.rssMb = rss ? Number(rss[1]) / 1024 : 0;
  }
  return tree;
}

/** Samples for `seconds` and reports, per process (name and pid: two windows are two WebKit web processes), the mean
 *  CPU and the RSS at start, end and peak. */
async function phase(name, seconds) {
  const series = new Map();
  let before = processes();
  let t0 = Date.now();
  const end = t0 + seconds * 1000;
  while (Date.now() < end) {
    await sleep(SAMPLE_MS);
    const now = processes();
    const t1 = Date.now();
    for (const [pid, p] of now) {
      const prev = before.get(pid);
      if (!prev) continue;
      const key = `${p.cmd} ${pid}`;
      const s = series.get(key) ?? { cpu: [], rss: [] };
      s.cpu.push(((p.ticks - prev.ticks) / TICKS_PER_S / ((t1 - t0) / 1000)) * 100);
      s.rss.push(p.rssMb);
      series.set(key, s);
    }
    [before, t0] = [now, t1];
  }
  console.log(`${name} (${seconds} s), virtual time ${(await virtualSeconds("Virtual time")).toFixed(1)} s`);
  for (const [key, s] of series) {
    const mean = s.cpu.reduce((a, b) => a + b, 0) / s.cpu.length;
    const [first, last, peak] = [s.rss[0], s.rss.at(-1), Math.max(...s.rss)];
    console.log(
      `  ${key.padEnd(24)} CPU ${mean.toFixed(1).padStart(5)} %  RSS ${first.toFixed(0)} → ${last.toFixed(0)} MB, peak ${peak.toFixed(0)}`,
    );
  }
}

/** Opens the PLC window from the dock, turns Monitoring on there with SOAK_MONITOR, and comes back to the main one. */
async function openPlcWindow() {
  const main = await browser.getWindowHandle();
  await dock("plc");
  await click(button("Open the PLC in a window"));
  await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2, { timeout: 15000 });
  await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== main));
  await browser.$(".monaco-host").waitForExist({ timeout: 15000 });
  if (MONITOR) await click(button("Monitoring (Ctrl+T)"));
  await browser.switchToWindow(main);
  await sleep(1000);
}

sessions(() =>
  app("release", PROJECT, async () => {
    await sleep(3000);
    if (PLC_WINDOW) await openPlcWindow();
    else if (MONITOR) {
      await dock("plc");
      await click(button("Monitoring (Ctrl+T)"));
      await sleep(1000);
    }
    if (FORCE) {
      // Through the variables table and the write/force bar, as a user forces it.
      await browser.execute((name) => {
        const rows = [...document.querySelectorAll('[aria-label="Variables"] tbody tr')];
        const row = rows.find((r) => r.querySelector('input[aria-label="Name"]')?.value === name);
        row.querySelector(".value button").click();
      }, FORCE);
      await click('//div[@role="group"][@aria-label="Value to write or force"]//button[normalize-space()="Force"]');
      await sleep(500);
      console.log(`forced ${FORCE}`);
    }
    await phase("Idle after start", PHASE_S);
    await click(button("Run"));
    if (await browser.$(button("Start")).isExisting()) await click(button("Start"));
    await phase("Run ×1, line running", PHASE_S);
    await click(button("Pause"));
    await selectSpeed("Max");
    await click(button("Run"));
    await phase("Run as fast as possible", PHASE_S);
    await click(button("Pause"));
    await phase("Paused", PHASE_S);
  }),
).catch((e) => {
  console.error(e);
  process.exit(1);
});
