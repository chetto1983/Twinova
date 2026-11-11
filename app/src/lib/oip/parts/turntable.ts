// Copied from Open Industry Project, examples/parts/turntable-conveyor/part.js at 1e7488c (MIT, Copyright (c)
// 2024-present Automation Standard LLC and Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript. Its
// `geometry()`, the round roller bed, and the links `rig()` draws: the deck disc and drum, four static single-pole legs
// on a ring with their foot plates, and the two drives (turntableDrive.ts). The colliders and the hidden convey lane
// are the physics', not drawn.
// The origin is the deck's centre at the cargo datum (y 0, the rollers' top), x along the lane at the deck's rest, z
// across, the floor at −standHeight. The deck, the bed on it, the side gearmotor and the ring gear are `link: "moving"`
// and drawn at the rest: the view turns them about +Y by the deck's angle.
import type { GeometryBuilder } from "../geometry/types";
import { cylinderTemplate } from "./legs";
import { type Instance, type MeshPart, yawQuat } from "./part";
import { drawLinks, type Link } from "./rigLinks";
import { type Roller, rollerBed } from "./roller";
import { clamp, HEAVY, PLATE, sideGearmotor, slewDrive } from "./turntableDrive";

export interface TurntableParams {
  /** The conveying lane across the deck; the disc is PLATE times as wide. */
  width: number;
  /** The cargo datum above the floor. */
  standHeight: number;
}

const DECK_Y = -0.122; // the deck link's centre, under the datum
// the drum under the deck plate: its top at the plate top (0.072 under the datum = deck-local +0.05), its full depth
// 0.428, and the gap it keeps above the floor on a stand too low for that depth
const PLATE_TOP = 0.072;
const DRUM_TOP = 0.05;
const DRUM_H = 0.428;
const DRUM_GAP = 0.05;
// leg ring radius: 0.62 on a normal deck, pulled in on a NARROW one so the legs stay under the disc (radius
// 0.75*width) and inside the drum. A fixed 0.62 puts the legs outside the plate below a ~0.9 m lane.
const LEG_R = 0.62;
const LEG_INSET = 0.13;
const LEG_R_MIN = 0.05;
// the pole's top, just under the plate top at −0.072, so it hides beneath the disc instead of poking through
const LEG_TOP = 0.1;

// ---- geometry(): the round roller bed ------------------------------------------------------------------------------
// Rollers fanned across the full plate diameter at the heavy pitch, each clipped to the plate circle (long across the
// lane, tapering to the round rim), end brackets on the tapered outer rollers the side rails don't reach, and a
// hex-bolt ring rimming the plate.
const RIM = 0.06; // rollers clip this far in from the plate rim
const BRACKET = 0.06; // length of one roller end-holder along the lane
const WALL = 0.01; // side-frame wall thickness

