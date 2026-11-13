// Online monitoring and the variables table on the webview's side (spec 2026-11-13-online-monitoring-design.md,
// decisions 4 to 11): the types `monitor.rs` and `declarations.rs` send, the values drawn in the code as Monaco
// decorations, and the small pure helpers the editor and the table share.
import type { Source } from "./sources";

/** What a write or a force changes (engine `controller_monitor.rs`, `Target`). */
export type Target = { kind: "variable"; program: string; path: string } | { kind: "address"; address: string };

/** Where a value is drawn in the code: after `column` of `line`, both 1-based (monitor.rs `Spot`). */
export interface Spot {
  line: number;
  column: number;
  condition: boolean;
  target: Target | null;
}

/** A row of the variables table the engine reads (monitor.rs `Row`). */
export interface Row {
  name: string;
  declaration: number | null;
  target: Target | null;
  instance: boolean;
}

export interface Watched {
  spots: Spot[];
  rows: Row[];
}

export interface Force {
  target: Target;
  label: string;
  value: string;
}

/** A declaration as the table edits it (declarations.rs `Declaration`). */
export interface Declaration {
  pou: string;
  name: string;
  class: string;
  qualifier: string | null;
  type_name: string;
  address: string | null;
  initial: string | null;
  comment: string | null;
}

export type Change =
  { op: "add"; row: Declaration } | { op: "delete"; index: number } | { op: "update"; index: number; row: Declaration };

/** The classes a table row can take, in the order IEC 61131-3 lists the blocks. */
export const CLASSES = ["VAR", "VAR_INPUT", "VAR_OUTPUT", "VAR_IN_OUT", "VAR_TEMP", "VAR_GLOBAL", "VAR_EXTERNAL"];

export const labelOf = (target: Target): string =>
  target.kind === "variable" ? `${target.program}.${target.path}` : target.address;

/** The values `monitor` sends, split as `watch_range` ordered them: the spots' first, then the rows'. */
export function split(watched: Watched, values: (string | null)[]) {
  const n = watched.spots.length;
  return { spots: values.slice(0, n), rows: values.slice(n, n + watched.rows.length) };
}

/** The class names of a value: a condition's colour, and a mark when its target is forced. */
export function valueClass(spot: Spot, value: string, forced: Set<string>): string {
  const classes = ["monitor-value"];
  if (spot.condition) classes.push(value === "TRUE" ? "monitor-true" : "monitor-false");
  if (spot.target && forced.has(labelOf(spot.target))) classes.push("monitor-forced");
  return classes.join(" ");
}

/** Monaco decorations drawing each value after its spot; a spot without a value draws nothing. */
export function decorations(spots: Spot[], values: (string | null)[], forced: Set<string>) {
  return spots.flatMap((spot, i) => {
    const value = values[i];
    if (value === null || value === undefined) return [];
    const at = {
      startLineNumber: spot.line,
      startColumn: spot.column,
      endLineNumber: spot.line,
      endColumn: spot.column,
    };
    return [
      { range: at, options: { after: { content: ` ${value}`, inlineClassName: valueClass(spot, value, forced) } } },
    ];
  });
}

/** The spot a click at `line` and `column` hits: Monaco reports a click on a drawn value at its spot's column. */
export const spotAt = (spots: Spot[], line: number, column: number): number =>
  spots.findIndex((s) => s.line === line && s.column === column);

/** The value a click gives a BOOL: its opposite. Any other value is typed. */
export const toggled = (value: string | null): string | null =>
  value === "TRUE" ? "FALSE" : value === "FALSE" ? "TRUE" : null;

/** The device a block's tab monitors: the selected instance, when it is of that block's family. */
export function deviceOf(
  source: Source,
  selected: string | null,
  instances: { id: string; definition: string }[],
): string | null {
  if (source.kind !== "behaviour" || !selected) return null;
  return instances.find((i) => i.id === selected && i.definition === source.key)?.id ?? null;
}

/** A new row of `pou`, named so as not to clash with `taken`. */
export function newRow(pou: string, taken: string[]): Declaration {
  let n = 1;
  while (taken.includes(`var${n}`)) n += 1;
  return {
    pou,
    name: `var${n}`,
    class: "VAR",
    qualifier: null,
    type_name: "BOOL",
    address: null,
    initial: null,
    comment: null,
  };
}
