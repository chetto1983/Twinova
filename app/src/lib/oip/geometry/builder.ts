// Copied from Open Industry Project, src/core/geometry/geometryBuilder.ts at 1e7488c (MIT, Copyright (c) 2024-present
// Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md). Twinova keeps the operations its generators use:
// loft, cylinder and subtract are left out; fillet's polyline rounding lives in fillet.ts.
//
// The GeometryBuilder implementation - the `g` handed to a part's geometry() (docs/procedural-geometry.md, Tier 2): a
// curated set of mid-level constructive ops a part composes into a portable MeshDescription.
//
// Pure + deterministic (no Date/Math.random/host handles), so the same builder runs identically on both
// shipping surfaces and the description is reproducible. Every op that allocates is vertex-budget-capped via
// meshDescription's GeometryBudgetError, the geometry analogue of the determinism guard + spawn caps.

import type { GeometryBuilder, MeshDescription, SurfaceBuilder } from "./types";
import { GeometryBudgetError, MAX_DESCRIPTION_VERTICES } from "./mesh";
import type { Vec3 } from "./mesh";
import { bevelBoxDescription, extrudeDescription, latheDescription, transformDescription } from "./shapes";
import { sweepProfileAlongPath } from "./sweep";
import { filletPolyline } from "./fillet";

/** default arc sub-segments per filleted corner */
const DEFAULT_ARC_SEG = 6;

/** Concatenate descriptions into one mesh, re-basing each part's indices onto the running vertex offset.
 *  Budget-checked against the combined vertex count (a merge can blow the cap even when each piece is fine). */
function mergeDescriptions(parts: readonly MeshDescription[]): MeshDescription {
  const positions: number[] = [];
  const indices: number[] = [];
  const uvs: number[] = [];
  let vertexOffset = 0;
  for (const part of parts) {
    const vCount = (part.positions.length / 3) | 0;
    if (vertexOffset + vCount > MAX_DESCRIPTION_VERTICES) throw new GeometryBudgetError(vertexOffset + vCount);
    for (let i = 0; i < part.positions.length; i++) positions.push(part.positions[i]!);
    for (let i = 0; i < part.uvs.length; i++) uvs.push(part.uvs[i]!);
    for (let i = 0; i < part.indices.length; i++) indices.push(part.indices[i]! + vertexOffset);
    vertexOffset += vCount;
  }
  return { positions, indices, uvs };
}

/** An axis-aligned box (full `size`, centred at `center`) as a 24-vertex description - 4 verts per face so each
 *  face shades flat (a leg/foot shows hard edges instead of smooth shading). CCW-outward winding → outward normals. */
function boxDescription(
  size: readonly [number, number, number],
  center: readonly [number, number, number],
): MeshDescription {
  const hx = size[0] / 2;
  const hy = size[1] / 2;
  const hz = size[2] / 2;
  const x0 = center[0] - hx;
  const x1 = center[0] + hx;
  const y0 = center[1] - hy;
  const y1 = center[1] + hy;
  const z0 = center[2] - hz;
  const z1 = center[2] + hz;
  const positions: number[] = [];
  const indices: number[] = [];
  const uvs: number[] = [];
  const face = (a: Vec3, b: Vec3, c: Vec3, d: Vec3): void => {
    const base = positions.length / 3;
    for (const p of [a, b, c, d]) positions.push(p[0], p[1], p[2]);
    uvs.push(0, 0, 1, 0, 1, 1, 0, 1);
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  };
  face([x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]); // +X
  face([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]); // -X
  face([x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]); // +Y (top)
  face([x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]); // -Y (bottom)
  face([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]); // +Z
  face([x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]); // -Z
  return { positions, indices, uvs };
}

/** Build a fresh GeometryBuilder. Stateless - a new one per geometry() call keeps the op surface pure. */
export function createGeometryBuilder(): GeometryBuilder {
  return {
    sweep(profile, path, opts): MeshDescription {
      // sweepProfileAlongPath already validates the profile/station counts and enforces the vertex budget.
      return sweepProfileAlongPath(profile, path, opts);
    },
    fillet(points, radius, arcSegments = DEFAULT_ARC_SEG): ReadonlyArray<Vec3> {
      return filletPolyline(points, radius, arcSegments);
    },
    arc(center, radius, start, end, segments = 8): ReadonlyArray<Vec3> {
      const seg = Math.max(1, Math.floor(segments));
      const out: Vec3[] = [];
      for (let i = 0; i <= seg; i++) {
        const a = start + (end - start) * (i / seg);
        out.push([center[0] + radius * Math.cos(a), center[1] + radius * Math.sin(a), center[2]]);
      }
      return out;
    },
    box(size, center): MeshDescription {
      return boxDescription(size, center);
    },
    surface(): SurfaceBuilder {
      // a fresh accumulator per call; the part pushes verts/tris then build()s. Budget-capped on each vertex
      // so a runaway loop can't allocate without bound (the low-level analogue of the sweep budget check).
      const positions: number[] = [];
      const uvs: number[] = [];
      const indices: number[] = [];
      return {
        vertex(x, y, z, u = 0, v = 0): number {
          const idx = positions.length / 3;
          if (idx >= MAX_DESCRIPTION_VERTICES) throw new GeometryBudgetError(idx + 1);
          positions.push(x, y, z);
          uvs.push(u, v);
          return idx;
        },
        triangle(a, b, c): void {
          indices.push(a, b, c);
        },
        quad(a, b, c, d): void {
          indices.push(a, b, c, a, c, d);
        },
        build(): MeshDescription {
          return { positions, indices, uvs };
        },
      };
    },
    merge(...parts: readonly MeshDescription[]): MeshDescription {
      return mergeDescriptions(parts);
    },
    // the solid-shape ops (geometryShapes.ts): engineered hardware as the numbers a drawing gives
    bevelBox(size, center, bevel, square): MeshDescription {
      return bevelBoxDescription(size, center, bevel, square);
    },
    lathe(profile, opts): MeshDescription {
      return latheDescription(profile, opts);
    },
    extrude(outline, from, to, opts): MeshDescription {
      return extrudeDescription(outline, from, to, opts);
    },
    transform(desc, opts): MeshDescription {
      return transformDescription(desc, opts);
    },
  };
}
