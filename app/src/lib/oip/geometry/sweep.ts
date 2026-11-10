// Copied from Open Industry Project, src/core/geometry/meshDescription.ts at 1e7488c (MIT, Copyright (c) 2024-present
// Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md): `sweepProfileAlongPath`, the conveyor-bed
// primitive. Twinova keeps what its belts and curves use: one section for the whole path, the level frame, and the
// caps. The loft (a section that changes along the path) and the transport frame (a path that climbs vertically) are
// left out; a cap is triangulated with the outline triangulation the extrusion already uses (shapes.ts).
import type { MeshDescription } from "./types";
import { GeometryBudgetError, MAX_DESCRIPTION_VERTICES, type Vec3 } from "./mesh";
import { triangulateWithHoles } from "./shapes";

type V2 = readonly [number, number];

const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const cross = (a: Vec3, b: Vec3): Vec3 => [
  a[1] * b[2] - a[2] * b[1],
  a[2] * b[0] - a[0] * b[2],
  a[0] * b[1] - a[1] * b[0],
];
const dot = (a: Vec3, b: Vec3): number => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const length3 = (a: Vec3): number => Math.hypot(a[0], a[1], a[2]);
const normalize = (a: Vec3): Vec3 => {
  const l = length3(a) || 1;
  return [a[0] / l, a[1] / l, a[2] / l];
};
const WORLD_UP: Vec3 = [0, 1, 0];

/** the section's width axis along tangent `t`, kept level */
function widthAt(t: Vec3): Vec3 {
  let w = cross(t, WORLD_UP);
  if (length3(w) < 1e-6) w = cross(t, [0, 0, 1]); // near-vertical fallback (conveyors never hit this)
  return normalize(w);
}

/** the turn (radians) past which a section corner is a hard edge; a 12-sided tube turns 30° and stays smooth */
const CREASE_TURN = (40 * Math.PI) / 180;

/** which section points are hard edges: a corner counts when it turns past CREASE_TURN */
function sharpCorners(section: readonly V2[]): boolean[] {
  const m = section.length;
  const cosLimit = Math.cos(CREASE_TURN);
  return section.map((b, j) => {
    const a = section[(j + m - 1) % m]!;
    const c = section[(j + 1) % m]!;
    const ux = b[0] - a[0],
      uy = b[1] - a[1],
      vx = c[0] - b[0],
      vy = c[1] - b[1];
    const lu = Math.hypot(ux, uy),
      lv = Math.hypot(vx, vy);
    return lu >= 1e-12 && lv >= 1e-12 && (ux * vx + uy * vy) / (lu * lv) < cosLimit;
  });
}

/** A simple polygon's triangles in either winding, indexing the polygon's own points. */
function triangulatePolygon(poly: readonly V2[]): [number, number, number][] {
  let area = 0;
  for (let i = 0; i < poly.length; i++) {
    const a = poly[i]!,
      b = poly[(i + 1) % poly.length]!;
    area += a[0] * b[1] - b[0] * a[1];
  }
  if (area > 0) return triangulateWithHoles(poly, []).triangles;
  const last = poly.length - 1;
  return triangulateWithHoles(poly.toReversed(), []).triangles.map(
    ([a, b, c]) => [last - a, last - b, last - c] as [number, number, number],
  );
}

/** Options for {@link sweepProfileAlongPath}. */
export interface SweepOptions {
  /** close both ends with the section itself. Off by default: a belt bed is open-ended (its drum wrap is built
   *  separately) and a piece that continues into another sweep hides its end anyway. Turn it on for anything
   *  that visibly stops (a guard, a frame rail), or its end shows as a hollow tube. */
  caps?: boolean;
}

/**
 * Loft a closed 2-D cross-section profile along a 3-D `path`: the conveyor-bed primitive (extrude a
 * belt/frame/guard section down the part's length, straight, inclined or curved). `profile` is the section as
 * `[u, w]` pairs, where `u` is the cross-section's up offset and `w` its width offset; `path` is the bed centreline as
 * ≥2 points.
 *
 * At each station the section is oriented by a path-aligned frame: tangent `T` (the path direction), width axis
 * `W = normalize(T × worldUp)` (kept horizontal so the deck never banks), and up axis `U = W × T`. A profile point
 * `[pu, pw]` goes to `centre + pu·U + pw·W`. Quads bridge consecutive rings; the ends stay open unless `opts.caps`.
 * `u` runs along normalized arc length, `v` around the profile.
 */
