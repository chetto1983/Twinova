// A Ladder program as the Rust crate `twinova-ladder` reads and writes it (`model.rs`, its serde form), and what the
// lowering tells the editor about it (`ladder_commands.rs` `Lowering`). The editor holds a file as this tree; Rust reads
// it from the text and prints it back (plan 2026-11-13-ladder-editor.md, decision 1).

export type ContactMode = "open" | "closed" | "rising" | "falling";
export type CoilMode = "normal" | "negated" | "set" | "reset" | "rising" | "falling";

/** A box's parameter as written: an input (`PT := T#2s`), an output bound to a place (`ET => elapsed`), or a bare name,
 *  which puts that pin on the rung. */
export interface Pin {
  name: string;
  value: string | null;
  out: boolean;
}

export type Element =
  | { kind: "contact"; mode: ContactMode; operand: string | null }
  | { kind: "coil"; mode: CoilMode; operand: string | null }
  | { kind: "box"; head: string | null; pins: Pin[] }
  | { kind: "branch"; paths: Element[][] };

export type Exit = { kind: "jump"; label: string } | { kind: "return" };

export interface Rung {
  comment: string | null;
  label: string | null;
  elements: Element[];
  exit: Exit | null;
}

export interface Program {
  declarations: string;
  rungs: Rung[];
}

/** An element: its rung, then the indices down the tree, an element's and a path's in turn. */
export interface ElementId {
  rung: number;
  path: number[];
}

/** How a box sits on its rung, as the lowering resolved it. */
export interface BoxPins {
  function: boolean;
  type_name: string;
  input: string | null;
  output: string;
}

export interface Lowering {
  power: { element: ElementId; variable: string }[];
  boxes: [ElementId, BoxPins][];
  errors: { element: ElementId; message: string }[];
}

/** Where the text stops reading as Ladder (`parse.rs` `ParseError`): a byte offset. */
export interface ParseError {
  message: string;
  offset: number;
}

/** A key for an element's place: its rung and path. */
export const keyOf = (rung: number, path: readonly number[]): string => [rung, ...path].join(".");
