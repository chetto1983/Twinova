// Copied from Open Industry Project, examples/parts/diverter/behavior.ts `geometry()` at 1e7488c (MIT, Copyright (c)
// 2024-present Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript. A guided
// pusher: a bearing block holds two guide rods and the rod of a drive cylinder, all three fixed to the blade and moving
// with it as one stage. The block is the same for every stroke; what the stroke sizes is what has to be as long as it:
// the guide rods (retracted, they stand out behind the block by the stroke, tied by a bar that comes up against the
// block at full reach), the cylinder's barrel behind the block, and the stand under both.
//
// Frame: local −Z is the push direction. The blade's face at rest is z = −1.103 for every stroke; the machine grows
// back from it along +Z. y 0 is 0.2 m below the running surface the pusher is mounted against; the floor is at
// −standHeight. The moving stage (`link: "moving"`, OIP's `partEnd` link) is drawn at rest, in the part's frame.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import type { MeshPart } from "./part";

type V2 = readonly [number, number];
type V3 = readonly [number, number, number];

export interface PusherParams {
  /** OIP's `divertDistance`: how far the blade travels, 0.1..1.5 m. */
  stroke: number;
  /** From the origin down to the floor. */
  standHeight: number;
}

/** A solid cylinder's profile, `r` round and `h` long. */
const disc = (r: number, h: number): V2[] => [
  [0, 0],
  [r, 0],
  [r, h],
  [0, h],
];

/** A ring 8 mm deep round a bore of radius `r`, `h` long: a bezel, a wiper. */
const collar = (r: number, h: number): V2[] => [
  [r, 0],
  [r + 0.008, 0],
  [r + 0.008, h],
  [r, h],
  [r, 0],
];

// along Z: the blade (rubber face, plate, the boss the rods mount to), the glands, the bearing block, then the
// cylinder barrel behind it
const BLADE_FRONT = -1.103,
  RUBBER = 0.012,
  PLATE = 0.025,
  BOSS = 0.035,
  GLAND = 0.04,
  BLOCK = 0.42;
const TIE = 0.03; // the tie bar's depth: at full reach it sits against the rear glands
const REAR_GLAND = 0.025; // a shallower gland round each guide rod where it leaves the block's rear face
const CAP = 0.03;
const PLATE_T = 0.012;
const BASE = 0.24; // the block's underside, just under the blade's bottom edge
const ROD_Y = 0.446,
  ROD_X = 0.13,
  GUIDE_R = 0.025; // the guide rods
const DRIVE_Y = 0.33,
  DRIVE_R = 0.016,
  BARREL_R = 0.045,
  CAP_W = 0.12; // the drive cylinder, below the tie bar's path
const BLADE_Y = 0.447,
  BLADE_W = 0.47,
  BLADE_H = 0.395;
const HW = 0.31,
  CAB_HW = 0.29,
  CAB_TOP = 0.5,
  TOP = 0.66;

const PORT_X = -CAP_W / 2 - 0.022; // the tip of each cylinder port's fitting, where its hose starts
const HOSE_R = 0.0055;
// [up, width], up first: the other order turns the tube inside out
const HOSE: V2[] = Array.from({ length: 10 }, (_, i): V2 => [
  HOSE_R * Math.cos((2 * Math.PI * i) / 10),
  HOSE_R * Math.sin((2 * Math.PI * i) / 10),
]);

/** Where the machine's pieces sit along Z for stroke `S`. */
function stations(S: number) {
  const bladeBack = BLADE_FRONT + RUBBER + PLATE + BOSS;
  const front = bladeBack + GLAND; // the block's front face, where the rods leave it
  const blockRear = front + BLOCK;
  const glandRear = blockRear + REAR_GLAND;
  const rodTail = glandRear + S + TIE; // the guide rods' back end, at rest
  const barrelEnd = blockRear + S + 0.04; // the barrel holds the piston's whole travel
  return { bladeBack, front, blockRear, glandRear, rodTail, barrelEnd };
}

type Stations = ReturnType<typeof stations>;

const tube = (g: GeometryBuilder, x: number, y: number, z0: number, length: number, r: number, segments = 20) =>
  g.lathe(disc(r, length), { axis: "z", center: [x, y, z0], segments });

/** Hex bolt heads, nuts and port bodies: a six-faceted disc. */
const hex = (g: GeometryBuilder, r: number, h: number, axis: "x" | "y" | "z", center: V3) =>
  g.lathe(disc(r, h), { axis, center, segments: 6, smooth: false });

