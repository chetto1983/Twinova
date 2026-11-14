// What the Ladder view (LadderEditor.svelte) reads off a file and its lowering.
import type { Selection } from "./edit";
import type { Resolve } from "./layout";
import type { LadderSpot } from "../monitor";
import { keyOf, type BoxPins, type Lowering } from "./model";

/** The line of byte `offset` in `text`, from 1: where a file stops reading as Ladder. */
export function lineOf(text: string, offset: number): number {
  const bytes = new TextEncoder().encode(text).slice(0, offset);
  return new TextDecoder().decode(bytes).split("\n").length;
}

/** How each box of rung `rung` sits on it, from the lowering. */
export function resolver(lowering: Lowering | null): (rung: number) => Resolve {
  const boxes = new Map<string, BoxPins>((lowering?.boxes ?? []).map(([at, pins]) => [keyOf(at.rung, at.path), pins]));
  return (rung) => (path) => boxes.get(keyOf(rung, path));
}

/** What keeps rung `rung` from lowering. */
export function rungErrors(lowering: Lowering | null, rung: number): string[] {
  return (lowering?.errors ?? []).filter((e) => e.element.rung === rung).map((e) => e.message);
}

/** What a click selects, read off the rung and the `data-path` it landed in: an element or a branch path, or the rung
 *  itself between them. Null off the rungs. */
export function selectionOf(rung: string | undefined, path: string | undefined): Selection | null {
  if (rung === undefined) return null;
  return { rung: Number(rung), path: path ? path.split(".").map(Number) : [] };
}

/** What a key does in the Ladder view, CODESYS's keys: delete, undo, redo, Left and Right along the elements, Escape,
 *  Enter or F2 to type the selected element's operand or rung's head, Ctrl+B to pick a box, and the clipboard's. */
export type KeyAction =
  "delete" | "undo" | "redo" | "left" | "right" | "escape" | "edit" | "box" | "copy" | "cut" | "paste";

export function keyAction(e: { key: string; ctrlKey: boolean; metaKey: boolean; shiftKey: boolean }): KeyAction | null {
  const ctrl = e.ctrlKey || e.metaKey;
  const key = e.key.toLowerCase();
  if (ctrl && key === "z") return e.shiftKey ? "redo" : "undo";
  if (ctrl && key === "y") return "redo";
  if (ctrl && key === "b") return "box";
  const clipboard: Record<string, KeyAction> = { c: "copy", x: "cut", v: "paste" };
  if (ctrl && clipboard[key]) return clipboard[key];
  if (ctrl) return null;
  const plain: Record<string, KeyAction> = {
    delete: "delete",
    backspace: "delete",
    arrowleft: "left",
    arrowright: "right",
    escape: "escape",
    enter: "edit",
    f2: "edit",
  };
  return plain[key] ?? null;
}

/** What the drawing of rung `rung` shows online (monitor_ladder.rs): whether each element's power out is on, and each
 *  operand's or box pin's value with the spot it comes from, by `keyOf` and `#pin`. Empty offline. */
export interface Online {
  power: Map<string, boolean>;
  values: Map<string, { value: string; spot: number }>;
}

export function onlineOf(spots: readonly LadderSpot[], values: readonly (string | null)[], rung: number): Online {
  const power = new Map<string, boolean>();
  const shown = new Map<string, { value: string; spot: number }>();
  spots.forEach((s, i) => {
    const value = values[i];
    if (s.element.rung !== rung || value === null || value === undefined) return;
    const key = keyOf(s.element.rung, s.element.path);
    if (s.power) power.set(key, value === "TRUE");
    else shown.set(s.pin === null ? key : `${key}#${s.pin}`, { value, spot: i });
  });
  return { power, values: shown };
}

/** Whether the power `feed` names is on in rung `rung`: the left rail's always, an element's as the scan left it;
 *  null while unknown (offline, or not read yet). */
export function powered(online: Online | null, rung: number, feed: number[] | null): boolean | null {
  if (!online) return null;
  if (feed === null) return true;
  return online.power.get(keyOf(rung, feed)) ?? null;
}
