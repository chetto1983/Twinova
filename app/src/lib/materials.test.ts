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
    expect(materialOf("belt", loader).map!.name).toMatch(/BeltFabric/);
    expect(materialOf("guard", loader).map).toBeNull();
  });

  it("maps the pusher's rods with the roller's metal and its label unflipped, as OIP loads a part's textures", () => {
    const rod = materialOf("pusherRod", loader);
    expect([rod.map!.name, rod.map!.flipY]).toEqual([expect.stringMatching(/RollerMetal/), true]);
    const label = materialOf("pusherLabel", loader);
    expect([label.map!.name, label.map!.flipY]).toEqual([expect.stringMatching(/DiverterLabel/), false]);
    expect(materialOf("pusherBarrel", loader).map).toBeNull();
  });

  it("gives a lamp its lit glow, dark until the view raises the intensity", () => {
    const red = materialOf("pusherLampRed", loader);
    expect([red.emissive.r, red.emissive.g, red.emissive.b]).toEqual([0.74902, 0.05098, 0.05098]);
    expect(red.emissiveIntensity).toBe(0);
    const green = materialOf("pusherLampGreen", loader);
    expect([green.emissive.r, green.emissive.g, green.emissive.b]).toEqual([0, 1, 0.28235]);
    const plain = materialOf("pusherButton", loader);
    expect([plain.emissive.getHex(), plain.emissiveIntensity]).toEqual([0, 1]);
  });
});

/** OIP's `lin` (examples/parts/laser-sensor/part.js, color-sensor/part.js): an authored sRGB colour, stored linear at 8
 *  decimals. */
const lin = (rgb: number[]): number[] =>
  rgb.map((c) => {
    const l = c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
    return Math.abs(l) < 1e-9 ? 0 : +l.toFixed(8);
  });

/** OIP's values, at 1e7488c: the finishes of src/render/parts/common.ts and the parts' rig.json materials. */
const OIP: Record<Material, [number | number[], number, number]> = {
  belt: [0xffffff, 0, 0.85],
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
  darkSteel: [[0.2204, 0.2204, 0.2204], 0.1, 0.5],
  plate: [[0.8809, 0.8809, 0.8809], 0.1, 0.4657],
  beltBlack: [[0.0122, 0.0122, 0.0122], 0, 1],
  bladeOrange: [[0.8006, 0.3333, 0.0482], 0, 0.7886],
  pusherHood: [[0.6377, 0.21222, 0.03189], 0.511, 0.361],
  pusherCabinet: [[0.03072, 0.03689, 0.0497], 0, 0.45],
  pusherFrame: [[0.21956, 0.22322, 0.22321], 1, 0.539],
  pusherBarrel: [[0.78, 0.8, 0.84], 0.75, 0.42],
  pusherRod: [0xffffff, 0.9, 0.3],
  pusherRubber: [[0.012, 0.012, 0.013], 0, 0.85],
  pusherHose: [[0.02, 0.16, 0.55], 0, 0.5],
  pusherLabel: [0xffffff, 0, 0.55],
  pusherButton: [[0.00402, 0.491, 0.63755], 0, 0.5],
  pusherLampRed: [[0.29182, 0.11443, 0.11192], 0, 0.654],
  pusherLampGreen: [[0.15295, 0.31398, 0.14995], 0, 0.612],
  turntablePaint: [[0.05, 0.075, 0.095], 0.15, 0.45],
  turntableCowl: [[0.015, 0.015, 0.017], 0.1, 0.55],
  turntableGrille: [[0.004, 0.004, 0.005], 0, 0.9],
  turntableSteel: [[0.7, 0.7, 0.72], 0.9, 0.28],
  turntablePocket: [[0.006, 0.006, 0.007], 0, 0.95],
  turntableRim: [[0.03, 0.03, 0.035], 0.4, 0.6],
  turntableNameplate: [[0.55, 0.56, 0.58], 0.7, 0.35],
  turntableGear: [[0.3, 0.3, 0.32], 0.8, 0.4],
  colorBody: [lin([0, 0.4, 0.5]), 0, 0.55],
  colorLens: [lin([0.45, 0, 0]), 0, 0.55],
  laserAmber: [lin([0.5, 0.35, 0.0]), 0, 0.55],
  laserWindow: [lin([0.736647, 0.736647, 0.929502]), 0, 0.55],
  laserDisplay: [lin([0.92, 0.4, 0.0]), 0, 0.45],
  laserLed: [lin([0.736647, 0.736647, 0.736647]), 0, 0.55],
  scannerHousing: [[0.479, 0.509, 0.539], 0.6, 0.35],
  scannerBeam: [[0.888, 0.508, 0.005], 0.1, 0.45],
  scannerHead: [[0.06, 0.065, 0.075], 0.2, 0.5],
  scannerWindow: [[0.18, 0.18, 0.18], 0, 0.3],
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
