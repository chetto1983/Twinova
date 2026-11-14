<script lang="ts">
  // One rung of a Ladder file drawn from its layout (ladder/layout.ts): rails, wires, contacts, coils, boxes and its
  // exit, as CODESYS's and TIA's Ladder draw them.
  // A click is read off `data-path` by the view (LadderEditor.svelte): an element's, or a branch path's on its wire.
  // Online (spec 2026-11-13-ladder-editor-design.md, decision 6, as LD2's): a wire or a symbol carrying power is thick
  // and green, one without it thick and dark, one not read yet thin; thickness tells as well as colour. Each operand's
  // and box pin's value is drawn under it, and a click on it prepares a write or a force (`data-spot`).
  import { L, OPERAND_MAX, textW, type Feed, type LNode, type RungLayout } from "./ladder/layout";
  import { powered, type Online } from "./ladder/view";
  import { keyOf } from "./ladder/model";
  let {
    layout,
    selected,
    rung = 0,
    online = null,
    forced = () => false,
  }: {
    layout: RungLayout;
    selected: number[] | null;
    rung?: number;
    online?: Online | null;
    forced?: (spot: number) => boolean;
  } = $props();
  const power = (feed: Feed): string => {
    const on = powered(online, rung, feed);
    return on === null ? "" : on ? "on" : "off";
  };
  const valueAt = (path: number[], pin: string | null = null) =>
    online?.values.get(pin === null ? keyOf(rung, path) : `${keyOf(rung, path)}#${pin}`);
  const picked = (path: number[]): boolean => !!selected && selected.length > 0 && path.join() === selected.join();

  /** An operand as drawn: `???` while blank, cut in the middle past what the box grows to fit. */
  const shown = (operand: string | null): string => {
    const text = operand ?? "???";
    if (text.length <= OPERAND_MAX) return text;
    const half = Math.floor((OPERAND_MAX - 1) / 2);
    return `${text.slice(0, half)}…${text.slice(text.length - half)}`;
  };
  const contactMark = { open: "", closed: "/", rising: "P", falling: "N" } as const;
  const coilMark = { normal: "", negated: "/", set: "S", reset: "R", rising: "P", falling: "N" } as const;
  const mid = (n: LNode) => n.x + n.w / 2;
  const row = (n: LNode, k: number) => n.y + L.BOX_HEAD + L.PIN_STEP * k + L.PIN_STEP / 2;
</script>

