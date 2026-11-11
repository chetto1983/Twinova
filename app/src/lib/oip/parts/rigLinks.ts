// Copied from Open Industry Project, src/render/parts/sdk/rig.ts `linkMesh` at 1e7488c (MIT, Copyright (c)
// 2024-present Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: how a rig's
// box and cylinder links are drawn. A box of `size`, or a cylinder of `radius` and `height` about its local +Y, turned
// by `quat` and centred at `offset` in the part's frame. The pieces of one material on one link merge into one mesh.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import { cylinderTemplate } from "./legs";
import type { Material, MeshPart, Quat, V3 } from "./part";

export type LinkShape = { box: V3 } | { radius: number; height: number };

export interface Link {
  shape: LinkShape;
  offset: V3;
  quat?: Quat;
  material: Material;
  /** It rides the moving link (MeshPart `link`). */
  moving?: boolean;
  /** A large cylinder, drawn at 64 facets: at 16 each facet of a ~1 m deck shows as a flat panel on its rim (rig.ts
   *  picks it from the radius authored at the default width, `FINE_CYLINDER_RADIUS` 0.2). */
  fine?: boolean;
}

/** A turn of `a` about +X. */
export function xQuat(a: number): Quat {
  return [Math.sin(a / 2), 0, 0, Math.cos(a / 2)];
}

function linkMesh(g: GeometryBuilder, l: Link): MeshDescription {
  const unit = "box" in l.shape ? g.box([1, 1, 1], [0, 0, 0]) : cylinderTemplate(g, l.fine ? 64 : 16, { axis: "y" });
  const scale: V3 = "box" in l.shape ? l.shape.box : [l.shape.radius, l.shape.height, l.shape.radius];
  return g.transform(unit, { scale, rotate: l.quat, translate: l.offset });
}

export function drawLinks(g: GeometryBuilder, links: readonly Link[]): MeshPart[] {
  const groups = new Map<string, { material: Material; moving: boolean; descs: MeshDescription[] }>();
  for (const l of links) {
    const moving = l.moving === true;
    const key = `${l.material}/${moving}`;
    const group = groups.get(key) ?? { material: l.material, moving, descs: [] };
    group.descs.push(linkMesh(g, l));
    groups.set(key, group);
  }
  return [...groups.values()].map(({ material, moving, descs }) => {
    const part: MeshPart = { desc: g.merge(...descs), material };
    if (moving) part.link = "moving";
    return part;
  });
}
