import * as THREE from "three";
import { drawnOf } from "./families";
import { materialOf } from "./materials";
import { toObject } from "./meshes";
import { shapeOf, type Definition, type Instance, type ProjectView } from "./project";

// Preview the same bundled definitions and mesh generators used by placement, using one short-lived WebGL context.
const definitions = Object.values(
  import.meta.glob<Definition>("../../src-tauri/library/components/*.json", { eager: true, import: "default" }),
);
const base = (key: string, id: string): Instance => ({
  id,
  name: id,
  definition: key,
  position: [0, 0, 0],
  rotation: [0, 0, 0, 1],
  parameters: {},
  signals: {},
});

export async function partPreviews(keys: string[]): Promise<Record<string, string>> {
  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setSize(256, 144);
  renderer.setPixelRatio(1);
  const manager = new THREE.LoadingManager();
  const loader = new THREE.TextureLoader(manager);
  const result: Record<string, string> = {};
  const defs = Object.fromEntries(definitions.map((d) => [`${d.id}@${d.version}`, d]));
  try {
    for (const key of keys) {
      const def = defs[key];
      if (!def) continue;
      const instance = base(key, "preview");
      const host = base("twinova.roller@1", "host");
      const project: ProjectView = {
        name: "",
        folder: null,
        instances: [instance, host],
        definitions: defs,
        changed_outside: [],
        recovered: false,
        program: [],
        behaviours: {},
        panel: [],
        mounts: [{ device: "preview", host: "host", at: 0 }],
        flows: [],
        merges: [],
        interfaces: {},
        notice: null,
        can_undo: false,
        can_redo: false,
      };
      const group = new THREE.Group();
      const drawn = drawnOf(project, instance);
      const materials: THREE.MeshStandardMaterial[] = [];
      const geometries: THREE.BufferGeometry[] = [];
      let loading = false;
      let completed!: () => void;
      const loaded = new Promise<void>((resolve) => (completed = resolve));
      manager.onStart = () => (loading = true);
      manager.onLoad = () => completed();
      if (drawn) {
        for (const part of drawn.parts) {
          const material = materialOf(part.material, loader);
          const object = toObject(part, material);
          group.add(object);
          materials.push(material);
          object.traverse((o) => {
            if (o instanceof THREE.Mesh) geometries.push(o.geometry);
          });
        }
      } else {
        const shape = shapeOf(instance, def);
        const geometry = new THREE.BoxGeometry(...shape.size);
        const material = new THREE.MeshStandardMaterial({
          color: key.includes("source") ? 0xd8ab5e : 0x7893a6,
          roughness: 0.6,
        });
        group.add(new THREE.Mesh(geometry, material));
        materials.push(material);
        geometries.push(geometry);
      }
      // The renderer and loading manager are shared; finish one preview before reusing them.
      // eslint-disable-next-line no-await-in-loop
      if (loading) await loaded;
      const scene = new THREE.Scene();
      scene.add(group, new THREE.HemisphereLight(0xddeeff, 0x6d6d69, 2.5));
      const light = new THREE.DirectionalLight(0xfff7e8, 3);
      light.position.set(4, 8, 6);
      scene.add(light);
      const bounds = new THREE.Box3().setFromObject(group);
      const centre = bounds.getCenter(new THREE.Vector3());
      const size = bounds.getSize(new THREE.Vector3());
      const radius = Math.max(size.length() / 2, 0.05);
      const camera = new THREE.PerspectiveCamera(32, 256 / 144, 0.001, 500);
      camera.position.copy(centre).addScaledVector(new THREE.Vector3(1.3, 1, 1.5).normalize(), radius * 3.8);
      camera.lookAt(centre);
      renderer.render(scene, camera);
      result[key] = renderer.domElement.toDataURL("image/png");
      for (const geometry of geometries) geometry.dispose();
      for (const material of materials) {
        material.map?.dispose();
        material.dispose();
      }
    }
  } finally {
    renderer.dispose();
    renderer.forceContextLoss();
  }
  return result;
}
