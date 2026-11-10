// The read-only view of an opened project (src-tauri/src/project_commands.rs, ProjectView) and the pure functions the
// scene tree, the inspector and the 3D view need. Format: docs/superpowers/specs/2026-11-08-project-format-design.md.

export type Vec3 = [number, number, number];
export type Quat = [number, number, number, number];
type Value = boolean | number;

export interface Instance {
  id: string;
  name: string;
  definition: string;
  position: Vec3;
  rotation: Quat;
  parameters: Record<string, Value>;
  signals: Record<string, string>;
}

export interface ParameterDef {
  name: string;
  type: "REAL" | "DINT" | "BOOL" | "TIME";
  unit: string | null;
  default: Value;
  min: number | null;
  max: number | null;
}

export interface SignalDef {
  name: string;
  type: "BOOL" | "DINT" | "REAL";
  direction: "plc_input" | "plc_output";
}

export interface Definition {
  id: string;
  version: number;
  generator: string;
  parameters: ParameterDef[];
  signals: SignalDef[];
}

export interface ProjectView {
  name: string;
  folder: string | null;
  instances: Instance[];
  definitions: Record<string, Definition>;
  changed_outside: string[];
  recovered: boolean;
  /** The line program's sources, [path, text]. */
  program: [string, string][];
  /** The behaviour block of each definition that has one, by `id@version`. */
  behaviours: Record<string, string>;
  panel: { name: string; address: string }[];
  /** Each mounted device's host and position along it (project_commands.rs `MountView`). */
  mounts: { device: string; host: string; at: number }[];
}

export interface ParameterRow {
  name: string;
  text: string;
  overridden: boolean;
}

export interface SignalRow {
  name: string;
  type: SignalDef["type"];
  direction: SignalDef["direction"];
  /** null when the signal is not bound yet. */
  address: string | null;
}

/** The family a definition id names, without its namespace: `twinova.belt` is `belt`. */
export function family(definitionId: string): string {
  return definitionId.slice(definitionId.lastIndexOf(".") + 1);
}

/** A TIME is stored in nanoseconds; it reads best in the largest unit that keeps it whole. */
export function formatTime(ns: number): string {
  for (const [unit, size] of [
    ["s", 1e9],
    ["ms", 1e6],
    ["µs", 1e3],
  ] as const) {
    if (ns !== 0 && ns % size === 0) return `${ns / size} ${unit}`;
  }
  return `${ns} ns`;
}

export function formatValue(def: ParameterDef, value: Value): string {
  if (def.type === "BOOL") return value ? "TRUE" : "FALSE";
  if (def.type === "TIME") return formatTime(Number(value));
  return def.unit ? `${value} ${def.unit}` : String(value);
}

/** Every parameter of the definition, with the instance's override where it has one. */
export function parameterRows(instance: Instance, def: Definition): ParameterRow[] {
  return def.parameters.map((p) => {
    const overridden = Object.hasOwn(instance.parameters, p.name);
    const value = overridden ? instance.parameters[p.name] : p.default;
    return { name: p.name, text: formatValue(p, value), overridden };
  });
}

export function signalRows(instance: Instance, def: Definition): SignalRow[] {
  return def.signals.map((s) => ({
    name: s.name,
    type: s.type,
    direction: s.direction,
    address: instance.signals[s.name] ?? null,
  }));
}

/**
 * Euler angles in degrees about X, Y and Z of a unit quaternion, decomposed in the order Y, X, Z, rounded to 0.1°, for
 * display only. With Y up, turning a part on the floor is a rotation about Y; the order YXZ (three.js's for Y-up scenes)
 * keeps that case regular and puts the singularity at ±90° about X, which floor layouts rarely use.
 */
const deg = (radians: number) => Math.round((radians * 1800) / Math.PI) / 10 || 0;

