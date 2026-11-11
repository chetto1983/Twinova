// Which instances OIP's generators draw (lib/oip/parts), and where, as the physics places them (physics/src/layout.rs,
// path.rs, world.rs):
// - a belt from its tail and a roller conveyor centred on its length, the surface at their `height`;
// - a curve from its entry stub's tip on the centreline. OIP draws a curve about its centre, entering at +X heading
//   −Z and turning left: Twinova's left curve is that drawing turned −90° about +Y; its right curve is the same drawing
//   run backwards (turned π/2 − sweep), never mirrored, so its faces keep their winding (and its fabric scrolls back);
// - a chain transfer and a sensor from their mount on the host, along a straight host or at an angle on a curve; a
//   sensor on a turntable stands still beside its deck, across its lane at its start angle; a through-beam's receiver
//   faces its emitter from the far edge; a barcode scanner's arch stands over its straight host;
// - a stopper in the gap between its roller host's rollers nearest its mount, its blade raised by its lift; a pusher
//   beside its host, its paddle face just outside the frame, pushing to its out side by its reach; a turntable from
//   its deck's centre, the deck turned by its angle (physics world_devices.rs).
// The other families keep their stand-in shapes (project.ts `shapeOf`).
import { createGeometryBuilder } from "./oip/geometry/builder";
import { beltGeometry } from "./oip/parts/belt";
import { beltCurveGeometry } from "./oip/parts/beltCurve";
import { chainTransferGeometry } from "./oip/parts/chainTransfer";
import type { CurveParams } from "./oip/parts/curve";
import type { MeshPart } from "./oip/parts/part";
import { barcodeScannerGeometry } from "./oip/parts/barcodeScanner";
import { LASER, laserGeometry } from "./oip/parts/laser";
import { PHOTOEYE, photoeyeGeometry } from "./oip/parts/photoeye";
import { pusherGeometry } from "./oip/parts/pusher";
import { rollerGap, rollerGeometry } from "./oip/parts/roller";
import { rollerCurveGeometry } from "./oip/parts/rollerCurve";
import { stopperGeometry } from "./oip/parts/stopper";
import { turntableGeometry } from "./oip/parts/turntable";
import type { Opening } from "./openings";
import { flag, real, type Instance, type ProjectView, type Quat, type Vec3 } from "./project";

/** The chain deck under its host's rollers, and its stroke (physics world.rs `TRANSFER_BELOW`, `TRANSFER_STROKE`). */
const TRANSFER_BELOW = 0.03;
export const TRANSFER_STROKE = 0.095;
/** A sensor's beam above its host's surface, and its lens out from the bed edge (world_sensors.rs `BEAM_HEIGHT`,
 *  `LENS_OUT`). */
const BEAM_HEIGHT = 0.05;
const LENS_OUT = 0.02;
/** The barcode scanner's arch: how far its columns stand clear of the host's edges, and its head's window over the
 *  surface (OIP's clearance). */
const ARCH_CLEARANCE = 0.2;
const ARCH_HEAD = 0.6;
/** The stopper's blade: OIP's clearance to its bed, and how far under the surface its top rests (physics
 *  world_devices.rs `BLADE_CLEARANCE`, `BLADE_LOWERED`). */
const BLADE_CLEARANCE = 0.03;
const BLADE_LOWERED = 0.05;
/** The pusher's paddle face outside its host's frame wall, and OIP's frame for it: the face at rest at −1.103 along
 *  its push (−Z), its origin 0.2 under the running surface (physics world_devices.rs `PADDLE_CLEARANCE`;
 *  diverter/behavior.ts). */
