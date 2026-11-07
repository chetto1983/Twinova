// The Tauri commands of src-tauri/src/commands.rs and the events of the Run loop.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Snapshot } from "./snapshot";

export const backend = {
  defaultProgram: () => invoke<string>("default_program"),
  build: (source: string) => invoke<Snapshot>("build", { source }),
  reset: () => invoke<Snapshot>("reset"),
  step: () => invoke<Snapshot>("step"),
  run: () => invoke<void>("run"),
  pause: () => invoke<void>("pause"),
  press: (start: boolean, stop: boolean) => invoke<void>("press", { start, stop }),
  setOutletBlocked: (blocked: boolean) => invoke<Snapshot>("set_outlet_blocked", { blocked }),
};

export const onSnapshot = (f: (s: Snapshot) => void): Promise<UnlistenFn> =>
  listen<Snapshot>("snapshot", (e) => f(e.payload));
export const onEngineError = (f: (error: string) => void): Promise<UnlistenFn> =>
  listen<string>("engine-error", (e) => f(e.payload));
export const onStopped = (f: () => void): Promise<UnlistenFn> => listen("stopped", () => f());
