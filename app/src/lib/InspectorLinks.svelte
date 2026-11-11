<script lang="ts">
  // The inspector's links of the selected instance: what each of its flow ports feeds or is fed by, and, for a device,
  // the conveyor it is mounted on and how far along it (an angle in degrees on a curve). Snapping sets these in 4.5b.
  import { m } from "../paraglide/messages.js";
  import { flowPorts, hostsFor, mountOf, mountsOnAHost } from "./inspector";
  import type { Edit, Endpoint, Instance, ProjectView } from "./project";

  let {
    project,
    instance,
    onEdit,
  }: { project: ProjectView; instance: Instance; onEdit: (op: Edit) => Promise<boolean> } = $props();

  const ports = $derived(flowPorts(project, instance));
  const mount = $derived(mountOf(project, instance.id));
  const key = (e: Endpoint | null): string => (e ? `${e[0]}.${e[1]}` : "");
  const NONE = "";

  function setPeer(port: string, value: string): void {
    const peer = value === NONE ? null : (value.split(".") as Endpoint);
    void onEdit({ op: "set_flow", id: instance.id, port, peer });
  }

  function setHost(host: string): void {
    void onEdit({ op: "set_mount", device: instance.id, host: host || null, at: mount?.at ?? 0 });
  }

  async function setAt(text: string, field: HTMLInputElement): Promise<void> {
    const at = Number(text.replace(",", "."));
    const ok =
      mount && Number.isFinite(at) && (await onEdit({ op: "set_mount", device: instance.id, host: mount.host, at }));
    if (!ok) field.value = String(mount?.at ?? "");
  }
</script>

{#if ports.length > 0}
  <h3 class="sub">{m.inspector_flow()}</h3>
  <dl>
    {#each ports as p (p.port)}
      <dt class="muted">{p.port} {p.kind === "flow_in" ? "←" : "→"}</dt>
      <dd>
        <select aria-label={p.port} value={key(p.peer)} onchange={(e) => setPeer(p.port, e.currentTarget.value)}>
          <option value={NONE}>{m.inspector_not_connected()}</option>
          {#each p.candidates as c (key(c))}<option value={key(c)}>{key(c)}</option>{/each}
        </select>
      </dd>
    {/each}
  </dl>
{/if}
{#if mountsOnAHost(project, instance)}
  <h3 class="sub">{m.inspector_mount()}</h3>
  <dl>
    <dt class="muted">{m.inspector_host()}</dt>
    <dd>
      <select
        aria-label={m.inspector_host()}
        value={mount?.host ?? NONE}
        onchange={(e) => setHost(e.currentTarget.value)}
      >
        <option value={NONE}>{m.inspector_not_mounted()}</option>
        {#each hostsFor(project, instance) as host (host)}<option value={host}>{host}</option>{/each}
      </select>
    </dd>
    {#if mount}
      <dt class="muted">{m.inspector_along()}</dt>
      <dd>
        <input
          class="data"
          aria-label={m.inspector_along()}
          value={mount.at}
          onchange={(e) => setAt(e.currentTarget.value, e.currentTarget)}
        />
      </dd>
    {/if}
  </dl>
{/if}

<style>
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 6px 12px;
    margin: 0 0 12px;
  }
  dd {
    margin: 0;
  }
</style>