const PADDLE_OUTSIDE = 0.01 + 0.0105;
const PADDLE_FACE = 1.103;
const PADDLE_BELOW = 0.2;
/** A turntable's lane runs from rim to rim, 0.75 widths each side of its centre (physics layout.rs `TURNTABLE_LANE`). */
const TURNTABLE_HALF_LANE = 0.75;
const PUSHER_OPENING = 0.75;
/** A curve's straight stub before and after its arc (physics path.rs `STUB`, OIP `ARC_END_EXT`). */
const STUB = 0.085;
const CURVES: Record<string, (g: ReturnType<typeof createGeometryBuilder>, p: CurveParams) => MeshPart[]> = {
  "twinova.belt_curve/1": beltCurveGeometry,
  "twinova.roller_curve/1": rollerCurveGeometry,
};

export interface Drawn {
  parts: MeshPart[];
  /** The pose the parts hang from: the instance's own, or its host's for a mounted device. */
  position: Vec3;
  rotation: Quat;
  /** Where the parts' origin sits in that frame. */
  offset: Vec3;
  /** How the parts are turned in that frame, about their origin. */
  turn?: Quat;
  /** How the parts tagged `moving` follow the device's position in the snapshot. */
  motion?: Motion;
}

/** A moving link's motion: it slides by the position times `along` (a lift or a reach, 0 to 1), or turns about +Y by
 *  the position in degrees (a turntable's deck). */
export type Motion = { slide: Vec3 } | { turn: true };

const yaw = (angle: number): Quat => [0, Math.sin(angle / 2), 0, Math.cos(angle / 2)];

function host(project: ProjectView, device: string): { instance: Instance; at: number } | null {
  const mount = project.mounts.find((m) => m.device === device);
  const instance = mount && project.instances.find((i) => i.id === mount.host);
  return mount && instance ? { instance, at: mount.at } : null;
}

/** The openings the pushers mounted on a conveyor cut in its guards, 0.75 m about each on its side (physics
 *  layout_devices.rs `PUSHER_OPENING`). */
function openings(project: ProjectView, on: Instance): Opening[] {
  return project.mounts.flatMap((m) => {
    const device = m.host === on.id ? project.instances.find((i) => i.id === m.device) : undefined;
    const def = device ? project.definitions[device.definition] : undefined;
    if (!device || !def || def.generator !== "twinova.pusher/1") return [];
    return [{ right: !flag(device, def, "out_right", false), centre: m.at, length: PUSHER_OPENING }];
  });
}

/** A conveyor's left (−z) and right guards. */
function guards(project: ProjectView, instance: Instance): [boolean, boolean] {
  const def = project.definitions[instance.definition]!;
  return [flag(instance, def, "left_guard", true), flag(instance, def, "right_guard", true)];
}

/** A curve's geometry: its centreline's radius, its sweep in radians, and whether it turns right. */
function arcOf(project: ProjectView, instance: Instance) {
  const def = project.definitions[instance.definition]!;
  const rc = real(instance, def, "inner_radius", 0.6) + real(instance, def, "width", 0.6) / 2;
  return {
    rc,
    sweep: (real(instance, def, "sweep", 90) * Math.PI) / 180,
    right: flag(instance, def, "turn_right", false),
  };
}

/** Where a device mounted `at` along its host hangs, `up` above its floor and `across` to the right of its centreline,
 *  and how it is turned: along a straight host, or at an angle (degrees into the arc) on a curve, facing along the arc
 *  (physics path.rs `Arc::frame`). */
function mountFrame(
  project: ProjectView,
  on: Instance,
  at: number,
  up: number,
  across: number,
): { offset: Vec3; turn?: Quat } {
  const generator = project.definitions[on.definition]?.generator ?? "";
  if (generator === "twinova.turntable/1") {
    const back = TURNTABLE_HALF_LANE * real(on, project.definitions[on.definition], "width", 0.6);
    return { offset: [at - back, up, across] };
  }
  if (!CURVES[generator]) return { offset: [at, up, across] };
  const { rc, right } = arcOf(project, on);
  const t = right ? -1 : 1;
  const a = (at * Math.PI) / 180;
  const offset: Vec3 = [
    STUB + rc * Math.sin(a) + across * Math.sin(t * a),
    up,
    -t * rc * (1 - Math.cos(a)) + across * Math.cos(t * a),
  ];
  return { offset, turn: yaw(t * a) };
}

