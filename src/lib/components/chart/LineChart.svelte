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
  let crosshairX = $state<number | null>(null);

  let colors = $derived(generateColors(yLabels.length));

  let allValues = $derived(data.flatMap(d => d.values));
  let scale = $derived(computeScale(allValues));

  let chartW = $derived(WIDTH - PADDING.left - PADDING.right);
  let chartH = $derived(HEIGHT - PADDING.top - PADDING.bottom - (showLegend ? LEGEND_HEIGHT : 0));

  function xPos(index: number): number {
    if (data.length <= 1) return chartW / 2;
    return (index / (data.length - 1)) * chartW;
  }

  function yPos(value: number): number {
    const range = scale.max - scale.min;
    if (range === 0) return chartH;
    return chartH - ((value - scale.min) / range) * chartH;
  }

  function pathForSeries(seriesIdx: number): string {
    if (data.length === 0) return '';
    const points = data.map((d, i) => `${xPos(i)},${yPos(d.values[seriesIdx] ?? 0)}`);
    return `M${points.join(' L')}`;
  }

  function handleDotEnter(label: string, yLabel: string, value: number, px: number, py: number) {
    tooltip = { x: px + PADDING.left, y: py + PADDING.top - 10, text: `${label}: ${yLabel} = ${value}` };
    crosshairX = px;
  }

  function handleDotLeave() {
    tooltip = null;
    crosshairX = null;
  }
</script>

<svg viewBox="0 0 {WIDTH} {HEIGHT}" class="line-chart">
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

    <!-- Axes -->
    <line x1="0" y1="0" x2="0" y2={chartH} stroke="var(--border-color)" stroke-width="1" />
    <line x1="0" y1={chartH} x2={chartW} y2={chartH} stroke="var(--border-color)" stroke-width="1" />

    <!-- Crosshair -->
    {#if crosshairX !== null}
      <line
        x1={crosshairX} y1="0"
        x2={crosshairX} y2={chartH}
        stroke="var(--text-muted)" stroke-width="0.5" stroke-dasharray="3,3"
      />
    {/if}

    <!-- Lines -->
    {#each yLabels as _, si}
      <path
        d={pathForSeries(si)}
        fill="none"
        stroke={colors[si]}
        stroke-width="2"
        stroke-linejoin="round"
        stroke-linecap="round"
      />
    {/each}

    <!-- Dots -->
    {#each data as point, di}
      {#each point.values as val, vi}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <circle
          cx={xPos(di)}
          cy={yPos(val)}
          r="3.5"
          fill={colors[vi]}
          stroke="var(--bg-secondary)"
          stroke-width="1.5"
          onmouseenter={() => handleDotEnter(point.label, yLabels[vi], val, xPos(di), yPos(val))}
          onmouseleave={handleDotLeave}
        />
      {/each}
    {/each}

    <!-- X axis labels -->
    {#each data as point, di}
      {#if data.length <= 20 || di % Math.ceil(data.length / 20) === 0}
        <text
          x={xPos(di)}
          y={chartH + 14}
          font-size="10"
          fill="var(--text-muted)"
          text-anchor="middle"
          font-family="var(--font-mono)"
          transform="rotate(-30, {xPos(di)}, {chartH + 14})"
        >{point.label.length > 12 ? point.label.substring(0, 12) + '...' : point.label}</text>
      {/if}
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
  .line-chart {
    width: 100%;
    height: 100%;
    user-select: none;
  }

  .line-chart circle {
    transition: r 0.1s ease;
  }

  .line-chart circle:hover {
    r: 5;
  }
</style>
