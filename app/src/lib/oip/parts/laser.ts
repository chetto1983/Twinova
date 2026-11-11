// Copied from Open Industry Project at 1e7488c (MIT, Copyright (c) 2024-present Automation Standard LLC and
// Contributors; THIRD_PARTY_NOTICES.md), typed for TypeScript: the laser distance sensor's housing
// (examples/parts/laser-sensor/part.js `geometry`). OIP draws its parts double-sided; three's default side is kept, as
// for the photo eye.
// Part-local metres: +Z the front (the beam axis), y 0 the underside of the connector, the beam at `LASER.emitY`.
import type { GeometryBuilder } from "../geometry/types";
import { circle, rrect } from "./outlines";
import type { MeshPart } from "./part";

export const LASER = {
  halfX: 0.057, // 114 mm wide across the body
  panel: 0.0008, // the label panel standing proud of each flank
  back: -0.0735,
  capZ: 0.04, // where the dark front cap meets the body
  front: 0.074, // the window face: the beam origin and the guard contact
  bodyY0: 0.038, // the housing's underside; the M12 plug hangs below it
  bodyY1: 0.152,
  emitY: 0.095, // the window's centre
};

/** A body with two mounting holes and a label panel on each flank, a dark front cap with the window and four screws,
 *  a top panel with a display and two status LEDs, and an M12 plug underneath. */
export function laserGeometry(g: GeometryBuilder): MeshPart[] {
  const X = LASER.halfX;
  const PANEL = LASER.panel;
  const BACK = LASER.back;
  const CAPZ = LASER.capZ;
  const FRONT = LASER.front;
  const Y0 = LASER.bodyY0;
  const Y1 = LASER.bodyY1;
  const YC = (Y0 + Y1) / 2;
  const body = g.extrude(
    [
      [Y0, BACK],
      [Y0, CAPZ],
      [Y1, CAPZ],
      [Y1, BACK],
    ],
    -X,
    X,
    {
      axis: "x",
      bevel: 0.002,
      square: [1],
      holes: [circle(Y0 + 0.02, BACK + 0.0155, 0.0033), circle(Y1 - 0.02, BACK + 0.0155, 0.0033)],
    },
  );
  const cap = g.bevelBox([2 * X, Y1 - Y0, FRONT - 0.002 - CAPZ], [0, YC, (CAPZ + FRONT - 0.002) / 2], 0.002, ["z-"]);
  const win = g.extrude(rrect(-0.04, YC - 0.04, 0.04, YC + 0.04, 0.002), FRONT - 0.002, FRONT, { bevel: 0.0005 });
  const screws = [];
  for (const sx of [-1, 1])
    for (const sy of [-1, 1])
      screws.push(
        g.lathe(
          [
            [0, 0],
            [0.0045, 0],
            [0.0045, 0.0006],
            [0.003, 0.0012],
            [0, 0.0012],
          ],
          { axis: "z", center: [sx * 0.0495, YC + sy * 0.0495, FRONT - 0.002], segments: 12 },
        ),
      );
  const panels = [-1, 1].map((s) =>
    g.extrude(rrect(Y0 + 0.012, BACK + 0.03, Y1 - 0.012, CAPZ - 0.008, 0.004), s * X, s * (X + PANEL), { axis: "x" }),
  );
  const top = g.extrude(rrect(-0.044, BACK + 0.01, 0.044, CAPZ - 0.008, 0.004), Y1, Y1 + 0.001, {
    axis: "y",
    bevel: 0.0003,
  });
  const display = g.extrude(rrect(-0.034, -0.012, 0.034, 0.022, 0.003), Y1 + 0.001, Y1 + 0.0016, { axis: "y" });
  const leds = [-1, 1].map((s) =>
    g.lathe(
      [
        [0, 0],
        [0.004, 0],
        [0.004, 0.0008],
        [0, 0.0012],
      ],
      { axis: "y", center: [s * 0.013, Y1 + 0.001, -0.038], segments: 16 },
    ),
  );
  const PZ = -0.01;
  const plug = g.lathe(
    [
      [0, 0],
      [0.0078, 0],
      [0.0085, 0.0008],
      [0.0085, 0.01],
      [0.009, 0.02],
      [0.009, Y0],
      [0, Y0],
    ],
    { axis: "y", center: [0, 0, PZ], segments: 24 },
  );
  const nut = g.lathe(
    [
      [0, 0.01],
      [0.0115, 0.01],
      [0.0115, 0.021],
      [0, 0.021],
    ],
    { axis: "y", center: [0, 0, PZ], segments: 6, smooth: false },
  );
  return [
    { desc: body, material: "laserAmber" },
    // OIP's `dark` is the photo eye's lens material, the same colour and finish.
    { desc: g.merge(cap, top, ...panels), material: "lens" },
    { desc: win, material: "laserWindow" },
    { desc: g.merge(plug, nut, ...screws), material: "black" },
    { desc: display, material: "laserDisplay" },
    { desc: g.merge(...leds), material: "laserLed" },
  ];
}
