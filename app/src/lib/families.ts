// Which instances OIP's generators draw (lib/oip/parts), and where: a roller conveyor centred on its length with the
// rollers' top at its `height`; a chain transfer and a photo eye from their mount on the host, as the physics places
// them (physics/src/world.rs). The other families keep their stand-in shapes (project.ts `shapeOf`) until theirs come
// (the belt with the curves, 4.4b).
import { createGeometryBuilder } from "./oip/geometry/builder";
import { chainTransferGeometry } from "./oip/parts/chainTransfer";
import type { MeshPart } from "./oip/parts/part";
import { PHOTOEYE, photoeyeGeometry } from "./oip/parts/photoeye";
import { rollerGeometry } from "./oip/parts/roller";
import { flag, real, type Instance, type ProjectView, type Quat, type Vec3 } from "./project";

/** The chain deck under its host's rollers, and its stroke (physics world.rs `TRANSFER_BELOW`, `TRANSFER_STROKE`). */
const TRANSFER_BELOW = 0.03;
export const TRANSFER_STROKE = 0.095;
/** A diffuse sensor's beam above its host's surface (world.rs `BEAM_HEIGHT`). */
const BEAM_HEIGHT = 0.05;

export interface Drawn {
  parts: MeshPart[];
  /** The pose the parts hang from: the instance's own, or its host's for a mounted device. */
  position: Vec3;
  rotation: Quat;
  /** Where the parts' origin sits in that frame. */
  offset: Vec3;
  /** A chain transfer's deck rises by its lift times this. */
  stroke?: number;
}

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

export function drawnOf(project: ProjectView, instance: Instance): Drawn | null {
  const def = project.definitions[instance.definition];
  const value = (inst: Instance, name: string, fallback: number) =>
    real(inst, project.definitions[inst.definition], name, fallback);
  const g = createGeometryBuilder();
  switch (def?.generator) {
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
      const offset: Vec3 = [on.at, surface + BEAM_HEIGHT - PHOTOEYE.emitY, -edge - 0.02 - PHOTOEYE.front];
      return { parts: photoeyeGeometry(g), position: on.instance.position, rotation: on.instance.rotation, offset };
    }
    default:
      return null;
  }
}