/** A push-in air fitting, 8 mm round and 12 mm long. */
const fitting = (g: GeometryBuilder, axis: "x" | "z", center: V3) =>
  g.lathe(disc(0.008, 0.012), { axis, center, segments: 12 });

/** The stand: four square-tube legs on foot plates under a top frame that runs the machine's length, a mounting
 *  plate under the block, a saddle under the barrel's end, and a rail round the legs low down; with the anchor bolts
 *  through the foot plates. */
function stand(g: GeometryBuilder, z: Stations, floor: number): { frame: MeshDescription; bolts: MeshDescription[] } {
  const LEG = 0.05,
    LEG_X = 0.27,
    FOOT = 0.12,
    FOOT_T = 0.012,
    RAIL = 0.04,
    RAIL_Y = floor + 0.12;
  const legZ = [z.front + 0.07, z.barrelEnd - 0.03] as const;
  const frameTop = BASE - PLATE_T,
    frameY = frameTop - RAIL / 2;
  const legH = frameTop - RAIL - floor - FOOT_T;
  const span = legZ[1] - legZ[0],
    spanMid = (legZ[0] + legZ[1]) / 2;
  const parts = [g.box([2 * HW, PLATE_T, BLOCK], [0, BASE - PLATE_T / 2, z.front + BLOCK / 2])];
  const bolts: MeshDescription[] = [];
  for (const x of [-LEG_X, LEG_X]) {
    for (const lz of legZ) {
      parts.push(g.box([LEG, legH, LEG], [x, floor + FOOT_T + legH / 2, lz]));
      parts.push(g.bevelBox([FOOT, FOOT_T, FOOT], [x, floor + FOOT_T / 2, lz], 0.003, ["y-"]));
      for (const dz of [-0.035, 0.035]) bolts.push(hex(g, 0.011, 0.007, "y", [x, floor + FOOT_T, lz + dz]));
    }
    parts.push(g.box([RAIL, RAIL, span + LEG], [x, frameY, spanMid]));
    parts.push(g.box([RAIL, RAIL, span - LEG], [x, RAIL_Y, spanMid]));
  }
  for (const lz of legZ) {
    parts.push(g.box([2 * LEG_X - LEG, RAIL, RAIL], [0, frameY, lz]));
    parts.push(g.box([2 * LEG_X - LEG, RAIL, RAIL], [0, RAIL_Y, lz]));
  }
  const saddleH = DRIVE_Y - BARREL_R - frameTop;
  parts.push(g.box([0.12, saddleH, 0.05], [0, frameTop + saddleH / 2, legZ[1]]));
  return { frame: g.merge(...parts), bolts };
}

/** The hood, drawn as its SIDE profile and run across the width: a rounded nose just behind the rod face, a rounded
 *  back, and a diagonal cut-away from the nose down toward the stand that leaves the cabinet showing under the rods.
 *  In the profile's (z, y): along the bottom, up the back, over the top, down the nose. */
function hood(g: GeometryBuilder, z: Stations): MeshDescription {
  const NOSE_R = 0.06,
    BACK_R = 0.07,
    LOW = BASE + 0.06;
  const nose = z.front + 0.06;
  const cut = nose + Math.min(0.2, 0.5 * (z.blockRear - BACK_R - nose)); // where the diagonal meets the hood's bottom edge
  const zy: V2[] = [
    [cut, LOW],
    [z.blockRear, LOW],
  ];
  for (const q of g.arc([z.blockRear - BACK_R, TOP - BACK_R, 0], BACK_R, 0, Math.PI / 2, 6)) zy.push([q[0], q[1]]);
  for (const q of g.arc([nose + NOSE_R, TOP - NOSE_R, 0], NOSE_R, Math.PI / 2, Math.PI, 6)) zy.push([q[0], q[1]]);
  zy.push([nose, CAB_TOP]);
  // an outline across X is (y, z): the swap mirrors it, so reverse it to keep it counter-clockwise
  return g.extrude(zy.map(([zz, y]): V2 => [y, zz]).toReversed(), -HW, HW, { axis: "x", bevel: 0.012 });
}

/** A gland on the block's face round each rod, and on the rear face round each guide rod, so the rods pass through a
 *  part rather than into the hood's bare panel (at full reach the tie bar stops against these); four bolts on the
 *  face of each front guide gland, and a rubber wiper ring where each rod leaves its gland. */
