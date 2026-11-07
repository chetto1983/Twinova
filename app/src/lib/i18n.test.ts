import { afterEach, describe, expect, it, vi } from "vitest";
import { m } from "../paraglide/messages.js";
import { currentLocale, initialLocale, languageName, locales, setLanguage } from "./i18n.svelte";

afterEach(() => {
  setLanguage("en");
  vi.unstubAllGlobals();
});

describe("initialLocale", () => {
  it("prefers the saved language", () => {
    expect(initialLocale("it", ["en-US"])).toBe("it");
  });

  it("falls back to the first system language available, by its language subtag", () => {
    expect(initialLocale(null, ["fr-FR", "it-IT", "en"])).toBe("it");
    expect(initialLocale("xx", ["EN-gb"])).toBe("en");
  });

  it("ends at the base language", () => {
    expect(initialLocale(null, ["fr", "de-DE"])).toBe("en");
    expect(initialLocale(null, [])).toBe("en");
  });
});

describe("setLanguage", () => {
  it("switches the messages without a reload, and remembers the choice", () => {
    expect(m.built_ok()).toBe("Built: ready to run");
    setLanguage("it");
    expect(currentLocale()).toBe("it");
    expect(m.built_ok()).not.toBe("Built: ready to run");
    expect(document.documentElement.lang).toBe("it");
    expect(localStorage.getItem("locale")).toBe("it");
  });
});

describe("languageName", () => {
  it("names every language in itself, capitalised", () => {
    expect(locales.map(languageName)).toEqual(["English", "Italiano"]);
  });

  it("falls back to the tag when the runtime has no name for it", () => {
    vi.stubGlobal("Intl", {
      DisplayNames: class {
        of = () => undefined;
      },
    });
    expect(languageName("it")).toBe("It");
  });
});
