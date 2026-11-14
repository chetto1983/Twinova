// Where the PLC is held (spec 2026-11-13-breakpoints-hold-design.md, decision 2), one answer for the window: a held
// scan sends no event, so the debugger is asked. The status bar, the debug keys and the editor's held line read it.
// Each asked on its own before, and a key pressed as the status bar showed HOLD could reach a debug bar that did not
// know of it yet: the key was dropped (CI, e2e/breakpoints.mjs, F11 9 ms before that bar's poll saw the stop).
import { debug } from "./backend";
import type { Hold } from "./debug";

export const POLL_MS = 200;

class Held {
  current = $state<Hold | null>(null);
  private watchers = 0;
  private timer: ReturnType<typeof setInterval> | undefined;

  /** Asks the debugger while one watcher at least is mounted; the function returned stops this one watching. */
  watch(): () => void {
    if (this.watchers++ === 0) this.timer = setInterval(() => void this.ask(), POLL_MS);
    return () => {
      if (--this.watchers === 0) clearInterval(this.timer);
    };
  }

  ask(): Promise<void> {
    return debug
      .hold()
      .then((h) => {
        this.current = h;
      })
      .catch(() => {
        this.current = null;
      });
  }
}

export const plcHold = new Held();
