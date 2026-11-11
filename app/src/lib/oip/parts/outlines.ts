// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the outlines its sensor housings are extruded from
// (examples/parts/_shared/sensorOutlines.js `rrect` and `circle`).
type V2 = readonly [number, number];

/** A rounded-corner rectangle, for windows and panels. */
export function rrect(x0: number, y0: number, x1: number, y1: number, r: number): V2[] {
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
export function circle(cy: number, cz: number, r: number): V2[] {
  const pts: V2[] = [];
  for (let k = 0; k < 16; k++) {
    const a = (k * Math.PI) / 8;
    pts.push([cy + r * Math.cos(a), cz + r * Math.sin(a)]);
  }
  return pts;
}
