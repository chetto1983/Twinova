// What an element's operand may be, from truST's completion (`ladder_completion`): the names that fit its role, the
// ones of the type it takes first. The order is Open Industry Project's (`src/plc/ladder/ladderComplete.ts` at 1e7488c,
// MIT; THIRD_PARTY_NOTICES.md): a contact or a coil takes a bit, so the BOOLs come first, and nothing is hidden, since
// a program may rightly name something else. Its second tier, a structure holding a wanted member, waits for members:
// truST offers none after a dot (spec 2026-11-13-ladder-editor-design.md, decision 4, amended 2026-11-14).
import type { Completion } from "../backend";
import type { Declaration } from "../monitor";
import type { Element } from "./model";

/** What an operand names: a variable for a contact or a coil; a block instance or a function for a box. */
export type Role = "bit" | "box";

export const roleOf = (el: Element | null): Role | null =>
  el?.kind === "contact" || el?.kind === "coil" ? "bit" : el?.kind === "box" ? "box" : null;

/** The text an element's operand field holds: a contact's or a coil's variable, a box's head. */
export const operandOf = (el: Element | null): string =>
  (el?.kind === "contact" || el?.kind === "coil" ? el.operand : el?.kind === "box" ? el.head : null) ?? "";

const KINDS: Record<Role, ReadonlySet<string>> = {
  bit: new Set(["variable"]),
  box: new Set(["variable", "function", "function_block"]),
};

/** The type a declared type holds, upper case: an array's element (`ARRAY [0..9] OF BOOL` is `BOOL`), a string's
 *  family without its length. */
export function elementType(type: string): string {
  return (/\bOF\s+(.+)$/i.exec(type)?.[1] ?? type).replace(/\[.*$/, "").trim().toUpperCase();
}

const isBool = (c: Completion): boolean => elementType(c.detail ?? "") === "BOOL";

/** The names truST offers for `typed` that fit `role`, each once, starting as typed; for a bit the BOOLs first, each
 *  group in truST's order. */
export function candidates(found: readonly Completion[], typed: string, role: Role): string[] {
  const start = typed.toLowerCase();
  const fit = found.filter((c) => KINDS[role].has(c.kind) && c.label.toLowerCase().startsWith(start));
  const names = [...new Map(fit.map((c) => [c.label, c])).values()];
  if (role === "box") return names.map((c) => c.label);
  return [...names.filter(isBool), ...names.filter((c) => !isBool(c))].map((c) => c.label);
}

/** Whether Auto Declare can declare `text`: one IEC identifier (no two underscores in a row), not a BOOL literal. */
export const declarable = (text: string): boolean =>
  /^[A-Za-z_][A-Za-z0-9_]*$/.test(text) && !/__/.test(text) && !/^(TRUE|FALSE)$/i.test(text);

/** The row Auto Declare adds for a contact's or a coil's undeclared operand `name` of program `pou`: a BOOL. */
export const boolRow = (pou: string, name: string): Declaration => ({
  pou,
  name,
  class: "VAR",
  qualifier: null,
  type_name: "BOOL",
  address: null,
  initial: null,
  comment: null,
});
