// A control's tooltip says what it does and, while it is disabled, why it does nothing now (spec
// 2026-11-26-ux-consolidation-design.md, decision 8): "Run: build the project first".

/** `name`, followed by `why` when the control is disabled for that reason. */
export const tip = (name: string, why: string | null): string => (why ? `${name}: ${why}` : name);
