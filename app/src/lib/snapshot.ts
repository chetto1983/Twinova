// What the backend sends (src-tauri/src/engine.rs `Snapshot`) and the pure helpers the view needs.

export interface Part {
  id: number;
  front_m: number;
}

export interface Snapshot {
  time_ms: number;
  parts: Part[];
  delivered: number;
  outlet_blocked: boolean;
  sensor: boolean;
  motor: boolean;
  jam: boolean | null;
  lag_ms: number;
}

/** Scene constants, the same as src-tauri/src/scene.rs. */
export const BELT_LENGTH_M = 3.0;
export const PART_LENGTH_M = 0.3;
export const PHOTOCELL_AT_M = 2.7;

/** Virtual time as m:ss.cc. */
export function formatTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 10));
  const centis = total % 100;
  const seconds = Math.floor(total / 100) % 60;
  const minutes = Math.floor(total / 6000);
  return `${minutes}:${String(seconds).padStart(2, "0")}.${String(centis).padStart(2, "0")}`;
}

/** Centre of a part along the belt, with the belt centred on x = 0 in the 3D view. */
export const partCentreX = (front_m: number): number => front_m - PART_LENGTH_M / 2 - BELT_LENGTH_M / 2;

/** Lag worth showing: a frame or two of jitter is not "behind". */
export const isBehind = (lag_ms: number): boolean => lag_ms > 100;

/** Camera distance that fits `halfWidth` metres horizontally and `halfHeight` vertically, for a vertical field of view
 *  of `vfovDeg` and the viewport's `aspect` (width / height). */
export function framingDistance(halfWidth: number, halfHeight: number, vfovDeg: number, aspect: number): number {
  const tanV = Math.tan((vfovDeg * Math.PI) / 360);
  const tanH = tanV * aspect;
  return Math.max(halfWidth / tanH, halfHeight / tanV);
}
