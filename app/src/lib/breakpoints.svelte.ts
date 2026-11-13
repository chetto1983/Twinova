// The session's breakpoints by file, as the editor shows them (specs 2026-11-13-breakpoints-hold-design.md and
// -breakpoints-complete-design.md). A tab with edits not built keeps its breakpoints in the editor, since the engine
// runs the built text; each Build sets them all again.
import { debug } from "./backend";
import type { Breakpoint } from "./debug";

export class Breakpoints {
  byFile = $state<Record<string, Breakpoint[]>>({});

  private waiting: (file: string) => boolean;
  private fail: (error: string) => void;

  /** `waiting(file)`: its tab has edits not built. `fail` shows a refusal, such as a condition that does not parse. */
  constructor(waiting: (file: string) => boolean, fail: (error: string) => void) {
    this.waiting = waiting;
    this.fail = fail;
  }

  of(file: string | null): Breakpoint[] {
    return file ? (this.byFile[file] ?? []) : [];
  }

  /** Some breakpoint is set; some is a logpoint. */
  get armed(): boolean {
    return Object.values(this.byFile).some((list) => list.length > 0);
  }
  get logging(): boolean {
    return Object.values(this.byFile).some((list) => list.some((b) => b.log));
  }

  /** Sets `file`'s breakpoints, and keeps them as the debugger placed them, each on its statement's line. */
  set(file: string, list: Breakpoint[]): void {
    if (this.waiting(file)) {
      this.byFile = { ...this.byFile, [file]: list };
      return;
    }
    debug
      .setBreakpoints(file, list)
      .then((set) => (this.byFile = { ...this.byFile, [file]: set }))
      .catch((e) => this.fail(String(e)));
  }

  /** Moved with their lines by an edit: kept in the editor until the next Build. */
  move(file: string, list: Breakpoint[]): void {
    this.byFile = { ...this.byFile, [file]: list };
  }

  /** Each file's breakpoints, set again on what was just built. */
  resend(): void {
    for (const [file, list] of Object.entries(this.byFile)) this.set(file, list);
  }
}
