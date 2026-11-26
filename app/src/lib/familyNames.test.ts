import { readdirSync } from "node:fs";
import { afterEach, describe, expect, it } from "vitest";
import { familyName, familyOf, namedFamilies } from "./familyNames";
import { setLanguage } from "./i18n.svelte";

/** The library's families, from its definition files: `twinova.belt@1.json` is `belt`. */
const library = readdirSync("src-tauri/library/components")
  .filter((f) => f.endsWith(".json"))
  .map((f) => familyOf(f.replace(/\.json$/, "")));

describe("component family names", () => {
  afterEach(() => setLanguage("en"));

  it("names every family of the library, and no other", () => {
    expect(library.toSorted()).toEqual(namedFamilies().toSorted());
  });

  it("names a family by key or id, in the window's language, without its version", () => {
    expect(familyName("twinova.sensor_diffuse@1")).toBe("Photo-eye");
    expect(familyName("twinova.sensor_diffuse")).toBe("Photo-eye");
    setLanguage("it");
    expect(familyName("twinova.sensor_diffuse@1")).toBe("Fotocellula");
  });

  it("falls back to the family's own words for one it does not name", () => {
    expect(familyName("vendor.spiral_conveyor@2")).toBe("spiral conveyor");
  });
});
