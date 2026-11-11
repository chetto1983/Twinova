// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the curved roller conveyor's look,
// examples/parts/curved-roller/part.js `geometry`:
// - the side rails, an L-rail swept along each edge of the arc, and the guards, swept the same way (curve.ts);
// - the rollers at the centreline pitch, each pointing at the centre, with their axle stubs and no drive bands
//   (roller.ts `rollerBed`, OIP's _shared/rollerBed.js);
// - the floor stands along the arc (curve.ts).
// Twinova's curves hand over at the end of a 0.085 m stub, where the physics carries the load (OIP `ARC_END_EXT`), so
// the rails and guards lead in and out over it; OIP draws the roller curve's 2 mm snap stub
// (`CURVED_ROLLER_END_EXT`) instead (spec 2026-11-10-curves-belt-options-design.md, amended in Task 5). Sensor-beam
// holes are left out. OIP's arc convention (curve.ts): the origin is the centre.
import type { GeometryBuilder } from "../geometry/types";
import { frameBottom, guardSection, railSections } from "./belt";
import { arcPath, arcStands, type CurveParams } from "./curve";
import type { MeshPart } from "./part";
import { type Roller, rollerBed, STRAIGHT_BED } from "./roller";

/** The physics' stub, OIP `ARC_END_EXT` (physics/src/path.rs `STUB`). */
const STUB = 0.085;
/** The roller pitch along the centreline (roller-conveyor/part.js `PITCH`), so the density matches a straight bed. */
const PITCH = 0.075;

export function rollerCurveGeometry(g: GeometryBuilder, p: CurveParams): MeshPart[] {
  const rc = p.innerRadius + p.width / 2;
  const hw = p.width / 2;
  const fb = frameBottom(p.frameHeight);
  const edge = (sign: number) => arcPath(rc + sign * hw, p.sweep, STUB);
  const rail = (sign: number) =>
    g.merge(...railSections(fb, 0, sign).map((s) => g.sweep(s, edge(sign), { caps: true })));
  const guards = [
    ...(p.inner ? [g.sweep(guardSection(-0.01, -1), edge(-1), { caps: true })] : []),
    ...(p.outer ? [g.sweep(guardSection(0.01, 1), edge(1), { caps: true })] : []),
  ];
  const length = Math.max(p.width - 0.004, 0.01); // the body recessed just behind the inner and outer frame walls
  const count = Math.max(2, Math.ceil(p.sweep / (PITCH / Math.max(rc, 0.01))) + 1);
  const rollers: Roller[] = [];
  for (let i = 0; i < count; i++) {
    const a = (p.sweep * i) / (count - 1);
    rollers.push({ x: rc * Math.cos(a), z: -rc * Math.sin(a), yaw: a + Math.PI / 2, length });
  }
  const parts: MeshPart[] = [{ desc: g.merge(rail(-1), rail(1)), material: "frame" }];
  if (guards.length > 0) parts.push({ desc: g.merge(...guards), material: "guard" });
  parts.push(...arcStands(g, p, fb), ...rollerBed(g, rollers, STRAIGHT_BED));
  return parts;
}
