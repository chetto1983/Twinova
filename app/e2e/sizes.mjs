// The visual rules of spec 2026-11-26-ux-consolidation-design.md (decision 8, U7) measured on the real app: a disabled
// control's tooltip says why, and in each dock tab of the M3 demo, running, no visible text is below 11 px, no visible
// control below WCAG 2.2's 24 x 24 px target, and the top bar's controls are at least 28 px high. Monaco's own widgets
// are its, not the app's, and are left out.
//
//   npx tauri build --debug --no-bundle && node e2e/sizes.mjs
//
// Needs what e2e/run.mjs needs.

import { browser, button, check, click, dock, eventually, expectEqual, session } from "./helpers.mjs";

/** What breaks a rule in the window as drawn now, one line each. The dock's resize grip spans the dock's whole width:
 *  its target is that strip, kept thin on purpose. A checkbox's target is the label around it. */
const breaches = () =>
  browser.execute(() => {
    const found = [];
    for (const e of document.querySelectorAll("body *")) {
      const r = e.getBoundingClientRect();
      const style = getComputedStyle(e);
      if (r.width === 0 || r.height === 0 || style.visibility === "hidden" || e.closest(".monaco-editor")) continue;
      const said = e.getAttribute("aria-label") || e.getAttribute("placeholder") || e.textContent || "";
      const label = `${e.parentElement?.className} > ${e.tagName.toLowerCase()}.${e.className} "${said.trim().slice(0, 40)}"`;
      if ([...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) && parseFloat(style.fontSize) < 11)
        found.push(`text ${style.fontSize} ${label}`);
      if (!e.matches("button:not(.resize), select, input")) continue;
      const t = ((e.type === "checkbox" && e.closest("label")) || e).getBoundingClientRect();
      if (t.width < 24 || t.height < 24) found.push(`target ${Math.round(t.width)}x${Math.round(t.height)} ${label}`);
      if (e.closest("header.top") && t.height < 28) found.push(`bar ${Math.round(t.height)} px ${label}`);
    }
    return found;
  });

await session("debug", async () => {
  await check("a disabled control says why: nothing to undo on a project just opened", async () =>
    expectEqual(await browser.$(button("Undo")).getAttribute("title"), "Undo: nothing to undo", "title"),
  );
  await click(button("Build"));
  await click(button("Run"));
  for (const tab of ["parts", "plc", "problems", "watch"]) {
    await check(`${tab}: text at least 11 px, controls at least 24 px, the bar's 28 px`, async () => {
      await dock(tab);
      await eventually(async () => expectEqual(await breaches(), [], "breaches"), tab, 5000);
    });
  }
});
