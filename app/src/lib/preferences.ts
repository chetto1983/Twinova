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
