// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the barcode scanner's arch
// (examples/parts/barcode-scanner/behavior.ts `geometry`). Its scan window and good-read beacon are drawn, not lit.
// Part-local metres: the arch's centre on the floor, +Z the direction of travel, its columns at ±X either side of the
// bed.
import type { GeometryBuilder, MeshDescription } from "../geometry/types";
import type { MeshPart, V3 } from "./part";

type Face = "x-" | "x+" | "y-" | "y+" | "z-" | "z+";

export interface ArchParams {
  /** The clear width between the columns. */
  width: number;
  /** The conveying surface's height under the arch. */
  surfaceHeight: number;
  /** From the surface up to the scanner head's window. */
  clearance: number;
}

/** Two aluminium profiles on anchored floor plates, a crossbeam on them, and the scanner head hung under its centre with
 *  its window at the clearance height. */
export function barcodeScannerGeometry(g: GeometryBuilder, p: ArchParams): MeshPart[] {
  const width = p.width;
  const s = p.surfaceHeight;
  const c = p.clearance;
  const PW = 0.05;
  const PD = 0.1;
  const FOOT_X = 0.14;
  const FOOT_Z = 0.22;
  const FOOT_T = 0.012;
  const B = 0.003;
  const HEAD_H = 0.12;
  const BEAM_H = 0.08;
  const HEAD_X = 0.24;
  const HEAD_Z = 0.09;
  const hw = width / 2;
  const beamBottom = s + c + HEAD_H;
  const top = beamBottom + BEAM_H;
  const win = s + c;
  const bb = (size: V3, center: V3, square?: Face[]): MeshDescription => g.bevelBox(size, center, B, square);
  // a hex head standing on a face: `face` its coordinate along the axis, sg the side it stands out to
  const hex = (axis: "y" | "z", center: V3, face: number, sg: number, af: number, h: number): MeshDescription => {
    const a = sg > 0 ? face : face - h;
    const b = sg > 0 ? face + h : face;
    const r = af / Math.sqrt(3);
    return g.lathe(
      [
        [0, a],
        [r, a],
        [r, b],
        [0, b],
      ],
      { axis, center, segments: 6, smooth: false },
    );
  };
  const dark: MeshDescription[] = [];
  const columns: MeshDescription[] = [];
  for (const side of [-1, 1]) {
    const cx = side * (hw + PW / 2);
    columns.push(
      bb([PW, beamBottom - FOOT_T, PD], [cx, (FOOT_T + beamBottom) / 2, 0], ["y-", "y+"]),
      bb([FOOT_X, FOOT_T, FOOT_Z], [cx, FOOT_T / 2, 0], ["y-"]),
    );
    // the profile's T-slots, one down each face
    const h0 = FOOT_T + 0.02;
    const h1 = beamBottom - 0.01;
    for (const f of [-1, 1]) {
      dark.push(g.box([0.0016, h1 - h0, 0.008], [cx + f * (PW / 2 + 0.0006), (h0 + h1) / 2, 0]));
      dark.push(g.box([0.008, h1 - h0, 0.0016], [cx, (h0 + h1) / 2, f * (PD / 2 + 0.0006)]));
      // two anchors through the foot plate, fore and aft of the profile
      dark.push(g.box([0.03, 0.003, 0.03], [cx, FOOT_T + 0.0015, f * 0.08]));
      dark.push(hex("y", [cx, 0, f * 0.08], FOOT_T + 0.003, 1, 0.02, 0.011));
      dark.push(hex("y", [cx, 0, f * 0.08], FOOT_T + 0.014, 1, 0.011, 0.008));
      // a corner bracket on each face where the column meets the beam, bolted into both
      const ix = side * hw;
      const zf = (f * PD) / 2;
      dark.push(
        g.extrude(
          [
            [ix, beamBottom],
            [ix - side * 0.08, beamBottom],
            [ix, beamBottom - 0.08],
          ],
          zf,
          zf + f * 0.004,
          { axis: "z" },
        ),
      );
      dark.push(hex("z", [ix - side * 0.052, beamBottom - 0.014, 0], zf + f * 0.004, f, 0.014, 0.006));
      dark.push(hex("z", [ix - side * 0.014, beamBottom - 0.052, 0], zf + f * 0.004, f, 0.014, 0.006));
    }
    // the beam's end cap
    dark.push(
      bb(
        [0.008, BEAM_H - 0.006, PD - 0.006],
        [side * (hw + PW + 0.004), beamBottom + BEAM_H / 2, 0],
        [side > 0 ? "x-" : "x+"],
      ),
    );
  }
  const beam = bb([width + 2 * PW, BEAM_H, PD], [0, beamBottom + BEAM_H / 2, 0]);
  // the head: a finned housing on a bracket bolted up into the beam, a bezel round its window, and its cable leaving
  // through a gland on top and running up into the beam
  const bodyTop = beamBottom - 0.028;
  dark.push(bb([HEAD_X - 0.02, bodyTop - win - 0.012, HEAD_Z], [0, (win + 0.012 + bodyTop) / 2, 0]));
  dark.push(bb([HEAD_X, 0.012, HEAD_Z + 0.006], [0, win + 0.016, 0]));
  for (const f of [-1, 1]) {
    for (let k = 0; k < 3; k++)
      dark.push(
        g.box(
          [0.012, bodyTop - win - 0.04, 0.004],
          [f * (HEAD_X / 2 - 0.004), (win + 0.03 + bodyTop) / 2, -0.03 + k * 0.03],
        ),
      );
    dark.push(hex("y", [f * 0.03, 0, f * 0.045], beamBottom, -1, 0.014, 0.006));
  }
  dark.push(bb([0.1, 0.028, 0.07], [0, bodyTop + 0.014, 0], ["y+"]));
  dark.push(
    g.lathe(
      [
        [0, bodyTop],
        [0.011, bodyTop],
        [0.011, bodyTop + 0.012],
        [0.007, bodyTop + 0.012],
        [0, bodyTop + 0.012],
      ],
      { axis: "y", center: [0.075, 0, 0], segments: 6, smooth: false },
    ),
  );
  dark.push(
    g.lathe(
      [
        [0, bodyTop + 0.012],
        [0.005, bodyTop + 0.012],
        [0.005, beamBottom],
        [0, beamBottom],
      ],
      { axis: "y", center: [0.075, 0, 0], segments: 10 },
    ),
  );
  // the beacon's collar on top of the beam
  dark.push(
    g.lathe(
      [
        [0, top],
        [0.037, top],
        [0.037, top + 0.008],
        [0, top + 0.008],
      ],
      { axis: "y", center: [0, 0, 0], segments: 20 },
    ),
  );
  // the head's window, the scan line, and the good-read beacon on top of the beam
  const window = g.box([HEAD_X - 0.04, 0.01, HEAD_Z - 0.03], [0, win + 0.005, 0]);
  const beacon = g.lathe(
    [
      [0, 0],
      [0.03, 0],
      [0.03, 0.03],
      [0.02, 0.05],
      [0, 0.056],
    ],
    { axis: "y", center: [0, top, 0], segments: 16 },
  );
  return [
    { desc: g.merge(...columns), material: "scannerHousing" },
    { desc: beam, material: "scannerBeam" },
    { desc: g.merge(...dark), material: "scannerHead" },
    { desc: g.merge(window, beacon), material: "scannerWindow" },
  ];
}
