import { describe, expect, it } from "vitest";
import type { Definition, Instance, Interfaces, ProjectView, Quat, Vec3 } from "./project";
import { RELEASE, REACH, candidate, editOf, flowGap, same, type Candidate } from "./snapping";

// Interfaces as Rust gives them (physics interfaces.rs, tested there against the fixtures): a belt's ends on its drum
// faces 85 mm past its run, at its 0.8 m surface, x out of the belt; its rail along its run.
const HALF_TURN: Quat = [0, 1, 0, 0];
const NO_TURN: Quat = [0, 0, 0, 1];
const yaw = (deg: number): Quat => {
  const h = (deg * Math.PI) / 360;
  return [0, Math.sin(h), 0, Math.cos(h)];
};

function belt(length: number, spurs = true): Interfaces {
  return {
    ends: [
      { name: "in", direction: "in", frame: { position: [-0.085, 0.8, 0], rotation: HALF_TURN }, on: null },
      { name: "out", direction: "out", frame: { position: [length + 0.085, 0.8, 0], rotation: NO_TURN }, on: null },
    ],
    host: {
      rail: { kind: "line", start: { position: [0, 0.8, 0], rotation: NO_TURN }, length },
      width: 0.6,
      accepts: ["twinova.sensor_diffuse/1", "twinova.chain_transfer/1"],
      spurs,
    },
  };
}

const def = (generator: string, device = false): Definition =>
  ({
    id: generator,
    generator,
    ports: device ? [{ name: "mount", kind: "mount_device" }] : [{ name: "in", kind: "flow_in" }],
  }) as unknown as Definition;

const inst = (id: string, definition: string, position: Vec3 = [0, 0, 0], rotation: Quat = NO_TURN): Instance =>
  ({ id, name: id, definition, position, rotation, parameters: {}, signals: {} }) as Instance;

/** belt_1 (3 m) at the origin and belt_2 (2 m) where the test puts it; a sensor; a curve. */
function view(extra: Partial<ProjectView> = {}): ProjectView {
  return {
    instances: [inst("belt_1", "belt"), inst("belt_2", "belt", [9, 0, 9]), inst("eye", "eye")],
    definitions: { belt: def("twinova.belt/1"), eye: def("twinova.sensor_diffuse/1", true) },
    interfaces: { belt_1: belt(3), belt_2: belt(2), eye: { ends: [], host: null } },
    flows: [],
    merges: [],
    mounts: [],
    ...extra,
  } as ProjectView;
}

/** Where belt_2's origin goes for its entry face to stand `dx` past belt_1's exit and `dz` aside, turned `deg`. */
function entryAt(dx: number, dz = 0, deg = 0): [Vec3, Quat] {
  const a = (deg * Math.PI) / 180;
  return [[3.085 + dx + 0.085 * Math.cos(a), 0, dz - 0.085 * Math.sin(a)], yaw(deg)];
}

