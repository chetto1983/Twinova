// End-to-end run of the open app driven over MCP (spec 2026-11-23-app-mcp-design.md, test plan; plan Task 6): the debug
// app starts through tauri-driver, `twinova-agent` attaches to it through the discovery file, and the TypeScript SDK's
// client, a second implementation of the protocol beside rmcp's, runs every command of the window's registry. After
// each, the window, read through WebDriver, shows the change. The headless agent answers the same client.
//
//   npx tauri build --debug --no-bundle && cargo build -p twinova-agent && node e2e/mcp.mjs

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  app,
  browser,
  check,
  eventually,
  expectEqual,
  out,
  sessions,
  statusSeconds,
  target,
  textOf,
} from "./helpers.mjs";

const AGENT = join(target, "debug", process.platform === "win32" ? "twinova-agent.exe" : "twinova-agent");
const work = mkdtempSync(join(tmpdir(), "twinova-mcp-"));
/** Every command of the window's registry: the run checks each was driven. */
const COMMANDS = [
  "block_sink",
  "breakpoints",
  "build",
  "edit",
  "export_archive",
  "force",
  "hold",
  "import_archive",
  "language",
  "list_examples",
  "monitoring",
  "new_project",
  "open_example",
  "open_file",
  "open_project",
  "pause",
  "plcopen_export",
  "plcopen_import",
  "press",
  "read_source",
  "redo",
  "release",
  "reset",
  "restore_state",
  "resume",
  "run",
  "run_to_cursor",
  "save_project",
  "save_state",
  "select",
  "show",
  "state",
  "step",
  "theme",
  "undo",
  "values",
  "variables_table",
  "view",
  "write",
  "write_source",
];
const driven = new Set();
let mcp;

/** An MCP client of the agent, started with `args`. */
async function agent(args) {
  const client = new Client({ name: "twinova-e2e", version: "1" });
  await client.connect(new StdioClientTransport({ command: AGENT, args, env: { ...process.env }, stderr: "pipe" }));
  return client;
}

/** Tool `name`'s answer as JSON, or its error thrown. */
async function tool(name, args = {}) {
  const result = await mcp.callTool({ name, arguments: args });
  const text = result.content.find((c) => c.type === "text")?.text ?? "";
  if (result.isError) throw new Error(text);
  return result.structuredContent ?? JSON.parse(text);
}

/** One of the window's commands, through `execute`. */
function run(command, params) {
  driven.add(command);
  return tool("execute", params === undefined ? { command } : { command, params });
}

async function refused(command, params) {
  try {
    await run(command, params);
  } catch (e) {
    return e.message;
  }
  throw new Error(`${command} was not refused`);
}

/** Waits until `read()` gives `expected`: the window shows a change a moment after the command answers. */
const shows = (what, read, expected, ms = 10_000) =>
  eventually(async () => expectEqual(await read(), expected, what), what, ms);

const projectName = () =>
  browser.execute(() => document.querySelector('header button[aria-haspopup="menu"] span').textContent);
const treeNames = () =>
  browser.execute(() => [...document.querySelectorAll("section.tree .item-name")].map((s) => s.textContent));
const pressed = (selector) => browser.execute((s) => document.querySelector(s)?.getAttribute("aria-pressed"), selector);
const fileShown = () =>
  browser.execute(() =>
    document.querySelector('[role="tablist"][aria-label="ST sources"] [aria-selected="true"]')?.textContent.trim(),
  );
/** Row `name` of the variables table: its value and whether it is forced. */
const cell = (name) =>
  browser.execute((n) => {
    const rows = [...document.querySelectorAll('[aria-label="Variables"] tbody tr')];
    const value = rows.find((r) => r.querySelector('input[aria-label="Name"]')?.value === n)?.querySelector(".value");
    return value ? [value.textContent.trim(), Boolean(value.querySelector(".forced"))] : null;
  }, name);
