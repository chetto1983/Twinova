<script lang="ts">
  // The 3D view of an opened project: each instance as its family's stand-in shape, sized from its parameters
  // (project.ts, shapeOf), at its pose. Read-only in 4.2; the selected instance is highlighted.
  import * as THREE from "three";
  import { frame, shapeOf, type ProjectView } from "./project";
  import Viewport from "./Viewport.svelte";

  let { project, selected, theme }: { project: ProjectView; selected: string | null; theme: string } = $props();

  const scene = new THREE.Scene();
  const content = new THREE.Group();
  scene.add(content);
  let requestRender = $state(() => {});
  const view = $derived(frame(project.instances));
  const target = $derived(new THREE.Vector3(...view.centre));

  function token(name: string): THREE.Color {
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return new THREE.Color(value || "#888");
  }

  /** Frees what the previous build gave the GPU: clearing the group alone would leak a geometry and a material per
   * instance at every selection change. */
  function dispose(): void {
    content.traverse((o) => {
      if (o instanceof THREE.Mesh) {
        o.geometry.dispose();
        (o.material as THREE.Material).dispose();
      }
    });
    content.clear();
  }

  function build(): void {
    dispose();
    for (const instance of project.instances) {
      const def = project.definitions[instance.definition];
      if (!def) continue;
      const shape = shapeOf(instance, def);
      const [x, y, z] = shape.size;
      const geometry =
        shape.kind === "box" ? new THREE.BoxGeometry(x, y, z) : new THREE.CylinderGeometry(x / 2, z / 2, y);
      const lit = instance.id === selected;
      const material = new THREE.MeshStandardMaterial({
        color: token(shape.token),
        emissive: lit ? token("--scene-selected") : new THREE.Color(0),
        emissiveIntensity: lit ? 0.6 : 0,
        roughness: 0.8,
      });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.castShadow = true;
      mesh.position.set(...shape.centre);
      const placed = new THREE.Group();
      placed.position.set(...instance.position);
      placed.quaternion.set(...instance.rotation);
      placed.add(mesh);
      content.add(placed);
    }
  }

  $effect(() => {
    void [project, selected, theme];
    build();
    requestRender();
  });

  $effect(() => dispose);
</script>

<Viewport {scene} {theme} {target} halfWidth={view.radius} bind:requestRender />
