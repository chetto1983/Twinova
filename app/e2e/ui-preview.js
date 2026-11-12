// Browser-only visual fixture. Never imported by the application or included in its production entry.
import { mockIPC } from "@tauri-apps/api/mocks";
import scene from "../src-tauri/project/tests/fixtures/valid/demo_m3/scene.json";
import program from "../src-tauri/project/tests/fixtures/valid/demo_m3/plc/Main.st?raw";
import behaviour from "../src-tauri/library/components/twinova.belt@1.st?raw";
const defs = Object.values(
  import.meta.glob("../src-tauri/library/components/*.json", { eager: true, import: "default" }),
);
const project = {
  name: "Conveyor commissioning",
  folder: null,
  instances: scene.instances,
  definitions: Object.fromEntries(defs.map((d) => [`${d.id}@${d.version}`, d])),
  program: [["plc/Main.st", program]],
  behaviours: { "twinova.belt@1": behaviour },
  panel: scene.controller.panel,
  mounts: [{ device: "sensor_a", host: "belt_a", at: 2.75 }],
  flows: scene.connections.filter((c) => c.kind === "flow"),
  merges: [],
  interfaces: {},
  notice: null,
  can_undo: false,
  can_redo: false,
  changed_outside: [],
  recovered: false,
};
const snapshot = {
  time_ms: 0,
  cartons: [],
  sinks: [{ id: "sink", delivered: 0, blocked: false }],
  actuators: [],
  conveyors: [],
  signals: [
    { name: "belt_a.run", address: "%QX0.0", value: "FALSE" },
    { name: "belt_a.running", address: "%IX0.1", value: "FALSE" },
  ],
  jam: false,
  lag_ms: 0,
};
mockIPC(
  (cmd, args) => {
    if (cmd === "open_launch_project") return null;
    if (cmd === "current_project") return structuredClone(project);
    if (cmd === "build") return { snapshot, warnings: [] };
    if (cmd === "scan_timing") return { scans: 0, last_us: 0, average_us: 0, maximum_us: 0, interval_ms: 10 };
    if (cmd === "catalogue")
      return defs.map((d) => ({
        key: `${d.id}@${d.version}`,
        group: /source|sink/.test(d.id)
          ? "flow"
          : /sensor|scanner/.test(d.id)
            ? "sensors"
            : /belt|roller|turntable/.test(d.id)
              ? "conveyors"
              : "devices",
      }));
    if (cmd === "edit") {
      const op = args.op;
      const instance = project.instances.find((i) => i.id === op.id);
      if (op.op === "rename") instance.name = op.name;
      if (op.op === "set_parameter") instance.parameters[op.name] = op.value;
      if (op.op === "move") {
        instance.position = op.position;
        instance.rotation = op.rotation;
      }
      project.can_undo = true;
      return structuredClone(project);
    }
    if (cmd === "reset" || cmd === "step") return snapshot;
    return undefined;
  },
  { shouldMockEvents: true },
);
localStorage.setItem("locale", "en");
await import("../src/main");
