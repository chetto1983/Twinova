// What the Ladder view (LadderEditor.svelte) reads off a file and its lowering.
import type { Resolve } from "./layout";
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
