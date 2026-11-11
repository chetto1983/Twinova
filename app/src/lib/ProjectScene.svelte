<script lang="ts">
  // The 3D view of a project: each instance as OIP's generator draws its family (families.ts), or as its stand-in
  // shape sized from its parameters (project.ts, shapeOf), at its pose, the selected one highlighted; each moving
  // device's link at its position (a transfer's or a stopper's lift, a pusher's reach, a turntable's angle), a belt's
  // fabric moved by its travel, and the products of the running line at the poses the physics gives them, each a unit
  // box scaled to its size in its paint (engine.rs `Snapshot.actuators`, `Snapshot.conveyors`, `Snapshot.cartons`).
  // Each instance's group carries its id, so the view picks it; the selection carries the gizmo (sceneEditing.ts). While
  // it drags, the join it would make is marked (snapping.ts), and its release makes it; a mounted device dragged off
  // every rail is unmounted where it lands (spec 2026-11-11-snapping-design.md).
  import { untrack } from "svelte";
  import * as THREE from "three";
  import { drawnOf, type Motion } from "./families";
  import { materialOf } from "./materials";
  import { toObject } from "./meshes";
  import { frame, shapeOf, type ProjectView, type Quat, type Vec3 } from "./project";
  import { sceneEditing, type GizmoMode } from "./sceneEditing";
  import { candidate, type Candidate } from "./snapping";
  import { yawQuat } from "./oip/parts/part";
  import { pusherLamps } from "./oip/parts/pusher";
  import { paintOf, scaled, scrollOffset, type Carton, type Snapshot } from "./snapshot";
  import Viewport from "./Viewport.svelte";

  let {
    project,
    selected,
    theme,
    cartons,
    actuators,
    conveyors,
    framing = 0,
    mode = "translate",
    dropping = null,
    snapping = true,
    onSelect = () => {},
    onMove = () => {},
    onDrop = () => {},
    onSnap = () => {},
    onUnmount = () => {},
    onCandidate = () => {},
  }: {
    project: ProjectView;
    selected: string | null;
    theme: string;
    cartons: Carton[];
    actuators: Snapshot["actuators"];
    conveyors: Snapshot["conveyors"];
    /** Changes when the view should frame the scene again: a project opened, not an edit. */
    framing?: number;
    mode?: GizmoMode;
    dropping?: string | null;
    onSelect?: (id: string | null) => void;
    onMove?: (id: string, position: Vec3, rotation: Quat) => void;
    onDrop?: (family: string, position: Vec3) => void;
    /** Snap while dragging. */
    snapping?: boolean;
    /** A drag released on a join. */
    onSnap?: (c: Candidate) => void;
    /** A mounted device released off every rail. */
    onUnmount?: (id: string, position: Vec3, rotation: Quat) => void;
    /** The join a drag would make now, or none. */
    onCandidate?: (c: Candidate | null) => void;
  } = $props();

  const scene = new THREE.Scene();
  const content = new THREE.Group();
  const products = new THREE.Group();
  scene.add(content, products);
  /** Each instance's placed group, by id: what the gizmo moves. */
  const placedGroups = new Map<string, THREE.Group>();
  /** Where a mounted device stands in its host's group, which the gizmo moves (families.ts `offset`). */
  const deviceOffsets = new Map<string, Vec3>();
  /** The join the drag would make, held until it lets go (snapping.ts `RELEASE`). */
  let held: Candidate | null = null;
  const marker = new THREE.Mesh(
    new THREE.SphereGeometry(0.06, 16, 12),
    new THREE.MeshBasicMaterial({ depthTest: false, transparent: true, opacity: 0.85 }),
  );
  marker.renderOrder = 1;
  marker.visible = false;
  scene.add(marker);

  /** Where `id` stands for snapping at the gizmo's pose: the instance's own pose, or a mounted device's place. */
  function standing(id: string, position: Vec3, rotation: Quat): Vec3 {
    const offset = deviceOffsets.get(id);
    if (!offset) return position;
    const at = new THREE.Vector3(offset[0], 0, offset[2]).applyQuaternion(new THREE.Quaternion(...rotation));
    return [position[0] + at.x, position[1] + at.y, position[2] + at.z];
  }

  function show(c: Candidate | null): void {
    held = c;
    marker.visible = c !== null;
    if (c) marker.position.set(...c.point);
    onCandidate(c);
    requestRender();
  }

  const editing = sceneEditing(scene, content, {
    onSelect: (id) => onSelect(id),
    onDrag: (id, position, rotation, free) => {
      const at = standing(id, position, rotation);
      show(snapping && !free ? candidate(project, id, at, rotation, held) : null);
    },
    onMove: (id, position, rotation, free) => {
      const at = standing(id, position, rotation);
      const c = snapping && !free ? candidate(project, id, at, rotation, held) : null;
      show(null);
      if (c) onSnap(c);
      else if (deviceOffsets.has(id)) onUnmount(id, at, rotation);
      else onMove(id, position, rotation);
    },
    dropping: () => dropping,
    onDrop: (family, position) => onDrop(family, position),
  });
  const cartonGeometry = new THREE.BoxGeometry(1, 1, 1);
  const cartonMaterial = new THREE.MeshStandardMaterial({ roughness: 0.9 });
  /** One material per paint (paintOf), the unpainted one the theme's. */
  const paints = new Map<string, THREE.MeshStandardMaterial>([["unpainted", cartonMaterial]]);

  function paint(color: Carton["color"]): THREE.MeshStandardMaterial {
    const key = paintOf(color);
    let material = paints.get(key);
    if (!material) {
      material = new THREE.MeshStandardMaterial({ roughness: 0.9, color: new THREE.Color(key) });
      paints.set(key, material);
    }
    return material;
  }
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
  /** "Frame all" asks for the scene as it is now; an edit does not move the camera. */
  let reframes = $state(0);
  const view = $derived.by(() => {
    void [framing, reframes];
    return untrack(() => frame(project.instances));
  });
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
    placedGroups.clear();
    deviceOffsets.clear();
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
      if (drawn && project.mounts.some((m) => m.device === instance.id)) deviceOffsets.set(instance.id, drawn.offset);
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
        place(instance.id, placed);
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
      place(instance.id, placed);
    }
    marker.material.color = token("--scene-selected");
    editing.attach(selected ? (placedGroups.get(selected) ?? null) : null);
  }

  /** Adds an instance's group to the view, named for picking. A mounted device is drawn from its host's pose, so its
   *  group is the host's frame: picking it selects the device, and the gizmo leaves it alone. */
  function place(id: string, placed: THREE.Group): void {
    placed.userData.instance = id;
    placedGroups.set(id, placed);
    content.add(placed);
  }

  $effect(() => {
    editing.setMode(mode);
  });

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
        mesh = new THREE.Mesh(cartonGeometry, paint(c.color));
        mesh.scale.set(...c.size);
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
    marker.geometry.dispose();
    marker.material.dispose();
    for (const material of paints.values()) material.dispose();
  });
</script>

<Viewport
  {scene}
  {theme}
  {target}
  halfWidth={view.radius}
  bind:requestRender
  interact={editing.interact}
  onFrameAll={() => (reframes += 1)}
/>