describe("end to end", () => {
  it("joins the nearest free end within 0.3 m and a quarter turn", () => {
    const c = candidate(view(), "belt_2", ...entryAt(0.2, 0.1, 30));
    expect(c).toMatchObject({ kind: "join", id: "belt_2", end: "in", peer: ["belt_1", "out"] });
    expect(c!.point).toEqual([3.085, 0.8, 0]);
    expect(c!.cost).toBeCloseTo(Math.hypot(0.2, 0.1) + 0.5 * (Math.PI / 6), 6);
    expect(candidate(view(), "belt_2", ...entryAt(0.31))).toBeNull();
    expect(candidate(view(), "belt_2", ...entryAt(0, 0, 91))).toBeNull();
    expect(candidate(view(), "belt_2", ...entryAt(0, 0, 89))).not.toBeNull();
    expect(REACH).toBe(0.3);
  });

  it("holds a shown join until 0.6 m, then lets it go", () => {
    const held = candidate(view(), "belt_2", ...entryAt(0.1));
    expect(candidate(view(), "belt_2", ...entryAt(0.5), held)).toMatchObject({ kind: "join", end: "in" });
    expect(candidate(view(), "belt_2", ...entryAt(RELEASE + 0.01), held)).toBeNull();
    expect(candidate(view(), "belt_2", ...entryAt(0.5), null)).toBeNull();
  });

  it("offers no two ins or two outs, nor an end another part already joins", () => {
    // belt_2 turned round: its exit lands on belt_1's exit.
    const [p] = entryAt(0);
    const exitOnExit: Vec3 = [p[0] - 0.17 + 2.17, 0, 0];
    expect(candidate(view(), "belt_2", exitOnExit, HALF_TURN)).toBeNull();
    const busy = view({ flows: [{ from: ["belt_1", "out"], to: ["other", "in"] }] });
    expect(candidate(busy, "belt_2", ...entryAt(0))).toBeNull();
    const mine = view({ flows: [{ from: ["belt_1", "out"], to: ["belt_2", "in"] }] });
    expect(candidate(mine, "belt_2", ...entryAt(0))).not.toBeNull();
  });

  it("joins an end given in its host's frame, as a transfer's side exit", () => {
    const v = view();
    v.interfaces.belt_1.ends.push({
      name: "out",
      direction: "out",
      frame: { position: [1, 0.8, -0.3], rotation: yaw(90) },
      on: "belt_1",
    });
    v.interfaces.transfer = v.interfaces.belt_1;
    v.interfaces.belt_1 = belt(3);
    v.instances.push(inst("transfer", "eye", [50, 0, 50]));
    const c = candidate(v, "belt_2", [1, 0, -0.3 - 0.1 - 0.085], yaw(90));
    expect(c).toMatchObject({ kind: "join", peer: ["transfer", "out"] });
  });
});

/** belt_2's origin, its exit face `gap` off belt_1's right flank 1.2 m along, running along −z. */
const exitNear = (gap: number): Vec3 => [1.2, 0, 0.3 + gap + 2.085];

/** belt_2's origin, turned `deg` about its exit face, which stands 0.1 m off belt_1's right flank 1.2 m along. */
function turnedAboutExit(deg: number): Vec3 {
  const a = (deg * Math.PI) / 180;
  return [1.2 - 2.085 * Math.cos(a), 0, 0.4 + 2.085 * Math.sin(a)];
}

describe("merging and mounting", () => {
  it("merges a spur's exit onto a straight's flank, square within 45°", () => {
    // belt_2 runs along −z towards belt_1's right flank (+z), its exit face `gap` off it, 1.2 m along.
    const c = candidate(view(), "belt_2", exitNear(0.1), yaw(90));
    expect(c).toMatchObject({ kind: "merge", spur: "belt_2", host: "belt_1", at: 1.2, side: "right" });
    expect(c!.point.map((v) => +v.toFixed(6))).toEqual([1.2, 0.8, 0.3]);
    expect(candidate(view(), "belt_2", exitNear(0.31), yaw(90))).toBeNull();
    expect(candidate(view(), "belt_2", [1.2, 0, 0.3 + 0.1 + 2.085], yaw(140))).toBeNull();
    const noSpurs = view({ interfaces: { ...view().interfaces, belt_1: belt(3, false) } });
    expect(candidate(noSpurs, "belt_2", exitNear(0.1), yaw(90))).toBeNull();
    const left = candidate(view(), "belt_2", [1.5, 0, -0.3 - 0.05 - 2.085], yaw(-90));
    expect(left).toMatchObject({ kind: "merge", at: 1.5, side: "left" });
  });

  it("mounts a device on the nearest rail that takes it, at its distance along", () => {
    expect(candidate(view(), "eye", [1.25, 0, -0.4], NO_TURN)).toMatchObject({
      kind: "mount",
      device: "eye",
      host: "belt_1",
      at: 1.25,
    });
    expect(candidate(view(), "eye", [1.25, 0, -0.61], NO_TURN)).toBeNull();
    expect(candidate(view(), "eye", [-0.2, 0, 0], NO_TURN)).toMatchObject({ at: 0 });
    const v = view();
    v.interfaces.belt_1.host!.accepts = [];
    v.interfaces.belt_2.host!.accepts = [];
    expect(candidate(v, "eye", [1.25, 0, 0], NO_TURN)).toBeNull();
  });

  it("mounts a device round a curve's arc, in degrees, either way", () => {
    for (const right of [false, true]) {
      const turn = right ? -1 : 1;
      const v = view();
      v.interfaces.belt_1.host!.rail = {
        kind: "arc",
        centre: [0, 0.8, -turn],
        start: [0, 0.8, 0],
        sweep: Math.PI / 2,
        right,
      };
      // 30° round the arc of radius 1, to its side.
      const a = Math.PI / 6;
      const at: Vec3 = [Math.sin(a), 0, -turn + turn * Math.cos(a)];
      expect(candidate(v, "eye", at, NO_TURN)).toMatchObject({ kind: "mount", host: "belt_1", at: 30 });
    }
  });
});

