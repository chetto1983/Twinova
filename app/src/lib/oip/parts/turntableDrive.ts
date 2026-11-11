// Copied from Open Industry Project, examples/parts/turntable-conveyor/part.js `rig()` at 1e7488c (MIT, Copyright (c)
// 2024-present Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the turntable's
// two drives, as the rig's drawn links in the part's frame (turntable.ts). The side gearmotor that drives the rollers
// rides the deck; the slew drive's ring gear turns with it, while its base beams, arm and gearmotor stand still.
import { type Link, type LinkShape, xQuat } from "./rigLinks";
import { type Material, type Quat, type V3, yawQuat } from "./part";

/** The deck disc's diameter over the lane width (TURNTABLE_PLATE_FACTOR). */
export const PLATE = 1.5;
/** ROLLER_CLASSES.heavy. */
export const HEAVY = { radius: 0.04, pitch: 0.13 };

export function clamp(v: number, lo: number, hi: number): number {
  return Math.min(Math.max(v, lo), hi);
}

const RZ90: Quat = [0, 0, Math.SQRT1_2, Math.SQRT1_2]; // cylinder local +Y → +X
const RX90: Quat = [Math.SQRT1_2, 0, 0, Math.SQRT1_2]; // cylinder local +Y → +Z

const box = (x: number, y: number, z: number): LinkShape => ({ box: [x, y, z] });
const cylinder = (radius: number, height: number): LinkShape => ({ radius, height });

// The side motor block - a compact right-angle gearmotor riding the deck: gearbox, C-face adapter, a finned
// induction motor, end bell, black fan cowl + grille, a terminal box on the outward flank, a nameplate. A pivot
// tracks the lane edge (z = width/2 + MOTOR_EDGE) so the whole assembly reflows with `width`.
//
// IT STAYS BELOW THE ROLLER TOPS. The motor stands beside the lane, where a load wider than the lane (a pallet's
// overhang) passes over it, and a decorative link has no collider. So the motor is SET INTO A POCKET in the deck plate
// (a dark opening with a steel rim, flush on the plate top) and sized so its highest point clears the cargo datum by
// 5 mm. The terminal box sits on the outward flank rather than on top for the same reason.
const MOTOR_EDGE = 0.1; // motor axis, out from the lane edge
// pivot-local frame: +x down the lane (motor axis), +z outward from the lane. The pivot sits 0.032 below the cargo
// datum, so the plate top is at y = −0.04 and the datum at +0.032.
const DATUM = 0.032;
const DECK_TOP = -0.04;
const TOP = DATUM - 0.005; // the motor's highest point: 5 mm under the roller tops
const BODY_R = 0.045;
const FIN_H = 0.011; // how far the cooling fins stand off the body
const AXIS_Y = TOP - BODY_R - FIN_H; // the top fins are the motor's highest point
// the drive line: the heavy rollers' axles are HEAVY.radius below the datum, and the side rail (0.01 thick) stands
// just outside the lane
const DRIVE_Y = DATUM - HEAVY.radius;
const RAIL_OUT = -MOTOR_EDGE + 0.01;
// SCALE WITH THE LANE. Full size at the default width and up; on a narrower turntable the whole assembly shrinks in
// proportion (s = width / MOTOR_W0, floored at MOTOR_S_MIN) so it stays on the plate. Every piece scales about a point
// that must not move: x about the drive shaft (x = 0), z about the side rail's outer face (the bearing stays bolted to
// the rail), y by its mode -
//   'deck'  about the plate top, so the motor shrinks DOWN into its pocket and stays under the roller tops;
//   'drive' about the roller axle line, so the shaft stays on the centre roller's axle;
//   'fixed' not at all (the pocket + rim lie on the plate; the gearbox keeps its height so the axle line, which is
//           fixed by the roller size, always lands inside it).
const MOTOR_W0 = 1.524;
const MOTOR_S_MIN = 0.5;
type YMode = "deck" | "drive" | "fixed";

const GB = { x: 0.1, y: TOP - (AXIS_Y - 0.05), z: 0.09 }; // gearbox case: top at TOP, 5 cm under the axis
const GB_Y = TOP - GB.y / 2;
const BODY_X0 = GB.x / 2 + 0.015; // motor body starts past the gearbox + adapter
const BODY_L = 0.15;
const BODY_X = BODY_X0 + BODY_L / 2;
const COWL_X0 = BODY_X0 + BODY_L + 0.012; // past the end bell
// cooling fins: radial ribs around the body, leaving the top (nameplate) and the outward flank (terminal box) clear,
// like a cast TEFC frame; the ones under the plate top are hidden in the pocket
const FIN_ANGLES = [35, 60, 120, 145, 215, 240, 270, 300, 325];
// the pocket: its footprint covers the whole motor (gearbox face to grille, lane side of the gearbox to the terminal
// box), drawn as a dark opening 1 mm proud of the plate top with a thin steel rim round it
const TB_Z = BODY_R + 0.018; // terminal box centre, out on the flank
const POCKET_X = [-GB.x / 2 - 0.012, COWL_X0 + 0.05] as const;
const POCKET_Z = [-GB.z / 2 - 0.012, TB_Z + 0.03] as const;
const PX = (POCKET_X[0] + POCKET_X[1]) / 2;
const PZ = (POCKET_Z[0] + POCKET_Z[1]) / 2;
const PL = POCKET_X[1] - POCKET_X[0];
const PW = POCKET_Z[1] - POCKET_Z[0];
const RIM_W = 0.012;
const RIM_H = 0.004;

