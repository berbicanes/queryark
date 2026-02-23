<script lang="ts">
  import type { ChartDataPoint } from '$lib/types/chart';
  import { generateColors, formatAxisLabel, computeScale } from '$lib/utils/chartHelpers';

  let {
    data = [],
    yLabels = [],
    showGrid = true,
    showLegend = true,
  }: {
    data: ChartDataPoint[];
    yLabels: string[];
    showGrid: boolean;
    showLegend: boolean;
  } = $props();

  const PADDING = { top: 20, right: 20, bottom: 60, left: 60 };
  const LEGEND_HEIGHT = 30;
  const WIDTH = 800;
  const HEIGHT = 400;

  let tooltip = $state<{ x: number; y: number; text: string } | null>(null);

  let colors = $derived(generateColors(yLabels.length));

  let allValues = $derived(data.flatMap(d => d.values));
  let scale = $derived(computeScale(allValues));

  let chartW = $derived(WIDTH - PADDING.left - PADDING.right);
  let chartH = $derived(HEIGHT - PADDING.top - PADDING.bottom - (showLegend ? LEGEND_HEIGHT : 0));

  let barGroupWidth = $derived(data.length > 0 ? chartW / data.length : 0);
  let barWidth = $derived(Math.max(2, Math.min(40, (barGroupWidth - 4) / Math.max(1, yLabels.length))));

  function yPos(value: number): number {
    const range = scale.max - scale.min;
    if (range === 0) return chartH;
    return chartH - ((value - scale.min) / range) * chartH;
  }

  function handleBarEnter(label: string, yLabel: string, value: number, x: number, y: number) {
    tooltip = { x: x + PADDING.left, y: y + PADDING.top - 10, text: `${label}: ${yLabel} = ${value}` };
  }

  function handleBarLeave() {
    tooltip = null;
  }
</script>

<svg viewBox="0 0 {WIDTH} {HEIGHT}" class="bar-chart">
  <g transform="translate({PADDING.left}, {PADDING.top})">
    <!-- Grid lines -->
    {#if showGrid}
      {#each scale.ticks as tick}
        <line
          x1="0" y1={yPos(tick)}
          x2={chartW} y2={yPos(tick)}
          stroke="var(--border-color)" stroke-width="0.5" opacity="0.4"
        />
      {/each}
    {/if}

    <!-- Y axis labels -->
    {#each scale.ticks as tick}
      <text
        x="-8" y={yPos(tick) + 3}
        font-size="10" fill="var(--text-muted)" text-anchor="end"
        font-family="var(--font-mono)"
      >{formatAxisLabel(tick)}</text>
    {/each}

    <!-- Y axis line -->
    <line x1="0" y1="0" x2="0" y2={chartH} stroke="var(--border-color)" stroke-width="1" />
    <!-- X axis line -->
    <line x1="0" y1={chartH} x2={chartW} y2={chartH} stroke="var(--border-color)" stroke-width="1" />

    <!-- Bars -->
    {#each data as point, di}
      {@const gx = di * barGroupWidth + barGroupWidth / 2}
      {#each point.values as val, vi}
        {@const bx = gx - (yLabels.length * barWidth) / 2 + vi * barWidth}
        {@const by = yPos(val)}
        {@const bh = chartH - by}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <rect
          x={bx}
          y={by}
          width={barWidth - 1}
          height={Math.max(0, bh)}
          fill={colors[vi]}
          rx="1"
          opacity="0.85"
          onmouseenter={() => handleBarEnter(point.label, yLabels[vi], val, bx, by)}
          onmouseleave={handleBarLeave}
        />
      {/each}

      <!-- X axis label -->
      <text
        x={gx}
        y={chartH + 14}
        font-size="10"
        fill="var(--text-muted)"
        text-anchor="middle"
        font-family="var(--font-mono)"
        transform="rotate(-30, {gx}, {chartH + 14})"
      >{point.label.length > 12 ? point.label.substring(0, 12) + '...' : point.label}</text>
    {/each}

    <!-- Tooltip -->
    {#if tooltip}
      <g transform="translate({tooltip.x - PADDING.left}, {tooltip.y - PADDING.top})">
        <rect
          x="-4" y="-14"
          width={tooltip.text.length * 6.5 + 8} height="18"
          rx="3" fill="var(--bg-primary)" stroke="var(--border-color)" stroke-width="0.5"
        />
        <text
          x="0" y="-1"
          font-size="10" fill="var(--text-primary)"
          font-family="var(--font-mono)"
        >{tooltip.text}</text>
      </g>
    {/if}

    <!-- Legend -->
    {#if showLegend && yLabels.length > 1}
      <g transform="translate(0, {chartH + 40})">
        {#each yLabels as label, i}
          <rect x={i * 120} y="0" width="10" height="10" rx="2" fill={colors[i]} />
          <text x={i * 120 + 14} y="9" font-size="10" fill="var(--text-secondary)">{label}</text>
        {/each}
      </g>
    {/if}
  </g>
</svg>

<style>
  .bar-chart {
    width: 100%;
    height: 100%;
    user-select: none;
  }

  .bar-chart rect:hover {
    opacity: 1 !important;
  }
</style>