describe("the cases at the edges", () => {
  it("treats an exit a spur already merges from as taken", () => {
    const merged = view({ merges: [{ spur: "belt_1", host: "elsewhere", at: 1, side: "left" }] });
    expect(candidate(merged, "belt_2", ...entryAt(0))).toBeNull();
    const back = view({ merges: [{ spur: "belt_1", host: "belt_2", at: 1, side: "left" }] });
    expect(candidate(back, "belt_2", ...entryAt(0))).not.toBeNull();
  });

  it("keeps a held join only while it is the one found, and merges only a conveyor", () => {
    const held = candidate(view(), "belt_2", ...entryAt(0.1));
    const other: Candidate = { ...held!, kind: "join", peer: ["belt_9", "out"] } as Candidate;
    // A different join held does not keep the one found beyond reach.
    expect(candidate(view(), "belt_2", ...entryAt(0.5), other)).toBeNull();
    const v = view();
    v.interfaces.belt_2 = { ...belt(2), host: null };
    expect(candidate(v, "belt_2", [1.2, 0, 0.3 + 0.1 + 2.085], yaw(90))).toBeNull();
    // A merge's cost is its move and its turn.
    const c = candidate(view(), "belt_2", [1.2, 0, 0.3 + 0.1 + 2.085], yaw(80));
    expect(c?.cost).toBeCloseTo(0.1 + 0.5 * (Math.PI / 18), 1);
    // Turned about its exit face 0.1 m off the flank: within 45° of square it merges, beyond it does not.
    expect(candidate(view(), "belt_2", turnedAboutExit(130), yaw(130))).toMatchObject({ kind: "merge" });
    expect(candidate(view(), "belt_2", turnedAboutExit(140), yaw(140))).toBeNull();
    expect(candidate(view(), "belt_2", [-0.2, 0, 0.4 + 2.085], yaw(90))).toBeNull();
    expect(candidate(view(), "belt_2", [3.2, 0, 0.4 + 2.085], yaw(90))).toBeNull();
  });

  it("mounts a device with ports of its own, as a chain transfer, and never on itself", () => {
    const v = view();
    v.definitions.transfer = {
      ...def("twinova.chain_transfer/1", true),
      ports: [
        { name: "mount", kind: "mount_device" },
        { name: "out", kind: "flow_out" },
      ],
    } as Definition;
    v.instances.push(inst("t", "transfer"));
    v.interfaces.t = { ends: [], host: null };
    expect(candidate(v, "t", [1, 0, 0], NO_TURN)).toMatchObject({ kind: "mount", host: "belt_1", at: 1 });
    expect(candidate(view(), "ghost", [1, 0, 0], NO_TURN)).toBeNull();
  });

  it("finds the angle round an arc that starts off the axes, and the point on it", () => {
    const v = view();
    // A left arc of radius 1 about (1, 0.8, 1), starting at 45° from +z.
    const s = Math.SQRT1_2;
    v.interfaces.belt_1.host!.rail = {
      kind: "arc",
      centre: [1, 0.8, 1],
      start: [1 + s, 0.8, 1 + s],
      sweep: Math.PI,
      right: false,
    };
    const a = Math.PI / 4 + Math.PI / 3;
    const c = candidate(v, "eye", [1 + Math.sin(a), 0, 1 + Math.cos(a)], NO_TURN);
    expect(c).toMatchObject({ kind: "mount", at: 60 });
    expect(c!.point.map((x) => +x.toFixed(6))).toEqual([
      +(1 + Math.sin(a)).toFixed(6),
      0.8,
      +(1 + Math.cos(a)).toFixed(6),
    ]);
    expect(candidate(v, "eye", [1 + 1.7 * Math.sin(a), 0, 1 + 1.7 * Math.cos(a)], NO_TURN)).toBeNull();
    expect(candidate(v, "eye", [1 + 1.25 * Math.sin(a), 0, 1 + 1.25 * Math.cos(a)], NO_TURN)).not.toBeNull();
  });
});

