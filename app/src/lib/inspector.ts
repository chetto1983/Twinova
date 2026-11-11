// The editable inspector's pure parts (spec 2026-11-11-editor-basics-design.md, "The view"): a parameter's value from the
// text typed for it and back, the heading about +Y, and the flow ports and mount an instance can be given. The format
// checks ranges, addresses and ports again at Build.
import {
  formatTime,
  type Endpoint,
  type Instance,
  type ParameterDef,
  type ProjectView,
  type Quat,
  type Value,
} from "./project";

const TIME_UNITS: Record<string, number> = { s: 1e9, ms: 1e6, µs: 1e3, us: 1e3, ns: 1 };

/** A duration typed as `2 s`, `300 ms`, `5 µs` or nanoseconds alone, in nanoseconds; null if it is none of those. */
export function parseTime(text: string): number | null {
  const match = /^\s*(\d+(?:[.,]\d+)?)\s*(s|ms|µs|us|ns)?\s*$/.exec(text);
  if (!match) return null;
  const ns = Number(match[1].replace(",", ".")) * TIME_UNITS[match[2] ?? "ns"];
  return Number.isInteger(ns) ? ns : null;
}

const number = (text: string): number | null => {
  const n = Number(text.trim().replace(",", "."));
  return text.trim() !== "" && Number.isFinite(n) ? n : null;
};

const items = (text: string): string[] =>
  text
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);

/** The value typed for a parameter: null takes the default back (an empty field), undefined is not a value of its
 *  type. A BOOL is a checkbox, never typed. */
export function parseValue(def: ParameterDef, text: string): Value | null | undefined {
  if (text.trim() === "") return null;
  switch (def.type) {
    case "REAL":
      return number(text) ?? undefined;
    case "DINT": {
      const n = number(text);
      return n !== null && Number.isInteger(n) ? n : undefined;
    }
    case "TIME":
      return parseTime(text) ?? undefined;
    case "DINT_LIST": {
      const list = items(text).map((item) => number(item));
      return list.every((n) => n !== null && Number.isInteger(n)) ? (list as number[]) : undefined;
    }
    case "COLOR_LIST": {
      const list = items(text).map((item) => item.toLowerCase());
      return list.every((c) => /^#[0-9a-f]{6}$/.test(c)) ? list : undefined;
    }
    case "BOOL":
      return undefined;
  }
}

/** What a parameter's field shows for its value: a duration in its unit, a list as its items. */
export function valueText(def: ParameterDef, value: Value): string {
  if (Array.isArray(value)) return value.join(", ");
  if (def.type === "TIME") return formatTime(Number(value));
  return String(value);
}

/** The heading about +Y of a rotation, in degrees, rounded to 0.1°: the inspector's field. */
export function headingOf([x, y, z, w]: Quat): number {
  const angle = Math.atan2(2 * (w * y + x * z), 1 - 2 * (y * y + x * x));
  return Math.round(((angle * 180) / Math.PI) * 10) / 10 || 0;
}

/** A turn of `degrees` about +Y. */
export function headingQuat(degrees: number): Quat {
  const half = (degrees * Math.PI) / 360;
  return [0, Math.sin(half), 0, Math.cos(half)];
}

export interface FlowPort {
  port: string;
  kind: "flow_in" | "flow_out";
  /** The port at its other end, if it is connected. */
  peer: Endpoint | null;
  /** The ports of other instances it could connect to: an input to an output, an output to an input. */
  candidates: Endpoint[];
}

const definitionOf = (project: ProjectView, instance: Instance) => project.definitions[instance.definition];

/** Each flow port of `instance`, with its peer and what it could connect to. */
export function flowPorts(project: ProjectView, instance: Instance): FlowPort[] {
  const ports = definitionOf(project, instance)?.ports ?? [];
  return ports.flatMap((p) => {
    if (p.kind !== "flow_in" && p.kind !== "flow_out") return [];
    const kind = p.kind;
    const here = (e: Endpoint) => e[0] === instance.id && e[1] === p.name;
    const flow = project.flows.find((f) => here(f.from) || here(f.to));
    const peer = flow ? (here(flow.from) ? flow.to : flow.from) : null;
    const opposite = kind === "flow_in" ? "flow_out" : "flow_in";
    const candidates = project.instances
      .filter((other) => other.id !== instance.id)
      .flatMap((other) =>
        (definitionOf(project, other)?.ports ?? [])
          .filter((q) => q.kind === opposite)
          .map((q): Endpoint => [other.id, q.name]),
      );
    return [{ port: p.name, kind, peer, candidates }];
  });
}

/** Whether `instance` is a device that mounts on a host. */
export const mountsOnAHost = (project: ProjectView, instance: Instance): boolean =>
  (definitionOf(project, instance)?.ports ?? []).some((p) => p.kind === "mount_device");

/** The instances a device can be mounted on: those with a rail. */
export function hostsFor(project: ProjectView, device: Instance): string[] {
  return project.instances
    .filter((i) => i.id !== device.id)
    .filter((i) => (definitionOf(project, i)?.ports ?? []).some((p) => p.kind === "mount_host"))
    .map((i) => i.id);
}

/** Where a device is mounted, if it is. */
export function mountOf(project: ProjectView, device: string): { host: string; at: number } | null {
  const mount = project.mounts.find((m) => m.device === device);
  return mount ? { host: mount.host, at: mount.at } : null;
}