/** The roller drive, at the deck's rest; every piece rides the deck. */
export function sideGearmotor(width: number): Link[] {
  const pivot: V3 = [0, -0.122 + 0.09, width / 2 + MOTOR_EDGE]; // deck-local y 0.09, the deck at −0.122
  const s = clamp(width / MOTOR_W0, MOTOR_S_MIN, 1);
  const about = (v: number, c: number) => c + (v - c) * s;
  const mp = (shape: LinkShape, o: V3, material: Material, yMode: YMode = "deck", quat?: Quat): Link => {
    const sized: LinkShape =
      "box" in shape
        ? { box: [shape.box[0] * s, yMode === "fixed" ? shape.box[1] : shape.box[1] * s, shape.box[2] * s] }
        : { radius: shape.radius * s, height: shape.height * s };
    const y = yMode === "fixed" ? o[1] : about(o[1], yMode === "drive" ? DRIVE_Y : DECK_TOP);
    const offset: V3 = [pivot[0] + o[0] * s, pivot[1] + y, pivot[2] + about(o[2], RAIL_OUT)];
    return { shape: sized, offset, quat, material, moving: true };
  };
  const fin = (deg: number): Link => {
    const a = (deg * Math.PI) / 180;
    const r = BODY_R + FIN_H / 2;
    const at: V3 = [BODY_X, AXIS_Y + Math.cos(a) * r, Math.sin(a) * r];
    return mp(box(BODY_L - 0.02, FIN_H, 0.006), at, "turntablePaint", "deck", xQuat(a));
  };
  const rimY = DECK_TOP + RIM_H / 2;
  const bearingBolts = [-1, 1].flatMap((sx) =>
    [-1, 1].map((sy) =>
      mp(
        cylinder(0.0035, 0.005),
        [sx * 0.017, DRIVE_Y + sy * 0.017, RAIL_OUT + 0.0105],
        "turntableSteel",
        "drive",
        RX90,
      ),
    ),
  );
  return [
    mp(box(PL, 0.002, PW), [PX, DECK_TOP + 0.001, PZ], "turntablePocket", "fixed"),
    mp(box(PL + 2 * RIM_W, RIM_H, RIM_W), [PX, rimY, POCKET_Z[0] - RIM_W / 2], "turntableRim", "fixed"),
    mp(box(PL + 2 * RIM_W, RIM_H, RIM_W), [PX, rimY, POCKET_Z[1] + RIM_W / 2], "turntableRim", "fixed"),
    mp(box(RIM_W, RIM_H, PW), [POCKET_X[0] - RIM_W / 2, rimY, PZ], "turntableRim", "fixed"),
    mp(box(RIM_W, RIM_H, PW), [POCKET_X[1] + RIM_W / 2, rimY, PZ], "turntableRim", "fixed"),
    // right-angle gearbox: the cast case, its top at the clearance line
    mp(box(GB.x, GB.y, GB.z), [0, GB_Y, 0], "turntablePaint", "fixed"),
    // output side (toward the lane): a shaft-mounted drive onto the CENTRE roller. The gearbox output sits on that
    // roller's axle line (DRIVE_Y, x = 0 - the bed's k = 0 roller), and the shaft runs through a flange bearing bolted
    // to the side rail's outer face into the roller end, so the motor visibly drives the bed.
    mp(cylinder(0.026, 0.012), [0, DRIVE_Y, -GB.z / 2 - 0.006], "turntablePaint", "drive", RX90),
    mp(
      cylinder(0.012, MOTOR_EDGE - GB.z / 2 + 0.01),
      [0, DRIVE_Y, (-GB.z / 2 - MOTOR_EDGE - 0.01) / 2],
      "turntableSteel",
      "drive",
      RX90,
    ),
    mp(cylinder(0.018, 0.01), [0, DRIVE_Y, (-GB.z / 2 - 0.012 + RAIL_OUT) / 2], "turntableSteel", "drive", RX90),
    mp(box(0.05, 0.05, 0.008), [0, DRIVE_Y, RAIL_OUT + 0.004], "turntablePaint", "drive"),
    ...bearingBolts,
    // C-face adapter flange between the gearbox and the motor
    mp(cylinder(BODY_R + 0.006, 0.015), [GB.x / 2 + 0.0075, AXIS_Y, 0], "turntablePaint", "deck", RZ90),
    // the motor: body, fins, non-drive end bell
    mp(cylinder(BODY_R, BODY_L), [BODY_X, AXIS_Y, 0], "turntablePaint", "deck", RZ90),
    ...FIN_ANGLES.map(fin),
    mp(cylinder(BODY_R + 0.005, 0.012), [BODY_X0 + BODY_L + 0.006, AXIS_Y, 0], "turntablePaint", "deck", RZ90),
    // fan cowl + its vent grille on the end face
    mp(cylinder(BODY_R + 0.004, 0.04), [COWL_X0 + 0.02, AXIS_Y, 0], "turntableCowl", "deck", RZ90),
    mp(cylinder(BODY_R - 0.008, 0.003), [COWL_X0 + 0.0415, AXIS_Y, 0], "turntableGrille", "deck", RZ90),
    // terminal box on the outward flank: box, lid, and a cable gland out its end
    mp(box(0.06, 0.05, 0.03), [BODY_X - 0.01, AXIS_Y, TB_Z], "turntablePaint"),
    mp(box(0.066, 0.056, 0.006), [BODY_X - 0.01, AXIS_Y, TB_Z + 0.018], "turntablePaint"),
    mp(cylinder(0.007, 0.016), [BODY_X + 0.028, AXIS_Y, TB_Z], "turntableCowl", "deck", RZ90),
    // nameplate on top of the body, between the fins
    mp(box(0.05, 0.002, 0.026), [BODY_X, AXIS_Y + BODY_R + 0.001, 0], "turntableNameplate"),
  ];
}

