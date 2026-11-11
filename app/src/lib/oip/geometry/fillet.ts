// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md): src/core/filletCorner.ts `filletCorner` and
// src/core/geometry/geometryBuilder.ts `filletPolyline`, the builder's `fillet` op. OIP shares the corner rule with
// its belt path (beltPath.ts), which Twinova does not copy; here it rounds a cable's bend (the pusher's).
import type { Vec3 } from "./mesh";

/**
 * The tangent arc that rounds one polyline corner: how far back from the corner the arc starts (`inset`) and the
 * radius it ends up with. The inset is capped at half of each adjacent edge, so the arcs of two neighbouring corners
 * never overlap. A capped corner gets a tighter radius than asked for.
 *
 * `turn` is the unsigned deflection at the corner, in radians, and must be > 0.
 */
export function filletCorner(
  radius: number,
  turn: number,
  inLen: number,
  outLen: number,
): { inset: number; radius: number } {
  const tanHalf = Math.tan(turn / 2);
  const want = radius * tanHalf;
  const inset = Math.min(want, inLen * 0.5, outLen * 0.5);
  // an uncapped corner keeps the radius exactly, rather than a value round-tripped through tan
  return { inset, radius: inset < want ? inset / tanHalf : radius };
}

/** Round the interior corners of an open polyline with tangent arcs of `radius`, in the X-Y plane (Z carried
 *  through). The general fillet primitive a part composes a bent path from (a belt centreline, a rounded
 *  bracket). The inset at each corner is clamped to half the shorter adjacent segment so tight corners /
 *  small radii stay well-formed. Endpoints are kept. */
export function filletPolyline(points: ReadonlyArray<Vec3>, radius: number, arcSegments: number): Vec3[] {
  const n = points.length;
  if (n < 3 || radius <= 0) return points.map((p): Vec3 => [p[0], p[1], p[2]]);
  const seg = Math.max(1, Math.floor(arcSegments));
  const out: Vec3[] = [[points[0]![0], points[0]![1], points[0]![2]]];
  for (let i = 1; i < n - 1; i++) {
    const A = points[i - 1]!;
    const P = points[i]!;
    const B = points[i + 1]!;
    let inx = P[0] - A[0];
    let iny = P[1] - A[1];
    let outx = B[0] - P[0];
    let outy = B[1] - P[1];
    const inLen = Math.hypot(inx, iny);
    const outLen = Math.hypot(outx, outy);
    if (inLen < 1e-9 || outLen < 1e-9) {
      out.push([P[0], P[1], P[2]]);
      continue;
    }
    inx /= inLen;
    iny /= inLen;
    outx /= outLen;
    outy /= outLen;
    const turn = Math.atan2(inx * outy - iny * outx, inx * outx + iny * outy); // signed deflection
    const half = Math.abs(turn) / 2;
    if (half < 1e-6) {
      out.push([P[0], P[1], P[2]]);
      continue;
    }
    const { inset, radius: rArc } = filletCorner(radius, Math.abs(turn), inLen, outLen); // rArc ≤ radius
    const sign = Math.sign(turn);
    const t1x = P[0] - inx * inset;
    const t1y = P[1] - iny * inset;
    const cx = t1x - iny * sign * rArc; // arc centre: perpendicular to the incoming dir, toward the turn side
    const cy = t1y + inx * sign * rArc;
    const v0x = t1x - cx;
    const v0y = t1y - cy;
    for (let k = 0; k <= seg; k++) {
      const a = (turn * k) / seg;
      const ca = Math.cos(a);
      const sa = Math.sin(a);
      out.push([cx + v0x * ca - v0y * sa, cy + v0x * sa + v0y * ca, P[2]]);
    }
  }
  out.push([points[n - 1]![0], points[n - 1]![1], points[n - 1]![2]]);
  return out;
}
