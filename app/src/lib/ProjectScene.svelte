<script lang="ts">
  // The 3D view of a project: each instance as OIP's generator draws its family (families.ts), or as its stand-in
  // shape sized from its parameters (project.ts, shapeOf), at its pose, the selected one highlighted; each moving
  // device's link at its position (a transfer's or a stopper's lift, a pusher's reach, a turntable's angle), a belt's
  // fabric moved by its travel, and the cartons of the running line at the poses the physics gives them (engine.rs
  // `Snapshot.actuators`, `Snapshot.conveyors`, `Snapshot.cartons`).
  import * as THREE from "three";
  import { drawnOf, type Motion } from "./families";
  import { materialOf } from "./materials";
  import { toObject } from "./meshes";
  import { frame, shapeOf, type ProjectView } from "./project";
  import { yawQuat } from "./oip/parts/part";
  import { pusherLamps } from "./oip/parts/pusher";
  import { CARTON_SIZE_M, scaled, scrollOffset, type Carton, type Snapshot } from "./snapshot";
  import Viewport from "./Viewport.svelte";

  let {
    project,
    selected,
    theme,
    cartons,
    actuators,
    conveyors,
  }: {
    project: ProjectView;
    selected: string | null;
    theme: string;
    cartons: Carton[];
    actuators: Snapshot["actuators"];
    conveyors: Snapshot["conveyors"];
  } = $props();

  const scene = new THREE.Scene();
  const content = new THREE.Group();
  const products = new THREE.Group();
  scene.add(content, products);
  const cartonGeometry = new THREE.BoxGeometry(CARTON_SIZE_M, CARTON_SIZE_M, CARTON_SIZE_M);
  const cartonMaterial = new THREE.MeshStandardMaterial({ roughness: 0.9 });
  /** One mesh per carton id, kept while the carton lives, so a snapshot moves meshes instead of rebuilding them. */
  const meshes = new Map<number, THREE.Mesh>();
  /** Each moving device's link group, and how it follows the device's position. */
  const links = new Map<string, { group: THREE.Group; motion: Motion }>();
  /** Each pusher's green and red lamp materials, lit as OIP lights them (pusherLamps). */
  const lamps = new Map<string, { green: THREE.MeshStandardMaterial[]; red: THREE.MeshStandardMaterial[] }>();
  /** Each conveyor's scrolling textures, and their tiles per metre of travel. */
  const fabrics = new Map<string, { map: THREE.Texture; tiles: number }[]>();
  const loader = new THREE.TextureLoader();
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
        const material = o.material as THREE.MeshStandardMaterial;
        material.map?.dispose();
        material.dispose();
      }
    });
    content.clear();
    links.clear();
    lamps.clear();
    fabrics.clear();
  }

  function build(): void {
    dispose();
    for (const instance of project.instances) {
      const def = project.definitions[instance.definition];
      if (!def) continue;
      const lit = instance.id === selected;
      const drawn = drawnOf(project, instance);
      if (drawn) {
        const group = new THREE.Group();
        group.position.set(...drawn.offset);
        if (drawn.turn) group.quaternion.set(...drawn.turn);
        const link = new THREE.Group();
        group.add(link);
        if (drawn.motion) links.set(instance.id, { group: link, motion: drawn.motion });
        for (const part of drawn.parts) {
          const material = materialOf(part.material, loader);
          if (lit) {
            material.emissive = token("--scene-selected");
            material.emissiveIntensity = 0.6;
          }
          if (!lit && (part.material === "pusherLampGreen" || part.material === "pusherLampRed")) {
            const own = lamps.get(instance.id) ?? { green: [], red: [] };
            own[part.material === "pusherLampGreen" ? "green" : "red"].push(material);
            lamps.set(instance.id, own);
          }
          if (part.scroll && material.map) {
            fabrics.set(instance.id, [...(fabrics.get(instance.id) ?? []), { map: material.map, tiles: part.scroll }]);
          }
          const mesh = toObject(part, material);
          mesh.castShadow = true;
          (part.link ? link : group).add(mesh);
        }
        const placed = new THREE.Group();
        placed.position.set(...drawn.position);
        placed.quaternion.set(...drawn.rotation);
        placed.add(group);
        content.add(placed);
        continue;
      }
      const shape = shapeOf(instance, def);
      const [x, y, z] = shape.size;
      const geometry =
        shape.kind === "box" ? new THREE.BoxGeometry(x, y, z) : new THREE.CylinderGeometry(x / 2, z / 2, y);
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
    cartonMaterial.color = token("--scene-product");
    requestRender();
  });

  $effect(() => {
    void [project, selected];
    for (const a of actuators) {
      const link = links.get(a.id);
      if (!link) continue;
      if ("slide" in link.motion) link.group.position.set(...scaled(link.motion.slide, a.value));
      else link.group.quaternion.set(...yawQuat((a.value * Math.PI) / 180));
      const lit = pusherLamps(a.value);
      for (const [colour, materials] of Object.entries(lamps.get(a.id) ?? {}))
        for (const m of materials) m.emissiveIntensity = lit[colour as "green" | "red"] ? 1 : 0;
    }
    requestRender();
  });

  $effect(() => {
    void [project, selected];
    for (const c of conveyors) {
      for (const fabric of fabrics.get(c.id) ?? []) fabric.map.offset.x = scrollOffset(c.travel, fabric.tiles);
    }
    requestRender();
  });

  $effect(() => {
    const live = new Set(cartons.map((c) => c.id));
    for (const [id, mesh] of meshes) {
      if (!live.has(id)) {
        products.remove(mesh);
        meshes.delete(id);
      }
    }
    for (const c of cartons) {
      let mesh = meshes.get(c.id);
      if (!mesh) {
        mesh = new THREE.Mesh(cartonGeometry, cartonMaterial);
        mesh.castShadow = true;
        meshes.set(c.id, mesh);
        products.add(mesh);
      }
      mesh.position.set(...c.position);
      mesh.quaternion.set(...c.rotation);
    }
    requestRender();
  });

  $effect(() => () => {
    dispose();
    cartonGeometry.dispose();
    cartonMaterial.dispose();
  });
</script>

<Viewport {scene} {theme} {target} halfWidth={view.radius} bind:requestRender />
