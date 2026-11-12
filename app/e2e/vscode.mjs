// End-to-end run of the sources saved outside Twinova (spec 2026-11-12-st-vscode-design.md, acceptance 7): a copy of the
// diverter line opened in the real app, run, and its files written as an editor such as VS Code saves them, in place
// and by renaming a temporary over them. The program goes online while the line runs on; a behaviour rebuilds the line
// from zero; edits made in Twinova hold a change until they are reverted.
//
//   npx tauri build --debug --no-bundle && node e2e/vscode.mjs

import { existsSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { join } from "node:path";
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
  virtualSeconds,
  textOf,
} from "./helpers.mjs";
import { setProgram } from "./editing.mjs";

const folder = fixtureCopy("two_belts_diverter");
const main = join(folder, "plc/Main.st");
const belt = join(folder, "components/twinova.belt@1.st");
const original = readFileSync(main, "utf8");
const stopped = original.replace("belt_b_run := TRUE;", "belt_b_run := FALSE; (* saved outside *)");

const notice = () => textOf(`//p[@role="status"]`);
const noticeSays = (words) =>
  eventually(
    async () => {
      const text = await notice();
      if (!text.includes(words)) throw new Error(text);
    },
    words,
    15_000,
  );
const time = () => virtualSeconds("Virtual time");
const tab = (label) => click(`//button[@role="tab"][normalize-space()="${label}"]`);
const editorText = () => browser.execute(() => document.querySelector("textarea").value);

/** Writes `text` to `path` as an editor that saves atomically does: a temporary beside it, renamed over it. */
function renameOver(path, text) {
  writeFileSync(`${path}.tmp`, text);
  renameSync(`${path}.tmp`, path);
}

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("Sources saved outside Twinova reach the running line");
    await check("the line builds, and its folder has the files for VS Code", async () => {
      await eventually(
        async () =>
          expectEqual(
            await browser
              .$("//footer[contains(@class,'statusbar')]/span[normalize-space()='Built: ready to run']")
              .isExisting(),
            true,
            "built",
          ),
        "build",
        10_000,
      );
      expectEqual(existsSync(join(folder, "trust-lsp.toml")), true, "trust-lsp.toml");
      expectEqual(existsSync(join(folder, ".twinova/lsp/field.st")), true, "field.st");
    });

    await selectSpeed("Max");
    await click(button("Run"));
    await eventually(async () => expectEqual((await time()) > 5, true, "running"), "5 s run", 30_000);

    await check("a program saved in place goes online, and the line runs on", async () => {
      const before = await time();
      expectEqual(await bit("belt_b.run"), "TRUE", "belt B before");
      writeFileSync(main, stopped);
      await noticeSays("took the new program online");
      await eventually(async () => expectEqual(await bit("belt_b.run"), "FALSE", "belt B"), "belt B stops", 10_000);
      const after = await time();
      if (after < before) throw new Error(`the time went back from ${before} to ${after}`);
      await tab("plc/Main.st");
      expectEqual((await editorText()).includes("(* saved outside *)"), true, "the tab shows the saved text");
    });

    await check("a program renamed over its file goes online too", async () => {
      renameOver(main, original);
      await eventually(async () => expectEqual(await bit("belt_b.run"), "TRUE", "belt B"), "belt B runs", 15_000);
    });

    await check("a behaviour saved outside rebuilds the line from zero, and Run goes on", async () => {
      const before = await time();
      const block = readFileSync(belt, "utf8");
      writeFileSync(belt, block.replace("(* Twinova belt conveyor", "(* Twinova's belt conveyor"));
      await noticeSays("the line was rebuilt from zero");
      const rebuilt = await time();
      if (rebuilt >= before) throw new Error(`no rebuild: ${before} s, then ${rebuilt} s`);
      await eventually(async () => expectEqual((await time()) > rebuilt, true, "running"), "Run goes on", 15_000);
    });

    await check("an edit made in Twinova holds a change on disk until it is reverted", async () => {
      await tab("plc/Main.st");
      await setProgram(`${original}(* typed in Twinova *)\n`);
      writeFileSync(main, stopped);
      await noticeSays("changed on disk while edited here");
      expectEqual(await bit("belt_b.run"), "TRUE", "nothing taken");
      await click(`//button[normalize-space()="Revert"]`);
      writeFileSync(main, `${stopped}(* saved again *)\n`);
      await eventually(async () => expectEqual(await bit("belt_b.run"), "FALSE", "belt B"), "taken", 15_000);
    });
    await click(button("Pause"));
  }),
);
