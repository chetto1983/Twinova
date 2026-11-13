// The PLC pane in a window of its own (spec 2026-11-13-plc-window-design.md; plc_window.rs). The main window keeps
// the texts and sends the window its state; the window sends back each edit, numbered, and its Build, Run and Step
// requests with its texts, so main runs them on what the window shows.
import { invoke } from "@tauri-apps/api/core";
import { emitTo, listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Problems } from "./analysis";
import type { Source } from "./sources";

/** The PLC window's label (plc_window.rs `LABEL`). */
export const PLC = "plc";
const MAIN = "main";

/** What the window shows: what Editor.svelte takes, the run state, and the last edit main took. */
export interface PlcState {
  sources: Source[];
  status: "none" | "ok" | "failed";
  problems: Problems;
  selected: string | null;
  instances: { id: string; definition: string }[];
  running: boolean;
  ready: boolean;
  ack: number;
}
/** The window's texts after its edit `seq`. */
export interface Edit {
  seq: number;
  sources: Source[];
}
export type Request = "build" | "run" | "step";
export interface Ask extends Edit {
  request: Request;
}

/** The window's side: it numbers its edits, and takes main's texts only once main has taken the last one, so an echo
 *  never undoes what was typed since (decision 2). */
export class Outbox {
  private seq = 0;
  next(sources: Source[]): Edit {
    this.seq += 1;
    return { seq: this.seq, sources };
  }
  takes(state: PlcState): boolean {
    return state.ack >= this.seq;
  }
}

/** Main's side: each edit once, newest wins; events between windows may arrive out of order. */
export class Inbox {
  ack = 0;
  take(edit: Edit): boolean {
    if (edit.seq <= this.ack) return false;
    this.ack = edit.seq;
    return true;
  }
  /** A new window counts from 1. */
  reset(): void {
    this.ack = 0;
  }
}

export const plcWindow = {
  open: () => invoke<void>("open_plc_window"),
  /** Closes the window as its title bar does (plc_window.rs `close_plc_window`). */
  close: () => invoke<boolean>("close_plc_window"),
  send: (state: PlcState) => emitTo(PLC, "plc-state", state),
  ready: () => emitTo(MAIN, "plc-ready", null),
  edit: (edit: Edit) => emitTo(MAIN, "plc-edit", edit),
  ask: (ask: Ask) => emitTo(MAIN, "plc-ask", ask),
  onState: (f: (state: PlcState) => void) => listen<PlcState>("plc-state", (e) => f(e.payload)),
};

/** Main's listeners: an edit taken, a request (its texts taken first), the window ready for its state, and gone. */
export function hostPlcWindow(
  inbox: Inbox,
  on: { take: (sources: Source[]) => void; request: (r: Request) => void; ready: () => void; closed: () => void },
): Promise<UnlistenFn>[] {
  const taken = (edit: Edit) => {
    if (inbox.take(edit)) on.take(edit.sources);
  };
  return [
    listen<Edit>("plc-edit", (e) => taken(e.payload)),
    listen<Ask>("plc-ask", (e) => {
      taken(e.payload);
      on.request(e.payload.request);
    }),
    listen("plc-ready", () => on.ready()),
    listen("plc-closed", () => on.closed()),
  ];
}
