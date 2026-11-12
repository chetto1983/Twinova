<script lang="ts">
  // A WebGL view of a three.js scene with orbit controls, shared by the M3 demo and the project view. It lays out an
  // industrial hall after Open Industry Project's (src/render/env.ts, MIT, THIRD_PARTY_NOTICES.md): a hemisphere light
  // with a cool sky and a warm ground bounce, a warm key light with soft shadows, a cool fill light, a floor with an
  // optional grid, and an axis gizmo. It renders on demand (three.js manual, "Rendering on demand"): a frame per
  // change, none while nothing moves; the shadow map is recomputed only with those frames. A view that is edited
  // (ProjectScene.svelte) installs its picking and its gizmo through `interact`, which receives the camera, the canvas
  // and the orbit controls, and returns its cleanup.
  import { onMount } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { ViewHelper } from "three/examples/jsm/helpers/ViewHelper.js";
  import { m } from "../paraglide/messages.js";
  import { Grid2X2, Scan } from "@lucide/svelte";
  import { warehouse } from "./warehouse";
  import type { Interaction } from "./picking";
  import { framingDistance } from "./snapshot";

  let {
    scene,
    target,
    halfWidth,
    halfHeight = 1.6,
    theme,
    requestRender = $bindable(() => {}),
    interact,
    onFrameAll,
  }: {
    scene: THREE.Scene;
    target: THREE.Vector3;
    halfWidth: number;
    halfHeight?: number;
    theme: string;
    requestRender?: () => void;
    interact?: (view: Interaction) => () => void;
    /** Frames the scene as it is now: a view whose target follows its content asks for a new target, which reframes. */
    onFrameAll?: () => void;
  } = $props();

  let host: HTMLDivElement;
  let grid = $state(false);
  let reframe = () => {};
  const camera = new THREE.PerspectiveCamera(40, 1, 0.1, 500);
  /** Viewing direction from the target: front, a little to the right and above. */
  const VIEW_DIRECTION = new THREE.Vector3(0.35, 0.6, 1).normalize();

  const hall = new THREE.Group();
  const hemi = new THREE.HemisphereLight(0xffffff, 0xffffff, 1.15);
  const key = new THREE.DirectionalLight(0xffffff, 2.1);
  const fill = new THREE.DirectionalLight(0xffffff, 1.0);
  const floorMaterial = new THREE.MeshStandardMaterial({ roughness: 0.95 });
  const floor = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), floorMaterial);
  let gridHelper: THREE.GridHelper | null = null;
  let building: ReturnType<typeof warehouse> | null = null;
  key.position.set(-8.1, 16.4, 8.1);
  fill.position.set(10, 14, -10);
  key.castShadow = true;
  key.shadow.mapSize.set(2048, 2048);
  key.shadow.bias = -0.0004;
  key.shadow.normalBias = 0.02;
  key.shadow.radius = 4;
  floor.rotation.x = -Math.PI / 2;
  floor.receiveShadow = true;
  hall.add(hemi, key, key.target, fill, fill.target, floor);

  function token(name: string): THREE.Color {
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return new THREE.Color(value || "#888");
  }

  /** Sizes the floor, the grid and the key light's shadow frustum to the scene around `target`. */
  function layoutHall(): void {
    const extent = Math.max(halfWidth * 4, 12);
    if (building) {
      hall.remove(building.group);
      building.dispose();
    }
    building = warehouse(extent);
    building.group.position.set(target.x, 0, target.z);
    hall.add(building.group);
    floor.scale.set(extent, extent, 1);
    floor.position.set(target.x, 0, target.z);
    key.target.position.set(target.x, 0, target.z);
    key.position.set(target.x - 8.1, 16.4, target.z + 8.1);
    fill.target.position.set(target.x, 0, target.z);
    const shadow = key.shadow.camera;
    const half = halfWidth + 2;
    [shadow.left, shadow.right, shadow.top, shadow.bottom] = [-half, half, half, -half];
    shadow.near = 0.5;
    shadow.far = 60;
    shadow.updateProjectionMatrix();
  }

  function paintHall(): void {
    scene.background = token("--scene-sky");
    hemi.color = token("--scene-hemi-sky");
    hemi.groundColor = token("--scene-hemi-ground");
    key.color = token("--scene-key-light");
    fill.color = token("--scene-fill-light");
    floorMaterial.color = token("--scene-floor");
    if (gridHelper) {
      hall.remove(gridHelper);
      gridHelper.dispose();
      gridHelper = null;
    }
    if (grid) {
      const size = Math.ceil(Math.max(halfWidth * 4, 10));
      gridHelper = new THREE.GridHelper(size, size, token("--scene-grid"), token("--scene-grid"));
      const material = gridHelper.material as THREE.Material;
      material.transparent = true;
      material.opacity = 0.35;
      material.depthWrite = false;
      gridHelper.position.set(target.x, 0.002, target.z);
      hall.add(gridHelper);
    }
  }

  $effect(() => {
    void [halfWidth, target];
    layoutHall();
    reframe();
    requestRender();
  });
  $effect(() => {
    void [theme, grid, halfWidth, target];
    paintHall();
    requestRender();
  });

  onMount(() => {
    scene.add(hall);
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.autoClear = false;
    host.appendChild(renderer.domElement);
    const controls = new OrbitControls(camera, renderer.domElement);
    const gizmo = new ViewHelper(camera, renderer.domElement);
    gizmo.location = { top: 8, right: 8, bottom: 0, left: null };
    const clock = new THREE.Clock();
    let frame = 0;
    requestRender = () => {
      frame ||= requestAnimationFrame(() => {
        frame = 0;
        const delta = clock.getDelta();
        if (gizmo.animating) {
          gizmo.update(delta);
          requestRender();
        }
        renderer.clear();
        renderer.render(scene, camera);
        gizmo.render(renderer);
      });
    };
    controls.addEventListener("change", requestRender);
    renderer.domElement.addEventListener("pointerup", (event) => {
      gizmo.center.copy(controls.target);
      if (gizmo.handleClick(event)) {
        clock.getDelta();
        requestRender();
      }
    });
    const resize = () => {
      const { clientWidth: w, clientHeight: h } = host;
      renderer.setSize(w, h);
      camera.aspect = w / Math.max(h, 1);
      camera.updateProjectionMatrix();
      requestRender();
    };
    reframe = () => {
      resize();
      controls.target.copy(target);
      const distance = framingDistance(halfWidth, halfHeight, camera.fov, camera.aspect);
      camera.position.copy(target).addScaledVector(VIEW_DIRECTION, distance);
      controls.update();
      requestRender();
    };
    reframe();
    const observer = new ResizeObserver(resize);
    observer.observe(host);
    const release = interact?.({ camera, canvas: renderer.domElement, controls, requestRender });
    return () => {
      release?.();
      observer.disconnect();
      cancelAnimationFrame(frame);
      gizmo.dispose();
      controls.dispose();
      renderer.dispose();
      building?.dispose();
      floor.geometry.dispose();
      floorMaterial.dispose();
      gridHelper?.dispose();
      key.dispose();
      scene.remove(hall);
    };
  });
</script>

<div class="view" bind:this={host}>
  <div class="tools">
    <button
      class="icon-button"
      title={m.view_grid()}
      aria-label={m.view_grid()}
      aria-pressed={grid}
      onclick={() => (grid = !grid)}><Grid2X2 size={16} /></button
    >
    <button
      class="icon-button"
      title={m.view_frame()}
      aria-label={m.view_frame()}
      onclick={() => (onFrameAll ? onFrameAll() : reframe())}><Scan size={16} /></button
    >
  </div>
</div>

<style>
  .view {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }
  .tools {
    position: absolute;
    top: 12px;
    left: 82px;
    display: flex;
    gap: 2px;
    padding: 3px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }
  .tools .icon-button {
    background: color-mix(in oklab, var(--color-surface) 80%, transparent);
  }
  .tools .icon-button[aria-pressed="true"] {
    background: var(--color-accent);
    color: var(--color-on-accent);
  }
</style>
