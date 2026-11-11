// Copied from Open Industry Project, examples/parts/blade-stop/part.js `geometry()` at 1e7488c (MIT, Copyright (c)
// 2024-present Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript. The blade
// stop, drawn: a static housing (slab, core wall with light panels, top rail, black strip, end plates) with a pneumatic
// cylinder at each end, and the moving assembly (the orange blade, its end posts and clamps, the guide rods down into
// the cylinders, the bolts). The origin is the host's running surface (y = 0) at the blade's centreline: +X the flow,
// across the blade's thickness; +Z across the lane, centred; +Y up. The moving link is drawn at its rest, lowered: OIP's
// `raise` joint at 0, with `airPressureHeight` at its 0.07 default, so the view raises it by lift × (raiseHeight + 0.05).
//
// The shapes are kept in native units. They are scaled by S and a native z maps in two bands: |z| ≤ 1 (the middle
// sections) stretches to ±cornerZ, and beyond that (the corner nodes) keeps its size and slides out with it.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import type { MeshPart } from "./part";

type V2 = readonly [number, number];
type Face = "x-" | "x+" | "y-" | "y+" | "z-" | "z+";

export interface StopperParams {
  width: number;
  raiseHeight: number;
}

const S = 0.45; // native → metres
const NATIVE_TOP = 0.2494; // blade/base top in native units
const NATIVE_BLADE_X = 0.1036; // blade centreline in native units
const LOWERED_TOP = -0.05; // blade-top rest height (below the belt surface)
const HOUSING_TOP = -0.12; // housing top pinned under the surface (clears every roller class)
// OIP's default mount of the moving assembly above the housing; Twinova does not expose it.
const AIR_PRESSURE_HEIGHT = LOWERED_TOP - HOUSING_TOP;
// The guide rods are authored for a 0.06 raiseHeight: this travel from the rest top.
const ROD_TRAVEL = 0.06 - LOWERED_TOP;
const X0 = -NATIVE_BLADE_X * S; // shifts the blade centreline to x = 0
const BLADE_Y = LOWERED_TOP - S * NATIVE_TOP; // the blade's native origin at rest
const BASE_Y = HOUSING_TOP - S * NATIVE_TOP; // the housing's native origin
const CORNER_REACH = S * 0.2; // cornerZ = width/2 − S·0.2, floored at S·0.01
const CORNER_MIN = S * 0.01;
const BEVEL = 0.002 * S; // every box has the model's 0.002 bevel
const D2R = Math.PI / 180;

