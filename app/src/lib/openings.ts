// The stretches of a straight conveyor's guard between the openings the pushers beside it cut (spec
// 2026-11-11-stopper-pusher-turntable-design.md, decision 2; OIP's guardOpenings.ts opens 0.75 m about a pusher). The
// physics cuts its guard colliders by the same rule (physics path.rs `spans`).

/** A gap in one guard: in the right guard (+z) or the left, about `centre` along the conveyor. */
export interface Opening {
  right: boolean;
  centre: number;
  length: number;
}

/** The pieces of the guard on the `right` or left side that runs from `from` to `to`, open where `openings` say. */
export function guardSpans(from: number, to: number, right: boolean, openings: readonly Opening[]): [number, number][] {
  const cuts = openings
    .filter((o) => o.right === right)
    .map((o) => [o.centre - o.length / 2, o.centre + o.length / 2] as const)
    .toSorted((a, b) => a[0] - b[0]);
  const out: [number, number][] = [];
  let start = from;
  for (const [a, b] of cuts) {
    if (a > start) out.push([start, Math.min(a, to)]);
    start = Math.max(start, b);
  }
  if (start < to) out.push([start, to]);
  return out;
}
