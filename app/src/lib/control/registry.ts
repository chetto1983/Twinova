// Everything the window does, as commands of one registry (spec 2026-11-23-app-mcp-design.md, decision 2, after
// SolidCraft's): each has a name, a sentence and its parameters as a JSON Schema, and runs what the window's own button
// runs, so a command from the control channel leaves the window as a click would. Components register their commands
// while they are mounted; `listenControl` runs the channel's requests and sends each answer back.
import type { UnlistenFn } from "@tauri-apps/api/event";
import { control } from "../backend";

/** A JSON Schema, as MCP clients read a tool's parameters. */
export type Schema = Record<string, unknown>;

export interface Command {
  name: string;
  description: string;
  params: Schema;
  run: (params: Record<string, unknown>) => Promise<unknown> | unknown;
}

const commands = new Map<string, Command>();

/** Tells the app this window's commands, so it sends their requests here (spec 2026-11-24-open-points-design.md,
 *  decision 3). */
const report = (): void => void control.commands(listed()).catch(() => {});

/** Registers `list`; the function returned takes them away again (a component unmounted). */
export function register(...list: Command[]): () => void {
  for (const c of list) commands.set(c.name, c);
  report();
  return () => {
    list.forEach((c) => commands.get(c.name) === c && commands.delete(c.name));
    report();
  };
}

/** The registry's commands, by name. */
export const listed = (): { name: string; description: string }[] =>
  [...commands.values()]
    .map(({ name, description }) => ({ name, description }))
    .toSorted((a, b) => a.name.localeCompare(b.name));

export function described(name: string): { name: string; description: string; params: Schema } {
  const c = commands.get(name);
  if (!c) throw new Error(`no command ${name}: list_commands names them`);
  return { name: c.name, description: c.description, params: c.params };
}

/** Runs command `method` with `params`: the registry's own two answer what it holds. A parameter the command's schema
 *  does not name, or one it requires and is missing, refuses the command before it runs. */
export async function dispatch(method: string, params: unknown): Promise<unknown> {
  const args = (params ?? {}) as Record<string, unknown>;
  if (method === "list_commands") return listed();
  if (method === "describe_command") return described(String(args.name));
  const c = commands.get(method);
  if (!c) throw new Error(`no command ${method}: list_commands names them`);
  const { properties = {}, required = [] } = c.params as { properties?: Schema; required?: string[] };
  const unknown = Object.keys(args).filter((k) => !(k in properties));
  if (unknown.length > 0)
    throw new Error(`${method} takes no ${unknown.join(", ")}: describe_command gives its parameters`);
  const missing = required.filter((k) => args[k] === undefined);
  if (missing.length > 0) throw new Error(`${method} needs ${missing.join(", ")}`);
  return (await c.run(args)) ?? null;
}

const text = (e: unknown): string => (e instanceof Error ? e.message : String(e));

/** Answers the control channel's requests (control.rs) until the function resolved is called; the channel holds its
 *  requests until this window listens, then hears it is ready. */
export const listenControl = (): Promise<UnlistenFn> =>
  control
    .onRequest(({ id, method, params }) => {
      dispatch(method, params).then(
        (value) => control.reply(id, true, value),
        (e: unknown) => control.reply(id, false, text(e)),
      );
    })
    .then(async (unlisten) => {
      await control.ready();
      return unlisten;
    });

/** The schema of an object with `properties`, each required unless named in `optional`. */
export function object(properties: Record<string, Schema>, optional: string[] = []): Schema {
  return {
    type: "object",
    properties,
    required: Object.keys(properties).filter((k) => !optional.includes(k)),
    additionalProperties: false,
  };
}

export const none: Schema = object({});
export const string = (description: string): Schema => ({ type: "string", description });
export const number = (description: string): Schema => ({ type: "number", description });
export const boolean = (description: string): Schema => ({ type: "boolean", description });
export const oneOf = (values: string[], description: string): Schema => ({ type: "string", enum: values, description });

/** Parameter `key` of `params` as a string, or the command's error naming it. */
export function stringOf(params: Record<string, unknown>, key: string): string {
  const value = params[key];
  if (typeof value !== "string") throw new Error(`${key} must be a string`);
  return value;
}