<svg width={layout.width} height={layout.height} viewBox="0 0 {layout.width} {layout.height}" role="img">
  <line class="rail {power(null)}" x1={L.RAIL_X} y1="0" x2={L.RAIL_X} y2={layout.height} />
  <line class="rail" x1={layout.width - L.RAIL_X} y1="0" x2={layout.width - L.RAIL_X} y2={layout.height} />
  {#each layout.wires as w, i (i)}<line class="wire {power(w.feed)}" x1={w.x1} y1={w.y1} x2={w.x2} y2={w.y2} />{/each}
  {#each layout.vlines as v, i (i)}<line class="wire {power(v.feed)}" x1={v.x} y1={v.y1} x2={v.x} y2={v.y2} />{/each}
  {#each layout.branches as b (b.path.join())}
    {#each b.legs as y, j (j)}
      {@const path = [...b.path, j]}
      <line
        class="leg"
        class:picked={picked(path)}
        data-path={path.join(".")}
        x1={b.x + 6}
        y1={y}
        x2={b.x + b.w - 6}
        y2={y}
      />
    {/each}
  {/each}
  {#each layout.nodes as n, i (i)}
    {@const cy = n.y + n.h / 2}
    {#if n.kind !== "exit"}
      {#if picked(n.path)}<rect class="picked" x={n.x - 4} y={n.y - 16} width={n.w + 8} height={n.h + 20} rx="3" />{/if}
      <!-- the whole symbol takes the click, its gaps too -->
      <rect class="hit" data-path={n.path.join(".")} x={n.x} y={n.y - 14} width={n.w} height={n.h + 14} />
    {/if}
    {#if n.kind === "contact"}
      <g class="contact" data-path={n.path.join(".")}>
        <title>{n.operand ?? "???"}</title>
        <line class="wire {power(n.feed)}" x1={n.x} y1={cy} x2={mid(n) - 7} y2={cy} />
        <line class="wire {power(n.path)}" x1={mid(n) + 7} y1={cy} x2={n.x + n.w} y2={cy} />
        <line class="symbol {power(n.feed)}" x1={mid(n) - 7} y1={n.y} x2={mid(n) - 7} y2={n.y + n.h} />
        <line class="symbol {power(n.path)}" x1={mid(n) + 7} y1={n.y} x2={mid(n) + 7} y2={n.y + n.h} />
        {#if n.mode === "closed"}
          <line class="symbol" x1={mid(n) + 5} y1={n.y + 3} x2={mid(n) - 5} y2={n.y + n.h - 3} />
        {:else if contactMark[n.mode]}
          <text class="mark" x={mid(n)} y={cy + 4}>{contactMark[n.mode]}</text>
        {/if}
        <text class="operand" class:blank={n.operand === null} x={mid(n)} y={n.y - 5}>{shown(n.operand)}</text>
      </g>
    {:else if n.kind === "coil"}
      <g class="coil" data-path={n.path.join(".")}>
        <title>{n.operand ?? "???"}</title>
        <line class="wire {power(n.feed)}" x1={n.x} y1={cy} x2={mid(n) - 9} y2={cy} />
        <line class="wire {power(n.path)}" x1={mid(n) + 9} y1={cy} x2={n.x + n.w} y2={cy} />
        <path class="symbol {power(n.path)}" d="M {mid(n) - 5} {n.y} Q {mid(n) - 12} {cy} {mid(n) - 5} {n.y + n.h}" />
        <path class="symbol {power(n.path)}" d="M {mid(n) + 5} {n.y} Q {mid(n) + 12} {cy} {mid(n) + 5} {n.y + n.h}" />
        {#if coilMark[n.mode]}<text class="mark" x={mid(n)} y={cy + 4}>{coilMark[n.mode]}</text>{/if}
        <text class="operand" class:blank={n.operand === null} x={mid(n)} y={n.y - 5}>{shown(n.operand)}</text>
      </g>
    {:else if n.kind === "box"}
      <g class="box" data-path={n.path.join(".")}>
        <rect class="symbol {power(n.path)}" x={n.x} y={n.y} width={n.w} height={n.h} rx="2" />
        <text class="title" x={mid(n)} y={n.y + 13}>{n.title}</text>
        {#if n.instance}<text class="operand" x={mid(n)} y={n.y - 5}>{n.instance}</text>{/if}
        {#each n.left as pin, k (k)}
          <text class="pin" x={n.x + 4} y={row(n, k) + 3}>{pin.name}</text>
          {#if pin.value !== null}<text class="value end" x={n.x - 4} y={row(n, k) - 3}>{pin.value}</text>{/if}
          {@const live = valueAt(n.path, pin.name)}
          {#if live}{@render reading(live, n.x - 4, row(n, k) + 10, "end")}{/if}
        {/each}
        {#each n.right as pin, k (k)}
          <text class="pin end" x={n.x + n.w - 4} y={row(n, k) + 3}>{pin.name}</text>
          {#if pin.value !== null}<text class="value" x={n.x + n.w + 4} y={row(n, k) - 3}>{pin.value}</text>{/if}
          {@const live = valueAt(n.path, pin.name)}
          {#if live}{@render reading(live, n.x + n.w + 4, row(n, k) + 10, "start")}{/if}
        {/each}
      </g>
    {:else}
      <text class="exit" x={n.x} y={n.y + n.h / 2 + 4}>{n.text}</text>
    {/if}
    {#if n.kind === "contact" || n.kind === "coil"}
      {@const live = valueAt(n.path)}
      {#if live}{@render reading(live, mid(n), n.y + n.h + 11, "middle")}{/if}
    {/if}
  {/each}
</svg>

{#snippet reading(live: { value: string; spot: number }, x: number, y: number, anchor: "start" | "middle" | "end")}
  <!-- a value read online sits on a pill, so it is not taken for a constant written on the pin -->
  {@const w = textW(live.value, 9) + 6}
  {@const left = anchor === "start" ? x - 3 : anchor === "end" ? x - w + 3 : x - w / 2}
  <g class="reading" class:true={live.value === "TRUE"} class:forced={forced(live.spot)} data-spot={live.spot}>
    <rect x={left} y={y - 9} width={w} height="12" rx="2" />
    <!-- the anchor goes in `style`: the stylesheet's `text-anchor: middle` would win over the attribute -->
    <text {x} {y} style="text-anchor: {anchor}">{live.value}</text>
  </g>
{/snippet}

<style>
  svg {
    display: block;
    overflow: visible;
  }
  .leg {
    stroke: transparent;
    stroke-width: 10;
    cursor: pointer;
  }
  .leg.picked {
    stroke: var(--color-accent-soft);
  }
  rect.picked {
    fill: var(--color-accent-soft);
    stroke: var(--color-ring);
    stroke-width: 1;
  }
  .hit {
    fill: transparent;
    cursor: pointer;
  }
  .wire.on,
  .symbol.on,
  .rail.on {
    stroke: var(--color-success);
    stroke-width: 3;
  }
  .wire.off,
  .symbol.off {
    stroke: var(--color-text-faint);
    stroke-width: 3;
  }
  /* a dead wire is dashed too, so colour is not the only sign */
  .wire.off {
    stroke-dasharray: 5 3;
  }
  .reading {
    cursor: pointer;
  }
  .reading rect {
    fill: var(--color-surface-3);
  }
  .reading text {
    font-size: 9px;
    fill: var(--color-text-muted);
  }
  .reading.true text {
    fill: var(--color-success);
  }
  .reading.forced rect {
    stroke: var(--color-warning);
  }
  .reading.forced text {
    fill: var(--color-warning);
    font-weight: 600;
  }
  .rail {
    stroke: var(--color-text-muted);
    stroke-width: 3;
  }
  .wire,
  .symbol {
    stroke: var(--color-text);
    stroke-width: 1.5;
    fill: none;
  }
  rect.symbol {
    fill: var(--color-surface-2);
  }
  text {
    fill: var(--color-text);
    font-family: var(--font-mono);
    font-size: 10px;
    text-anchor: middle;
  }
  .title {
    font-weight: 600;
    font-size: 11px;
  }
  .pin {
    text-anchor: start;
    fill: var(--color-text-muted);
  }
  .value {
    text-anchor: start;
  }
  .end {
    text-anchor: end;
  }
  .mark {
    font-weight: 600;
  }
  .operand.blank {
    fill: var(--color-warning);
  }
  .exit {
    text-anchor: start;
    font-weight: 600;
    font-size: 11px;
  }
</style>
