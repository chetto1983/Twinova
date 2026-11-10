// Copied from Open Industry Project, src/core/parts/partApi.ts at 1e7488c (MIT, Copyright (c) 2024-present Automation
// Standard LLC and Contributors; THIRD_PARTY_NOTICES.md): the mesh description and the geometry builder's surface,
// reduced to the operations Twinova's generators use.

/** A portable, three.js-shaped mesh: flat buffers a host turns into a BufferGeometry. */
export interface MeshDescription {
  /** flat [x,y,z, …] vertex positions (metres, part-local) */
  positions: number[];
  /** triangle vertex indices (CCW = front) */
  indices: number[];
  /** flat [u,v, …] per-vertex texture coordinates */
  uvs: number[];
}

type V3 = readonly [number, number, number];
type V2 = readonly [number, number];

/** The `g` a part's geometry() composes its mesh with. */
export interface GeometryBuilder {
  merge(...parts: readonly MeshDescription[]): MeshDescription;
  box(size: V3, center: V3): MeshDescription;
  surface(): SurfaceBuilder;
  bevelBox(
    size: V3,
    center: V3,
    bevel: number,
    square?: readonly ("x-" | "x+" | "y-" | "y+" | "z-" | "z+")[],
  ): MeshDescription;
  lathe(
    profile: ReadonlyArray<V2>,
    opts?: { segments?: number; axis?: "x" | "y" | "z"; center?: V3; startAngle?: number; smooth?: boolean },
  ): MeshDescription;
  extrude(
    outline: ReadonlyArray<V2>,
    from: number,
    to: number,
    opts?: {
      axis?: "x" | "y" | "z";
      holes?: ReadonlyArray<ReadonlyArray<V2>>;
      bevel?: number;
      square?: readonly number[];
    },
  ): MeshDescription;
  transform(
    desc: MeshDescription,
    opts: { scale?: number | V3; rotate?: readonly [number, number, number, number]; translate?: V3 },
  ): MeshDescription;
}

/** A hand-driven vertex/triangle accumulator: add vertices, wire them into triangles/quads (CCW = front), build. */
export interface SurfaceBuilder {
  vertex(x: number, y: number, z: number, u?: number, v?: number): number;
  triangle(a: number, b: number, c: number): void;
  quad(a: number, b: number, c: number, d: number): void;
  build(): MeshDescription;
}
