// The box picker's catalogue (`ladder_catalog`, truST's own lists) as the picker shows it: filtered, grouped by the
// table of IEC 61131-3 Ed.3 a standard box belongs to, a new instance named, and the call it starts from. Ported in
// part from nautilus (`tools/vscode-iec/webview-ui/src/LdBlockPicker.svelte` at 2db093e, Apache-2.0, Copyright 2026
// Joy Automation; THIRD_PARTY_NOTICES.md): the filter's order, a free instance name, the call's pins editable before
// the insert.
import type { Element, Pin } from "./model";

export interface CatalogPin {
  name: string;
  type_name: string;
  bool: boolean;
}

export interface CatalogEntry {
  name: string;
  block: boolean;
  own: boolean;
  inputs: CatalogPin[];
  outputs: CatalogPin[];
  variadic: boolean;
  table: number | null;
}

/** The picker's groups, in its order: the project's own first, then the standard library as IEC's tables run. */
export const GROUPS = [
  "project",
  "timer",
  "counter",
  "edge",
  "bistable",
  "comparison",
  "math",
  "bitwise",
  "selection",
  "string",
  "time",
  "conversion",
  "other",
] as const;
export type Group = (typeof GROUPS)[number];

const BY_TABLE: Record<number, Group> = {
  22: "conversion",
  28: "math",
  29: "math",
  30: "bitwise",
  31: "bitwise",
  32: "selection",
  33: "comparison",
  34: "string",
  35: "time",
  36: "time",
  43: "bistable",
  44: "edge",
  45: "counter",
  46: "timer",
};

export const groupOf = (e: CatalogEntry): Group => (e.own ? "project" : (BY_TABLE[e.table ?? 0] ?? "other"));

/** The entries `filter` finds, nautilus's order: the names it starts first, then the ones it is inside. */
export function filtered(entries: readonly CatalogEntry[], filter: string): CatalogEntry[] {
  const t = filter.trim().toLowerCase();
  if (t === "") return [...entries];
  const starts = entries.filter((e) => e.name.toLowerCase().startsWith(t));
  const inside = entries.filter((e) => !e.name.toLowerCase().startsWith(t) && e.name.toLowerCase().includes(t));
  return [...starts, ...inside];
}

/** `entries` by group, in the picker's order, empty groups left out. */
export function grouped(entries: readonly CatalogEntry[]): [Group, CatalogEntry[]][] {
  return GROUPS.map((g): [Group, CatalogEntry[]] => [g, entries.filter((e) => groupOf(e) === g)]).filter(
    ([, list]) => list.length > 0,
  );
}

/** A name no declaration in `taken` has yet, for a new instance of `type`: `TON_1`, then `TON_2`… */
export function freeInstance(type: string, taken: readonly string[]): string {
  const used = new Set(taken.map((t) => t.toLowerCase()));
  let n = 1;
  while (used.has(`${type}_${n}`.toLowerCase())) n++;
  return `${type}_${n}`;
}

/** The pins a new box starts from, as its call writes them, to fill before the insert. A block sits on the rung by its
 *  first BOOL input and output. A comparison starts the rung by its OUT; another function sits on it by EN and ENO,
 *  its result going where `OUT =>` says. */
export function template(e: CatalogEntry): string {
  if (e.block) {
    const rung = [e.inputs.find((p) => p.bool), e.outputs.find((p) => p.bool)];
    return rung.flatMap((p) => (p ? [p.name] : [])).join(", ");
  }
  const inputs = e.inputs.map((p) => `${p.name} := `);
  if (groupOf(e) === "comparison") return [...inputs, "OUT"].join(", ");
  return ["EN", ...inputs, "OUT => ", "ENO"].join(", ");
}

const pinText = (p: Pin): string => (p.value === null ? p.name : `${p.name} ${p.out ? "=>" : ":="} ${p.value}`);

/** A box as its call is written (`print.rs`): `T1(IN, PT := T#2s, Q)`. */
export const callOf = (el: Extract<Element, { kind: "box" }>): string =>
  `${el.head ?? "?"}(${el.pins.map(pinText).join(", ")})`;
