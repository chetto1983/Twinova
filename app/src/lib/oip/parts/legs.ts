// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the conveyor floor stand every legged conveyor draws,
// examples/parts/_shared/legs.js `legPieces`, at src/render/parts/common.ts `legStations`, its bolts as `hexBoltGeo` (a
// six-sided cylinder). A stand is placed at a station and turned to it, as the straight belt and the roller set it
// along their length and the curves along their arc (examples/parts/curved-conveyor/part.js `legAt`).
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import type { Instance, MeshPart, Quat, V3 } from "./part";

/** A capped unit cylinder along Z (or `opts.axis`), centred: the axle stub, and with six segments OIP's hex bolt head. */
export function cylinderTemplate(
  g: GeometryBuilder,
  segments: number,
  opts: { axis?: "y" | "z"; startAngle?: number; smooth?: boolean } = {},
): MeshDescription {
  return g.lathe(
    [
      [0, -0.5],
      [1, -0.5],
      [1, 0.5],
      [0, 0.5],
    ],
    { axis: "z", segments, ...opts },
  );
}

const LEG_TAIL_OFFSET = 0.15;
const LEG_SPACING = 3.05;
const LEG_CLEARANCE = 1.0;

/** The stations along a span of `span` metres, from its start: the end insets and the regular spacing between. */
export function legStations(span: number): number[] {
  const sMin = LEG_TAIL_OFFSET;
  const sMax = span - LEG_TAIL_OFFSET;
  const out = [sMin];
  const first = Math.ceil((sMin + LEG_CLEARANCE) / LEG_SPACING) * LEG_SPACING;
  const last = Math.floor((sMax - LEG_CLEARANCE) / LEG_SPACING) * LEG_SPACING;
  for (let s = first; s <= last + 1e-6; s += LEG_SPACING) out.push(s);
  if (sMax >= 0 && Math.abs(sMax - sMin) > 1e-6) out.push(sMax);
  return out;
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

/** Where a stand stands: its centre on the floor, its turn about +Y, and the way its knee braces lean (±1, 0 none). */
export interface Station {
  position: V3;
  yaw: number;
  dir: number;
}

/** `a` then `b`: the rotation b, then a. */
function compose(a: Quat, b: Quat): Quat {
  const [ax, ay, az, aw] = a;
  const [bx, by, bz, bw] = b;
  return [
    aw * bx + ax * bw + ay * bz - az * by,
    aw * by - ax * bz + ay * bw + az * bx,
    aw * bz + ax * by - ay * bx + az * bw,
    aw * bw - ax * bx - ay * by - az * bz,
  ];
}

/** The stands at `at`, for a bed `hw` half wide whose frame bottom is `height` above the floor and `frameDepth` below
 *  its deck. */
export function stands(g: GeometryBuilder, hw: number, height: number, frameDepth: number, at: Station[]): MeshPart[] {
  const boxes: Instance[] = [];
  const bolts: Instance[] = [];
  const up: Quat = [-Math.SQRT1_2, 0, 0, Math.SQRT1_2]; // the bolt's axis z → y
  for (const { position, yaw, dir } of at) {
    const turn: Quat = [0, Math.sin(yaw / 2), 0, Math.cos(yaw / 2)];
    const place = (cx: number, cy: number, cz: number): V3 => [
      position[0] + cx * Math.cos(yaw) + cz * Math.sin(yaw),
      position[1] + cy,
      position[2] - cx * Math.sin(yaw) + cz * Math.cos(yaw),
    ];
    const pieces = legPieces(hw, height, dir, frameDepth);
    for (const [cx, cy, cz, sx, sy, sz, rz] of pieces.boxes) {
      boxes.push({
        position: place(cx, cy, cz),
        quaternion: compose(turn, [0, 0, Math.sin(rz / 2), Math.cos(rz / 2)]),
        scale: [sx, sy, sz],
      });
    }
    for (const [cx, cy, cz, r, depth, isUp] of pieces.bolts) {
      bolts.push({
        position: place(cx, cy, cz),
        quaternion: compose(turn, isUp ? up : [0, 0, 0, 1]),
        scale: [r, r, depth],
      });
    }
  }
  return [
    { desc: g.box([1, 1, 1], [0, 0, 0]), material: "leg", instances: boxes },
    { desc: cylinderTemplate(g, 6), material: "bolt", instances: bolts },
  ];
}
