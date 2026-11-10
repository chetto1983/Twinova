// Copied from Open Industry Project, src/core/geometry/meshDescription.ts at 1e7488c (MIT, Copyright (c) 2024-present
// Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md): the vertex budget every geometry op is capped by.

/** Hard ceiling on one description's vertices, the geometry analogue of the determinism guard and the
 *  per-tick spawn caps: a runaway geometry function can't allocate without bound. Tune as real parts arrive. */
export const MAX_DESCRIPTION_VERTICES = 200_000;

/** Thrown when a geometry op would exceed {@link MAX_DESCRIPTION_VERTICES}. */
export class GeometryBudgetError extends Error {
  constructor(requested: number) {
    super(`geometry exceeds the vertex budget: ${requested} > ${MAX_DESCRIPTION_VERTICES}`);
    this.name = "GeometryBudgetError";
  }
}

/** A 3-D point on a sweep path (metres, part-local). */
export type Vec3 = readonly [number, number, number];
