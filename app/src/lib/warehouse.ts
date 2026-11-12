import * as THREE from "three";

/** Static context only: the warehouse never enters picking, snapping or the simulation's physics world. */
export function warehouse(width: number): { group: THREE.Group; dispose: () => void } {
  const group = new THREE.Group();
  const cube = new THREE.BoxGeometry(1, 1, 1);
  const materials = {
    wall: new THREE.MeshStandardMaterial({ color: 0xa7b0b6, roughness: 0.92 }),
    steel: new THREE.MeshStandardMaterial({ color: 0x536575, roughness: 0.65, metalness: 0.45 }),
    trim: new THREE.MeshStandardMaterial({ color: 0x8594a0, roughness: 0.55, metalness: 0.5 }),
    yellow: new THREE.MeshStandardMaterial({ color: 0xd5af48, roughness: 0.85 }),
    orange: new THREE.MeshStandardMaterial({ color: 0xd28749, roughness: 0.7 }),
    carton: new THREE.MeshStandardMaterial({ color: 0xb99a73, roughness: 0.98 }),
    glass: new THREE.MeshStandardMaterial({
      color: 0xaecad9,
      roughness: 0.3,
      emissive: 0x7e9da9,
      emissiveIntensity: 0.22,
    }),
  };
  function box(
    size: [number, number, number],
    at: [number, number, number],
    material: keyof typeof materials,
  ): THREE.Mesh {
    const mesh = new THREE.Mesh(cube, materials[material]);
    mesh.scale.set(...size);
    mesh.position.set(...at);
    mesh.receiveShadow = true;
    group.add(mesh);
    return mesh;
  }
  const edge = width / 2;
  box([width, 6, 0.15], [0, 3, -edge], "wall");
  box([0.15, 6, width], [-edge, 3, 0], "wall");
  for (let y = 0.6; y < 6; y += 0.65) {
    box([width, 0.025, 0.04], [0, y, -edge + 0.09], "trim");
    box([0.04, 0.025, width], [-edge + 0.09, y, 0], "trim");
  }
  for (let x = -edge; x <= edge; x += 4) {
    box([0.16, 6.1, 0.22], [x, 3.05, -edge + 0.18], "steel");
    box([0.25, 0.85, 0.3], [x, 0.425, -edge + 0.18], "yellow");
    box([2.6, 1.05, 0.08], [x + 1.6, 4.8, -edge + 0.11], "glass");
    box([0.11, 0.16, width], [x, 6, 0], "steel");
  }
  for (let z = -edge; z <= edge; z += 4) {
    box([0.22, 6.1, 0.16], [-edge + 0.18, 3.05, z], "steel");
    box([0.3, 0.85, 0.25], [-edge + 0.18, 0.425, z], "yellow");
  }
  // A marked service aisle, with storage against the back wall, leaves the work area clear.
  box([0.07, 0.008, width - 1], [edge - 2, 0.007, 0], "yellow");
  box([width - 2, 0.008, 0.07], [0, 0.007, -edge + 2.8], "yellow");
  for (let z = -2.5; z < 2.5; z += 0.45) box([1.35, 0.008, 0.21], [edge - 1, 0.009, z], "wall");
  for (let bay = 0; bay < 2; bay++) {
    const x = -edge + 2.3 + bay * 3.2;
    for (const dx of [-1.35, 1.35])
      for (const dz of [-0.5, 0.5]) box([0.08, 3.8, 0.08], [x + dx, 1.9, -edge + 1.1 + dz], "steel");
    for (const y of [0.25, 1.5, 2.75]) {
      box([2.8, 0.09, 1.15], [x, y, -edge + 1.1], "orange");
      for (const dx of [-0.88, 0, 0.88]) box([0.75, 0.8, 0.85], [x + dx, y + 0.45, -edge + 1.1], "carton");
    }
  }
  return {
    group,
    dispose: () => {
      cube.dispose();
      Object.values(materials).forEach((m) => m.dispose());
    },
  };
}