export function stopperGeometry(g: GeometryBuilder, p: StopperParams): MeshPart[] {
  const cz = Math.max(p.width / 2 - CORNER_REACH, CORNER_MIN);
  // Cylinders are sized to the stroke. The model's cylinder + guide rods are authored for ROD_TRAVEL (rest top to
  // raiseHeight); a longer one would pull the rods clean out of the cylinders and leave the blade floating. So the extra
  // travel lengthens the rods and the cylinder body DOWNWARD by the same amount (same sheath at full raise as the
  // default), stopping 1 cm above the housing's floor slab.
  const travel = p.raiseHeight - (AIR_PRESSURE_HEIGHT + HOUSING_TOP);
  const room = AIR_PRESSURE_HEIGHT + (0.6124 - 0.3385) * S - 0.01;
  const ext = Math.max(0, Math.min(travel - ROD_TRAVEL, room)) / S; // native units
  const mz = (z: number): number => {
    const a = Math.abs(z);
    return a <= 1 ? z * cz : Math.sign(z) * (cz + (a - 1) * S);
  };
  const X = (x: number): number => x * S + X0;
  // a chamfered box from native extents at a mount height (every box ends in the rigid corner region, so its chamfer
  // is B·S all round)
  const box = (
    list: MeshDescription[],
    yOff: number,
    [x0, x1]: V2,
    [y0, y1]: V2,
    [z0, z1]: V2,
    square?: readonly Face[],
  ): void => {
    const a = [X(x0), y0 * S + yOff, mz(z0)];
    const b = [X(x1), y1 * S + yOff, mz(z1)];
    const size = [b[0] - a[0], b[1] - a[1], b[2] - a[2]] as const;
    list.push(g.bevelBox(size, [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (a[2] + b[2]) / 2], BEVEL, square));
  };
  // a round rod along y about native (x, z): profile [r, y] in native units, 8 facets, smooth
  const rodY = (list: MeshDescription[], yOff: number, x: number, z: number, prof: readonly V2[]): void => {
    const scaled = prof.map((q): V2 => [q[0] * S, q[1] * S]);
    list.push(g.lathe(scaled, { axis: "y", center: [X(x), yOff, mz(z)], segments: 8 }));
  };
  // a hex bolt along z at native (x, y): profile [r, z] for the +z side; the −z side is its mirror, walked the other
  // way so the outside stays on the right
  const hexZ = (
    list: MeshDescription[],
    x: number,
    y: number,
    prof: readonly V2[],
    deg: number,
    side: number,
  ): void => {
    const mapped = prof.map((q): V2 => [q[0] * S, mz(side * q[1])]);
    if (side < 0) mapped.reverse();
    const center = [X(x), y * S + BLADE_Y, 0] as const;
    list.push(g.lathe(mapped, { axis: "z", center, segments: 6, startAngle: deg * D2R, smooth: false }));
  };
  // a 45° flare strip where a panel meets the slab or rail, the full run long, facing away from corner c
  const flare = (list: MeshDescription[], p0: V2, q0: V2, c0: V2): void => {
    const s = g.surface();
    const z0 = mz(-1.15);
    const z1 = mz(1.15);
    const [P, Q, C] = [p0, q0, c0].map((v): V2 => [X(v[0]), v[1] * S + BASE_Y]);
    const a = s.vertex(P[0], P[1], z0);
    const b = s.vertex(Q[0], Q[1], z0);
    const cc = s.vertex(Q[0], Q[1], z1);
    const d = s.vertex(P[0], P[1], z1);
    // a→b→cc→d faces the right-hand side of P→Q; keep that if it points away from c, else wind it the other way
    const away = (P[0] - C[0]) * (Q[1] - P[1]) - (P[1] - C[1]) * (Q[0] - P[0]) > 0;
    if (away) s.quad(a, b, cc, d);
    else s.quad(a, d, cc, b);
    list.push(s.build());
  };

  const steel: MeshDescription[] = [];
  const plate: MeshDescription[] = [];
  const black: MeshDescription[] = [];
  // housing: bottom slab, the core wall (light panel faces both sides), the top rail on the +x side, the black strip on
  // top, and an end plate closing each end; faces meeting a neighbour flush stay square
  box(steel, BASE_Y, [0, 0.4], [-0.6606, -0.6124], [-1.15, 1.15]);
  box(steel, BASE_Y, [0.2682, 0.4], [0.2083, 0.2293], [-1.15, 1.15], ["x-", "y+"]);
  box(plate, BASE_Y, [0.2, 0.2662], [-0.6124, 0.2293], [-1.15, 1.15], ["y-", "y+"]);
  box(black, BASE_Y, [0.2, 0.4], [0.2293, 0.2494], [-1.152, 1.152], ["y-"]);
  flare(plate, [0.2662, -0.6104], [0.2682, -0.6124], [0.2662, -0.6124]);
  flare(plate, [0.2, -0.6104], [0.198, -0.6124], [0.2, -0.6124]);
  flare(plate, [0.2662, 0.2063], [0.2682, 0.2083], [0.2662, 0.2083]);
  box(steel, BASE_Y, [0, 0.4], [-0.6606, 0.2494], [1.15, 1.2]);
  box(steel, BASE_Y, [0, 0.4], [-0.6606, 0.2494], [-1.2, -1.15]);
  // pneumatic cylinders: two black blocks, the round body between them, a cap on top (the bottom block and the body
  // reach down by ext for a long stroke)
  const airY = BASE_Y + AIR_PRESSURE_HEIGHT;
  for (let side = -1; side <= 1; side += 2) {
    const zs: V2 = side > 0 ? [1.0463, 1.1527] : [-1.1527, -1.0463];
    box(black, airY, [0.0506, 0.1569], [-0.1177, -0.0761], zs);
    box(black, airY, [0.0506, 0.1569], [-0.3385 - ext, -0.2969 - ext], zs);
    rodY(plate, airY, 0.10375, side * 1.0995, [
      [0, -0.3042 - ext],
      [0.0431, -0.3042 - ext],
      [0.0431, -0.1104],
      [0, -0.1104],
    ]);
    rodY(steel, airY, 0.10375, side * 1.09945, [
      [0, -0.083],
      [0.02615, -0.083],
      [0.02615, -0.0652],
      [0.02405, -0.0632],
      [0, -0.0632],
    ]);
  }

  const bBlade: MeshDescription[] = [];
  const bSteel: MeshDescription[] = [];
  const bPlate: MeshDescription[] = [];
  box(bBlade, BLADE_Y, [0.0786, 0.1288], [0, 0.2461], [-1.0843, 1.0843]);
  for (let sd = -1; sd <= 1; sd += 2) {
    const zr = (z0: number, z1: number): V2 => (sd > 0 ? [z0, z1] : [-z1, -z0]);
    // the end post the blade is bolted to, and the clamp at its foot
    box(bSteel, BLADE_Y, [0.0786, 0.1288], [-0.0437, 0.2462], zr(1.0877, 1.1113));
    box(bSteel, BLADE_Y, [0.086, 0.1214], [-0.0561, -0.0111], zr(1.0758, 1.1232));
    // the guide rod down into the cylinder
    rodY(bPlate, BLADE_Y, 0.10375, sd * 1.09945, [
      [0, -0.3271 - ext],
      [0.01445, -0.3271 - ext],
      [0.01445, -0.0556],
      [0, -0.0556],
    ]);
    // the hex bolt through the clamp, and the two bolt heads on the post's outer face
    const clampBolt: V2[] = [
      [0, 1.0701],
      [0.00989, 1.0701],
      [0.01212, 1.0721],
      [0.01212, 1.1255],
      [0.00989, 1.1275],
      [0, 1.1275],
    ];
    hexZ(bPlate, 0.10293, -0.03405, clampBolt, 49.8, sd);
    hexZ(bPlate, 0.10367, 0.21218, boltHead(0.01056, 0.00832), 10.8, sd);
    hexZ(bPlate, 0.10367, 0.02902, boltHead(0.01061, 0.00824), 32, sd);
  }

  return [
    { desc: g.merge(...steel), material: "darkSteel" },
    { desc: g.merge(...plate), material: "plate" },
    { desc: g.merge(...black), material: "beltBlack" },
    { desc: g.merge(...bBlade), material: "bladeOrange", link: "moving" },
    { desc: g.merge(...bSteel), material: "darkSteel", link: "moving" },
    { desc: g.merge(...bPlate), material: "plate", link: "moving" },
  ];
}

/** A bolt head on the post's outer face: profile [r, z], its chamfer from radius r to rc at the outer end. */
function boltHead(r: number, rc: number): V2[] {
  return [
    [0, 1.1091],
    [r, 1.1091],
    [r, 1.1145],
    [rc, 1.1165],
    [0, 1.1165],
  ];
}
