<script lang="ts">
  // A WebGL view of a three.js scene with orbit controls, shared by the M3 demo and the project view. It adds the
  // lights, frames `target` whatever the panel's shape, and renders on demand (three.js manual, "Rendering on
  // demand"): a frame per change, none while nothing moves.
  import { onMount } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { framingDistance } from "./snapshot";

  let {
    scene,
    target,
    halfWidth,
    halfHeight = 1,
    requestRender = $bindable(() => {}),
  }: {
    scene: THREE.Scene;
    target: THREE.Vector3;
    halfWidth: number;
    halfHeight?: number;
    requestRender?: () => void;
  } = $props();

  let host: HTMLDivElement;
  const camera = new THREE.PerspectiveCamera(40, 1, 0.1, 200);
  /** Viewing direction from the target: front, a little to the right and above. */
  const VIEW_DIRECTION = new THREE.Vector3(0.35, 0.6, 1).normalize();

  onMount(() => {
    const lights = [new THREE.HemisphereLight(0xffffff, 0x444444, 2.2), new THREE.DirectionalLight(0xffffff, 1.6)];
    lights[1].position.set(3, 6, 4);
    scene.add(...lights);
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    host.appendChild(renderer.domElement);
    const controls = new OrbitControls(camera, renderer.domElement);
    let frame = 0;
    requestRender = () => {
      frame ||= requestAnimationFrame(() => {
        frame = 0;
        renderer.render(scene, camera);
      });
    };
    controls.addEventListener("change", requestRender);
    const resize = () => {
      const { clientWidth: w, clientHeight: h } = host;
      renderer.setSize(w, h);
      camera.aspect = w / Math.max(h, 1);
      camera.updateProjectionMatrix();
      controls.target.copy(target);
      const distance = framingDistance(halfWidth, halfHeight, camera.fov, camera.aspect);
      camera.position.copy(target).addScaledVector(VIEW_DIRECTION, distance);
      controls.update();
      requestRender();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(host);
    return () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
      controls.dispose();
      renderer.dispose();
      scene.remove(...lights);
    };
  });
</script>

<div class="view" bind:this={host}></div>

<style>
  .view {
    width: 100%;
    height: 100%;
    min-height: 320px;
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
</style>
