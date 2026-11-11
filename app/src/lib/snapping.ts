// Snapping in the view (spec 2026-11-11-snapping-design.md, "The view"): for a part posed by a drag, the join it would
// make, if any: one of its ends onto a free end of another part, a spur's exit onto a straight's flank, or a device
// onto a conveyor's rail. The interfaces come from Rust, each in its own instance's frame; the view only names the
// pair, and Rust computes the pose. The thresholds are OIP's (src/core/types/base.ts at 0572868): an end or a flank
// engages within 0.3 m (SNAP_CONVEYOR_VISIBLE_THRESHOLD) and a quarter turn (a flank: 45°), a shown join is held until
// 0.6 m (SNAP_CONVEYOR_RELEASE), and the cheapest wins, its move plus 0.5 per radian of turn.
import * as THREE from "three";
import type { Edit, End, Endpoint, Frame, ProjectView, Quat, Side, Vec3 } from "./project";

export const REACH = 0.3;
export const RELEASE = 0.6;
const TURN_COST = 0.5;
const END_TURN = Math.PI / 2;
const FLANK_TURN = Math.PI / 4;

export type Candidate =
  | { kind: "join"; id: string; end: string; peer: Endpoint; point: Vec3; cost: number }
  | { kind: "merge"; spur: string; host: string; at: number; side: Side; point: Vec3; cost: number }
  | { kind: "mount"; device: string; host: string; at: number; point: Vec3; cost: number };

/** A pose as a three.js matrix. */
function matrix(position: readonly number[], rotation: readonly number[]): THREE.Matrix4 {
  const q = new THREE.Quaternion(rotation[0], rotation[1], rotation[2], rotation[3]);
  return new THREE.Matrix4().compose(new THREE.Vector3(...position), q, new THREE.Vector3(1, 1, 1));
}

const frameMatrix = (f: Frame): THREE.Matrix4 => matrix(f.position, f.rotation);

/** A frame's origin and its x, horizontal, in the world. */
function placed(base: THREE.Matrix4, f: Frame): { at: THREE.Vector3; out: THREE.Vector3 } {
  const m = base.clone().multiply(frameMatrix(f));
  const at = new THREE.Vector3().setFromMatrixPosition(m);
  const out = new THREE.Vector3(1, 0, 0).transformDirection(m).setY(0).normalize();
  return { at, out };
}

const poseOf = (project: ProjectView, id: string): THREE.Matrix4 | null => {
  const inst = project.instances.find((i) => i.id === id);
  return inst ? matrix(inst.position, inst.rotation) : null;
};

/** Where `id`'s end `end` stands in the world, its own pose `pose` when given. */
function worldEnd(project: ProjectView, id: string, end: End, pose?: THREE.Matrix4) {
  const base = end.on ? poseOf(project, end.on) : (pose ?? poseOf(project, id));
  return base ? placed(base, end.frame) : null;
}

/** Whether an end already feeds or is fed by something other than `except`. */
function taken(project: ProjectView, [id, port]: Endpoint, except: string): boolean {
  const at = (e: Endpoint) => e[0] === id && e[1] === port;
  return (
    project.flows.some((f) => (at(f.from) && f.to[0] !== except) || (at(f.to) && f.from[0] !== except)) ||
    project.merges.some((m) => m.spur === id && port === "out" && m.host !== except)
  );
}

const flat = (v: THREE.Vector3): THREE.Vector3 => new THREE.Vector3(v.x, 0, v.z);
const point = (v: THREE.Vector3): Vec3 => [v.x, v.y, v.z];
const cheaper = (a: Candidate | null, b: Candidate | null): Candidate | null => (!b || (a && a.cost <= b.cost) ? a : b);

/** The cheapest end-to-end join of `id` at `pose`: each of its ends onto each free, compatible end of another part. */
function joinOf(project: ProjectView, id: string, pose: THREE.Matrix4, reach: number): Candidate | null {
  let best: Candidate | null = null;
  for (const mine of project.interfaces[id]?.ends ?? []) {
    if (mine.on) continue;
    const m = worldEnd(project, id, mine, pose)!;
    for (const [peer, i] of Object.entries(project.interfaces)) {
      if (peer === id) continue;
      for (const theirs of i.ends) {
        if (mine.direction !== "either" && mine.direction === theirs.direction) continue;
        if (taken(project, [peer, theirs.name], id)) continue;
        const t = worldEnd(project, peer, theirs);
        if (!t) continue;
        const move = flat(m.at).distanceTo(flat(t.at));
        const turn = m.out.angleTo(t.out.clone().negate());
        if (move > reach || turn > END_TURN + 1e-9) continue;
        const cost = move + TURN_COST * turn;
        best = cheaper(best, { kind: "join", id, end: mine.name, peer: [peer, theirs.name], point: point(t.at), cost });
      }
    }
  }
  return best;
}

