// Editing in the 3D view (spec 2026-11-11-editor-basics-design.md, "The view"): a click selects the instance under the
// cursor, or clears the selection on empty floor; three's TransformControls moves the selection on the floor or turns it
// about +Y in steps of 15°, one edit when the drag ends; a family dragged from the catalogue lands where it is released.
// While a drag lasts, each pose is reported, and whether Alt is held to place freely (spec 2026-11-11-snapping-design.md).
// TransformControls emits `dragging-changed` (three 0.187, examples/jsm/controls/TransformControls.js, its
// "propertyname-changed" events), which hands the pointer from the orbit controls to the gizmo and back.
import * as THREE from "three";
import { TransformControls } from "three/examples/jsm/controls/TransformControls.js";
import { isClick, picked, pose, type Interaction } from "./picking";
import type { Quat, Vec3 } from "./project";

/** OIP's default rotation step (src/core/types/base.ts `DEFAULT_ROTATE_SNAP_DEG`). */
const ROTATE_STEP = THREE.MathUtils.degToRad(15);
const FLOOR = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);

export type GizmoMode = "translate" | "rotate";

/** The render request until the viewport hands its own. */
const nothing = (): void => {};

export interface EditingHandlers {
  /** The instance clicked, or null on empty floor. */
  onSelect: (id: string | null) => void;
  /** The gizmo moved the instance to this pose; `free` while Alt is held. */
  onDrag: (id: string, position: Vec3, rotation: Quat, free: boolean) => void;
  /** A gizmo drag ended: the instance's new pose. */
  onMove: (id: string, position: Vec3, rotation: Quat, free: boolean) => void;
  /** The catalogue family being dragged, if any. */
  dropping: () => string | null;
  /** A dragged family released over the floor at `position`. */
  onDrop: (family: string, position: Vec3) => void;
}

export interface Editing {
  interact: (view: Interaction) => () => void;
  /** Puts the gizmo on an instance's group, or takes it away. */
  attach: (group: THREE.Object3D | null) => void;
  setMode: (mode: GizmoMode) => void;
}

export function sceneEditing(scene: THREE.Scene, content: THREE.Object3D, handlers: EditingHandlers): Editing {
  let gizmo: TransformControls | null = null;
  let target: THREE.Object3D | null = null;
  let mode: GizmoMode = "translate";
  let render: () => void = nothing;

  function showMode(): void {
    if (!gizmo) return;
    gizmo.setMode(mode);
    // On the floor: x and z only; turning: about +Y only.
    gizmo.showX = mode === "translate";
    gizmo.showZ = mode === "translate";
    gizmo.showY = mode === "rotate";
    render();
  }

  function attach(group: THREE.Object3D | null): void {
    target = group;
    if (!gizmo) return;
    if (group) gizmo.attach(group);
    else gizmo.detach();
    render();
  }

  function interact({ camera, canvas, controls, requestRender }: Interaction): () => void {
    render = requestRender;
    const controlsOfGizmo = new TransformControls(camera, canvas);
    gizmo = controlsOfGizmo;
    controlsOfGizmo.setRotationSnap(ROTATE_STEP);
    const helper = controlsOfGizmo.getHelper();
    scene.add(helper);
    let usedGizmo = false;
    let free = false;
    const onAlt = (event: KeyboardEvent): void => {
      free = event.altKey;
    };
    const posed = (report: EditingHandlers["onMove"]): void => {
      const object = controlsOfGizmo.object;
      const id = object?.userData.instance;
      if (!object || typeof id !== "string") return;
      const placed = pose(object.position.toArray(), object.quaternion.toArray());
      report(id, placed.position, placed.rotation, free);
    };
    controlsOfGizmo.addEventListener("change", requestRender);
    controlsOfGizmo.addEventListener("objectChange", () => posed(handlers.onDrag));
    controlsOfGizmo.addEventListener("dragging-changed", (event) => {
      const dragging = Boolean(event.value);
      controls.enabled = !dragging;
      if (dragging) {
        usedGizmo = true;
        return;
      }
      posed(handlers.onMove);
    });
    const raycaster = new THREE.Raycaster();
    const pointer = new THREE.Vector2();
    let down = { x: 0, y: 0 };
    const aim = (event: PointerEvent): void => {
      const rect = canvas.getBoundingClientRect();
      pointer.set(
        ((event.clientX - rect.left) / rect.width) * 2 - 1,
        -((event.clientY - rect.top) / rect.height) * 2 + 1,
      );
      raycaster.setFromCamera(pointer, camera);
    };
    const onDown = (event: PointerEvent): void => {
      down = { x: event.clientX, y: event.clientY };
    };
    const onUp = (event: PointerEvent): void => {
      aim(event);
      const family = handlers.dropping();
      if (family) {
        const at = raycaster.ray.intersectPlane(FLOOR, new THREE.Vector3());
        if (at) handlers.onDrop(family, pose(at.toArray(), [0, 0, 0, 1]).position);
        return;
      }
      if (usedGizmo) {
        usedGizmo = false;
        return;
      }
      if (isClick(down, { x: event.clientX, y: event.clientY })) {
        handlers.onSelect(picked(raycaster.intersectObject(content, true)));
      }
    };
    canvas.addEventListener("pointerdown", onDown);
    canvas.addEventListener("pointerup", onUp);
    window.addEventListener("keydown", onAlt);
    window.addEventListener("keyup", onAlt);
    showMode();
    attach(target);
    return () => {
      canvas.removeEventListener("pointerdown", onDown);
      canvas.removeEventListener("pointerup", onUp);
      window.removeEventListener("keydown", onAlt);
      window.removeEventListener("keyup", onAlt);
      scene.remove(helper);
      controlsOfGizmo.dispose();
      gizmo = null;
    };
  }

  return {
    interact,
    attach,
    setMode: (next) => {
      mode = next;
      showMode();
    },
  };
}
