import { describe, expect, it } from "vitest";
import * as THREE from "three";
import type { Material } from "./oip/parts/part";
import { colorOf, materialOf, SPECS } from "./materials";

/** A loader that hands back a texture without fetching it, as the view's loader does once the image arrives. */
const loader = {
  load: (url: string) => Object.assign(new THREE.Texture(), { name: url }),
} as unknown as THREE.TextureLoader;

describe("OIP's materials", () => {
  it("reads a finish's hex as sRGB and a part's colour as linear", () => {
    expect(colorOf(SPECS.frame).getHex()).toBe(0x56a7c8);
    const band = colorOf(SPECS.band);
    expect([band.r, band.g, band.b]).toEqual([0.6939, 0.1844, 0.0242]);
  });

  it("maps the roller tube and the chain links, tiled, and leaves the plain finishes unmapped", () => {
    const roller = materialOf("roller", loader);
    expect([roller.metalness, roller.roughness, roller.side]).toEqual([0.8, 0.34, THREE.DoubleSide]);
    expect(roller.map!.name).toMatch(/RollerMetal/);
    expect([roller.map!.wrapS, roller.map!.wrapT, roller.map!.colorSpace]).toEqual([
      THREE.RepeatWrapping,
      THREE.RepeatWrapping,
      THREE.SRGBColorSpace,
    ]);
    expect(materialOf("chain", loader).map!.name).toMatch(/ChainLink/);
    expect(materialOf("guard", loader).map).toBeNull();
  });
});

/** OIP's values, at 1e7488c: the finishes of src/render/parts/common.ts and the parts' rig.json materials. */
const OIP: Record<Material, [number | number[], number, number]> = {
  frame: [0x56a7c8, 0.35, 0.45],
  guard: [0xeef1f5, 0.5, 0.4],
  leg: [0xbcc1c7, 0.4, 0.62],
  bolt: [0xeceef2, 0.85, 0.28],
  roller: [0xffffff, 0.8, 0.34],
  band: [[0.6939, 0.1844, 0.0242], 0, 0.55],
  chain: [0xffffff, 0, 1],
  support: [[0.4629, 0.4629, 0.4629], 1, 0.6264],
  chainFrame: [[0.0879, 0.0822, 0.4813], 0.7692, 0.7088],
  body: [[0.63770563, 0.21222135, 0.03189377], 0.511, 0.361],
  black: [[0.00232198, 0.00232198, 0.00232198], 0, 0.55],
  lens: [[0.00232198, 0.00116099, 0.00116099], 0, 0.35],
  pot: [[0.26327341, 0.26327341, 0.26327341], 0.3, 0.45],
  steel: [[0.47699978, 0.50707851, 0.53823552], 0.8, 0.38],
};

describe("the transcription of OIP's materials", () => {
  it.each(Object.entries(OIP))(
    "%s keeps OIP's colour, metalness and roughness",
    (name, [color, metalness, roughness]) => {
      const m = materialOf(name as Material, loader);
      const want =
        typeof color === "number" ? new THREE.Color(color) : new THREE.Color().setRGB(color[0], color[1], color[2]);
      expect(m.color.equals(want)).toBe(true);
      expect([m.metalness, m.roughness]).toEqual([metalness, roughness]);
    },
  );
});
