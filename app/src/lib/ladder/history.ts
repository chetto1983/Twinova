// The Ladder view's undo and redo, over the file's text: an edit of the drawing rewrites the whole file, and Monaco's own
// history does not survive a text set from outside it (`setValue`).

const LIMIT = 100;

export class History {
  private past: string[] = [];
  private future: string[] = [];

  /** Text `before` an edit, kept to go back to; a new edit drops what Redo would have restored. */
  record(before: string): void {
    this.past = [...this.past, before].slice(-LIMIT);
    this.future = [];
  }

  /** The text before the last edit, `current` kept for Redo; null with nothing to undo. */
  undo(current: string): string | null {
    const text = this.past.pop();
    if (text === undefined) return null;
    this.future.push(current);
    return text;
  }

  /** The text the last Undo went back from, `current` kept for Undo; null with nothing to redo. */
  redo(current: string): string | null {
    const text = this.future.pop();
    if (text === undefined) return null;
    this.past.push(current);
    return text;
  }

  get canUndo(): boolean {
    return this.past.length > 0;
  }

  get canRedo(): boolean {
    return this.future.length > 0;
  }
}