function roundBed(g: GeometryBuilder, width: number): MeshPart[] {
  const R = HEAVY.radius;
  const deckR = (width * PLATE) / 2;
  const laneHalf = width / 2;
  const rBed = Math.max(deckR - RIM, 0);
  const kMax = Math.floor(rBed / HEAVY.pitch);
  const rollers: Roller[] = [];
  const tapered: { s: number; half: number }[] = [];
  for (let k = -kMax; k <= kMax; k++) {
    const s = k * HEAVY.pitch;
    const half = Math.min(laneHalf, Math.sqrt(Math.max(rBed * rBed - s * s, 0)));
    if (half <= 0.02) continue; // a sliver at the very rim
    rollers.push({ x: s, z: 0, yaw: 0, length: half * 2 });
    if (half < laneHalf - 0.01) tapered.push({ s, half }); // shorter than the lane: past the side rails
  }
  const parts = rollerBed(g, rollers, { radius: R });
  // side-frame steel off the roller radius: from just below the plate top up to the frame top
  const plateTopY = -1.8 * R;
  const baseY = plateTopY - 0.01;
  const frameTopY = -0.3 * R;
  const railH = Math.max(frameTopY - baseY, 0.02);
  const railCY = baseY + railH / 2;
  const boxes: Instance[] = [];
  const railLen = 2 * Math.sqrt(Math.max(rBed * rBed - laneHalf * laneHalf, 0));
  if (railLen > 0.05) {
    const zc = laneHalf + WALL / 2;
    for (const z of [zc, -zc]) boxes.push({ position: [0, railCY, z], scale: [railLen, railH, WALL] });
  }
  for (const { s, half } of tapered) {
    const z = half + WALL / 2;
    for (const zz of [z, -z]) boxes.push({ position: [s, railCY, zz], scale: [BRACKET, railH, WALL] });
  }
  parts.push({ desc: g.box([1, 1, 1], [0, 0, 0]), material: "frame", instances: boxes });
  // the perimeter bolt ring: hex studs seated on the plate rim, each pointing radially outward, one per ~0.25 m
  const count = Math.max(8, Math.round((2 * Math.PI * deckR) / 0.25));
  const boltY = plateTopY - 0.06;
  const rBolt = deckR + 0.005;
  const bolts: Instance[] = [];
  for (let i = 0; i < count; i++) {
    const a = (2 * Math.PI * i) / count;
    bolts.push({
      position: [Math.cos(a) * rBolt, boltY, Math.sin(a) * rBolt],
      quaternion: yawQuat(Math.PI / 2 - a),
      scale: [0.012, 0.012, 0.01],
    });
  }
  const hex = cylinderTemplate(g, 6, { startAngle: Math.PI / 2, smooth: false });
  parts.push({ desc: hex, material: "bolt", instances: bolts });
  for (const part of parts) part.link = "moving";
  return parts;
}

// ---- rig(): the deck, the drum and the stand ----------------------------------------------------------------------
function deckAndStand(width: number, standHeight: number): Link[] {
  const discR = (width * PLATE) / 2;
  const legR = clamp(discR - LEG_INSET, LEG_R_MIN, LEG_R);
  const ring = [0, 1, 2, 3].map((i) => (i / 4) * Math.PI * 2 + Math.PI / 4);
  // On a LOW stand the drum stops DRUM_GAP above the floor instead of sinking through it: its depth and centre both
  // track standHeight, and clamp at the same stand height (0.55), above which nothing changes.
  const drumH = clamp(standHeight - (PLATE_TOP + DRUM_GAP), 0.02, DRUM_H);
  const drumY = clamp(DRUM_TOP + (PLATE_TOP + DRUM_GAP) / 2 - standHeight / 2, DRUM_TOP - DRUM_H / 2, DRUM_TOP - 0.01);
  return [
    { shape: { radius: discR, height: 0.1 }, offset: [0, DECK_Y, 0], material: "frame", moving: true, fine: true },
    {
      shape: { radius: discR - 0.05, height: drumH },
      offset: [0, DECK_Y + drumY, 0],
      material: "frame",
      moving: true,
      fine: true,
    },
    // the poles: their tops at −LEG_TOP, their feet at the floor, so raising the turntable regrows them
    ...ring.map((a): Link => ({
      shape: { box: [0.1, Math.max(standHeight - LEG_TOP, 0.05), 0.1] },
      offset: [Math.cos(a) * legR, -0.5 * standHeight - LEG_TOP / 2, Math.sin(a) * legR],
      material: "leg",
    })),
    ...ring.map((a): Link => ({
      shape: { box: [0.26, 0.02, 0.26] },
      offset: [Math.cos(a) * legR, -standHeight, Math.sin(a) * legR],
      material: "leg",
    })),
    ...slewDrive(width, standHeight, legR),
    ...sideGearmotor(width),
  ];
}

export function turntableGeometry(g: GeometryBuilder, p: TurntableParams): MeshPart[] {
  return [...roundBed(g, p.width), ...drawLinks(g, deckAndStand(p.width, p.standHeight))];
}
