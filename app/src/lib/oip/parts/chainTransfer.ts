// Copied from Open Industry Project, examples/parts/chain-transfer/part.js `geometry()` at 1e7488c (MIT, Copyright (c)
// 2024-present Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript. The chain
// lanes, drawn: each lane is built in its own frame and native units (+X along the lane, +Y up, +Z across), then
// turned into place (lane +X → part −Z, the divert direction), thinned across to a 4 cm lane, and the lanes merge per
// material. The origin is the conveying deck's top; the view raises the whole bank by the lift.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import type { MeshPart } from "./part";

type V2 = readonly [number, number];

export interface ChainTransferParams {
  chains: number;
  distance: number;
  laneLength: number;
}

const NATIVE_LANE_TOP = 0.0058;
const LANE_Y = -NATIVE_LANE_TOP;

export function chainTransferGeometry(g: GeometryBuilder, p: ChainTransferParams): MeshPart[] {
  const n = Math.max(1, Math.round(p.chains));
  const pitch = p.distance;
  const half = Math.max(p.laneLength, 0.1) / 2;
  const EX = Math.max(half - 0.1096, 0.02); // the run's half-length: the feet reach 0.1096 past it, so the lane measures laneLength overall
  const SA = 0.04 / 0.087; // across: the model's 0.087 channel thinned to the app's 4 cm lane
  const PIVOT = -0.094309; // the sprocket axle, below the strand crown
  const YAW = [0, Math.SQRT1_2, 0, Math.SQRT1_2] as const; // +90° about +Y: lane +X → part −Z, lane +Z → part +X
  const LINKS_PER_M = 16;
  const frame: MeshDescription[] = [];
  const support: MeshDescription[] = [];
  const chain: MeshDescription[] = [];

  // ---- the frame (blue): two rails under the lane, the U channel the chain runs in, its round-topped end caps,
  // and a foot at each end (two tapered gussets + a slotted mounting flange outboard of each)
  for (let s = -1; s <= 1; s += 2)
    frame.push(g.bevelBox([2 * EX, 0.1561, 0.01], [0, -0.27835, s * 0.0554], 0.001, ["x-", "x+"]));
  const channel: V2[] = [
    [0.0221, -0.28],
    [0.0241, -0.278],
    [0.0241, -0.002],
    [0.0261, 0],
    [0.0398, 0],
    [0.0418, -0.002],
    [0.0418, -0.298],
    [0.0398, -0.3],
    [-0.0398, -0.3],
    [-0.0418, -0.298],
    [-0.0418, -0.002],
    [-0.0398, 0],
    [-0.0261, 0],
    [-0.0241, -0.002],
    [-0.0241, -0.278],
    [-0.0221, -0.28],
  ];
  frame.push(
    g.extrude(
      channel.map((q): V2 => [q[1], q[0]]),
      -EX,
      EX,
      { axis: "x" },
    ),
  ); // outline in (y, z)
  const capWall: V2[] = [
    [0, 0],
    [0.1, 0],
    [0.1, 0.2],
  ];
  for (let k = 1; k <= 4; k++)
    capWall.push([0.1 * Math.cos((k * Math.PI) / 8), 0.2 + 0.1 * Math.sin((k * Math.PI) / 8)]);
  const gusset: V2[] = [
    [0, 0],
    [0.1006, 0],
    [0.1006, -0.0761],
    [0.0672, -0.0791],
    [0.0381, -0.0943],
    [0.0229, -0.1234],
    [0.0196, -0.1556],
    [0, -0.1561],
  ];
  const FOOT_Y = -0.200264;
  for (let e = -1; e <= 1; e += 2) {
    const X0 = e * EX;
    const out = (pts: readonly V2[], dy: number): V2[] => pts.map((q): V2 => [X0 + e * q[0], q[1] + dy]);
    // end cap: two walls (the seam edge onto the channel stays square) and the floor between them
    frame.push(g.extrude(out(capWall, -0.3), 0.0241, 0.0418, { bevel: 0.002, square: [capWall.length - 1] }));
    frame.push(g.extrude(out(capWall, -0.3), -0.0418, -0.0241, { bevel: 0.002, square: [capWall.length - 1] }));
    frame.push(
      g.extrude(
        out(
          [
            [0, 0],
            [0.098, 0],
            [0.1, 0.002],
            [0.1, 0.018],
            [0.098, 0.02],
            [0, 0.02],
          ],
          -0.3,
        ),
        -0.0241,
        0.0241,
      ),
    );
    // foot: gussets (square where they meet the rail and the flange), then a flange tab with a hex hole each side
    frame.push(g.extrude(out(gusset, FOOT_Y), 0.0504, 0.0604, { bevel: 0.001, square: [gusset.length - 1, 1] }));
    frame.push(g.extrude(out(gusset, FOOT_Y), -0.0604, -0.0504, { bevel: 0.001, square: [gusset.length - 1, 1] }));
    for (let sz = -1; sz <= 1; sz += 2) {
      const z0 = sz * 0.0504;
      const z1 = sz * 0.1248;
      const cy = FOOT_Y - 0.0361;
      const cz = sz * 0.0876;
      const hex: V2[] = [];
      for (let h = 0; h < 6; h++)
        hex.push([cy + 0.0137 * Math.cos((h * Math.PI) / 3), cz + 0.0137 * Math.sin((h * Math.PI) / 3)]);
      const tab: V2[] = [
        [FOOT_Y - 0.0761, z0],
        [FOOT_Y, z0],
        [FOOT_Y, z1],
        [FOOT_Y - 0.0761, z1],
      ];
      frame.push(g.extrude(tab, X0 + e * 0.0996, X0 + e * 0.1096, { axis: "x", holes: [hex], bevel: 0.001 }));
    }
    // ---- the sprocket (grey) the chain turns round
    support.push(
      g.lathe(
        [
          [0, -0.0142],
          [0.07485, -0.0142],
          [0.07485, 0.0142],
          [0, 0.0142],
        ],
        { axis: "z", center: [X0 + e * 0.01045, PIVOT, 0], segments: 16 },
      ),
    );
  }

  // ---- the chain: one loop of stations (an outer and an inner point each), walked from the middle of the bottom
  // run toward −X, up round the −X sprocket, along the top run, down round the +X one and back — so +U runs along
  // the top toward +X. U is the outer edge's arc length in tiles; the seam (U wraps) sits under the lane.
  const BEND: [V2, V2][] = [
    [
      [0, -0.1],
      [0, -0.0772],
    ],
    [
      [0.0488, -0.0925],
      [0.04, -0.0714],
    ],
    [
      [0.0813, -0.0708],
      [0.0651, -0.0546],
    ],
    [
      [0.103, -0.0383],
      [0.0818, -0.0296],
    ],
    [
      [0.1106, 0],
      [0.0877, 0],
    ],
  ];
  const turn = BEND.concat(
    BEND.slice(0, 4)
      .toReversed()
      .map((b): [V2, V2] => [
        [b[0][0], -b[0][1]],
        [b[1][0], -b[1][1]],
      ]),
  ); // bottom → top
  const st: { o: V2; i: V2 }[] = [
    { o: [0, PIVOT - 0.1], i: [0, PIVOT - 0.0772] },
    { o: [-EX, PIVOT - 0.1], i: [-EX, PIVOT - 0.0772] },
  ];
  for (let t1 = 1; t1 < turn.length; t1++)
    st.push({ o: [-EX - turn[t1][0][0], PIVOT + turn[t1][0][1]], i: [-EX - turn[t1][1][0], PIVOT + turn[t1][1][1]] });
  st.push({ o: [EX, PIVOT + 0.1], i: [EX, PIVOT + 0.0772] });
  for (let t2 = turn.length - 2; t2 >= 0; t2--)
    st.push({ o: [EX + turn[t2][0][0], PIVOT + turn[t2][0][1]], i: [EX + turn[t2][1][0], PIVOT + turn[t2][1][1]] });
  st.push({ o: [0, PIVOT - 0.1], i: [0, PIVOT - 0.0772] });
  const u = [0];
  for (let a = 1; a < st.length; a++)
    u.push(u[a - 1] + Math.hypot(st[a].o[0] - st[a - 1].o[0], st[a].o[1] - st[a - 1].o[1]) * LINKS_PER_M);
  const Z = 0.0148;
  const surf = g.surface();
  // per station: the outer and inner broad faces (the link photo's top half) and the two flanks (its side view)
  const V = st.map((q, j) => ({
    ob: [surf.vertex(q.o[0], q.o[1], -Z, u[j], 0.9842), surf.vertex(q.o[0], q.o[1], Z, u[j], 0.5021)],
    ib: [surf.vertex(q.i[0], q.i[1], -Z, u[j], 0.5021), surf.vertex(q.i[0], q.i[1], Z, u[j], 0.9842)],
    fm: [surf.vertex(q.o[0], q.o[1], -Z, u[j], 0.0043), surf.vertex(q.i[0], q.i[1], -Z, u[j], 0.4941)],
    fp: [surf.vertex(q.o[0], q.o[1], Z, u[j], 0.4941), surf.vertex(q.i[0], q.i[1], Z, u[j], 0.0043)],
  }));
  // wind a quad a-b-c-d (along the loop from a to b, then across) to face the direction want, given its first two points
  const quad = (
    a: number,
    b: number,
    c: number,
    d: number,
    pa: V2,
    pb: V2,
    across: readonly number[],
    want: readonly number[],
  ): void => {
    const ex = pb[0] - pa[0];
    const ey = pb[1] - pa[1];
    const nx = ey * across[2];
    const ny = -ex * across[2];
    const nz = ex * across[1] - ey * across[0]; // (b − a) × (d − a)
    if (nx * want[0] + ny * want[1] + nz * want[2] >= 0) surf.quad(a, b, c, d);
    else surf.quad(a, d, c, b);
  };
  for (let j = 0; j + 1 < st.length; j++) {
    const q0 = st[j];
    const q1 = st[j + 1];
    const A = V[j];
    const B = V[j + 1];
    const mx = (q0.o[0] + q1.o[0]) / 2;
    const my = (q0.o[1] + q1.o[1]) / 2;
    const core = [mx - Math.max(-EX, Math.min(EX, mx)), my - PIVOT, 0]; // away from the loop's middle: out of the chain
    quad(A.ob[0], B.ob[0], B.ob[1], A.ob[1], q0.o, q1.o, [0, 0, 2 * Z], core);
    quad(A.ib[0], B.ib[0], B.ib[1], A.ib[1], q0.i, q1.i, [0, 0, 2 * Z], [-core[0], -core[1], 0]);
    quad(A.fm[0], B.fm[0], B.fm[1], A.fm[1], q0.o, q1.o, [q0.i[0] - q0.o[0], q0.i[1] - q0.o[1], 0], [0, 0, -1]);
    quad(A.fp[0], B.fp[0], B.fp[1], A.fp[1], q0.o, q1.o, [q0.i[0] - q0.o[0], q0.i[1] - q0.o[1], 0], [0, 0, 1]);
  }
  chain.push(surf.build());

  // ---- every lane: the same pieces, turned into place and stepped across the part, centred
  const lane = (list: readonly MeshDescription[]): MeshDescription => g.merge(...list);
  const one = { frame: lane(frame), support: lane(support), chain: lane(chain) };
  const all: Record<keyof typeof one, MeshDescription[]> = { frame: [], support: [], chain: [] };
  for (let li = 0; li < n; li++) {
    const x = (li - (n - 1) / 2) * pitch;
    for (const key of ["frame", "support", "chain"] as const) {
      all[key].push(g.transform(one[key], { scale: [1, 1, SA], rotate: YAW, translate: [x, LANE_Y, 0] }));
    }
  }
  return [
    { desc: lane(all.frame), material: "chainFrame" },
    { desc: lane(all.support), material: "support" },
    { desc: lane(all.chain), material: "chain", scroll: LINKS_PER_M },
  ];
}