function glands(g: GeometryBuilder, z: Stations) {
  const wiperZ = z.front - GLAND - 0.006;
  const bodies = [g.bevelBox([0.07, 0.07, GLAND], [0, DRIVE_Y, z.front - GLAND / 2], 0.006, ["z+"])];
  const wipers = [g.lathe(collar(DRIVE_R, 0.006), { axis: "z", center: [0, DRIVE_Y, wiperZ], segments: 20 })];
  const bolts: MeshDescription[] = [];
  for (const gx of [-ROD_X, ROD_X]) {
    bodies.push(g.bevelBox([0.09, 0.09, GLAND], [gx, ROD_Y, z.front - GLAND / 2], 0.006, ["z+"]));
    bodies.push(g.bevelBox([0.09, 0.09, REAR_GLAND], [gx, ROD_Y, z.blockRear + REAR_GLAND / 2], 0.006, ["z-"]));
    for (const wz of [wiperZ, z.glandRear])
      wipers.push(g.lathe(collar(GUIDE_R, 0.006), { axis: "z", center: [gx, ROD_Y, wz], segments: 20 }));
    for (const bx of [-0.035, 0.035])
      for (const by of [-0.035, 0.035])
        bolts.push(hex(g, 0.011, 0.007, "z", [gx + bx, ROD_Y + by, z.front - GLAND - 0.007]));
  }
  return { bodies: g.merge(...bodies), wipers: g.merge(...wipers), bolts };
}

/** The moving stage, at rest: the blade (rubber face, plate, a boss across the guide rods with a rib up and down from
 *  each, a boss on the drive rod), the two guide rods with the tie bar across their back ends, and the drive rod,
 *  whose back end stays inside the barrel at rest and inside the block at full reach. OIP draws it in its partEnd
 *  link's frame, offset [0, 0.447, −0.853] from the part's. */
function stage(g: GeometryBuilder, z: Stations, S: number): MeshPart[] {
  const zPlate = BLADE_FRONT + RUBBER,
    zBoss = zPlate + PLATE;
  const rubber = g.box([BLADE_W - 0.02, BLADE_H - 0.02, RUBBER], [0, BLADE_Y, BLADE_FRONT + RUBBER / 2]);
  const plate = g.bevelBox([BLADE_W, BLADE_H, PLATE], [0, BLADE_Y, zPlate + PLATE / 2], 0.008, ["z-"]);
  const fittings = g.merge(
    g.bevelBox([2 * ROD_X + 0.1, 0.08, BOSS], [0, ROD_Y, zBoss + BOSS / 2], 0.006, ["z-"]),
    g.bevelBox([0.03, BLADE_H - 0.06, BOSS * 0.8], [-ROD_X, BLADE_Y, zBoss + BOSS * 0.4], 0.004, ["z-"]),
    g.bevelBox([0.03, BLADE_H - 0.06, BOSS * 0.8], [ROD_X, BLADE_Y, zBoss + BOSS * 0.4], 0.004, ["z-"]),
    g.bevelBox([0.07, 0.07, BOSS], [0, DRIVE_Y, zBoss + BOSS / 2], 0.006, ["z-"]),
    g.bevelBox([2 * ROD_X + 0.1, 0.07, TIE], [0, ROD_Y, z.rodTail - TIE / 2], 0.006),
  );
  const rods = g.merge(
    tube(g, -ROD_X, ROD_Y, z.bladeBack, z.rodTail - TIE - z.bladeBack, GUIDE_R),
    tube(g, ROD_X, ROD_Y, z.bladeBack, z.rodTail - TIE - z.bladeBack, GUIDE_R),
    tube(g, 0, DRIVE_Y, z.bladeBack, z.blockRear + S - 0.02 - z.bladeBack, DRIVE_R, 16),
  );
  return [
    { desc: rods, material: "pusherRod", link: "moving" },
    { desc: plate, material: "pusherHood", link: "moving" },
    { desc: fittings, material: "pusherFrame", link: "moving" },
    { desc: rubber, material: "pusherRubber", link: "moving" },
  ];
}

/** The drive cylinder: a barrel between two square end caps, held together by four tie rods with a nut at the back,
 *  and an air port on the −X side of each cap (a hex body and a push-in fitting, the hose starting at its tip). The
 *  front cap is bolted to the block's rear face. */
