<script lang="ts">
  import type { ChartDataPoint } from '$lib/types/chart';
  import { generateColors } from '$lib/utils/chartHelpers';

  let {
    data = [],
    showLegend = true,
  }: {
    data: ChartDataPoint[];
    showLegend: boolean;
  } = $props();

  const WIDTH = 800;
  const HEIGHT = 400;
  const CX = 280;
  const CY = 200;
  const RADIUS = 150;
  const INNER_RADIUS = 60;

  let tooltip = $state<{ x: number; y: number; text: string } | null>(null);
  let hoverIndex = $state<number | null>(null);

  // Use first value from each data point
  let values = $derived(data.map(d => Math.abs(d.values[0] ?? 0)));
  let total = $derived(values.reduce((s, v) => s + v, 0));
  let colors = $derived(generateColors(data.length));

  interface SliceInfo {
    path: string;
    labelX: number;
    labelY: number;
    percent: number;
    label: string;
    value: number;
    midAngle: number;
  }

  let slices = $derived.by((): SliceInfo[] => {
    if (total === 0) return [];
    const result: SliceInfo[] = [];
    let startAngle = -Math.PI / 2;

    for (let i = 0; i < values.length; i++) {
      const fraction = values[i] / total;
      const angle = fraction * Math.PI * 2;
      const endAngle = startAngle + angle;
      const midAngle = startAngle + angle / 2;

      const r = hoverIndex === i ? RADIUS + 8 : RADIUS;
      const ir = INNER_RADIUS;

      const x1 = CX + Math.cos(startAngle) * r;
      const y1 = CY + Math.sin(startAngle) * r;
      const x2 = CX + Math.cos(endAngle) * r;
      const y2 = CY + Math.sin(endAngle) * r;
      const ix1 = CX + Math.cos(endAngle) * ir;
      const iy1 = CY + Math.sin(endAngle) * ir;
      const ix2 = CX + Math.cos(startAngle) * ir;
      const iy2 = CY + Math.sin(startAngle) * ir;

      const largeArc = angle > Math.PI ? 1 : 0;

      const path = [
        `M ${x1} ${y1}`,
        `A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2}`,
        `L ${ix1} ${iy1}`,
        `A ${ir} ${ir} 0 ${largeArc} 0 ${ix2} ${iy2}`,
        'Z'
      ].join(' ');

      const labelR = RADIUS + 24;
      const labelX = CX + Math.cos(midAngle) * labelR;
      const labelY = CY + Math.sin(midAngle) * labelR;

      result.push({
        path,
        labelX,
        labelY,
        percent: fraction * 100,
        label: data[i].label,
        value: values[i],
        midAngle,
      });

      startAngle = endAngle;
    }
    return result;
  });

  function handleSliceEnter(i: number, label: string, value: number, percent: number) {
    hoverIndex = i;
    const slice = slices[i];
    if (slice) {
      tooltip = {
        x: CX + Math.cos(slice.midAngle) * (RADIUS * 0.6),
        y: CY + Math.sin(slice.midAngle) * (RADIUS * 0.6),
        text: `${label}: ${value} (${percent.toFixed(1)}%)`,
      };
    }
  }

  function handleSliceLeave() {
    hoverIndex = null;
    tooltip = null;
  }
</script>

<svg viewBox="0 0 {WIDTH} {HEIGHT}" class="pie-chart">
  <!-- Slices -->
  {#each slices as slice, i}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <path
      d={slice.path}
      fill={colors[i]}
      stroke="var(--bg-secondary)"
      stroke-width="2"
      opacity={hoverIndex === null || hoverIndex === i ? 0.85 : 0.5}
      onmouseenter={() => handleSliceEnter(i, slice.label, slice.value, slice.percent)}
      onmouseleave={handleSliceLeave}
    />
  {/each}

  <!-- Center label -->
  <text x={CX} y={CY - 6} text-anchor="middle" font-size="16" font-weight="600" fill="var(--text-primary)">
    {total.toLocaleString()}
  </text>
  <text x={CX} y={CY + 12} text-anchor="middle" font-size="10" fill="var(--text-muted)">total</text>

  <!-- Percentage labels (for larger slices) -->
  {#each slices as slice, i}
    {#if slice.percent > 5}
      <text
        x={slice.labelX}
        y={slice.labelY}
        text-anchor="middle"
        font-size="10"
        fill="var(--text-secondary)"
        font-family="var(--font-mono)"
      >{slice.percent.toFixed(1)}%</text>
    {/if}
  {/each}

  <!-- Tooltip -->
  {#if tooltip}
    <g transform="translate({tooltip.x}, {tooltip.y})">
      <rect
        x={-(tooltip.text.length * 3.25 + 4)} y="-14"
        width={tooltip.text.length * 6.5 + 8} height="18"
        rx="3" fill="var(--bg-primary)" stroke="var(--border-color)" stroke-width="0.5"
      />
      <text
        x={-(tooltip.text.length * 3.25)} y="-1"
        font-size="10" fill="var(--text-primary)"
        font-family="var(--font-mono)"
      >{tooltip.text}</text>
    </g>
  {/if}

  <!-- Legend -->
  {#if showLegend}
    <g transform="translate({CX + RADIUS + 60}, {CY - Math.min(data.length * 10, RADIUS)})">
      {#each data as point, i}
        {#if i < 15}
          <rect x="0" y={i * 20} width="10" height="10" rx="2" fill={colors[i]} />
          <text
            x="16" y={i * 20 + 9}
            font-size="11" fill="var(--text-secondary)"
          >{point.label.length > 20 ? point.label.substring(0, 20) + '...' : point.label}</text>
        {/if}
      {/each}
      {#if data.length > 15}
        <text x="0" y={15 * 20 + 9} font-size="10" fill="var(--text-muted)">
          +{data.length - 15} more...
        </text>
      {/if}
    </g>
  {/if}
</svg>

<style>
  .pie-chart {
    width: 100%;
    height: 100%;
    user-select: none;
  }

  .pie-chart path {
    transition: opacity 0.15s ease;
    cursor: pointer;
  }

  .pie-chart path:hover {
    opacity: 1 !important;
  }
</style>
