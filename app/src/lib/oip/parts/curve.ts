// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: what the curved belt and the curved roller conveyor
// share in examples/parts/curved-conveyor/part.js and curved-roller/part.js — the arc convention, the path their rails
// and guards are swept along (the bed's lead-in and lead-out stubs included, as _shared/guardArc.js sweeps a guard with
// no sensor hole), and the floor stands at stations along the arc (`legAt`).
//
// OIP's arc convention: the origin is the arc's centre, the inlet at +X heading −Z, the arc P(t) = (r·cos t, 0, −r·sin t)
// for t in [0, sweep], its tangent (−sin t, 0, −cos t) and its outward radial P(t) / r.
import type { GeometryBuilder } from "../geometry/types";
import { legStations, stands } from "./legs";
import type { MeshPart, V3 } from "./part";

export interface CurveParams {
  innerRadius: number;
  width: number;
  /** In radians. */
  sweep: number;
  /** The frame's depth below the surface (the belt loop's, or the roller frame's). */
  frameHeight: number;
  /** The conveying surface above the floor. */
  standHeight: number;
  /** Its inner and outer guards. */
  inner: boolean;
  outer: boolean;
}

/** The arc's tessellation: at least 8 pieces, one per 5°. */
export function arcSegments(sweep: number): number {
  return Math.max(8, Math.ceil((sweep * 180) / Math.PI / 5));
}

/** The arc of radius `r` from the inlet's lead-in stub to the outlet's lead-out stub, `stub` long each. */
export function arcPath(r: number, sweep: number, stub: number): V3[] {
  const n = arcSegments(sweep);
  const path: V3[] = [[r, 0, stub]]; // t = 0 lead-in: −tangent(0) = +Z
  for (let i = 0; i <= n; i++) {
    const t = (sweep * i) / n;
    path.push([r * Math.cos(t), 0, -r * Math.sin(t)]);
  }
  path.push([r * Math.cos(sweep) - Math.sin(sweep) * stub, 0, -r * Math.sin(sweep) - Math.cos(sweep) * stub]);
  return path;
}

/** The stands along the arc's centreline: the end insets and the regular spacing in arc length, each turned to the
 *  arc, with no knee braces (one reaching along the tangent would miss the curving frame). */
export function arcStands(g: GeometryBuilder, p: CurveParams, frameBottom: number): MeshPart[] {
  const rc = p.innerRadius + p.width / 2;
  const arcLen = rc * p.sweep;
  const height = p.standHeight + frameBottom;
  if (height <= 0.05 || arcLen <= 0) return [];
  const at = legStations(arcLen)
    .filter((s) => s >= 0.05 && s <= arcLen - 0.05)
    .map((s) => {
      const t = s / rc;
      // leg-local x along the tangent and z along the radial: a turn of t + π/2 about +Y
      return { position: [rc * Math.cos(t), -p.standHeight, -rc * Math.sin(t)] as V3, yaw: t + Math.PI / 2, dir: 0 };
    });
  return at.length > 0 ? stands(g, p.width / 2, height, -frameBottom, at) : [];
}
