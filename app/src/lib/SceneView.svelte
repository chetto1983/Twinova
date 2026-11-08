<script lang="ts">
  // The 3D view of the belt. It only renders the backend snapshot: no physics here.
  import * as THREE from "three";
  import { BELT_LENGTH_M, PART_LENGTH_M, PHOTOCELL_AT_M, partCentreX, type Snapshot } from "./snapshot";
  import Viewport from "./Viewport.svelte";

  let { snapshot, theme }: { snapshot: Snapshot | null; theme: string } = $props();

  const BELT_TOP = 0.8;
  const BELT_WIDTH = 0.6;
  const PART_SIZE = 0.25;

  const scene = new THREE.Scene();
  const TARGET = new THREE.Vector3(0, BELT_TOP, 0);
  const materials = {
    belt: new THREE.MeshStandardMaterial({ roughness: 0.9 }),
    frame: new THREE.MeshStandardMaterial({ roughness: 0.6, metalness: 0.4 }),
    part: new THREE.MeshStandardMaterial({ roughness: 0.7 }),
    beam: new THREE.MeshStandardMaterial({ emissiveIntensity: 1 }),
  };
  const parts = new Map<number, THREE.Mesh>();
  const partGeometry = new THREE.BoxGeometry(PART_LENGTH_M, PART_SIZE, PART_SIZE * 1.4);
  let beam: THREE.Mesh;
  let requestRender = $state(() => {});

  /** Colours only from the Aura tokens (CLAUDE.md), read again when the theme changes. */
  function applyTokens(): void {
    const css = getComputedStyle(document.documentElement);
    const token = (name: string) => new THREE.Color(css.getPropertyValue(name).trim() || "#888");
    materials.frame.color = token("--scene-frame");
    materials.belt.color = token("--scene-belt");
    materials.part.color = token("--scene-product");
    paintBeam();
  }

  function paintBeam(): void {
    const css = getComputedStyle(document.documentElement);
    const lit = snapshot?.sensor ?? false;
    const colour = new THREE.Color(css.getPropertyValue(lit ? "--color-success" : "--color-danger").trim() || "#888");
    materials.beam.color = colour;
    materials.beam.emissive = colour;
    materials.beam.emissiveIntensity = lit ? 1.2 : 0.25;
  }

  function addStatic(): void {
    const belt = new THREE.Mesh(new THREE.BoxGeometry(BELT_LENGTH_M, 0.08, BELT_WIDTH), materials.belt);
    belt.position.y = BELT_TOP - 0.04;
    scene.add(belt);
    for (const x of [-1, 1]) {
      for (const z of [-1, 1]) {
        const leg = new THREE.Mesh(new THREE.BoxGeometry(0.06, BELT_TOP - 0.08, 0.06), materials.frame);
        leg.position.set((x * (BELT_LENGTH_M - 0.2)) / 2, (BELT_TOP - 0.08) / 2, (z * (BELT_WIDTH - 0.06)) / 2);
        scene.add(leg);
      }
    }
    const sensorX = PHOTOCELL_AT_M - BELT_LENGTH_M / 2;
    for (const z of [-1, 1]) {
      const post = new THREE.Mesh(new THREE.BoxGeometry(0.05, 0.4, 0.05), materials.frame);
      post.position.set(sensorX, BELT_TOP + 0.2, (z * (BELT_WIDTH + 0.1)) / 2);
      scene.add(post);
    }
    beam = new THREE.Mesh(new THREE.CylinderGeometry(0.008, 0.008, BELT_WIDTH + 0.1), materials.beam);
    beam.rotation.x = Math.PI / 2;
    beam.position.set(sensorX, BELT_TOP + PART_SIZE / 2, 0);
    scene.add(beam);
  }

  function syncParts(s: Snapshot): void {
    const alive = new Set(s.parts.map((p) => p.id));
    for (const [id, mesh] of parts) {
      if (!alive.has(id)) {
        scene.remove(mesh);
        parts.delete(id);
      }
    }
    for (const p of s.parts) {
      let mesh = parts.get(p.id);
      if (!mesh) {
        mesh = new THREE.Mesh(partGeometry, materials.part);
        mesh.castShadow = true;
        mesh.position.y = BELT_TOP + PART_SIZE / 2;
        scene.add(mesh);
        parts.set(p.id, mesh);
      }
      mesh.position.x = partCentreX(p.front_m);
    }
  }

  $effect(() => {
    if (snapshot) syncParts(snapshot);
    paintBeam();
    requestRender();
  });

  $effect(() => {
    void theme;
    applyTokens();
    requestRender();
  });

  addStatic();
  scene.traverse((o) => (o.castShadow = o instanceof THREE.Mesh));
</script>

<!-- Fit the belt plus a margin whatever the panel's shape. -->
<Viewport {scene} {theme} target={TARGET} halfWidth={BELT_LENGTH_M / 2 + 0.4} bind:requestRender />
