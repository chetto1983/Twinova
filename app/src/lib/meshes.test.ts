import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { partsBounds, toBufferGeometry, toObject } from "./meshes";
import { createGeometryBuilder } from "./oip/geometry/builder";
import { GeometryBudgetError, MAX_DESCRIPTION_VERTICES } from "./oip/geometry/mesh";
import { chainTransferGeometry } from "./oip/parts/chainTransfer";
import { photoeyeGeometry } from "./oip/parts/photoeye";
import { rollerGeometry } from "./oip/parts/roller";

const r = (x: number) => Math.round(x * 1e4) / 1e4;
const rollerParams = { length: 2, width: 0.6, frameHeight: 0.17, standHeight: 0.8 };
/** The leg pieces and bolts a roller conveyor's stands are drawn from, at a stand height. */
const stands = (standHeight: number) =>
  rollerGeometry(createGeometryBuilder(), { ...rollerParams, standHeight })
    .slice(-2)
    .map((p) => p.instances!.length);

const bounds = (geometry: THREE.BufferGeometry) => {
  geometry.computeBoundingBox();
  const box = geometry.boundingBox!;
  return [box.min.toArray(), box.max.toArray()].map((v) => v.map((x) => Math.round(x * 1e4) / 1e4));
};

describe("mesh descriptions in three.js", () => {
  it("keeps the vertices, the triangles and the texture coordinates, and derives the normals", () => {
    const g = createGeometryBuilder();
    const geometry = toBufferGeometry(g.box([2, 1, 0.5], [0, 0.5, 0]));
    expect(geometry.getAttribute("position").count).toBe(24);
    expect(geometry.getAttribute("uv").count).toBe(24);
    expect(geometry.getIndex()!.count).toBe(36);
    const normal = geometry.getAttribute("normal");
    expect([normal.getX(0), normal.getY(0), normal.getZ(0)]).toEqual([1, 0, 0]);
    expect(bounds(geometry)).toEqual([
      [-1, 0, -0.25],
      [1, 1, 0.25],
    ]);
  });
});

describe("OIP's geometry budget", () => {
  it("refuses a surface past the vertex budget instead of allocating without bound", () => {
    const surface = createGeometryBuilder().surface();
    for (let k = 0; k < MAX_DESCRIPTION_VERTICES; k++) surface.vertex(0, 0, 0);
    expect(() => surface.vertex(0, 0, 0)).toThrow(GeometryBudgetError);
    expect(() => surface.vertex(0, 0, 0)).toThrow("geometry exceeds the vertex budget: 200001 > 200000");
  });
});

describe("OIP's chain transfer", () => {
  it("spans its lanes across the part and its lane length along the divert, under the deck's top", () => {
    const parts = chainTransferGeometry(createGeometryBuilder(), { chains: 2, distance: 0.33, laneLength: 0.6 });
    expect(parts.map((p) => [p.material, p.scroll])).toEqual([
      ["chainFrame", undefined],
      ["support", undefined],
      ["chain", 16],
    ]);
    const all = partsBounds(parts);
    // OIP's rig.json: the lane bottom at −0.3622 under the deck; the flanges 0.1248 across, thinned by 0.04 / 0.087.
    const flange = 0.1248 * (0.04 / 0.087);
    expect([r(all.min.x), r(all.max.x)]).toEqual([r(-0.165 - flange), r(0.165 + flange)]);
    // Along the divert, the lane measures laneLength to its feet (OIP's comment); the chain's half-loop, 0.1106 out from
    // the run's end against the feet's 0.1096 (part.js BEND), passes them by 1 mm.
    expect([r(all.min.z), r(all.max.z)]).toEqual([-0.301, 0.301]);
    expect(r(all.min.y)).toBe(-0.3622);
    expect(Math.abs(all.max.y)).toBeLessThan(0.001);
  });
});

