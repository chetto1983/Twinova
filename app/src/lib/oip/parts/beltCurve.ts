// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the curved belt conveyor's look,
// examples/parts/curved-conveyor/part.js `geometry`:
// - the belt bed: a ring strip (the top annulus and its inner and outer sides down the belt's depth) with a rounded
//   end-drum nose at each end, u in metres of the centreline so the fabric scrolls;
// - the side rails, an L-rail swept along the arc, and the guards, swept the same way (curve.ts);
// - the floor stands along the arc (curve.ts).
// Sensor-beam holes in the guards are left out: Twinova's beam does not cut them (spec
// 2026-11-10-curves-belt-options-design.md, decision 3). OIP's arc convention (curve.ts): the origin is the centre.
import type { GeometryBuilder } from "../geometry/types";
import { BELT_TILES_PER_M, frameBottom, guardSection, railSections } from "./belt";
import { arcPath, arcSegments, arcStands, type CurveParams } from "./curve";
import type { MeshPart } from "./part";

interface Ring {
  ix: number;
  iz: number;
  ox: number;
  oz: number;
  u: number;
}

/** The bed: the ring strip over the arc, and a drum nose over each end. */
function bed(g: GeometryBuilder, p: CurveParams, stub: number) {
  const { innerRadius: ir, width: w, sweep } = p;
  const rc = ir + w / 2;
  const bt = p.frameHeight;
  const n = arcSegments(sweep);
  const sfc = g.surface();
  const ri = ir + 0.01,
    ro = ir + w - 0.01;
  const rings: Ring[] = [];
  for (let i = 0; i <= n; i++) {
    const a = (sweep * i) / n;
    rings.push({ ix: ri * Math.cos(a), iz: -ri * Math.sin(a), ox: ro * Math.cos(a), oz: -ro * Math.sin(a), u: a * rc });
  }
  type Vertex = readonly [number, number, number, number, number];
  const quad = (a: Vertex, b: Vertex, c: Vertex, d: Vertex) =>
    sfc.quad(sfc.vertex(...a), sfc.vertex(...b), sfc.vertex(...c), sfc.vertex(...d));
  for (let i = 0; i < rings.length - 1; i++) {
    const r0 = rings[i]!,
      r1 = rings[i + 1]!;
    quad(
      [r0.ix, 0, r0.iz, r0.u, 0],
      [r0.ox, 0, r0.oz, r0.u, 1],
      [r1.ox, 0, r1.oz, r1.u, 1],
      [r1.ix, 0, r1.iz, r1.u, 0],
    );
    quad(
      [r0.ox, 0, r0.oz, r0.u, 1],
      [r0.ox, -bt, r0.oz, r0.u, 0],
      [r1.ox, -bt, r1.oz, r1.u, 0],
      [r1.ox, 0, r1.oz, r1.u, 1],
    );
    quad(
      [r0.ix, 0, r0.iz, r0.u, 0],
      [r1.ix, 0, r1.iz, r1.u, 0],
      [r1.ix, -bt, r1.iz, r1.u, 1],
      [r0.ix, -bt, r0.iz, r0.u, 1],
    );
  }
  // the end-drum nose: the cross-section wrapped from the top (y 0) down under (y −2·stub), bulging stub along the
  // end's outward tangent — the belt curling over a pulley
  const nose = (r: Ring, dx: number, dz: number, uSign: number) => {
    const DRUM = 12;
    let pi: [number, number, number] = [r.ix, 0, r.iz];
    let po: [number, number, number] = [r.ox, 0, r.oz];
    let pu = r.u;
    for (let k = 1; k <= DRUM; k++) {
      const th = (Math.PI * k) / DRUM,
        along = stub * Math.sin(th),
        y = -stub + stub * Math.cos(th);
      const ci: [number, number, number] = [r.ix + dx * along, y, r.iz + dz * along];
      const co: [number, number, number] = [r.ox + dx * along, y, r.oz + dz * along];
      const cu = r.u + uSign * stub * th;
      quad([...pi, pu, 0], [...po, pu, 1], [...co, cu, 1], [...ci, cu, 0]);
      pi = ci;
      po = co;
      pu = cu;
    }
  };
  nose(rings[0]!, 0, 1, -1); // back: the outward tangent at t = 0 is +Z
  nose(rings[rings.length - 1]!, -Math.sin(sweep), -Math.cos(sweep), 1); // front: the tangent at the sweep
  return sfc.build();
}

export function beltCurveGeometry(g: GeometryBuilder, p: CurveParams): MeshPart[] {
  const rc = p.innerRadius + p.width / 2;
  const hw = p.width / 2;
  const stub = p.frameHeight / 2; // the end-drum radius: the tangent lead-in and lead-out
  const fb = frameBottom(p.frameHeight);
  const path = arcPath(rc, p.sweep, stub);
  // the profile's width coordinate is radial along the arc: −1 the inner rail, +1 the outer
  const rail = (sign: number) =>
    g.merge(...railSections(fb, sign * hw, sign).map((s) => g.sweep(s, path, { caps: true })));
  const parts: MeshPart[] = [
    { desc: bed(g, p, stub), material: "belt", scroll: BELT_TILES_PER_M },
    { desc: g.merge(rail(-1), rail(1)), material: "frame" },
  ];
  const guards = [
    ...(p.inner ? [g.sweep(guardSection(-(hw + 0.01), -1), path, { caps: true })] : []),
    ...(p.outer ? [g.sweep(guardSection(hw + 0.01, 1), path, { caps: true })] : []),
  ];
  if (guards.length > 0) parts.push({ desc: g.merge(...guards), material: "guard" });
  parts.push(...arcStands(g, p, fb));
  return parts;
}
