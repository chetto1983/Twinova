// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the roller conveyor's look.
// - the rollers: examples/parts/roller-conveyor/part.js `layoutRollers` (its straight branch: a skewed bed is drawn square) and
//   examples/parts/_shared/rollerBed.js (tubes, axle stubs and drive bands, drawn as instanced banks). The band
//   layout's checks for skewed or clipped rollers are left out with the skew: on a straight bed they never apply;
// - the side frame, its flanges and the guards: the boxes of examples/parts/roller-conveyor/rig.json;
// - the floor stands (legs.ts) at its stations along the bed.
// The origin is the rollers' top (the conveying surface), x along the flow centred on the bed, y up, z across.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import { cylinderTemplate, legStations, stands } from "./legs";
import { type Instance, type MeshPart, type V3, yawQuat } from "./part";

type V2 = readonly [number, number];

export interface RollerParams {
  length: number;
  width: number;
  frameHeight: number;
  /** The conveying surface above the floor (OIP's standHeight). */
  standHeight: number;
  /** Its left (−z) and right guards. */
  guards: readonly [boolean, boolean];
}

/** The rendered roller radius and pitch (roller-conveyor/part.js `R`, `PITCH`). */
export const ROLLER_R = 0.024;
const PITCH = 0.075;
const STUB_R = ROLLER_R * 0.34;

// ---- rollerBed.js -------------------------------------------------------------------------------------------------
// The roller's profile, [radius, z] from the axle out to the tube, in the template's native units (radius 0.12).
const ROLLER_PROFILE: V2[] = [
  [0, 0.938],
  [0.0689, 0.938],
  [0.0718, 0.9362],
  [0.0787, 0.9199],
  [0.1072, 0.9199],
  [0.1072, 0.9351],
  [0.1092, 0.9371],
  [0.118, 0.9371],
  [0.12, 0.9351],
];
const ROLLER_TEMPLATE_R = 0.12;
const ROLLER_TEMPLATE_BODY = 2 * 0.9371;
const BAND_CORD_PER_R = 1 / 5;
const BAND_INSET_PER_R = 5 / 6;
const BAND_SPACING_PER_R = 3 / 4;

interface Roller {
  x: number;
  z: number;
  yaw: number;
  length: number;
}

function rollerTemplate(g: GeometryBuilder): MeshDescription {
  const walk: V2[] = [];
  for (const q of ROLLER_PROFILE) walk.push([q[0], -q[1]]);
  for (let k = ROLLER_PROFILE.length - 1; k >= 0; k--) walk.push([ROLLER_PROFILE[k][0], ROLLER_PROFILE[k][1]]);
  return g.lathe(walk, { axis: "z", segments: 18, startAngle: Math.PI / 18 });
}

function bandArcTemplate(g: GeometryBuilder, lr: number, t: number): MeshDescription {
  const s = g.surface();
  const T = 10;
  const N = 6;
  const rows: number[][] = [];
  for (let i = 0; i <= T; i++) {
    const u = (Math.PI * i) / T;
    const row: number[] = [];
    for (let j = 0; j < N; j++) {
      const v = (2 * Math.PI * j) / N;
      const rr = lr + t * Math.cos(v);
      row.push(s.vertex(-rr * Math.sin(u), rr * Math.cos(u), t * Math.sin(v)));
    }
    rows.push(row);
  }
  for (let a = 0; a < T; a++)
    for (let b = 0; b < N; b++) s.quad(rows[a][b], rows[a + 1][b], rows[a + 1][(b + 1) % N], rows[a][(b + 1) % N]);
  return s.build();
}

function bandStrandTemplate(g: GeometryBuilder): MeshDescription {
  return g.lathe(
    [
      [1, -0.5],
      [1, 0.5],
    ],
    { axis: "x", segments: 6 },
  );
}

interface Band {
  px: number;
  pz: number;
  qx: number;
  qz: number;
  span: number;
}

/** Where the drive bands go, from the rollers in bed order: a groove line at the +Z wall, pairs on alternating grooves. */
function layoutRollerBands(
  rollers: readonly Roller[],
  fullAxle: number,
  radius: number,
): { cord: number; bands: Band[] } {
  const zWall = fullAxle / 2;
  const cord = radius * BAND_CORD_PER_R;
  const inset = radius * BAND_INSET_PER_R;
  const spacing = radius * BAND_SPACING_PER_R;
  const grooves = [zWall - inset, zWall - inset - spacing];
  const out: Band[] = [];
  let minSpan = Infinity;
  for (let k = 0; k + 1 < rollers.length; k++) {
    const a = rollers[k];
    const b = rollers[k + 1];
    const sin = Math.sin(a.yaw);
    const cos = Math.cos(a.yaw);
    const zg = grooves[k % 2];
    const ta = (zg - a.z) / cos;
    const px = a.x + ta * sin;
    const pz = zg;
    const d = (b.x - px) * cos - (b.z - pz) * sin; // across to b's axle, along the axle normal (cos, -sin)
    minSpan = Math.min(minSpan, Math.abs(d));
    out.push({ px, pz, qx: px + d * cos, qz: pz - d * sin, span: Math.abs(d) });
  }
  return { cord, bands: out.filter((b) => b.span <= minSpan * 1.5) };
}