describe("OIP's roller conveyor", () => {
  const parts = rollerGeometry(createGeometryBuilder(), rollerParams);

  it("draws its rollers, stubs and bands as banks, at the roller pitch under the surface", () => {
    // n = ⌊(length − 2R) / pitch⌋ + 1 rollers, two stubs each, a band per neighbouring pair (two arcs, two strands).
    // Two stands (0.15 m in from each end); at 0.625 m each has 10 boxes (telescoping leg, foot, bracket, channel,
    // a cross brace) and 14 bolts (adjustment, anchor, three on the bracket per side).
    expect(parts.map((p) => [p.material, p.instances?.length])).toEqual([
      ["roller", 27],
      ["roller", 54],
      ["band", 52],
      ["band", 52],
      ["frame", undefined],
      ["guard", undefined],
      ["leg", 20],
      ["bolt", 28],
    ]);
    const tubes = parts[0].instances!;
    expect(tubes[1].position[0] - tubes[0].position[0]).toBeCloseTo(0.075, 9);
    expect(tubes[0].position[1]).toBe(-0.024);
    // Each tube reaches the surface: radius 0.024 on axles 0.024 down.
    const tube = partsBounds([parts[0]]);
    expect(tube.max.y).toBeCloseTo(0, 3);
  });

  it("spans its length, its frame and guards, and stands on the floor", () => {
    const all = partsBounds(parts);
    // rig.json: the guard's lip tops out at 0.1475 + 0.0025 and reaches 0.025 + 0.01 past the half width.
    // Along x the bed's ends, except the drive bands of the end rollers: each loop runs round its roller on the side away
    // from its neighbour, lr + cord/2 past the last axle at 13 × 0.075 (rollerBed.js).
    const loop = 0.975 + 0.024 + 0.4 * 0.0048 + 0.0024;
    expect([r(all.min.x), r(all.max.x)]).toEqual([r(-loop), r(loop)]);
    // The foot plates stand on a 3 mm gap above the floor (legs.js).
    expect([r(all.min.y), r(all.max.y)]).toEqual([-0.797, 0.15]);
    // Across, the stands' foot plates (6" wide, centred under the uprights at hw + 0.01 − 0.0445 / 2) pass the guard lip.
    const foot = 0.3 + 0.01 - 0.0445 / 2 + 0.0508;
    expect(all.min.z).toBeCloseTo(-foot, 6);
    expect(all.max.z).toBeCloseTo(foot, 6);
  });

  it("fits its stands to their height: no telescoping leg when low, knee braces when tall", () => {
    // Per station (two of them), legs.js: low, an upright, a foot and a bracket a side and the channel, with two anchor
    // and three bracket bolts a side; tall, the telescoping leg, foot, bracket and a knee brace a side, the channel
    // and one cross brace, with two adjustment, two anchor, three bracket and two knee-brace bolts a side.
    expect(stands(0.5)).toEqual([2 * 7, 2 * 10]);
    expect(stands(1.6)).toEqual([2 * 12, 2 * 18]);
  });

  it("places an instance with no turn or scale of its own as given", () => {
    const g = createGeometryBuilder();
    const object = toObject(
      { desc: g.box([1, 1, 1], [0, 0, 0]), material: "leg", instances: [{ position: [1, 2, 3] }] },
      new THREE.MeshBasicMaterial(),
    );
    const m = new THREE.Matrix4();
    (object as THREE.InstancedMesh).getMatrixAt(0, m);
    expect(m.equals(new THREE.Matrix4().makeTranslation(1, 2, 3))).toBe(true);
  });

  it("instances a mesh once per transform", () => {
    const object = toObject(parts[0], new THREE.MeshBasicMaterial());
    expect(object).toBeInstanceOf(THREE.InstancedMesh);
    expect((object as THREE.InstancedMesh).count).toBe(27);
  });
});

describe("OIP's photo eye", () => {
  it("stands its housing on its connector, the pot at its height, the bracket down the rail", () => {
    const parts = photoeyeGeometry(createGeometryBuilder());
    expect(parts.map((p) => p.material)).toEqual(["body", "lens", "black", "pot", "steel"]);
    const all = partsBounds(parts);
    // sensorPhotoeye.js: SENSOR_HEIGHT 0.16 at the pot's slot; the bracket's toe down to −0.07 on the wall face, 20 mm
    // out from the lens face less the wall's 10 mm; the housing's back at −0.036.
    expect(all.max.y).toBeCloseTo(0.16, 6);
    expect(all.min.y).toBeCloseTo(-0.07, 6);
    expect(all.min.z).toBeCloseTo(-0.036, 6);
    expect(all.max.z).toBeCloseTo(0.037 + 0.02 - 0.01, 6);
  });
});