export function eulerDegrees([x, y, z, w]: Quat): Vec3 {
  const m11 = 1 - 2 * (y * y + z * z);
  const m13 = 2 * (x * z + y * w);
  const m21 = 2 * (x * y + z * w);
  const m22 = 1 - 2 * (x * x + z * z);
  const m23 = 2 * (y * z - x * w);
  const m31 = 2 * (x * z - y * w);
  const m33 = 1 - 2 * (x * x + y * y);
  const pitch = Math.asin(-Math.max(-1, Math.min(1, m23)));
  // Stryker disable next-line EqualityOperator: `<=` differs only at |m23| = 0.9999999 exactly, which no test can hit.
  if (Math.abs(m23) < 0.9999999) {
    return [deg(pitch), deg(Math.atan2(m13, m33)), deg(Math.atan2(m21, m22))];
  }
  return [deg(pitch), deg(Math.atan2(-m31, m11)), 0];
}

/** A simple stand-in for a family's geometry: a box or a cylinder, in the instance's local frame. */
export interface Shape {
  kind: "box" | "cylinder";
  size: Vec3;
  centre: Vec3;
  /** A scene colour token (app.css, `--scene-*`). */
  token: string;
}

const DECK_M = 0.08;
/** OIP's chain-transfer deck: 0.06 m deep (examples/parts/chain-transfer/rig.json at 1e7488c). */
const CHAIN_DECK_M = 0.06;

export function real(instance: Instance, def: Definition, name: string, fallback: number): number {
  const p = def.parameters.find((q) => q.name === name);
  const value = Object.hasOwn(instance.parameters, name) ? instance.parameters[name] : p?.default;
  return typeof value === "number" ? value : fallback;
}

/** The first generator per family (spec decision: native, versioned generators): sizes come from the parameters. */
export function shapeOf(instance: Instance, def: Definition): Shape {
  const length = real(instance, def, "length", 1);
  const width = real(instance, def, "width", 0.6);
  const top = real(instance, def, "height", 0.8);
  switch (def.generator) {
    case "twinova.belt/1":
    case "twinova.roller/1":
      return {
        kind: "box",
        size: [length, DECK_M, width],
        centre: [length / 2, top - DECK_M / 2, 0],
        token: "--scene-belt",
      };
    case "twinova.chain_transfer/1": {
      // Its deck sits 3 cm under its host's rollers (OIP's chain transfer); the host's height is the default top.
      const along = real(instance, def, "chains", 2) * real(instance, def, "distance", 0.33);
      const across = real(instance, def, "lane_length", 0.6);
      return {
        kind: "box",
        size: [along, CHAIN_DECK_M, across],
        centre: [0, top - 0.03 - CHAIN_DECK_M / 2, 0],
        token: "--scene-device",
      };
    }
    case "twinova.sensor_diffuse/1":
      return { kind: "cylinder", size: [0.06, 0.3, 0.06], centre: [0, top + 0.15, 0], token: "--scene-sensor" };
    case "twinova.source/1":
      return { kind: "box", size: [0.4, 0.4, 0.4], centre: [0, 0.2, 0], token: "--scene-device" };
    case "twinova.sink/1":
      return { kind: "box", size: [0.4, 0.4, 0.4], centre: [0, 0.2, 0], token: "--scene-sink" };
    default:
      return { kind: "box", size: [0.3, 0.3, 0.3], centre: [0, 0.15, 0], token: "--color-warning" };
  }
}

/** The centre and the half-extent of the scene's footprint, to frame it in the view. */
export function frame(instances: Instance[]): { centre: Vec3; radius: number } {
  if (instances.length === 0) return { centre: [0, 0, 0], radius: 1 };
  const xs = instances.map((i) => i.position[0]);
  const zs = instances.map((i) => i.position[2]);
  const [x0, x1, z0, z1] = [Math.min(...xs), Math.max(...xs), Math.min(...zs), Math.max(...zs)];
  return { centre: [(x0 + x1) / 2, 0.8, (z0 + z1) / 2], radius: Math.max(x1 - x0, z1 - z0) / 2 + 1 };
}