/** A sensor's housing on its host's rail, its lens 20 mm out from the bed edge and its beam `BEAM_HEIGHT` above the
 *  surface, facing across (+z): OIP seats the eye so (spec 2026-11-10, "The view"). */
function beamSensor(
  project: ProjectView,
  instance: Instance,
  parts: MeshPart[],
  housing: { emitY: number; front: number },
): Drawn | null {
  const on = host(project, instance.id);
  if (!on) return null;
  const def = project.definitions[on.instance.definition];
  const surface = real(on.instance, def, "height", 0.8);
  const edge = real(on.instance, def, "width", 0.6) / 2;
  const up = surface + BEAM_HEIGHT - housing.emitY;
  const placed = mountFrame(project, on.instance, on.at, up, -edge - LENS_OUT - housing.front);
  return { parts, position: on.instance.position, rotation: on.instance.rotation, ...placed };
}

export function drawnOf(project: ProjectView, instance: Instance): Drawn | null {
  const def = project.definitions[instance.definition];
  const value = (inst: Instance, name: string, fallback: number) =>
    real(inst, project.definitions[inst.definition], name, fallback);
  const g = createGeometryBuilder();
  const curve = def && CURVES[def.generator];
  if (curve) {
    const { rc, sweep, right } = arcOf(project, instance);
    const [left, rightGuard] = guards(project, instance);
    const height = value(instance, "height", 0.8);
    const parts = curve(g, {
      innerRadius: value(instance, "inner_radius", 0.6),
      width: value(instance, "width", 0.6),
      sweep,
      frameHeight: value(instance, "frame_height", 0.17),
      standHeight: height,
      inner: right ? rightGuard : left,
      outer: right ? left : rightGuard,
    });
    return {
      parts: right ? parts.map((p) => (p.scroll ? { ...p, scroll: -p.scroll } : p)) : parts,
      position: instance.position,
      rotation: instance.rotation,
      offset: [STUB, height, right ? rc : -rc],
      turn: yaw(right ? Math.PI / 2 - sweep : -Math.PI / 2),
    };
  }
  switch (def?.generator) {
    case "twinova.belt/1": {
      const height = value(instance, "height", 0.8);
      const parts = beltGeometry(g, {
        length: value(instance, "length", 3),
        width: value(instance, "width", 0.6),
        frameHeight: value(instance, "frame_height", 0.17),
        standHeight: height,
        guards: guards(project, instance),
        openings: openings(project, instance),
      });
      return { parts, position: instance.position, rotation: instance.rotation, offset: [0, height, 0] };
    }
    case "twinova.roller/1": {
      const length = value(instance, "length", 2);
      const height = value(instance, "height", 0.8);
      const parts = rollerGeometry(g, {
        length,
        width: value(instance, "width", 0.6),
        frameHeight: value(instance, "frame_height", 0.17),
        standHeight: height,
        guards: guards(project, instance),
        openings: openings(project, instance),
      });
      return { parts, position: instance.position, rotation: instance.rotation, offset: [length / 2, height, 0] };
    }
    case "twinova.chain_transfer/1": {
      const on = host(project, instance.id);
      if (!on) return null;
      const parts = chainTransferGeometry(g, {
        chains: value(instance, "chains", 2),
        distance: value(instance, "distance", 0.33),
        laneLength: value(instance, "lane_length", 0.6),
      });
      const top = value(on.instance, "height", 0.8) - TRANSFER_BELOW;
      return {
        parts: parts.map((part) => Object.assign(part, { link: "moving" as const })),
        position: on.instance.position,
        rotation: on.instance.rotation,
        offset: [on.at, top, 0],
        motion: { slide: [0, TRANSFER_STROKE, 0] },
      };
    }
    case "twinova.stopper/1": {
      const on = host(project, instance.id);
      if (!on) return null;
      const raiseHeight = value(instance, "raise_height", 0.04);
      const parts = stopperGeometry(g, { width: value(on.instance, "width", 0.6) - BLADE_CLEARANCE, raiseHeight });
      const gap = rollerGap(value(on.instance, "length", 2), on.at);
      return {
        parts,
        position: on.instance.position,
        rotation: on.instance.rotation,
        offset: [gap, value(on.instance, "height", 0.8), 0],
        motion: { slide: [0, raiseHeight + BLADE_LOWERED, 0] },
      };
    }
    case "twinova.pusher/1": {
      const on = host(project, instance.id);
      if (!on) return null;
      const stroke = value(instance, "stroke", 0.75);
      const height = value(on.instance, "height", 0.8);
      const parts = pusherGeometry(g, { stroke, standHeight: height - PADDLE_BELOW });
      // OIP pushes along its −Z: towards the host's −z from its right side, turned half round from its left.
      const side = flag(instance, def!, "out_right", false) ? -1 : 1;
      const z = side * (value(on.instance, "width", 0.6) / 2 + PADDLE_OUTSIDE + PADDLE_FACE);
      return {
        parts,
        position: on.instance.position,
        rotation: on.instance.rotation,
        offset: [on.at, height - PADDLE_BELOW, z],
        turn: side > 0 ? undefined : yaw(Math.PI),
        motion: { slide: [0, 0, -stroke] },
      };
    }
    case "twinova.turntable/1": {
      const height = value(instance, "height", 0.8);
      const parts = turntableGeometry(g, { width: value(instance, "width", 0.6), standHeight: height });
      return {
        parts,
        position: instance.position,
        rotation: instance.rotation,
        offset: [0, height, 0],
        motion: { turn: true },
      };
    }
    case "twinova.sensor_diffuse/1":
      return beamSensor(project, instance, photoeyeGeometry(g), PHOTOEYE);
    case "twinova.sensor_color/1":
      return beamSensor(project, instance, photoeyeGeometry(g, { body: "colorBody", lens: "colorLens" }), PHOTOEYE);
    case "twinova.sensor_laser/1":
      return beamSensor(project, instance, laserGeometry(g), LASER);
    case "twinova.sensor_through_beam/1": {
      // The receiver is the emitter's housing turned to face it from the far edge: lens to lens, the host's width
      // and 20 mm each side (physics world_sensors.rs `LENS_OUT`).
      const on = host(project, instance.id);
      if (!on) return null;
      const span = 2 * (value(on.instance, "width", 0.6) / 2 + LENS_OUT + PHOTOEYE.front);
      const emitter = photoeyeGeometry(g);
      const receiver = emitter.map((part) => ({
        ...part,
        desc: g.transform(part.desc, { rotate: yaw(Math.PI), translate: [0, 0, span] }),
      }));
      return beamSensor(project, instance, [...emitter, ...receiver], PHOTOEYE);
    }
    case "twinova.scanner_barcode/1": {
      // OIP's arch, its columns clear of the host by 0.2 m each side and its head 0.6 m over the surface, turned so its
      // travel (+Z) runs along the host.
      const on = host(project, instance.id);
      if (!on) return null;
      const parts = barcodeScannerGeometry(g, {
        width: value(on.instance, "width", 0.6) + 2 * ARCH_CLEARANCE,
        surfaceHeight: value(on.instance, "height", 0.8),
        clearance: ARCH_HEAD,
      });
      const placed = mountFrame(project, on.instance, on.at, 0, 0);
      return {
        parts,
        position: on.instance.position,
        rotation: on.instance.rotation,
        ...placed,
        turn: yaw(Math.PI / 2),
      };
    }
    default:
      return null;
  }
}
