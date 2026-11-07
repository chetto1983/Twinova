// Fails when the built frontend holds more than one Paraglide runtime. Two copies split the
// locale state: the app sets the language in one, the messages read the other (seen on the
// Windows build on 2026-11-06, in chetto1983/ModbusTool, where this check comes from). The
// runtime's own error text marks each copy.
//
//   cd app && node ../scripts/check-single-runtime.mjs [dist-dir]

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const MARKER = "No locale found";
const assets = join(process.argv[2] ?? "dist", "assets");
const copies = readdirSync(assets)
  .filter((name) => name.endsWith(".js"))
  .reduce((count, name) => count + readFileSync(join(assets, name), "utf8").split(MARKER).length - 1, 0);

if (copies !== 1) {
  console.error(`Expected one Paraglide runtime in ${assets}, found ${copies}`);
  process.exit(1);
}
console.log(`One Paraglide runtime in ${assets}`);