const valueOf = async (name) => (await run("values")).find((row) => row.name === name)?.value;
const holdLine = async () => (await run("hold"))?.line ?? null;
/** A batch's step renaming instance `id`. */
const rename = (id) => ({ command: "edit", params: { op: { op: "rename", id, name: "X" } } });
/** Whether the editor shows the glyph margin, where breakpoints are set. */
const margin = () => browser.execute(() => document.querySelector(".monaco-host").editor.getRawOptions().glyphMargin);
const MONITOR = 'button[aria-label="Monitoring (Ctrl+T)"]';
const VARIABLES = 'button[aria-label="Variables"]';

async function theAgentAttaches() {
  await check("the agent attaches to the open app and offers the window's tools", async () => {
    const { tools } = await mcp.listTools();
    const names = tools.map((t) => t.name).toSorted();
    expectEqual(names, ["batch", "describe_command", "execute", "list_commands", "screenshot", "state"], "tools");
    const listed = (await tool("list_commands")).map((c) => c.name);
    expectEqual(listed, COMMANDS, "list_commands");
    const described = await tool("describe_command", { name: "open_example" });
    expectEqual(described.params.required, ["id"], "open_example's schema");
  });
  await check("an unknown command, an unknown parameter and a missing one are refused by name", async () => {
    expectEqual(await refused("fly"), "no command fly: list_commands names them", "unknown command");
    expectEqual(await refused("step", { ms: 5 }), "step takes no ms: describe_command gives its parameters", "ms");
    expectEqual(await refused("open_example", {}), "open_example needs id", "missing id");
  });
}

async function theScene() {
  await check("list_examples and open_example open the Ladder line, which the window shows", async () => {
    expectEqual((await run("list_examples")).length, 7, "examples");
    await run("open_example", { id: "ladder_line" });
    await shows("project", projectName, "Two belts and a chain transfer, in Ladder");
  });
  await check("edit, undo and redo rename an instance in the tree", async () => {
    await run("edit", { op: { op: "rename", id: "belt_a", name: "Belt One" } });
    await shows("renamed", async () => (await treeNames()).includes("Belt One"), true);
    await run("undo");
    await shows("undone", async () => (await treeNames()).includes("Belt A"), true);
    await run("redo");
    await shows("redone", async () => (await treeNames()).includes("Belt One"), true);
    expectEqual((await run("state")).project.can_undo, true, "can undo");
    await run("undo");
  });
  await check("select picks the instance in the tree", async () => {
    await run("select", { id: "belt_b" });
    await shows("selected", () => textOf("section.tree li.on .item-name"), "Belt B");
    await run("select", { id: null });
    await shows("none", () => browser.$("section.tree li.on").isExisting(), false);
  });
  await check("a batch that fails stops and undoes its edits", async () => {
    let failed;
    try {
      await tool("batch", { commands: [rename("belt_a"), rename("nope")] });
    } catch (e) {
      failed = JSON.parse(e.message);
    }
    expectEqual([failed?.failed.index, failed?.undone], [1, 1], "failed and undone");
    await shows("kept", async () => (await treeNames()).includes("Belt A"), true);
  });
}

