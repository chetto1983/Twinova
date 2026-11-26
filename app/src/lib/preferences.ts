// User preferences kept across sessions. localStorage can be unavailable (blocked storage):
// the preference then lasts for the session only.

export function readPreference(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function writePreference(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    // See the module comment.
  }
}

/** The window's theme: the one chosen in Settings, else light, as Visual Components and TIA Portal open (spec
 *  2026-11-26-ux-consolidation-design.md, answer 3). */
export const savedTheme = (): "light" | "dark" => (readPreference("theme") === "dark" ? "dark" : "light");
