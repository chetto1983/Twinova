// The materials OIP draws these parts in, at 1e7488c (MIT; THIRD_PARTY_NOTICES.md): the parts' own (their rig.json
// `materials`, linear RGB) and the conveyor finishes of src/render/parts/common.ts (sRGB hex). OIP's finishes are a
// triplanar shader over its Metal.webp; Twinova keeps their colour, metalness and roughness on three's standard
// material, and maps the two textures whose meshes carry real UVs: the roller tube and the chain links.
import * as THREE from "three";
import chainLink from "../assets/oip/ChainLink.webp";
import rollerMetal from "../assets/oip/RollerMetal.webp";
import type { Material } from "./oip/parts/part";

interface Spec {
  /** sRGB hex (a finish) or linear RGB (a part's material). */
  color: number | readonly [number, number, number];
  metalness: number;
  roughness: number;
  map?: string;
}

export const SPECS: Record<Material, Spec> = {
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
    material.map = map;
  }
  return material;
}