async function theLadderPane() {
  await check("build builds, as the status bar says", async () => {
    expectEqual((await run("build")).build, "ok", "build");
    await shows("built", () => textOf(".statusbar .build-status"), "Built: ready to run");
  });
  await check("show, open_file and view show the PLC tab, the Ladder file and its rungs", async () => {
    await run("show", { panel: "plc" });
    await shows("PLC tab", () => browser.$("#dock-plc").getAttribute("aria-selected"), "true");
    await run("open_file", { file: "plc/Main.ld" });
    await shows("file", fileShown, "plc/Main.ld");
    await run("view", { view: "ladder" });
    await shows("drawn", () => pressed("button.ladder"), "true");
    await shows("rungs", async () => (await browser.$$("[data-rung]")).length > 0, true);
    await run("view", { view: "text" });
    await shows("text", () => pressed("button.ladder"), "false");
  });
  await check(
    "a Ladder file takes no breakpoint: no glyph margin, no Run to cursor, and the command refused",
    async () => {
      await shows("no glyph margin", margin, false);
      const cursor = await browser.$('button[aria-label="Run to cursor (Ctrl+F10)"]');
      expectEqual(await cursor.isEnabled(), false, "Run to cursor enabled");
      expectEqual(
        await refused("breakpoints", { file: "plc/Main.ld", lines: [30] }),
        "plc/Main.ld: a Ladder file takes no breakpoints yet",
        "breakpoints",
      );
      expectEqual(
        await refused("run_to_cursor", { file: "plc/Main.ld", line: 30 }),
        "plc/Main.ld: a Ladder file takes no breakpoints yet",
        "run_to_cursor",
      );
    },
  );
  await check("variables_table and monitoring set the pane's toggles", async () => {
    await run("variables_table", { on: false });
    await shows("table off", () => pressed(VARIABLES), "false");
    await run("variables_table", { on: true });
    await shows("table on", () => pressed(VARIABLES), "true");
    await run("monitoring", { on: true });
    await shows("monitoring", () => pressed(MONITOR), "true");
  });
  const detected = { program: "Main", path: "detected_in" };
  await check("force, values and release: the table shows the value forced, then lets it go", async () => {
    expectEqual((await run("force", { target: detected, value: "TRUE" }))[0].label, "Main.detected_in", "force");
    await run("step");
    expectEqual(await valueOf("detected_in"), "TRUE", "values");
    await shows("forced", () => cell("detected_in"), ["TRUE", true]);
    expectEqual(await run("release", { target: detected }), [], "forces left");
    await shows("released", async () => (await cell("detected_in"))?.[1], false);
  });
  await check("write sets a variable once, as the table shows", async () => {
    await run("write", { target: { program: "Main", path: "count" }, value: "5" });
    await run("step");
    expectEqual(await valueOf("count"), "5", "values");
    await shows("written", async () => (await cell("count"))?.[0], "5");
  });
  await check("plcopen_export writes the Ladder file, plcopen_import brings it back", async () => {
    const xml = join(work, "main.xml");
    await run("plcopen_export", { target: xml, file: "plc/Main.ld" });
    expectEqual(readFileSync(xml, "utf8").includes("<LD"), true, "an <LD> body");
    const imported = await run("plcopen_import", { source: xml });
    expectEqual([imported.replaced, imported.added], [["plc/Main.ld"], []], "imported");
  });
}

async function theLine() {
  await check(
    "reset, step, save_state, restore_state, run and pause move the line, as the status bar shows",
    async () => {
      expectEqual(await run("reset"), { time_ms: 0 }, "reset");
      expectEqual(await run("step"), { time_ms: 10 }, "step");
      await shows("time", statusSeconds, 0.01);
      await run("save_state");
      await run("step");
      expectEqual(await run("restore_state"), { time_ms: 10 }, "restored");
      await run("run", { speed: null });
      await shows("running", () => textOf(".statusbar .status"), "Running");
      await run("pause");
      await shows("paused", () => textOf(".statusbar .status"), "Ready");
      expectEqual((await run("state")).running, false, "state");
    },
  );
  await check("on the M3 demo, press starts the belt and block_sink blocks the sink", async () => {
    await run("open_example", { id: "demo_m3" });
    await shows("project", projectName, "M3 demo");
    await run("build");
    await run("open_file", { file: "plc/Main.st" });
    await run("press", { address: "%IX1.0" });
    await run("step");
    expectEqual(await valueOf("motor"), "TRUE", "motor");
    await shows("motor", async () => (await cell("motor"))?.[0], "TRUE");
    await run("block_sink", { sink: 0, blocked: true });
    expectEqual((await run("state")).snapshot.sinks[0].blocked, true, "blocked");
  });
  await check("breakpoints, resume and run_to_cursor hold the line where asked, as the status bar says", async () => {
    await shows("an ST file's glyph margin", margin, true);
    expectEqual(await run("breakpoints", { file: "plc/Main.st", lines: [22] }), [{ line: 22 }], "set");
    await run("run");
    await shows("held", holdLine, 22);
    await shows("status", () => textOf(".statusbar .hold"), "HOLD at plc/Main.st:22");
    await run("resume", { how: "over" });
    await shows("over", holdLine, 23);
    await run("breakpoints", { lines: [] });
    await run("run_to_cursor", { line: 27 });
    await shows("cursor", holdLine, 27);
    await run("resume", { how: "continue" });
    await shows("running", holdLine, null);
    await run("pause");
    await run("reset");
  });
  await check("read_source and write_source edit the program, which the next build takes", async () => {
    const text = await run("read_source", { file: "plc/Main.st" });
    expectEqual(text.includes("PROGRAM Main"), true, "read");
    await run("write_source", { file: "plc/Main.st", text: `${text}(* driven over MCP *)\n` });
    await shows("edited", () => browser.$('[role="tab"].tab.edited').isExisting(), true);
    await run("build");
    expectEqual((await run("state")).sources.find((s) => s.file === "plc/Main.st").edited, false, "built");
  });
}

