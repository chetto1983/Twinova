// The 4.2 acceptance through the real app (plan 2026-11-08-component-format.md, Task 5): open each valid fixture
// from the command line (spec 2026-11-08-project-format-design.md, decision 8), inspect it, save it through the
// menu's Save and open the saved folder again; start on each invalid fixture and check that its diagnostic is shown
// and the M3 demo stays usable. tauri-driver cannot drive the native dialogs, so Save as, Export and Import are
// left to the command tests.
//
//   npx tauri build --debug --no-bundle && node e2e/project.mjs
//
// Needs what e2e/run.mjs needs.

import { cpSync, mkdtempSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { app, browser, check, click, eventually, expectEqual, root, sessions } from "./helpers.mjs";

const fixtures = join(root, "src-tauri/project/tests/fixtures");
const VALID = ["demo_m3", "two_belts_diverter"];
const INVALID = JSON.parse(readFileSync(join(fixtures, "invalid.json"), "utf8"));

const textOf = async (selector) => (await browser.$(selector)).getText();
const treeNames = () =>
  browser.execute(() =>
    [...document.querySelectorAll("section.tree button > span:first-child")].map((span) => span.textContent),
  );
const notice = () => textOf(`//p[@role="status"]`);
const projectButton = `//header//button[@aria-haspopup="menu"]`;

async function shows(selector, what) {
  await eventually(async () => expectEqual(await browser.$(selector).isExisting(), true, what), what, 10_000);
}

/** The manifest and every file it lists: what a save writes and what the fixture holds. */
function projectFiles(folder) {
  const manifest = JSON.parse(readFileSync(join(folder, "twinova.json"), "utf8"));
  return ["twinova.json", ...Object.keys(manifest.files)].map((path) => [path, readFileSync(join(folder, path))]);
}

async function inspectsTheScene(scene) {
  const names = scene.instances.map((instance) => instance.name);
  await check("the tree lists every instance of scene.json", () =>
    eventually(async () => expectEqual(await treeNames(), names, "tree"), "tree", 10_000),
  );
  const wired = scene.instances.find((instance) => Object.keys(instance.signals ?? {}).length > 0);
  await check(`the inspector shows ${wired.name}'s addresses`, async () => {
    await click(`//section[contains(@class,"tree")]//button[span[1][normalize-space()="${wired.name}"]]`);
    const shown = await textOf(`//section[contains(@class,"inspector")]`);
    for (const address of Object.values(wired.signals)) {
      if (!shown.includes(address)) throw new Error(`${address} is not in "${shown}"`);
    }
  });
}

async function roundTrip(name) {
  console.log(`Open, save and reopen ${name}`);
  const source = join(fixtures, "valid", name);
  const folder = join(mkdtempSync(join(tmpdir(), "twinova-project-")), name);
  cpSync(source, folder, { recursive: true });
  // Re-indented as an editor would, without touching a value (JSON.stringify would write 2.0 as 2): the open reports
  // it and the save writes the canonical form back.
  const authored = readFileSync(join(folder, "scene.json"), "utf8");
  writeFileSync(
    join(folder, "scene.json"),
    authored.replace(/^ +/gm, (indent) => indent + indent),
  );
  const scene = JSON.parse(authored);

  await app("debug", [`--project=${folder}`], async () => {
    await inspectsTheScene(scene);
    await check("the hand edit is reported as changed outside Twinova", async () => {
      const text = await notice();
      if (!text.includes("Changed outside Twinova since the last save: scene.json")) throw new Error(text);
    });
    await check("Save writes to the opened folder without a dialog", async () => {
      await click(projectButton);
      await click(`//button[@role="menuitem"][normalize-space()="Save"]`);
      await eventually(
        async () => {
          if (!(await notice()).includes(`Saved to ${folder}`)) throw new Error(await notice());
        },
        "saved",
        10_000,
      );
    });
  });
  await check("the saved folder holds the fixture's bytes", () => {
    const saved = Object.fromEntries(projectFiles(folder));
    for (const [path, bytes] of projectFiles(source)) {
      if (!saved[path]?.equals(bytes)) throw new Error(`${path} differs from the fixture`);
    }
  });
  // Windows has no file modes: a renamed file takes the folder's ACL.
  if (process.platform !== "win32") {
    await check("the saved files keep their mode", () => {
      for (const [path] of projectFiles(source)) {
        const mode = (folderPath) => (statSync(join(folderPath, path)).mode & 0o777).toString(8);
        expectEqual(mode(folder), mode(source), path);
      }
    });
  }
  await app("debug", [`--project=${folder}`], async () => {
    await inspectsTheScene(scene);
    await check("the saved folder reopens with nothing changed outside", async () => {
      const text = await notice();
      if (text.includes("Changed outside")) throw new Error(text);
    });
  });
}

async function refused(name, diagnostic) {
  console.log(`Start on ${name}`);
  await app("debug", [`--project=${join(fixtures, "invalid", name)}`], async () => {
    await check(`${name} is refused with its diagnostic`, () =>
      eventually(
        async () => {
          const shown = await textOf(`//pre[contains(@class,"error-line")]`);
          if (!shown.includes(diagnostic)) throw new Error(`"${diagnostic}" is not in "${shown}"`);
        },
        "diagnostic",
        10_000,
      ),
    );
    await check("the M3 demo stays current and built", async () => {
      const current = await textOf(projectButton);
      if (!current.startsWith("M3 demo")) throw new Error(`the current project is "${current}"`);
      await shows("//*[normalize-space()='Built: ready to run']", "build");
    });
  });
}

sessions(async () => {
  for (const name of VALID) await roundTrip(name);
  for (const [name, diagnostic] of Object.entries(INVALID)) await refused(name, diagnostic);
}).catch((e) => {
  console.error(e);
  process.exit(1);
});
