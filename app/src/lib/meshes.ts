// OIP's mesh descriptions (lib/oip/geometry) into three.js, as OIP's renderer does: flat buffers with normals derived
// here, and a bank (rollers, leg pieces) as one instanced draw.
import * as THREE from "three";
import type { MeshDescription } from "./oip/geometry/types";
import type { MeshPart } from "./oip/parts/part";

export function toBufferGeometry(desc: MeshDescription): THREE.BufferGeometry {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.Float32BufferAttribute(desc.positions, 3));
  geometry.setAttribute("uv", new THREE.Float32BufferAttribute(desc.uvs, 2));
  geometry.setIndex(desc.indices);
  geometry.computeVertexNormals();
  return geometry;
}

/** A part as a mesh, or an instanced mesh at its transforms. */
export function toObject(part: MeshPart, material: THREE.Material): THREE.Mesh {
  const geometry = toBufferGeometry(part.desc);
  if (!part.instances) return new THREE.Mesh(geometry, material);
  const mesh = new THREE.InstancedMesh(geometry, material, part.instances.length);
  const m = new THREE.Matrix4();
  part.instances.forEach((inst, k) => {
    const q = inst.quaternion ?? [0, 0, 0, 1];
    const s = inst.scale ?? [1, 1, 1];
    m.compose(new THREE.Vector3(...inst.position), new THREE.Quaternion(...q), new THREE.Vector3(...s));
    mesh.setMatrixAt(k, m);
  });
  mesh.computeBoundingBox();
  return mesh;
}

/** The box around a set of parts, instances included. */
export function partsBounds(parts: readonly MeshPart[]): THREE.Box3 {
  const all = new THREE.Box3();
  for (const part of parts) {
    const object = toObject(part, new THREE.MeshBasicMaterial());
    if (object instanceof THREE.InstancedMesh) all.union(object.boundingBox!);
    else {
      object.geometry.computeBoundingBox();
      all.union(object.geometry.boundingBox!);
    }
  }
  return all;
}