function cylinder(g: GeometryBuilder, z: Stations) {
  const CAP_L = 0.04,
    NUT = 0.008,
    machineRear = z.barrelEnd + CAP; // the nuts end the machine
  const capFront = z.blockRear + CAP_L / 2,
    capRear = machineRear - NUT - CAP_L / 2;
  const TIE_R = 0.006,
    TIE_OFF = CAP_W / 2 - 0.014;
  const parts = [
    tube(g, 0, DRIVE_Y, z.blockRear + CAP_L, machineRear - NUT - z.blockRear - 2 * CAP_L, BARREL_R, 24),
    g.bevelBox([CAP_W, CAP_W, CAP_L], [0, DRIVE_Y, capFront], 0.006, ["z-"]),
    g.bevelBox([CAP_W, CAP_W, CAP_L], [0, DRIVE_Y, capRear], 0.006),
  ];
  for (const tx of [-TIE_OFF, TIE_OFF]) {
    for (const ty of [-TIE_OFF, TIE_OFF]) {
      parts.push(tube(g, tx, DRIVE_Y + ty, z.blockRear, machineRear - z.blockRear - 0.002, TIE_R, 8));
      parts.push(hex(g, 0.011, NUT, "z", [tx, DRIVE_Y + ty, machineRear - NUT]));
    }
  }
  for (const cz of [capFront, capRear])
    parts.push(hex(g, 0.012, 0.01, "x", [-CAP_W / 2 - 0.01, DRIVE_Y, cz]), fitting(g, "x", [PORT_X, DRIVE_Y, cz]));
  return { barrel: g.merge(...parts), capFront, capRear };
}

/** The valve that switches the cylinder, on the block's rear face beside it, and its two hoses: one from the front
 *  port straight across, one from the rear port along the barrel's side and round into the valve's back. Each hose
 *  starts inside its port's fitting and ends inside the valve, behind a fitting of the same size on the valve's face,
 *  so neither end shows a gap or an open tube end. */
function valve(g: GeometryBuilder, z: Stations, ports: { capFront: number; capRear: number }) {
  const VALVE = [0.06, 0.05, 0.05] as const,
    VALVE_X = -0.21;
  const HOSE_BEND = 0.05,
    SEAT = 0.004;
  const body = g.bevelBox(VALVE, [VALVE_X, DRIVE_Y, z.blockRear + VALVE[2] / 2], 0.004, ["z-"]);
  const valveFace = VALVE_X + VALVE[0] / 2,
    valveBack = z.blockRear + VALVE[2];
  const fittings = g.merge(
    fitting(g, "x", [valveFace, DRIVE_Y, ports.capFront]),
    fitting(g, "z", [VALVE_X, DRIVE_Y, valveBack]),
  );
  const hoseFront = g.sweep(HOSE, [
    [PORT_X + SEAT, DRIVE_Y, ports.capFront],
    [valveFace - SEAT, DRIVE_Y, ports.capFront],
  ]);
  // the rear hose runs out along −X, bends a quarter turn round a centre inside the corner, and runs along −Z to the
  // valve's back face
  const bendC = [VALVE_X + HOSE_BEND, ports.capRear - HOSE_BEND] as const;
  const rearPath: V3[] = [[PORT_X + SEAT, DRIVE_Y, ports.capRear]];
  for (let i = 0; i <= 8; i++) {
    const a = Math.PI / 2 + (Math.PI / 2) * (i / 8);
    rearPath.push([bendC[0] + HOSE_BEND * Math.cos(a), DRIVE_Y, bendC[1] + HOSE_BEND * Math.sin(a)]);
  }
  rearPath.push([VALVE_X, DRIVE_Y, valveBack - SEAT]);
  return { body, fittings, hoses: g.merge(hoseFront, g.sweep(HOSE, rearPath)) };
}

/** The control box on the block's side (its back is where the rods come out): two buttons and the two lenses, a steel
 *  bezel round each and a visor over each lens so it reads in a bright hall; a cable gland under it, its cable bending
 *  down and into the cabinet below the hood. */
