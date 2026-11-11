// Picking in the 3D view (spec 2026-11-11-editor-basics-design.md, "The view"): each instance's group carries its id,
// so the first hit's ancestors name the instance under the cursor; a gizmo's pose goes back to the project as a position
// and a quaternion.
import type { PerspectiveCamera } from "three";
import type { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { Quat, Vec3 } from "./project";

/** What the viewport gives a view that is edited (Viewport.svelte `interact`). */
export interface Interaction {
  camera: PerspectiveCamera;
  canvas: HTMLCanvasElement;
  controls: OrbitControls;
  requestRender: () => void;
}

/** An object in the view: its own data and its parent. */
export interface Node {
  userData: Record<string, unknown>;
  parent: Node | null;
}

/** The instance a hit object belongs to: the nearest ancestor carrying an `instance` id. */
export function instanceOf(hit: Node | null): string | null {
  for (let node = hit; node; node = node.parent) {
    const id = node.userData.instance;
    if (typeof id === "string") return id;
  }
  return null;
}

/** The first hit that belongs to an instance, or null on empty floor. */
export function picked(hits: readonly { object: Node }[]): string | null {
  for (const hit of hits) {
    const id = instanceOf(hit.object);
    if (id) return id;
  }
  return null;
}

/** A pose as the project stores it, rounded to the millimetre and to 1e-9 of a quaternion component, so a gizmo
 *  drag does not write float noise into the scene file. */
const mm = (v: number): number => Math.round(v * 1000) / 1000 || 0;
const q = (v: number): number => Math.round(v * 1e9) / 1e9 || 0;

export function pose(position: readonly number[], rotation: readonly number[]): { position: Vec3; rotation: Quat } {
  return {
    position: [mm(position[0]), mm(position[1]), mm(position[2])],
    rotation: [q(rotation[0]), q(rotation[1]), q(rotation[2]), q(rotation[3])],
  };
}

/** A click, not a drag: the pointer moved at most this many pixels between press and release. */
export const CLICK_SLOP_PX = 4;

export const isClick = (down: { x: number; y: number }, up: { x: number; y: number }): boolean =>
  Math.hypot(up.x - down.x, up.y - down.y) <= CLICK_SLOP_PX;
