<script lang="ts">
  // Snapping's controls (spec 2026-11-11-snapping-design.md, "The view"): whether a drag snaps, the gap a new join
  // leaves between the faces, and the join the drag would make now.
  import { m } from "../paraglide/messages.js";
  import type { Candidate } from "./snapping";

  let {
    enabled = $bindable(true),
    gap = $bindable(0),
    candidate = null,
  }: { enabled?: boolean; gap?: number; candidate?: Candidate | null } = $props();

  /** The gap field in millimetres, from 0 to the 150 mm a join reaches. */
  function setGap(text: string, field: HTMLInputElement): void {
    const mm = Number(text.replace(",", "."));
    if (Number.isFinite(mm) && mm >= 0 && mm <= 150) gap = mm / 1000;
    field.value = String(Math.round(gap * 1000));
  }

  const describe = (c: Candidate): string => {
    const mm = String(Math.round(gap * 1000));
    switch (c.kind) {
      case "join":
        return m.snap_join({ peer: c.peer.join("."), gap: mm });
      case "merge":
        return m.snap_merge({
          host: c.host,
          side: c.side === "left" ? m.snap_left() : m.snap_right(),
          at: c.at,
          gap: mm,
        });
      case "mount":
        return m.snap_mount({ host: c.host, at: c.at });
    }
  };
</script>

<div class="row snap">
  <label class="row"><input type="checkbox" bind:checked={enabled} />{m.snap_toggle()}</label>
  <label class="row"
    >{m.snap_gap()}
    <input
      class="data gap"
      aria-label={m.snap_gap()}
      value={Math.round(gap * 1000)}
      onchange={(e) => setGap(e.currentTarget.value, e.currentTarget)}
    /></label
  >
  <span class="muted" role="status">{candidate ? describe(candidate) : ""}</span>
</div>

<style>
  .snap {
    gap: 12px;
    flex-wrap: wrap;
  }
  .gap {
    width: 5ch;
  }
</style>