// THE SLEW DRIVE - what turns the deck: a ring gear on the underside of the rotating frame, and a gearmotor on the
// FIXED base whose pinion meshes with it. So: a ring gear under the drum (on the deck, so it turns with it), an X of
// base beams joining the legs just under it, and a vertical gearmotor at angle 0 (between two legs) with its pinion on
// the ring's edge, carried by an arm off the beams' crossing.
//
// Heights hang from the drum bottom, each piece at −0.5 + k. The motor reaches SLEW_REACH below it, so on a stand too
// low to fit it the whole drive RIDES UP instead of reaching the floor: y = max(−0.5 + k, −standHeight + SLEW_REACH +
// FLOOR_GAP + k). It then sits up inside the drum (hidden) with only the cowl showing under it. Widths track the lane
// like the legs: the ring radius is min(0.5, 0.75*width − 0.3) (floored at 0.05), so the ring, pinion and motor stay
// under a narrow deck and clear of the leg ring.
const SLEW_REACH = 0.298; // drum bottom → grille face
const FLOOR_GAP = 0.007;
const RING_R = 0.5;
const RING_H = 0.025;
const PINION_R = 0.045;
const SLEW_X = RING_R + PINION_R - 0.004; // pinion + motor axis, radially out on +x (meshing the ring edge)

/** The deck's slew drive at the deck's rest, its base beams reaching the legs on a ring of `legR`. */
export function slewDrive(width: number, standHeight: number, legR: number): Link[] {
  const slewY = (k: number) => Math.max(-standHeight + SLEW_REACH + FLOOR_GAP + k, -0.5 + k);
  // a value that tracks the ring radius: v at full size, v − (RING_R − ring(w)) on a narrow deck
  const ring = (v: number, floor: number) => clamp((width * PLATE) / 2 + v - RING_R - 0.3, floor, v);
  const slewX = ring(SLEW_X, SLEW_X - RING_R + 0.05);
  const sp = (shape: LinkShape, k: number, material: Material): Link => ({
    shape,
    offset: [slewX, slewY(k), 0],
    material,
  });
  const ringY = slewY(-RING_H / 2);
  const beamY = slewY(-0.06);
  return [
    // the ring gear: a machined toothed band round a dark web, turning with the deck
    {
      shape: cylinder(ring(RING_R, 0.05), RING_H),
      offset: [0, ringY, 0],
      material: "turntableGear",
      moving: true,
      fine: true,
    },
    {
      shape: cylinder(ring(RING_R - 0.02, 0.03), RING_H + 0.002),
      offset: [0, ringY, 0],
      material: "turntableRim",
      moving: true,
      fine: true,
    },
    // the fixed base: an X of beams leg to leg, 1 cm under the ring
    ...[1, -1].map((sgn): Link => ({
      shape: box(2 * legR, 0.05, 0.08),
      offset: [0, beamY, 0],
      quat: yawQuat((sgn * Math.PI) / 4),
      material: "leg",
    })),
    // the arm from the beams' crossing out to the slew motor
    { shape: box(slewX, 0.04, 0.06), offset: [slewX / 2, beamY, 0], material: "leg" },
    // the slew gearmotor, standing on its axis: pinion on the ring edge, gearbox, motor body, cowl, grille
    sp(cylinder(PINION_R, RING_H), -RING_H / 2, "turntableSteel"),
    sp(box(0.1, 0.07, 0.1), -0.06, "turntablePaint"),
    sp(cylinder(0.042, 0.16), -0.175, "turntablePaint"),
    sp(cylinder(0.046, 0.04), -0.275, "turntableCowl"),
    sp(cylinder(0.036, 0.003), -SLEW_REACH + 0.0015, "turntableGrille"),
  ];
}
