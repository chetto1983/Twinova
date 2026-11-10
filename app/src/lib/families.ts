// Which instances OIP's generators draw (lib/oip/parts), and where, as the physics places them (physics/src/layout.rs,
// path.rs, world.rs):
// - a belt from its tail and a roller conveyor centred on its length, the surface at their `height`;
// - a curve from its entry stub's tip on the centreline. OIP draws a curve about its centre, entering at +X heading
//   −Z and turning left: Twinova's left curve is that drawing turned −90° about +Y; its right curve is the same drawing
//   run backwards (turned π/2 − sweep), never mirrored, so its faces keep their winding (and its fabric scrolls back);
// - a chain transfer and a photo eye from their mount on the host, along a straight host or at an angle on a curve.
// The other families keep their stand-in shapes (project.ts `shapeOf`).
import { createGeometryBuilder } from "./oip/geometry/builder";
import { beltGeometry } from "./oip/parts/belt";
import { beltCurveGeometry } from "./oip/parts/beltCurve";
import { chainTransferGeometry } from "./oip/parts/chainTransfer";
import type { CurveParams } from "./oip/parts/curve";
import type { MeshPart } from "./oip/parts/part";
import { PHOTOEYE, photoeyeGeometry } from "./oip/parts/photoeye";
import { rollerGeometry } from "./oip/parts/roller";
import { rollerCurveGeometry } from "./oip/parts/rollerCurve";
import { flag, real, type Instance, type ProjectView, type Quat, type Vec3 } from "./project";

/** The chain deck under its host's rollers, and its stroke (physics world.rs `TRANSFER_BELOW`, `TRANSFER_STROKE`). */
const TRANSFER_BELOW = 0.03;
export const TRANSFER_STROKE = 0.095;
/** A diffuse sensor's beam above its host's surface (world.rs `BEAM_HEIGHT`). */
const BEAM_HEIGHT = 0.05;
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
  /** A chain transfer's deck rises by its lift times this. */
  stroke?: number;
}

const yaw = (angle: number): Quat => [0, Math.sin(angle / 2), 0, Math.cos(angle / 2)];

function host(project: ProjectView, device: string): { instance: Instance; at: number } | null {
  const mount = project.mounts.find((m) => m.device === device);
  const instance = mount && project.instances.find((i) => i.id === mount.host);
  return mount && instance ? { instance, at: mount.at } : null;
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
  if (!CURVES[project.definitions[on.definition]?.generator ?? ""]) return { offset: [at, up, across] };
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
        parts,
        position: on.instance.position,
        rotation: on.instance.rotation,
        offset: [on.at, top, 0],
        stroke: TRANSFER_STROKE,
      };
    }
    case "twinova.sensor_diffuse/1": {
      const on = host(project, instance.id);
      if (!on) return null;
      // The lens on the beam: OIP seats the eye 20 mm out from the bed edge, facing across it (+z), and draws its beam
      // at emitY above its origin; Twinova's beam runs BEAM_HEIGHT above the surface (spec 2026-11-10, "The view").
      const surface = value(on.instance, "height", 0.8);
      const edge = value(on.instance, "width", 0.6) / 2;
      const up = surface + BEAM_HEIGHT - PHOTOEYE.emitY;
      const placed = mountFrame(project, on.instance, on.at, up, -edge - 0.02 - PHOTOEYE.front);
      return { parts: photoeyeGeometry(g), position: on.instance.position, rotation: on.instance.rotation, ...placed };
    }
    default:
      return null;
  }
}
