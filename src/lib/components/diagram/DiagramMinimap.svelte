<script lang="ts">
  import type { DiagramTable } from '$lib/types/diagram';
  import { TABLE_WIDTH, HEADER_HEIGHT, COL_ROW_HEIGHT } from '$lib/utils/diagramLayout';

  let {
    tables,
    viewBox,
    onnavigate,
  }: {
    tables: DiagramTable[];
    viewBox: { x: number; y: number; width: number; height: number };
    onnavigate: (x: number, y: number) => void;
  } = $props();

  const MINIMAP_W = 140;
  const MINIMAP_H = 90;

  let bounds = $derived.by(() => {
    if (tables.length === 0) return { x: 0, y: 0, w: 800, h: 600 };
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const t of tables) {
      const h = HEADER_HEIGHT + t.columns.length * COL_ROW_HEIGHT;
      minX = Math.min(minX, t.x);
      minY = Math.min(minY, t.y);
      maxX = Math.max(maxX, t.x + TABLE_WIDTH);
      maxY = Math.max(maxY, t.y + h);
    }
    const pad = 20;
    return { x: minX - pad, y: minY - pad, w: maxX - minX + pad * 2, h: maxY - minY + pad * 2 };
  });

  let scale = $derived(Math.min(MINIMAP_W / bounds.w, MINIMAP_H / bounds.h));

  function mapX(x: number): number {
    return (x - bounds.x) * scale;
  }
  function mapY(y: number): number {
    return (y - bounds.y) * scale;
  }

  function handleClick(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    const worldX = mx / scale + bounds.x;
    const worldY = my / scale + bounds.y;
    onnavigate(worldX - viewBox.width / 2, worldY - viewBox.height / 2);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="minimap" onclick={handleClick}>
  <svg width={MINIMAP_W} height={MINIMAP_H}>
    <!-- Tables as small rects -->
    {#each tables as t}
      {@const h = HEADER_HEIGHT + t.columns.length * COL_ROW_HEIGHT}
      <rect
        x={mapX(t.x)}
        y={mapY(t.y)}
        width={TABLE_WIDTH * scale}
        height={h * scale}
        fill="var(--bg-tertiary, var(--bg-secondary))"
        stroke="var(--border-color)"
        stroke-width="0.5"
      />
    {/each}
    <!-- Viewport rect -->
    <rect
      x={mapX(viewBox.x)}
      y={mapY(viewBox.y)}
      width={viewBox.width * scale}
      height={viewBox.height * scale}
      fill="rgba(122, 162, 247, 0.15)"
      stroke="var(--accent)"
      stroke-width="1"
    />
  </svg>
</div>

<style>
  .minimap {
    position: absolute;
    bottom: 12px;
    right: 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    cursor: crosshair;
    overflow: hidden;
    opacity: 0.85;
    transition: opacity var(--transition-fast);
  }

  .minimap:hover {
    opacity: 1;
  }
</style>