/** The bed's rollers: tubes, two axle stubs each, and the drive bands of a powered bed. */
function rollerBed(g: GeometryBuilder, rollers: readonly Roller[], fullAxle: number): MeshPart[] {
  const R = ROLLER_R;
  const y = -R; // the 'roller' link: the axles a radius under the conveying surface
  const sxy = R / ROLLER_TEMPLATE_R;
  const stub = { radius: STUB_R, length: 0.045, protrude: 0.0095 };
  const bodies: Instance[] = [];
  const stubs: Instance[] = [];
  for (const r of rollers) {
    const q = yawQuat(r.yaw);
    bodies.push({
      position: [r.x, y, r.z],
      quaternion: q,
      scale: [sxy, sxy, Math.max(0.01, r.length) / ROLLER_TEMPLATE_BODY],
    });
    const ax = Math.sin(r.yaw);
    const az = Math.cos(r.yaw);
    const so = r.length / 2 + stub.protrude;
    const ss: V3 = [stub.radius, stub.radius, stub.length];
    stubs.push({ position: [r.x + ax * so, y, r.z + az * so], quaternion: q, scale: ss });
    stubs.push({ position: [r.x - ax * so, y, r.z - az * so], quaternion: q, scale: ss });
  }
  const laid = layoutRollerBands(rollers, fullAxle, R);
  const lr = R + laid.cord * 0.4; // seated on the tube: the cord's inner edge just under the facet flats
  const arcs: Instance[] = [];
  const strands: Instance[] = [];
  for (const bd of laid.bands) {
    const yaw = Math.atan2(-(bd.qz - bd.pz), bd.qx - bd.px); // band frame: local X toward q, local Z the axle
    arcs.push({ position: [bd.qx, y, bd.qz], quaternion: yawQuat(yaw + Math.PI) });
    arcs.push({ position: [bd.px, y, bd.pz], quaternion: yawQuat(yaw) });
    const mx = (bd.px + bd.qx) / 2;
    const mz = (bd.pz + bd.qz) / 2;
    const sc: V3 = [bd.span, laid.cord / 2, laid.cord / 2];
    strands.push({ position: [mx, y + lr, mz], quaternion: yawQuat(yaw), scale: sc });
    strands.push({ position: [mx, y - lr, mz], quaternion: yawQuat(yaw), scale: sc });
  }
  return [
    { desc: rollerTemplate(g), material: "roller", instances: bodies },
    { desc: cylinderTemplate(g, 16), material: "roller", instances: stubs },
    { desc: bandArcTemplate(g, lr, laid.cord / 2), material: "band", instances: arcs },
    { desc: bandStrandTemplate(g), material: "band", instances: strands },
  ];
}

// ---- roller-conveyor/part.js `layoutRollers`, straight ------------------------------------------------------------
function layoutRollers(p: RollerParams): Roller[] {
  const axle = p.width - 0.004; // the body recessed just behind the frame walls
  const n = Math.max(1, Math.floor((p.length - 2 * ROLLER_R) / PITCH) + 1);
  const out: Roller[] = [];
  for (let d = 0; d < n; d++) out.push({ x: (d - (n - 1) / 2) * PITCH, z: 0, yaw: 0, length: axle });
  return out;
}

// ---- rig.json: the side frame walls, flanges, guards and their lips (box links; size and offset from the params) ---
function frame(g: GeometryBuilder, p: RollerParams): MeshPart[] {
  const L = p.length;
  const hw = p.width / 2;
  const walls: MeshDescription[] = [];
  const guards: MeshDescription[] = [];
  for (const [s, guarded] of [
    [-1, p.guards[0]],
    [1, p.guards[1]],
  ] as const) {
    walls.push(g.box([L, p.frameHeight, 0.01], [0, -p.frameHeight / 2 - 0.005, s * (hw + 0.005)]));
    walls.push(g.box([L, 0.005, 0.02], [0, -0.0025, s * (hw + 0.02)]));
    if (!guarded) continue;
    guards.push(g.box([L, 0.155, 0.005], [0, 0.0725, s * (hw + 0.0125)]));
    guards.push(g.box([L, 0.005, 0.02], [0, 0.1475, s * (hw + 0.025)]));
  }
  const out: MeshPart[] = [{ desc: g.merge(...walls), material: "frame" }];
  if (guards.length > 0) out.push({ desc: g.merge(...guards), material: "guard" });
  return out;
}

export function rollerGeometry(g: GeometryBuilder, p: RollerParams): MeshPart[] {
  const frameDepth = p.frameHeight + 0.005;
  const at = legStations(p.length).map((x) => ({
    position: [x - p.length / 2, -p.standHeight, 0] as const,
    yaw: 0,
    dir: x <= p.length / 2 ? 1 : -1,
  }));
  const legs = stands(g, p.width / 2, Math.max(0.05, p.standHeight - frameDepth), frameDepth, at);
  return [...rollerBed(g, layoutRollers(p), p.width - 0.004), ...frame(g, p), ...legs];
}
