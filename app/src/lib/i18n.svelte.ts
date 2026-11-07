// Interface language, as in chetto1983/ModbusTool (src/lib/i18n.svelte.ts).
//
// Paraglide's message functions call getLocale() on every call. Pointing it at a Svelte $state
// makes every m.*() in the markup reactive, so the language changes without the document
// reload that Paraglide's own setLocale() does, which would restart the simulation.

import {
  baseLocale,
  locales,
  overwriteGetLocale,
  overwriteSetLocale,
  setLocale,
  toLocale,
  type Locale,
} from "../paraglide/runtime";
import { readPreference, writePreference } from "./preferences";

export { locales, type Locale };

const PREFERENCE = "locale";

/** The saved language, else the first system language we have, else the base one. */
export function initialLocale(saved: string | null, system: readonly string[]): Locale {
  for (const tag of [saved, ...system]) {
    const locale = toLocale(tag) ?? toLocale(tag?.split("-")[0]);
    if (locale) return locale;
  }
  return baseLocale;
}

const i18n = $state({ locale: initialLocale(readPreference(PREFERENCE), navigator.languages) });

function apply(locale: Locale): void {
  i18n.locale = locale;
  document.documentElement.lang = locale;
}

overwriteGetLocale(() => i18n.locale);
overwriteSetLocale((locale) => {
  apply(locale);
  writePreference(PREFERENCE, locale);
});
apply(i18n.locale);

export const currentLocale = (): Locale => i18n.locale;

export const setLanguage = (locale: Locale): void => void setLocale(locale);

/** The language's own name ("English", "Italiano"), so a new locale needs no code. */
export function languageName(locale: Locale): string {
  const name = new Intl.DisplayNames([locale], { type: "language" }).of(locale) ?? locale;
  return name.charAt(0).toLocaleUpperCase(locale) + name.slice(1);
}