describe("edits and gaps", () => {
  it("makes each candidate's edit with the gap", () => {
    const join: Candidate = { kind: "join", id: "b", end: "in", peer: ["a", "out"], point: [0, 0, 0], cost: 0 };
    const merge: Candidate = { kind: "merge", spur: "b", host: "a", at: 1, side: "left", point: [0, 0, 0], cost: 0 };
    const mount: Candidate = { kind: "mount", device: "e", host: "a", at: 2, point: [0, 0, 0], cost: 0 };
    expect(editOf(join, 0.02)).toEqual({ op: "join", id: "b", end: "in", peer: ["a", "out"], gap: 0.02 });
    expect(editOf(merge, 0.04)).toEqual({ op: "merge", spur: "b", host: "a", at: 1, side: "left", gap: 0.04 });
    expect(editOf(mount, 0.04)).toEqual({ op: "set_mount", device: "e", host: "a", at: 2 });
    expect(same(join, { ...join, cost: 9 })).toBe(true);
    expect(same(join, { ...join, peer: ["a", "in"] })).toBe(false);
    expect(same(merge, { ...merge, side: "right" })).toBe(false);
    expect(same(mount, { ...mount, at: 5 })).toBe(true);
    expect(same(join, merge) || same(null, join) || same(join, null)).toBe(false);
    expect(same(join, { ...join, end: "out" })).toBe(false);
    expect(same(merge, { ...merge, at: 9 })).toBe(true);
    expect(same(merge, { ...merge, host: "z" })).toBe(false);
    expect(same(mount, { ...mount, host: "z" })).toBe(false);
    expect(same(mount, join) || same(merge, mount) || same(join, mount)).toBe(false);
  });

  it("measures a flow's gap face to face, for conveyors only", () => {
    const v = view({ instances: [inst("belt_1", "belt"), inst("belt_2", "belt", entryAt(0.02)[0])] });
    expect(flowGap(v, ["belt_1", "out"], ["belt_2", "in"])).toBeCloseTo(0.02, 6);
    expect(flowGap(v, ["belt_1", "out"], ["eye", "in"])).toBeNull();
    expect(flowGap(v, ["belt_1", "side"], ["belt_2", "in"])).toBeNull();
    expect(flowGap(v, ["belt_1", "out"], ["ghost", "in"])).toBeNull();
    v.interfaces.belt_2 = { ...belt(2), host: null };
    expect(flowGap(v, ["belt_1", "out"], ["belt_2", "in"])).toBeNull();
    v.interfaces.belt_2 = belt(2);
    v.interfaces.belt_1 = { ...belt(3), host: null };
    expect(flowGap(v, ["belt_1", "out"], ["belt_2", "in"])).toBeNull();
  });
});