/** The cheapest merge of conveyor `id`'s exit at `pose` onto the flank of a straight that takes spurs. */
function mergeOf(project: ProjectView, id: string, pose: THREE.Matrix4, reach: number): Candidate | null {
  const exit = project.interfaces[id]?.ends.find((e) => e.name === "out" && !e.on);
  if (!exit || !project.interfaces[id]?.host) return null;
  const m = worldEnd(project, id, exit, pose)!;
  let best: Candidate | null = null;
  for (const [host, i] of Object.entries(project.interfaces)) {
    const h = i.host;
    const base = poseOf(project, host);
    if (host === id || !h?.spurs || h.rail.kind !== "line" || !base) continue;
    const rail = base.clone().multiply(frameMatrix(h.rail.start));
    const local = m.at.clone().applyMatrix4(rail.clone().invert());
    if (local.x < 0 || local.x > h.rail.length) continue;
    const sign = local.z >= 0 ? 1 : -1;
    const move = Math.abs(Math.abs(local.z) - h.width / 2);
    const outward = new THREE.Vector3(0, 0, sign).transformDirection(rail).setY(0).normalize();
    const turn = m.out.angleTo(outward.clone().negate());
    if (move > reach || turn > FLANK_TURN + 1e-9) continue;
    const at = Math.round(local.x * 1000) / 1000;
    const flank = new THREE.Vector3(at, 0, (sign * h.width) / 2).applyMatrix4(rail);
    const side: Side = sign > 0 ? "right" : "left";
    best = cheaper(best, {
      kind: "merge",
      spur: id,
      host,
      at,
      side,
      point: point(flank),
      cost: move + TURN_COST * turn,
    });
  }
  return best;
}

/** The nearest rail that takes device `id` (of `generator`), standing at `at` in the world: along a straight's run,
 *  or round a curve's arc in degrees. */
function mountOf(project: ProjectView, id: string, generator: string, at: THREE.Vector3): Candidate | null {
  let best: Candidate | null = null;
  for (const [host, i] of Object.entries(project.interfaces)) {
    const h = i.host;
    const base = poseOf(project, host);
    if (host === id || !h || !h.accepts.includes(generator) || !base) continue;
    const inv = base.clone().invert();
    const local = at.clone().applyMatrix4(inv);
    let along: number;
    let off: number;
    let on: THREE.Vector3;
    if (h.rail.kind === "line") {
      const r = local.clone().applyMatrix4(frameMatrix(h.rail.start).invert());
      along = Math.min(Math.max(r.x, 0), h.rail.length);
      off = Math.hypot(r.x - along, Math.max(0, Math.abs(r.z) - h.width / 2));
      on = new THREE.Vector3(along, 0, 0).applyMatrix4(frameMatrix(h.rail.start));
    } else {
      const c = new THREE.Vector3(...h.rail.centre);
      const s = new THREE.Vector3(...h.rail.start).sub(c);
      const p = local.clone().sub(c).setY(0);
      const turn = h.rail.right ? -1 : 1;
      let angle = Math.atan2(s.x * p.z - s.z * p.x, s.x * p.x + s.z * p.z) * -turn;
      angle = Math.min(Math.max(angle, 0), h.rail.sweep);
      const radius = flat(s).length();
      off = Math.max(0, Math.abs(p.length() - radius) - h.width / 2);
      along = THREE.MathUtils.radToDeg(angle);
      on = s
        .clone()
        .applyAxisAngle(new THREE.Vector3(0, 1, 0), turn * angle)
        .add(c);
    }
    if (off > REACH) continue;
    const rounded = Math.round(along * 1000) / 1000;
    const world = on.applyMatrix4(base);
    best = cheaper(best, { kind: "mount", device: id, host, at: rounded, point: point(world), cost: off });
  }
  return best;
}

/** Whether two candidates name the same join. */
export function same(a: Candidate | null, b: Candidate | null): boolean {
  if (!a || !b || a.kind !== b.kind) return false;
  if (a.kind === "join" && b.kind === "join") return a.end === b.end && a.peer.join() === b.peer.join();
  if (a.kind === "merge" && b.kind === "merge") return a.host === b.host && a.side === b.side;
  return a.kind === "mount" && b.kind === "mount" && a.host === b.host;
}

/** The join `id` would make posed at `position` and `rotation` (for a mounted device, where it stands): a held one
 *  until 0.6 m, else the cheapest within reach; none while placing freely. */
export function candidate(
  project: ProjectView,
  id: string,
  position: Vec3,
  rotation: Quat,
  held: Candidate | null = null,
): Candidate | null {
  const inst = project.instances.find((i) => i.id === id);
  const def = inst && project.definitions[inst.definition];
  if (!def) return null;
  if (def.ports.some((p) => p.kind === "mount_device")) {
    return mountOf(project, id, def.generator, new THREE.Vector3(...position));
  }
  const pose = matrix(position, rotation);
  const near = (reach: number) => cheaper(joinOf(project, id, pose, reach), mergeOf(project, id, pose, reach));
  const kept = held && near(RELEASE);
  return kept && same(kept, held) ? kept : near(REACH);
}

/** The edit a candidate makes, with the gap between the faces. */
export function editOf(c: Candidate, gap: number): Edit {
  switch (c.kind) {
    case "join":
      return { op: "join", id: c.id, end: c.end, peer: c.peer, gap };
    case "merge":
      return { op: "merge", spur: c.spur, host: c.host, at: c.at, side: c.side, gap };
    case "mount":
      return { op: "set_mount", device: c.device, host: c.host, at: c.at };
  }
}

/** The gap between the faces a flow joins, in metres, when both are conveyors' ends; null otherwise. */
export function flowGap(project: ProjectView, from: Endpoint, to: Endpoint): number | null {
  const end = ([id, name]: Endpoint) => project.interfaces[id]?.ends.find((e) => e.name === name);
  const [a, b] = [end(from), end(to)];
  if (!a || !b || !project.interfaces[from[0]]?.host || !project.interfaces[to[0]]?.host) return null;
  const [wa, wb] = [worldEnd(project, from[0], a), worldEnd(project, to[0], b)];
  return wa && wb ? Math.round(wb.at.clone().sub(wa.at).dot(wa.out) * 1e6) / 1e6 : null;
}
