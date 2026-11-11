// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the belt conveyor's look, examples/parts/belt-conveyor/
// part.js `geometry`, on a straight path (Twinova's belts have one segment):
// - the belt band: the loop over both end drums as one surface, its u in metres along the loop (OIP's bent-path
//   branch, which also draws a straight belt; its straight branch draws instanced templates whose u a per-instance
//   attribute rebuilds, a shader Twinova does not have), so the fabric tiles and scrolls by texture offset;
// - the side rails and the guards: the swept unit templates, one instance per side (`steelPiece`);
// - the floor stands at the belt's stations (legs.ts), and the frame web bolts.
// Holes and mitres, rail extents and cuts, leg rests and exclusions are left out: no Twinova part has them (spec
// 2026-11-10-curves-belt-options-design.md, decision 3). A guard opens where a pusher stands (openings.ts).
// The origin is the belt's tail on its conveying surface, x along the flow, y up, z across (left −z).
import { guardSpans, type Opening } from "../../openings";
import type { GeometryBuilder } from "../geometry/types";
import { cylinderTemplate, legStations, stands } from "./legs";
import type { Instance, MeshPart, V3 } from "./part";

export interface BeltParams {
  length: number;
  width: number;
  /** The belt loop's depth (OIP's height): the drum diameter, and the side frame's depth. */
  frameHeight: number;
  /** The conveying surface above the floor. */
  standHeight: number;
  /** Its left (−z) and right guards. */
  guards: readonly [boolean, boolean];
  /** Where its guards open for the pushers beside it. */
  openings?: readonly Opening[];
}

/** The fabric's period along the belt, in metres (OIP `BELT_TEX_PERIOD`). */
export const BELT_TILES_PER_M = 1;
const UNIT_X: V3[] = [
  [0, 0, 0],
  [1, 0, 0],
];
const CAPS = { caps: true };

/** The guard's section, z outward from its inner face: the 5 mm sheet, its top turned out into a 25 mm return. */
export function guardSection(zIn: number, sign: number): [number, number][] {
  const LIP = 0.025,
    LIP_T = 0.005;
  const z1 = zIn + sign * 0.005,
    z2 = zIn + sign * LIP;
  return [
    [-0.005, zIn],
    [-0.005, z1],
    [0.15 - LIP_T, z1],
    [0.15 - LIP_T, z2],
    [0.15, z2],
    [0.15, zIn],
  ];
}

/** The frame's bottom below the surface, rounded to the micrometre as OIP rounds it. */
export function frameBottom(frameHeight: number): number {
  return -Math.round((frameHeight + 0.005) * 1e6) / 1e6;
}

/** The L-rail's wall and flange, z outward from the bed edge. */
export function railSections(fb: number, zIn: number, sign: number): [number, number][][] {
  const z0 = zIn,
    z1 = zIn + sign * 0.01,
    z2 = zIn + sign * 0.03;
  return [
    [
      [fb, z0],
      [fb, z1],
      [-0.005, z1],
      [-0.005, z0],
    ],
    [
      [-0.005, z1],
      [-0.005, z2],
      [0, z2],
      [0, z1],
    ],
  ];
}

/** A straight template piece from x `a` to `b` at side `sign`, `zOff` out: the −z side is the piece run backwards
 *  and turned 180° about its up axis, never a mirror scale, so its winding and normals stay right. */
function steelPiece(a: number, b: number, sign: number, zOff: number): Instance {
  if (sign < 0) return { position: [b, 0, -zOff], scale: [b - a, 1, 1], quaternion: [0, 1, 0, 0] };
  return { position: [a, 0, zOff], scale: [b - a, 1, 1] };
}

/** The belt band: the top run, over the head drum, the return run, and over the tail drum, u in metres. */
function band(g: GeometryBuilder, length: number, r: number, bw: number) {
  const loop: V3[] = [
    [0, 0, 0],
    [length, 0, 0],
  ];
  loop.push(...g.arc([length, -r, 0], r, Math.PI / 2, -Math.PI / 2, 8).slice(1));
  loop.push([length, -2 * r, 0], [0, -2 * r, 0]);
  loop.push(...g.arc([0, -r, 0], r, (3 * Math.PI) / 2, Math.PI / 2, 8).slice(1, -1));
  loop.push([0, 0, 0]);
  const sfc = g.surface();
  let prev: [number, number] | null = null;
  let u = 0;
  loop.forEach((p, i) => {
    if (i > 0) u += Math.hypot(p[0] - loop[i - 1]![0], p[1] - loop[i - 1]![1]);
    const a = sfc.vertex(p[0], p[1], -bw, u, 0);
    const b = sfc.vertex(p[0], p[1], bw, u, 1);
    if (prev) sfc.quad(prev[0], prev[1], b, a);
    prev = [a, b];
  });
  return sfc.build();
}

export function beltGeometry(g: GeometryBuilder, p: BeltParams): MeshPart[] {
  const L = p.length;
  const hw = p.width / 2;
  const r = p.frameHeight / 2; // the drum radius: frame and guards reach it past each end
  const fb = frameBottom(p.frameHeight);
  const sides = [-1, 1] as const;
  const rails = g.merge(...railSections(fb, 0, 1).map((s) => g.sweep(s, UNIT_X, CAPS)));
  const parts: MeshPart[] = [
    { desc: band(g, L, r, hw - 0.005), material: "belt", scroll: BELT_TILES_PER_M },
    { desc: rails, material: "frame", instances: sides.map((s) => steelPiece(-r, L + r, s, hw)) },
  ];
  const guarded = sides.filter((s) => p.guards[s < 0 ? 0 : 1]);
  if (guarded.length > 0) {
    const guard = g.sweep(guardSection(0, 1), UNIT_X, CAPS);
    const pieces = guarded.flatMap((s) =>
      guardSpans(-r, L + r, s > 0, p.openings ?? []).map(([a, b]) => steelPiece(a, b, s, hw + 0.01)),
    );
    parts.push({ desc: guard, material: "guard", instances: pieces });
  }
  // the stands: the end insets and the regular spacing, one centred post on a belt too short for two
  const height = p.standHeight + fb;
  if (height > 0.05) {
    const xs = L - 0.3 < 0.1 ? [L / 2] : legStations(L);
    const at = xs.map((x) => ({ position: [x, -p.standHeight, 0] as V3, yaw: 0, dir: x <= L / 2 ? 1 : -1 }));
    parts.push(...stands(g, hw, height, -fb, at));
  }
  // the web bolts: pairs every 1.2 m down each rail's mid-wall
  const yMid = fb / 2,
    br = 0.012;
  const gap = Math.max(0.001, Math.min(0.04, -0.005 - 0.003 - br - yMid, yMid - br - 0.003 - fb));
  const n = Math.max(2, Math.round(L / 1.2));
  const bolts: Instance[] = [];
  for (let i = 0; i < n; i++) {
    const x = ((i + 0.5) / n) * L;
    for (const s of sides)
      for (const d of [-gap, gap]) bolts.push({ position: [x, yMid + d, s * (hw + 0.013)], scale: [br, br, 0.01] });
  }
  parts.push({ desc: cylinderTemplate(g, 6), material: "bolt", instances: bolts });
  return parts;
}
