// The editor's keys (spec 2026-11-11-editor-basics-design.md, "The view"): Delete removes the selection, Ctrl+Z undoes,
// Ctrl+Y and Ctrl+Shift+Z redo (Cmd on macOS), W moves and E turns with the gizmo. None of them while the focus is where
// text is typed: a field, a list, the ST editor.

export type Shortcut = "undo" | "redo" | "remove" | "translate" | "rotate";

/** Whether the focus is somewhere keys type text. */
export function typing(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
}

export function shortcutOf(event: KeyboardEvent): Shortcut | null {
  if (typing(event.target)) return null;
  const key = event.key.toLowerCase();
  if (event.ctrlKey || event.metaKey) {
    if (key === "z") return event.shiftKey ? "redo" : "undo";
    if (key === "y") return "redo";
    return null;
  }
  if (event.altKey) return null;
  if (key === "delete" || key === "backspace") return "remove";
  if (key === "w") return "translate";
  if (key === "e") return "rotate";
  return null;
}
