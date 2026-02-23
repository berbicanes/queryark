<script lang="ts">
  import type { CellValue, ColumnDef } from '$lib/types/query';
  import type { ChartType } from '$lib/types/chart';
  import { prepareChartData, detectDefaults } from '$lib/utils/chartHelpers';
  import BarChart from './BarChart.svelte';
  import LineChart from './LineChart.svelte';
  import PieChart from './PieChart.svelte';

  let {
    columns = [],
    rows = [],
  }: {
    columns: ColumnDef[];
    rows: CellValue[][];
  } = $props();

  let chartType = $state<ChartType>('bar');
  let showGrid = $state(true);
  let showLegend = $state(true);

  // Auto-detect defaults when columns change
  let defaults = $derived(detectDefaults(columns));
  let xColumn = $state('');
  let yColumns = $state<string[]>([]);

  $effect(() => {
    if (defaults.xColumn && !xColumn) {
      xColumn = defaults.xColumn;
    }
    if (defaults.yColumns.length > 0 && yColumns.length === 0) {
      yColumns = defaults.yColumns;
    }
  });

  // Reset selections when columns change
  $effect(() => {
    const colNames = new Set(columns.map(c => c.name));
    if (!colNames.has(xColumn)) {
      xColumn = defaults.xColumn;
    }
    yColumns = yColumns.filter(y => colNames.has(y));
    if (yColumns.length === 0) {
      yColumns = defaults.yColumns;
    }
  });

  let chartData = $derived(prepareChartData(columns, rows, xColumn, yColumns));

  function toggleYColumn(colName: string) {
    if (yColumns.includes(colName)) {
      if (yColumns.length > 1) {
        yColumns = yColumns.filter(y => y !== colName);
      }
    } else {
      yColumns = [...yColumns, colName];
    }
  }

  function handleExportSvg() {
    const svgEl = document.querySelector('.chart-content svg');
    if (!svgEl) return;
    const svgData = new XMLSerializer().serializeToString(svgEl);
    const blob = new Blob([svgData], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'chart.svg';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="chart-panel">
  <div class="chart-config">
    <div class="config-group">
      <label class="config-label">Type</label>
      <div class="type-toggles">
        <button class="type-btn" class:active={chartType === 'bar'} onclick={() => chartType = 'bar'}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="1" y="8" width="3" height="7" rx="0.5" /><rect x="6" y="4" width="3" height="11" rx="0.5" /><rect x="11" y="1" width="3" height="14" rx="0.5" />
          </svg>
          Bar
        </button>
        <button class="type-btn" class:active={chartType === 'line'} onclick={() => chartType = 'line'}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <polyline points="1,12 5,6 9,9 15,2" />
          </svg>
          Line
        </button>
        <button class="type-btn" class:active={chartType === 'pie'} onclick={() => chartType = 'pie'}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="8" cy="8" r="6" /><path d="M8,2 L8,8 L14,8" />
          </svg>
          Pie
        </button>
      </div>
    </div>

    <div class="config-separator"></div>

    <div class="config-group">
      <label class="config-label">X Axis</label>
      <select class="config-select" bind:value={xColumn}>
        {#each columns as col}
          <option value={col.name}>{col.name}</option>
        {/each}
      </select>
    </div>

    <div class="config-group">
      <label class="config-label">Y Axis</label>
      <div class="y-columns">
        {#each columns as col}
          {#if col.name !== xColumn}
            <button
              class="y-chip"
              class:active={yColumns.includes(col.name)}
              onclick={() => toggleYColumn(col.name)}
            >{col.name}</button>
          {/if}
        {/each}
      </div>
    </div>

    <div class="config-separator"></div>

    <div class="config-group config-toggles">
      <label class="toggle-label">
        <input type="checkbox" bind:checked={showGrid} />
        Grid
      </label>
      <label class="toggle-label">
        <input type="checkbox" bind:checked={showLegend} />
        Legend
      </label>
    </div>

    <button class="export-btn" onclick={handleExportSvg} title="Export as SVG">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="7 10 12 15 17 10"></polyline>
        <line x1="12" y1="15" x2="12" y2="3"></line>
      </svg>
      SVG
    </button>
  </div>

  <div class="chart-content">
    {#if chartData.length === 0}
      <div class="chart-empty">
        <span class="text-muted">No data to chart. Select numeric Y-axis columns.</span>
      </div>
    {:else if chartType === 'bar'}
      <BarChart data={chartData} yLabels={yColumns} {showGrid} {showLegend} />
    {:else if chartType === 'line'}
      <LineChart data={chartData} yLabels={yColumns} {showGrid} {showLegend} />
    {:else if chartType === 'pie'}
      <PieChart data={chartData} {showLegend} />
    {/if}
  </div>
</div>

<style>
  .chart-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .chart-config {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
    flex-wrap: wrap;
    min-height: 36px;
  }

  .config-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .config-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    white-space: nowrap;
  }

  .type-toggles {
    display: flex;
    gap: 2px;
  }

  .type-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    font-size: 11px;
    font-family: var(--font-sans);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .type-btn.active {
    background: var(--bg-active);
    color: var(--accent);
    border-color: var(--accent);
  }

  .type-btn:hover:not(.active) {
    background: var(--bg-hover);
  }

  .config-select {
    padding: 2px 6px;
    font-size: 11px;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    outline: none;
    max-width: 140px;
  }

  .y-columns {
    display: flex;
    gap: 3px;
    flex-wrap: wrap;
    max-width: 300px;
  }

  .y-chip {
    padding: 2px 6px;
    font-size: 10px;
    font-family: var(--font-mono);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 10px;
    white-space: nowrap;
  }

  .y-chip.active {
    background: rgba(122, 162, 247, 0.15);
    color: var(--accent);
    border-color: var(--accent);
  }

  .y-chip:hover:not(.active) {
    background: var(--bg-hover);
  }

  .config-separator {
    width: 1px;
    height: 20px;
    background: var(--border-color);
  }

  .config-toggles {
    gap: 8px;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .toggle-label input {
    width: 12px;
    height: 12px;
    accent-color: var(--accent);
  }

  .export-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    font-size: 10px;
    font-family: var(--font-sans);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    margin-left: auto;
  }

  .export-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .chart-content {
    flex: 1;
    overflow: hidden;
    padding: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .chart-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }
</style>
