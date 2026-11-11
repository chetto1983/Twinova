// What the backend sends (src-tauri/src/engine.rs `Snapshot`, commands.rs `Built`) and the pure helpers the view needs.

export interface Carton {
  id: number;
  /** World position of its centre, m. */
  position: [number, number, number];
  rotation: [number, number, number, number];
  /** Full extents along its own x (the flow it was made on), y and z, m. */
  size: [number, number, number];
  /** sRGB `[r, g, b]` 0–255, or null for an unpainted product. */
  color: [number, number, number] | null;
}

export interface SinkState {
  id: string;
  delivered: number;
  blocked: boolean;
}

/** A scene signal or a panel button, with its value in the controller's image. */
export interface SignalState {
  name: string;
  address: string;
  value: string;
}

export interface Snapshot {
  time_ms: number;
  cartons: Carton[];
  sinks: SinkState[];
  /** Each moving device's position: a chain transfer's or a stopper's lift and a pusher's reach, 0 to 1, or a
   *  turntable's deck angle in degrees, counter-clockwise seen from above. */
  actuators: { id: string; value: number }[];
  /** Each conveyor's surface travel along its flow since Build, m: its fabric scrolls by it. */
  conveyors: { id: string; travel: number }[];
  signals: SignalState[];
  jam: boolean | null;
  lag_ms: number;
}

/** What Build returns: the line at time zero and what the program leaves undeclared. */
export interface Built {
  snapshot: Snapshot;
  warnings: string[];
}

/** The key of a product's paint, one material each: `#rrggbb`, or `unpainted` for the theme's product colour. */
export const paintOf = (color: Carton["color"]): string =>
  color ? `#${color.map((c) => c.toString(16).padStart(2, "0")).join("")}` : "unpainted";

/** A scrolling texture's offset along u after `travel` metres at `tilesPerMetre` (OIP functionMesh.ts: the offset walks
 *  backwards, so the pattern moves forward with the surface). */
export const scrollOffset = (travel: number, tilesPerMetre: number): number => -((travel * tilesPerMetre) % 1);

/** Every product the line's sinks have taken. */
export const deliveredOf = (s: Snapshot | null): number => s?.sinks.reduce((n, sink) => n + sink.delivered, 0) ?? 0;

/** Virtual time as m:ss.cc. */
export function formatTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 10));
  const centis = total % 100;
  const seconds = Math.floor(total / 100) % 60;
  const minutes = Math.floor(total / 6000);
  return `${minutes}:${String(seconds).padStart(2, "0")}.${String(centis).padStart(2, "0")}`;
}

/** Run speeds, as multiples of the wall clock; null runs as fast as possible (spec 2026-11-08-run-modes-design.md). */
export const SPEEDS: readonly (number | null)[] = [0.25, 0.5, 1, 2, 4, 10, null];

/** "×0.5" in the locale's number format, or `max` for as fast as possible. */
export const formatSpeed = (speed: number | null, locale: string, max: string): string =>
  speed === null ? max : `×${new Intl.NumberFormat(locale).format(speed)}`;

/** Lag worth showing: a frame or two of jitter is not "behind". */
export const isBehind = (lag_ms: number): boolean => lag_ms > 100;

/** Camera distance that fits `halfWidth` metres horizontally and `halfHeight` vertically, for a vertical field of view
 *  of `vfovDeg` and the viewport's `aspect` (width / height). */
export function framingDistance(halfWidth: number, halfHeight: number, vfovDeg: number, aspect: number): number {
  const tanV = Math.tan((vfovDeg * Math.PI) / 360);
  const tanH = tanV * aspect;
  return Math.max(halfWidth / tanH, halfHeight / tanV);
}

/** `v` times `k`: where a sliding link stands at a device's position. */
export function scaled(v: readonly [number, number, number], k: number): [number, number, number] {
  return [v[0] * k, v[1] * k, v[2] * k];
}
