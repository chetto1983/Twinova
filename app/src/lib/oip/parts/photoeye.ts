// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the diffuse photo eye's housing and its bracket
// (examples/parts/_shared/sensorPhotoeye.js `photoeyeHousing`, with sensorOutlines.js `rrect` and `circle`). The
// retro-reflective mode's reflector is OIP's only, and is left out.
// Part-local metres: +Z the front (the beam axis), y 0 the underside of the connector, the beam at `PHOTOEYE.emitY`.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import type { MeshPart } from "./part";

type V2 = readonly [number, number];
type V3 = readonly [number, number, number];

export const PHOTOEYE = {
  halfX: 0.0225, // 45 mm wide
  back: -0.036,
  front: 0.037, // the lens face: the beam origin and the guard contact
  bodyY0: 0.036, // the housing's underside; the M8 plug hangs below it
  bodyY1: 0.15,
  emitY: 0.1, // inside the lens window
};

const BRACKET = {
  gauge: 0.003, // the sheet
  width: 0.052, // the toe across the beam, from the plate: clear of the lens window by 7.5 mm
  wall: 0.01, // u of the wall's face, which the toe's back lies on
  clearGuard: 0.023, // u the plate keeps out past above the lip
  clearLip: 0.033, // ...and past at the lip
  lipY: [0.015, 0.02], // the lip's underside and top, in part-local y
  toeTop: -0.025, // where the plate steps in to the toe
  bottom: -0.07, // the toe's foot
  bolts: [-0.06, -0.035], // its two bolts into the wall
  back: 0.046, // u of the plate's back edge
};

/** A rounded-corner rectangle, for windows and panels. */
function rrect(x0: number, y0: number, x1: number, y1: number, r: number): V2[] {
  const pts: V2[] = [];
  const corners: [number, number, number][] = [
    [x1 - r, y0 + r, -Math.PI / 2],
    [x1 - r, y1 - r, 0],
    [x0 + r, y1 - r, Math.PI / 2],
    [x0 + r, y0 + r, Math.PI],
  ];
  for (const [cx, cy, a0] of corners)
    for (let k = 0; k <= 3; k++) {
      const a = a0 + (k * Math.PI) / 6;
      pts.push([cx + r * Math.cos(a), cy + r * Math.sin(a)]);
    }
  return pts;
}

/** A 16-sided circle, for mounting holes. */
function circle(cy: number, cz: number, r: number): V2[] {
  const pts: V2[] = [];
  for (let k = 0; k < 16; k++) {
    const a = (k * Math.PI) / 8;
    pts.push([cy + r * Math.cos(a), cz + r * Math.sin(a)]);
  }
  return pts;
}

function hexBolt(g: GeometryBuilder, axis: "x" | "z", center: V3): MeshDescription {
  return g.lathe(
    [
      [0, 0],
      [0.0065, 0],
      [0.0065, 0.004],
      [0, 0.004],
    ],
    { axis, center, segments: 6, smooth: false },
  );
}

/** The side plate in the x plane [x0, x0 + gauge], from yTop down, its face nearest the bed at uIn above the lip. */
function sidePlate(
  g: GeometryBuilder,
  x0: number,
  yTop: number,
  uIn: number,
  u2z: (u: number) => number,
): MeshDescription {
  const B = BRACKET;
  const outline: V2[] = [
    [yTop, uIn],
    [yTop, B.back],
    [B.bottom, B.back],
    [B.bottom, B.wall + B.gauge],
    [B.toeTop, B.wall + B.gauge],
    [B.toeTop, B.clearLip],
    [B.lipY[1] + 0.003, B.clearLip],
    [B.lipY[1] + 0.003, uIn],
  ];
  return g.extrude(
    outline.map(([y, u]): V2 => [y, u2z(u)]),
    x0,
    x0 + B.gauge,
    { axis: "x" },
  );
}

