import { remote } from "webdriverio";
import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { createServer } from "vite";

const output = resolve(process.env.TEMP, "twinova-ui-review");
mkdirSync(output, { recursive: true });
const server = process.argv.includes("--reuse")
  ? null
  : await createServer({ server: { host: "127.0.0.1", port: 1421 } });
await server?.listen();
const browser = await remote({
  logLevel: "error",
  capabilities: {
    browserName: "msedge",
    "ms:edgeOptions": { args: ["--headless=new", "--window-size=1440,900", "--disable-gpu"] },
  },
});
try {
  await browser.url("http://127.0.0.1:1421/e2e/ui-preview.html");
  await browser.$(".stage canvas").waitForExist({ timeout: 60000 });
  await browser.waitUntil(async () => (await browser.$$(".preview img")).length >= 5, {
    timeout: 60000,
    timeoutMsg: "3D thumbnails did not load",
  });
  await browser.saveScreenshot(resolve(output, "01-workspace.png"));
  await browser.$('.tree .item[title^="Belt A"]').click();
  await browser.$(".inspector .triple").waitForExist();
  await browser.saveScreenshot(resolve(output, "02-inspector.png"));
  await browser.$('input[aria-label="Filter properties…"]').setValue("speed");
  if (
    !(await browser.$('input[aria-label="speed"]').isDisplayed()) ||
    (await browser.$('input[aria-label="length"]').isExisting())
  )
    throw new Error("Inspector filtering failed");
  await browser.$('input[aria-label="Filter properties…"]').click();
  await browser.keys(["Control", "a"]);
  await browser.keys("Backspace");
  await browser.$('input[aria-label="length"]').waitForExist();
  await browser.$('input[aria-label="speed"]').setValue("0.75");
  await browser.keys("Tab");
  await browser.waitUntil(async () => (await browser.$('input[aria-label="speed"]').getValue()) === "0.75");
  const speedHandle = await browser.$('button[aria-label="Adjust speed"]');
  await browser
    .action("pointer")
    .move({ origin: speedHandle })
    .down()
    .move({ origin: speedHandle, x: 20, duration: 200 })
    .up()
    .perform();
  await browser.waitUntil(async () => (await browser.$('input[aria-label="speed"]').getValue()) === "0.95");
  await browser.$('input[aria-label="speed"]').click();
  await browser.keys(["Shift", "ArrowUp"]);
  await browser.waitUntil(async () => (await browser.$('input[aria-label="speed"]').getValue()) === "0.951");
  await browser.$('input[aria-label="speed"]').click();
  await browser.keys(["Control", "a"]);
  await browser.keys("999");
  await browser.keys("Escape");
  if ((await browser.$('input[aria-label="speed"]').getValue()) !== "0.951")
    throw new Error(`Escape did not cancel edit: ${await browser.$('input[aria-label="speed"]').getValue()}`);
  await browser.saveScreenshot(resolve(output, "02-inspector.png"));
  await browser.$(".inspector").saveScreenshot(resolve(output, "02-numeric-inspector.png"));
  await browser.$('button[aria-label="Lock editing Belt A"]').click();
  if (await browser.$('input[aria-label="speed"]').isEnabled()) throw new Error("Locked inspector is editable");
  await browser.$('button[aria-label="Unlock editing Belt A"]').click();
  await browser.$('button[aria-label="Hide Belt A"]').click();
  if (!(await browser.$('button[aria-label="Show Belt A"]').isExisting())) throw new Error("Visibility toggle failed");
  await browser.$('button[aria-label="Show Belt A"]').click();
  await browser.$("#dock-plc").click();
  await browser.$("textarea").waitForDisplayed();
  await browser.saveScreenshot(resolve(output, "03-plc.png"));
  await browser.$("#dock-watch").click();
  await browser.$(".timing").waitForDisplayed();
  await browser.saveScreenshot(resolve(output, "04-watch.png"));
  await browser.$('button[aria-label="Collapse panel"]').click();
  if (await browser.$(".timing").isDisplayed()) throw new Error("Dock did not collapse");
  await browser.$('button[aria-label="Expand panel"]').click();
  const before = (await browser.$(".dock").getSize()).height;
  await browser.$('button[aria-label="Resize panel"]').click();
  await browser.keys("ArrowUp");
  if ((await browser.$(".dock").getSize()).height <= before) throw new Error("Keyboard resize failed");
  await browser.keys("ArrowDown");
  await browser.$(".lang").selectByAttribute("value", "it");
  await browser.$("#dock-parts").click();
  await browser.$(".top .utilities button:last-child").click();
  await browser.saveScreenshot(resolve(output, "05-light-it.png"));
  await browser.setWindowSize(1100, 700);
  await browser.saveScreenshot(resolve(output, "06-compact.png"));
  const layout = await browser.execute(() => ({
    overflow: document.documentElement.scrollWidth > window.innerWidth,
    canvas: document.querySelector(".stage canvas").getBoundingClientRect().toJSON(),
    images: document.querySelectorAll(".preview img").length,
  }));
  if (layout.overflow || layout.canvas.width < 250 || layout.canvas.height < 180)
    throw new Error(JSON.stringify(layout));
  console.log(JSON.stringify({ output, layout }));
} finally {
  await browser.deleteSession();
  await server?.close();
}