async function theWindow() {
  await check("theme and language change the window", async () => {
    await run("theme", { theme: "light" });
    await shows("light", () => browser.execute(() => document.documentElement.dataset.theme), "light");
    await run("theme", { theme: "dark" });
    await run("language", { language: "it" });
    await shows("Italiano", () => browser.$("header select.lang").getValue(), "it");
    await run("language", { language: "en" });
    await shows("English", () => browser.$("header select.lang").getValue(), "en");
  });
  await check("export_archive, new_project, import_archive, save_project and open_project", async () => {
    const zip = join(work, "demo.zip");
    await run("export_archive", { file: zip });
    expectEqual(existsSync(zip), true, "archive");
    await run("new_project");
    await shows("empty", treeNames, []);
    await run("import_archive", { file: zip });
    await shows("imported", projectName, "M3 demo");
    const folder = join(work, "saved");
    await run("save_project", { folder });
    expectEqual(existsSync(join(folder, "twinova.json")), true, "saved");
    await run("new_project");
    await run("open_project", { folder });
    await shows("opened", projectName, "M3 demo");
    expectEqual((await run("state")).project.folder, folder, "its folder");
  });
  await check("screenshot answers a PNG of the window as drawn", async () => {
    const result = await mcp.callTool({ name: "screenshot", arguments: {} });
    const image = result.content.find((c) => c.type === "image");
    expectEqual(image?.mimeType, "image/png", "mime type");
    const png = Buffer.from(image.data, "base64");
    writeFileSync(join(out, "mcp-screenshot.png"), png);
    expectEqual(png.subarray(1, 4).toString(), "PNG", "signature");
    const size = [png.readUInt32BE(16), png.readUInt32BE(20)];
    const window = await browser.execute(() => [innerWidth, innerHeight].map((n) => Math.round(n * devicePixelRatio)));
    expectEqual(size, window, "the window's size");
  });
  await check("every command of the registry was driven", () => {
    expectEqual(
      COMMANDS.filter((c) => !driven.has(c)),
      [],
      "not driven",
    );
  });
}

async function headless() {
  await check("the headless agent answers the same client, arrays included", async () => {
    expectEqual((await tool("list_examples")).length, 7, "examples");
    await tool("open_example", { id: "sort_line" });
    expectEqual((await tool("step", { ms: 100 })).time_ms, 100, "step");
  });
}

await sessions(async () => {
  console.log("The open app driven over MCP");
  await app("debug", [], async () => {
    mcp = await agent([]);
    try {
      await theAgentAttaches();
      await theScene();
      await theLadderPane();
      await theLine();
      await theWindow();
    } finally {
      await mcp.close();
    }
  });
  console.log("The headless agent");
  mcp = await agent(["--headless"]);
  try {
    await headless();
  } finally {
    await mcp.close();
  }
});