export function photoeyeGeometry(g: GeometryBuilder): MeshPart[] {
  const X = PHOTOEYE.halfX;
  const BACK = PHOTOEYE.back;
  const FRONT = PHOTOEYE.front;
  const Y0 = PHOTOEYE.bodyY0;
  const Y1 = PHOTOEYE.bodyY1;
  const CH = 0.024; // the chamfer across the lower back corner
  // the housing: its side profile (y, z) run across the width, with two mounting holes through it
  const body = g.extrude(
    [
      [Y0, BACK + CH],
      [Y0, FRONT],
      [Y1, FRONT],
      [Y1, BACK],
      [Y0 + CH, BACK],
    ],
    -X,
    X,
    {
      axis: "x",
      bevel: 0.0015,
      holes: [circle(Y0 + 0.012, FRONT - 0.011, 0.0042), circle(Y1 - 0.012, FRONT - 0.011, 0.0042)],
    },
  );
  // the lens window on the face, the beam height inside it
  const lens = g.extrude(rrect(-0.017, 0.062, 0.017, 0.114, 0.004), FRONT, FRONT + 0.0008, { bevel: 0.0003 });
  // the top cap, stopping short of the face, and the sensitivity pot in it
  const cap = g.bevelBox([2 * X, 0.005, FRONT - 0.004 - BACK], [0, Y1 + 0.0025, (BACK + FRONT - 0.004) / 2], 0.001, [
    "y-",
  ]);
  const pot = g.lathe(
    [
      [0, 0],
      [0.0055, 0],
      [0.0055, 0.0038],
      [0.0047, 0.0045],
      [0, 0.0045],
    ],
    { axis: "y", center: [0, Y1 + 0.005, -0.012], segments: 20 },
  );
  const slot = g.box([0.008, 0.0008, 0.0012], [0, Y1 + 0.0096, -0.012]);
  // the M8 plug under the housing: a turned body and a hex coupling nut
  const PZ = (BACK + CH + FRONT) / 2;
  const plug = g.lathe(
    [
      [0, 0],
      [0.0048, 0],
      [0.0052, 0.0005],
      [0.0052, 0.009],
      [0.0056, 0.009],
      [0.0056, 0.019],
      [0.0068, 0.022],
      [0.0068, Y0],
      [0, Y0],
    ],
    { axis: "y", center: [0, 0, PZ], segments: 20 },
  );
  const nut = g.lathe(
    [
      [0, 0.009],
      [0.0072, 0.009],
      [0.0072, 0.018],
      [0, 0.018],
    ],
    { axis: "y", center: [0, 0, PZ], segments: 6, smooth: false },
  );
  // the bracket: a side plate profiled round the rail's guard and lip, a toe bolted to its wall, and a hex bolt head
  // through each of the housing's holes
  const BT = BRACKET.gauge;
  const BW = BRACKET.width;
  const U2Z = (u: number): number => FRONT + 0.02 - u; // this side: z falls as u grows
  const HZ = FRONT - 0.011; // the housing's holes
  const toe = g.box(
    [BW, BRACKET.toeTop - BRACKET.bottom, BT],
    [X + BW / 2, (BRACKET.bottom + BRACKET.toeTop) / 2, (U2Z(BRACKET.wall) + U2Z(BRACKET.wall + BT)) / 2],
  );
  const toeBolts = BRACKET.bolts.map((y) =>
    hexBolt(g, "z", [X + BW * 0.62, y, Math.min(U2Z(BRACKET.wall + BT), U2Z(BRACKET.wall + BT + 0.004))]),
  );
  const bracket = g.merge(
    sidePlate(g, X, Y1 - 0.002, BRACKET.clearGuard, U2Z),
    toe,
    ...toeBolts,
    hexBolt(g, "x", [X + BT, Y0 + 0.012, HZ]),
    hexBolt(g, "x", [X + BT, Y1 - 0.012, HZ]),
  );
  return [
    { desc: body, material: "body" },
    { desc: lens, material: "lens" },
    { desc: g.merge(cap, slot, plug, nut), material: "black" },
    { desc: pot, material: "pot" },
    { desc: bracket, material: "steel" },
  ];
}
