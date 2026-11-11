// The materials OIP draws these parts in, at 1e7488c (MIT; THIRD_PARTY_NOTICES.md): the parts' own (their rig.json
// `materials`, linear RGB) and the conveyor finishes of src/render/parts/common.ts (sRGB hex). OIP's finishes are a
// triplanar shader over its Metal.webp; Twinova keeps their colour, metalness and roughness on three's standard
// material, and maps the textures whose meshes carry real UVs: the roller tube, the chain links, the belt's weave
// (common.ts `beltWebglMaterial`), the pusher's guide and drive rods (lathed, so their UVs run round and along them) and
// its warning label. The pusher's barrel mixes faces with and without UVs, so it keeps OIP's colour without the metal.
import * as THREE from "three";
import beltFabric from "../assets/oip/BeltFabric.jpg";
import chainLink from "../assets/oip/ChainLink.webp";
import diverterLabel from "../assets/oip/DiverterLabel.webp";
import rollerMetal from "../assets/oip/RollerMetal.webp";
import type { Material } from "./oip/parts/part";

interface Spec {
  /** sRGB hex (a finish) or linear RGB (a part's material). */
  color: number | readonly [number, number, number];
  metalness: number;
  roughness: number;
  map?: string;
  /** OIP loads a part's textures with flipY off (src/render/parts/sdk/materials.ts), and its label is drawn for that. */
  flipY?: false;
  /** A lamp's lit glow, linear RGB (OIP's `lamps[].emissive`): set dark, the view lights it with emissiveIntensity 1. */
  emissive?: readonly [number, number, number];
}

export const SPECS: Record<Material, Spec> = {
  belt: { color: 0xffffff, metalness: 0, roughness: 0.85, map: beltFabric },
  frame: { color: 0x56a7c8, metalness: 0.35, roughness: 0.45 },
  guard: { color: 0xeef1f5, metalness: 0.5, roughness: 0.4 },
  leg: { color: 0xbcc1c7, metalness: 0.4, roughness: 0.62 },
  bolt: { color: 0xeceef2, metalness: 0.85, roughness: 0.28 },
  roller: { color: 0xffffff, metalness: 0.8, roughness: 0.34, map: rollerMetal },
  band: { color: [0.6939, 0.1844, 0.0242], metalness: 0, roughness: 0.55 },
  chain: { color: 0xffffff, metalness: 0, roughness: 1, map: chainLink },
  support: { color: [0.4629, 0.4629, 0.4629], metalness: 1, roughness: 0.6264 },
  chainFrame: { color: [0.0879, 0.0822, 0.4813], metalness: 0.7692, roughness: 0.7088 },
  body: { color: [0.63770563, 0.21222135, 0.03189377], metalness: 0.511, roughness: 0.361 },
  black: { color: [0.00232198, 0.00232198, 0.00232198], metalness: 0, roughness: 0.55 },
  lens: { color: [0.00232198, 0.00116099, 0.00116099], metalness: 0, roughness: 0.35 },
  pot: { color: [0.26327341, 0.26327341, 0.26327341], metalness: 0.3, roughness: 0.45 },
  steel: { color: [0.47699978, 0.50707851, 0.53823552], metalness: 0.8, roughness: 0.38 },
  // the blade stop's (examples/parts/blade-stop/rig.json: steel, plate, belt, blade)
  darkSteel: { color: [0.2204, 0.2204, 0.2204], metalness: 0.1, roughness: 0.5 },
  plate: { color: [0.8809, 0.8809, 0.8809], metalness: 0.1, roughness: 0.4657 },
  beltBlack: { color: [0.0122, 0.0122, 0.0122], metalness: 0, roughness: 1 },
  bladeOrange: { color: [0.8006, 0.3333, 0.0482], metalness: 0, roughness: 0.7886 },
  // the pusher's (examples/parts/diverter/rig.json); its DiverterMetal.webp is RollerMetal.webp, byte for byte
  pusherHood: { color: [0.6377, 0.21222, 0.03189], metalness: 0.511, roughness: 0.361 },
  pusherCabinet: { color: [0.03072, 0.03689, 0.0497], metalness: 0, roughness: 0.45 },
  pusherFrame: { color: [0.21956, 0.22322, 0.22321], metalness: 1, roughness: 0.539 },
  pusherBarrel: { color: [0.78, 0.8, 0.84], metalness: 0.75, roughness: 0.42 },
  pusherRod: { color: 0xffffff, metalness: 0.9, roughness: 0.3, map: rollerMetal },
  pusherRubber: { color: [0.012, 0.012, 0.013], metalness: 0, roughness: 0.85 },
  pusherHose: { color: [0.02, 0.16, 0.55], metalness: 0, roughness: 0.5 },
  pusherLabel: { color: 0xffffff, metalness: 0, roughness: 0.55, map: diverterLabel, flipY: false },
  pusherButton: { color: [0.00402, 0.491, 0.63755], metalness: 0, roughness: 0.5 },
  pusherLampRed: {
    color: [0.29182, 0.11443, 0.11192],
    metalness: 0,
    roughness: 0.654,
    emissive: [0.74902, 0.05098, 0.05098],
  },
  pusherLampGreen: { color: [0.15295, 0.31398, 0.14995], metalness: 0, roughness: 0.612, emissive: [0, 1, 0.28235] },
  // the turntable's drives (examples/parts/turntable-conveyor/part.js `rig()`: PAINT, BLACK, GRILLE, STEEL, POCKET, RIM,
  // NAMEPLATE, GEAR), flat colours rather than a steel finish
  turntablePaint: { color: [0.05, 0.075, 0.095], metalness: 0.15, roughness: 0.45 },
  turntableCowl: { color: [0.015, 0.015, 0.017], metalness: 0.1, roughness: 0.55 },
  turntableGrille: { color: [0.004, 0.004, 0.005], metalness: 0, roughness: 0.9 },
  turntableSteel: { color: [0.7, 0.7, 0.72], metalness: 0.9, roughness: 0.28 },
  turntablePocket: { color: [0.006, 0.006, 0.007], metalness: 0, roughness: 0.95 },
  turntableRim: { color: [0.03, 0.03, 0.035], metalness: 0.4, roughness: 0.6 },
  turntableNameplate: { color: [0.55, 0.56, 0.58], metalness: 0.7, roughness: 0.35 },
  turntableGear: { color: [0.3, 0.3, 0.32], metalness: 0.8, roughness: 0.4 },
};

export function colorOf(spec: Spec): THREE.Color {
  return typeof spec.color === "number" ? new THREE.Color(spec.color) : new THREE.Color().setRGB(...spec.color);
}

/** A new material for `name`; the caller disposes it. */
export function materialOf(name: Material, loader: THREE.TextureLoader): THREE.MeshStandardMaterial {
  const spec = SPECS[name];
  const material = new THREE.MeshStandardMaterial({
    color: colorOf(spec),
    metalness: spec.metalness,
    roughness: spec.roughness,
    side: THREE.DoubleSide,
  });
  if (spec.map) {
    const map = loader.load(spec.map);
    map.wrapS = THREE.RepeatWrapping;
    map.wrapT = THREE.RepeatWrapping;
    map.colorSpace = THREE.SRGBColorSpace;
    if (spec.flipY === false) map.flipY = false;
    material.map = map;
  }
  if (spec.emissive) {
    material.emissive.setRGB(...spec.emissive);
    material.emissiveIntensity = 0;
  }
  return material;
}
