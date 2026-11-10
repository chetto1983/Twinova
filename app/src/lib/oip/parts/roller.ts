// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the roller conveyor's look.
// - the rollers: examples/parts/roller-conveyor/part.js `layoutRollers` (its straight branch: skew is 4.4b) and
//   examples/parts/_shared/rollerBed.js (tubes, axle stubs and drive bands, drawn as instanced banks). The band
//   layout's checks for skewed or clipped rollers are left out with the skew: on a straight bed they never apply;
// - the side frame, its flanges and the guards: the boxes of examples/parts/roller-conveyor/rig.json;
// - the floor stands: examples/parts/_shared/legs.js `legPieces` at src/render/parts/common.ts `legStations`, the
//   bolts as its `hexBoltGeo` (a six-sided cylinder).
// The origin is the rollers' top (the conveying surface), x along the flow centred on the bed, y up, z across.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import { type Instance, type MeshPart, type V3, yawQuat } from "./part";

type V2 = readonly [number, number];

export interface RollerParams {
  length: number;
  width: number;
  frameHeight: number;
  /** The conveying surface above the floor (OIP's standHeight). */
  standHeight: number;
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

/** A capped unit cylinder along Z, centred: the axle stub, and with six segments OIP's hex bolt head. */
function cylinderTemplate(g: GeometryBuilder, segments: number): MeshDescription {
  return g.lathe(
    [
      [0, -0.5],
      [1, -0.5],
      [1, 0.5],
      [0, 0.5],
    ],
    { axis: "z", segments },
  );
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
  for (const s of [-1, 1]) {
    walls.push(g.box([L, p.frameHeight, 0.01], [0, -p.frameHeight / 2 - 0.005, s * (hw + 0.005)]));
    walls.push(g.box([L, 0.005, 0.02], [0, -0.0025, s * (hw + 0.02)]));
    guards.push(g.box([L, 0.155, 0.005], [0, 0.0725, s * (hw + 0.0125)]));
    guards.push(g.box([L, 0.005, 0.02], [0, 0.1475, s * (hw + 0.025)]));
  }
  return [
    { desc: g.merge(...walls), material: "frame" },
    { desc: g.merge(...guards), material: "guard" },
  ];
}

// ---- legs.js `legPieces` at common.ts `legStations` ---------------------------------------------------------------
const LEG_TAIL_OFFSET = 0.15;
const LEG_SPACING = 3.05;
const LEG_CLEARANCE = 1.0;

function legStations(span: number): number[] {
  const sMin = LEG_TAIL_OFFSET;
  const sMax = span - LEG_TAIL_OFFSET;
  const out = [sMin];
  const first = Math.ceil((sMin + LEG_CLEARANCE) / LEG_SPACING) * LEG_SPACING;
  const last = Math.floor((sMax - LEG_CLEARANCE) / LEG_SPACING) * LEG_SPACING;
  for (let s = first; s <= last + 1e-6; s += LEG_SPACING) out.push(s);
  if (sMax >= 0 && Math.abs(sMax - sMin) > 1e-6) out.push(sMax);
  return out.map((s) => s - span / 2);
}

/** A stand's pieces in its own frame (floor at y 0): boxes [cx, cy, cz, sx, sy, sz, rz] and bolts [cx, cy, cz, r, depth, up]. */
function legPieces(hw: number, height: number, dir: number, frameDepth: number) {
  const boxes: number[][] = [];
  const bolts: [number, number, number, number, number, boolean][] = [];
  const box = (x0: number, y0: number, z0: number, x1: number, y1: number, z1: number) =>
    boxes.push([
      (x0 + x1) / 2,
      (y0 + y1) / 2,
      (z0 + z1) / 2,
      Math.max(x1 - x0, 1e-4),
      Math.max(y1 - y0, 1e-4),
      Math.max(z1 - z0, 1e-4),
      0,
    ]);
  const bolt = (x: number, y: number, z: number, up: boolean) => bolts.push([x, y, z, 0.009, 0.007, up]);
  const FACE = hw + 0.01; // the side frame's outer face: the uprights stand flush under it
  const LW = 0.0508;
  const LD = Math.min(0.0445, Math.max(hw - 0.02, 0.02));
  const PT = 0.009;
  const CH = 0.076;
  const TELE = height > 0.45 ? 0.3 : 0; // the telescoping inner leg's top, on a stand tall enough to adjust
  const RISE = Math.max(0, Math.min(0.127, frameDepth - 0.01)); // bracket rise up the web
  for (const s of [1, -1]) {
    const zo = s * FACE;
    const zi = s * (FACE - LD);
    const zc = s * (FACE - LD / 2);
    const lo = Math.min(zo, zi);
    const hi = Math.max(zo, zi);
    if (TELE > 0) {
      box(-0.042, PT, lo + 0.004, 0.042, TELE, hi - 0.004); // inner leg
      box(-LW, TELE - 0.05, lo, LW, height, hi); // outer leg, sleeved over it
      bolt(-0.025, TELE - 0.025, zo + s * 0.0035, false);
      bolt(0.025, TELE - 0.025, zo + s * 0.0035, false);
    } else box(-LW, PT, lo, LW, height, hi);
    box(-0.0762, 0.003, zc - 0.0508, 0.0762, PT, zc + 0.0508); // foot plate
    bolt(-0.055, PT + 0.0035, zc, true);
    bolt(0.055, PT + 0.0035, zc, true);
    const bo = zo + s * 0.006;
    box(-0.019, height - CH, Math.min(zo, bo), 0.019, height + RISE, Math.max(zo, bo)); // mounting bracket on the frame web
    if (0.04 + 0.009 <= RISE) bolt(0, height + 0.04, bo + s * 0.0035, false);
    if (0.1 + 0.009 <= RISE) bolt(0, height + 0.1, bo + s * 0.0035, false);
    bolt(0, height - CH / 2, bo + s * 0.0035, false);
    if (dir !== 0 && height > 1.0) {
      // knee brace: 1-1/4" angle from the upright up to the frame web, outside both
      const d = Math.min(0.6, 0.45 * height);
      const ay = height - d;
      const bx = dir * d;
      const by = height + 0.04;
      const len = Math.hypot(bx, by - ay);
      const ux = bx / len;
      const uy = (by - ay) / len;
      const kz = s * (FACE + 0.016 + 0.006);
      boxes.push([bx / 2, (ay + by) / 2, kz, len, 0.032, 0.032, Math.atan2(by - ay, bx)]);
      bolt(ux * 0.03, ay + uy * 0.03, kz + s * 0.0195, false);
      bolt(bx - ux * 0.03, by - uy * 0.03, kz + s * 0.0195, false);
    }
  }
  const zin = FACE - LD + 0.005; // the uprights' inner faces, lapped 5 mm
  box(-0.0222, height - CH, -zin, 0.0222, height, zin); // support channel across under the bed
  if (height >= 0.6) {
    // horizontal cross braces, clear of the telescoping joint
    const n = Math.min(Math.max(Math.floor(height / 0.9), 1), 3);
    const y0 = TELE + 0.1;
    const y1 = height - CH - 0.1;
    if (y1 > y0) {
      for (let k = 0; k < n; k++) {
        const y = y0 + ((k + 1) * (y1 - y0)) / (n + 1);
        box(-0.0222, y - 0.019, -zin, 0.0222, y + 0.019, zin);
      }
    }
  }
  return { boxes, bolts };
}

/** The stands under the bed, at the floor (y = −standHeight in the bed's frame). */
function legs(g: GeometryBuilder, p: RollerParams): MeshPart[] {
  const frameDepth = p.frameHeight + 0.005;
  const height = Math.max(0.05, p.standHeight - frameDepth);
  const boxes: Instance[] = [];
  const bolts: Instance[] = [];
  const up: readonly [number, number, number, number] = [-Math.SQRT1_2, 0, 0, Math.SQRT1_2]; // the bolt's axis z → y
  for (const x of legStations(p.length)) {
    const dir = x <= 0 ? 1 : -1;
    const pieces = legPieces(p.width / 2, height, dir, frameDepth);
    for (const [cx, cy, cz, sx, sy, sz, rz] of pieces.boxes) {
      boxes.push({
        position: [x + cx, cy - p.standHeight, cz],
        quaternion: [0, 0, Math.sin(rz / 2), Math.cos(rz / 2)],
        scale: [sx, sy, sz],
      });
    }
    for (const [cx, cy, cz, r, depth, isUp] of pieces.bolts) {
      bolts.push({
        position: [x + cx, cy - p.standHeight, cz],
        quaternion: isUp ? up : [0, 0, 0, 1],
        scale: [r, r, depth],
      });
    }
  }
  return [
    { desc: g.box([1, 1, 1], [0, 0, 0]), material: "leg", instances: boxes },
    { desc: cylinderTemplate(g, 6), material: "bolt", instances: bolts },
  ];
}

export function rollerGeometry(g: GeometryBuilder, p: RollerParams): MeshPart[] {
  return [...rollerBed(g, layoutRollers(p), p.width - 0.004), ...frame(g, p), ...legs(g, p)];
}