function panel(g: GeometryBuilder, z: Stations) {
  const PANEL_Y = 0.44,
    PANEL_Z = z.blockRear - 0.11,
    PANEL_D = 0.03,
    face = HW + PANEL_D;
  const box = g.bevelBox([PANEL_D + 0.01, 0.22, 0.18], [HW + PANEL_D / 2 - 0.005, PANEL_Y, PANEL_Z], 0.006, ["x-"]);
  const knob = (y: number, zz: number, profile: readonly V2[]) =>
    g.lathe(profile, { axis: "x", center: [face, y, zz], segments: 16 });
  const bezel = (y: number, zz: number, r: number) =>
    g.lathe(collar(r, 0.005), { axis: "x", center: [face, y, zz], segments: 16 });
  const visor = (y: number) => g.box([0.024, 0.005, 0.05], [face + 0.012, y + 0.028, PANEL_Z - 0.045]);
  const BUTTON = disc(0.024, 0.015);
  const LENS: V2[] = [
    [0, 0],
    [0.02, 0],
    [0.02, 0.01],
    [0.014, 0.018],
    [0, 0.02],
  ];
  const glandY = PANEL_Y - 0.11,
    cableX = HW + PANEL_D / 2;
  const cableGland = g.lathe(
    [
      [0, 0],
      [0.011, 0],
      [0.011, 0.012],
      [0.008, 0.02],
      [0, 0.02],
    ],
    { axis: "y", center: [cableX, glandY - 0.02, PANEL_Z], segments: 6, smooth: false },
  );
  const cablePath = g.fillet(
    [
      [cableX, glandY - 0.02, PANEL_Z],
      [cableX, BASE + 0.025, PANEL_Z],
      [CAB_HW - 0.01, BASE + 0.025, PANEL_Z],
    ],
    0.025,
  );
  return {
    body: g.merge(box, visor(0.48), visor(0.4), cableGland, g.sweep(HOSE, cablePath)),
    bezels: g.merge(
      bezel(0.48, PANEL_Z + 0.045, 0.024),
      bezel(0.4, PANEL_Z + 0.045, 0.024),
      bezel(0.48, PANEL_Z - 0.045, 0.02),
      bezel(0.4, PANEL_Z - 0.045, 0.02),
    ),
    buttons: g.merge(knob(0.48, PANEL_Z + 0.045, BUTTON), knob(0.4, PANEL_Z + 0.045, BUTTON)),
    lampRed: knob(0.48, PANEL_Z - 0.045, LENS),
    lampGreen: knob(0.4, PANEL_Z - 0.045, LENS),
  };
}

/** A louvred vent on the block's other side: a dark back plate with slats across it. */
function vent(g: GeometryBuilder, z: Stations) {
  const VENT_Z = z.blockRear - 0.13,
    VENT_Y = 0.52;
  const slats: MeshDescription[] = [];
  for (let i = 0; i < 4; i++)
    slats.push(g.box([0.012, 0.012, 0.15], [-HW - 0.006, VENT_Y - 0.0375 + i * 0.025, VENT_Z]));
  return { back: g.box([0.006, 0.1, 0.16], [-HW - 0.002, VENT_Y, VENT_Z]), slats: g.merge(...slats) };
}

export function pusherGeometry(g: GeometryBuilder, p: PusherParams): MeshPart[] {
  const S = Math.min(1.5, Math.max(0.1, p.stroke));
  const floor = -(p.standHeight > 0 ? p.standHeight : 0.3847);
  const z = stations(S);
  const base = stand(g, z, floor);
  // the bearing block: a dark cabinet, stopping 1 cm short of the hood's rear face so the two never share a plane
  const cabinet = g.box(
    [2 * CAB_HW, CAB_TOP - BASE, BLOCK - 0.01],
    [0, (BASE + CAB_TOP) / 2, z.front + BLOCK / 2 - 0.005],
  );
  const front = glands(g, z);
  const drive = cylinder(g, z);
  const air = valve(g, z, drive);
  const controls = panel(g, z);
  const louvre = vent(g, z);
  // the warning label: a thin box whose outward face carries a whole texture (the builder maps every box face 0..1),
  // on the back face above the rods' path (the tie bar comes up to y 0.48 at full reach)
  const label = g.box([0.1, 0.1, 0.003], [0, 0.538, z.blockRear + 0.0015]);
  return [
    { desc: g.merge(base.frame, controls.bezels), material: "pusherFrame" },
    { desc: g.merge(cabinet, controls.body, louvre.back, air.body, front.wipers), material: "pusherCabinet" },
    { desc: g.merge(hood(g, z), front.bodies, louvre.slats), material: "pusherHood" },
    { desc: g.merge(drive.barrel, air.fittings, ...base.bolts, ...front.bolts), material: "pusherBarrel" },
    { desc: air.hoses, material: "pusherHose" },
    { desc: label, material: "pusherLabel" },
    { desc: controls.buttons, material: "pusherButton" },
    { desc: controls.lampRed, material: "pusherLampRed" },
    { desc: controls.lampGreen, material: "pusherLampGreen" },
    ...stage(g, z, S),
  ];
}

/** The panel's lenses, read off the home end of stroke the way a real pusher's reed switches are: green while the
 *  blade is home, red the moment it leaves, held through the whole travel out and back. */
export function pusherLamps(reach: number): { green: boolean; red: boolean } {
  const home = reach <= 1e-4;
  return { green: home, red: !home };
}