export function sweepProfileAlongPath(
  profile: readonly V2[],
  path: readonly Vec3[],
  opts?: SweepOptions,
): MeshDescription {
  if (profile.length < 3) throw new Error("sweepProfileAlongPath: profile needs ≥3 points (a closed section)");
  if (path.length < 2) throw new Error("sweepProfileAlongPath: need ≥2 stations to sweep between");
  const n = path.length;
  const m = profile.length;
  const caps = opts?.caps === true;
  // A section corner that turns more than CREASE_TURN is a hard edge: each edge leaving it gets a vertex of its own,
  // so the host's averaged normals never blend two faces of an angular section.
  const sharp = sharpCorners(profile);
  // ring layout: point j's vertex for the edge arriving at it is inV[j], for the edge leaving it outV[j]
  const inV: number[] = [];
  const outV: number[] = [];
  let ringSize = 0;
  for (let j = 0; j < m; j++) {
    inV.push(ringSize);
    if (sharp[j]) ringSize++;
    outV.push(ringSize);
    ringSize++;
  }
  const vertCount = ringSize * n + (caps ? 2 * m : 0);
  if (vertCount > MAX_DESCRIPTION_VERTICES) throw new GeometryBudgetError(vertCount);

  // tangent at a station: the adjacent segment at an end, the central difference in the interior
  const tangentAt = (i: number): Vec3 =>
    normalize(
      i === 0
        ? sub(path[1]!, path[0]!)
        : i === n - 1
          ? sub(path[n - 1]!, path[n - 2]!)
          : sub(path[i + 1]!, path[i - 1]!),
    );

  // cumulative arc length → a uniform u, so texture density doesn't bunch on a curve
  const arc: number[] = [0];
  for (let i = 1; i < n; i++) arc[i] = arc[i - 1]! + length3(sub(path[i]!, path[i - 1]!));
  const total = arc[n - 1]! || 1;
  // v runs along the cross-section perimeter arc-length, so a texture maps by real distance around the profile
  const vAt: number[] = [];
  let perim = 0;
  for (let j = 0; j < m; j++) {
    vAt.push(perim);
    const a = profile[j]!;
    const b = profile[(j + 1) % m]!;
    perim += Math.hypot(b[0] - a[0], b[1] - a[1]);
  }
  const vTotal = perim || 1;

  const positions: number[] = [];
  const uvs: number[] = [];
  for (let i = 0; i < n; i++) {
    const c = path[i]!;
    const t = tangentAt(i);
    const w = widthAt(t);
    const up = cross(w, t);
    const u = arc[i]! / total;
    for (let j = 0; j < m; j++) {
      const [pu, pw] = profile[j]!;
      const x = c[0] + up[0] * pu + w[0] * pw,
        y = c[1] + up[1] * pu + w[1] * pw,
        z = c[2] + up[2] * pu + w[2] * pw;
      for (let k = sharp[j] ? 2 : 1; k > 0; k--) {
        positions.push(x, y, z);
        uvs.push(u, vAt[j]! / vTotal);
      }
    }
  }
  // quads between consecutive rings; the profile is closed, so j wraps modulo m
  const indices: number[] = [];
  for (let i = 0; i < n - 1; i++) {
    const a = i * ringSize;
    const b = (i + 1) * ringSize;
    for (let j = 0; j < m; j++) {
      const p0 = outV[j]!;
      const p1 = inV[(j + 1) % m]!;
      indices.push(a + p0, a + p1, b + p1, a + p0, b + p1, b + p0);
    }
  }
  if (caps) {
    // each end closed by the section itself, on vertices of its own so it shades flat instead of averaging with
    // the walls. Wound so its normal faces out of the solid: back along the path at the start, on along it at the end.
    const tris = triangulatePolygon(profile);
    const cap = (ring: number, outward: Vec3): void => {
      const at = (k: number): number => (ring * ringSize + outV[k]!) * 3;
      const p = (k: number): Vec3 => [positions[at(k)]!, positions[at(k) + 1]!, positions[at(k) + 2]!];
      let facing = 0;
      for (const [a, b, c] of tris) facing += dot(cross(sub(p(b), p(a)), sub(p(c), p(a))), outward);
      if (tris.length === 0 || Math.abs(facing) < 1e-18) return;
      const base = positions.length / 3;
      for (let j = 0; j < m; j++) {
        const v = at(j);
        positions.push(positions[v]!, positions[v + 1]!, positions[v + 2]!);
        uvs.push(profile[j]![1], profile[j]![0]);
      }
      const flip = facing < 0;
      for (const [i, j, k] of tris) indices.push(base + i, base + (flip ? k : j), base + (flip ? j : k));
    };
    const t0 = tangentAt(0);
    cap(0, [-t0[0], -t0[1], -t0[2]]);
    cap(n - 1, tangentAt(n - 1));
  }
  return { positions, indices, uvs };
}
