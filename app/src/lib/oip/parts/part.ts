// The submeshes a generator returns, after Open Industry Project's MeshPart (src/core/parts/partApi.ts at 1e7488c, MIT;
// THIRD_PARTY_NOTICES.md): a mesh description, the material it is drawn in, and, for a bank such as rollers or leg
// pieces, the transforms it is drawn at.
import type { MeshDescription } from "../geometry/types";

export type V3 = readonly [number, number, number];
export type Quat = readonly [number, number, number, number];

export type Material =
  | "belt"
  | "frame"
  | "chainFrame"
  | "support"
  | "chain"
  | "roller"
  | "band"
  | "guard"
  | "leg"
  | "bolt"
  | "body"
  | "lens"
  | "black"
  | "pot"
  | "steel"
  | "darkSteel"
  | "plate"
  | "beltBlack"
  | "bladeOrange"
  | "pusherHood"
  | "pusherCabinet"
  | "pusherFrame"
  | "pusherBarrel"
  | "pusherRod"
  | "pusherRubber"
  | "pusherHose"
  | "pusherLabel"
  | "pusherButton"
  | "pusherLampRed"
  | "pusherLampGreen"
  | "turntablePaint"
  | "turntableCowl"
  | "turntableGrille"
  | "turntableSteel"
  | "turntablePocket"
  | "turntableRim"
  | "turntableNameplate"
  | "turntableGear";

export interface Instance {
  position: V3;
  quaternion?: Quat;
  scale?: V3;
}

export interface MeshPart {
  desc: MeshDescription;
  material: Material;
  /** One draw at each transform. */
  instances?: Instance[];
  /** Texture tiles per metre that scroll at the surface speed (a chain's links, a belt's weave). */
  scroll?: number;
  /** It rides the device's moving link (OIP's rig link: a blade, a paddle, a turntable's deck), which the view poses
   *  from the snapshot; drawn at the link's rest. */
  link?: "moving";
}

/** A turn of `yaw` about +Y. */
export function yawQuat(yaw: number): Quat {
  return [0, Math.sin(yaw / 2), 0, Math.cos(yaw / 2)];
}
