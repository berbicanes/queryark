<script lang="ts">
  import type { Tab } from '$lib/types/tabs';
  import type { CellValue } from '$lib/types/query';
  import { compareResults, type ResultCompareRow, type ResultCompareResult } from '$lib/utils/resultCompare';

  let { tab }: { tab: Tab } = $props();

  const ROW_HEIGHT = 32;
  const VISIBLE_ROWS = 25;

  let scrollTop = $state(0);
  let containerEl: HTMLDivElement;
  let filter = $state<'all' | 'identical' | 'changed' | 'source-only' | 'target-only'>('all');
  let matchColumns = $state<number[]>([]);

  let compResult = $derived.by((): ResultCompareResult | null => {
    const src = tab.compareSourceResult;
    const tgt = tab.compareTargetResult;
    if (!src || !tgt) return null;
    return compareResults(src, tgt, matchColumns.length > 0 ? matchColumns : undefined);
  });

  let filteredRows = $derived.by((): ResultCompareRow[] => {
    if (!compResult) return [];
    if (filter === 'all') return compResult.rows;
    return compResult.rows.filter(r => r.status === filter);
  });

  let startIdx = $derived(Math.floor(scrollTop / ROW_HEIGHT));
  let visibleRows = $derived(filteredRows.slice(startIdx, startIdx + VISIBLE_ROWS + 2));
  let totalHeight = $derived(filteredRows.length * ROW_HEIGHT);
  let offsetY = $derived(startIdx * ROW_HEIGHT);

  function cellText(cell: CellValue | undefined): string {
    if (!cell) return '';
    switch (cell.type) {
      case 'Null': return 'NULL';
      case 'Bool': return String(cell.value);
      case 'Int': return String(cell.value);
      case 'Float': return String(cell.value);
      case 'Text': return cell.value.substring(0, 100);
      case 'Timestamp': return cell.value;
      case 'Json': return cell.value.substring(0, 100);
      case 'Binary': return `[${cell.value.length} bytes]`;
      case 'LargeText': return cell.value.preview.substring(0, 100);
      case 'LargeJson': return cell.value.preview.substring(0, 100);
      case 'LargeBinary': return `[${cell.value.full_length} bytes]`;
    }
  }

  function toggleMatchColumn(idx: number) {
    if (matchColumns.includes(idx)) {
      matchColumns = matchColumns.filter(i => i !== idx);
    } else {
      matchColumns = [...matchColumns, idx];
    }
  }

  function handleScroll() {
    scrollTop = containerEl.scrollTop;
  }
</script>

<div class="result-compare-tab">
  <!-- Header -->
  <div class="compare-header">
    <div class="header-info">
      <div class="source-label">
        <span class="label-badge source">Source</span>
        <span class="sql-preview">{tab.compareSourceResult?.sql?.substring(0, 60) ?? ''}</span>
      </div>
      <span class="vs">vs</span>
      <div class="target-label">
        <span class="label-badge target">Target</span>
        <span class="sql-preview">{tab.compareTargetResult?.sql?.substring(0, 60) ?? ''}</span>
      </div>
    </div>

    {#if compResult}
      <div class="stats-bar">
        <button class="stat-chip" class:active={filter === 'all'} onclick={() => filter = 'all'}>
          All ({compResult.rows.length})
        </button>
        <button class="stat-chip identical" class:active={filter === 'identical'} onclick={() => filter = 'identical'}>
          Identical ({compResult.stats.identical})
        </button>
        <button class="stat-chip changed" class:active={filter === 'changed'} onclick={() => filter = 'changed'}>
          Changed ({compResult.stats.changed})
        </button>
        <button class="stat-chip source-only" class:active={filter === 'source-only'} onclick={() => filter = 'source-only'}>
          Source only ({compResult.stats.sourceOnly})
        </button>
        <button class="stat-chip target-only" class:active={filter === 'target-only'} onclick={() => filter = 'target-only'}>
          Target only ({compResult.stats.targetOnly})
        </button>
      </div>
    {/if}
  </div>

  <!-- Match columns selector -->
  {#if compResult}
    <div class="match-bar">
      <span class="match-label">Match by:</span>
      {#each compResult.columns as col, i}
        <button
          class="match-chip"
          class:active={matchColumns.includes(i)}
          onclick={() => toggleMatchColumn(i)}
        >{col.name}</button>
      {/each}
      {#if matchColumns.length === 0}
        <span class="match-hint">Positional matching (click columns for key-based)</span>
      {/if}
    </div>
  {/if}

  <!-- Grid -->
  {#if compResult}
    <div class="compare-grid" bind:this={containerEl} onscroll={handleScroll}>
      <div class="scroll-spacer" style="height: {totalHeight}px">
        <table class="diff-table" style="transform: translateY({offsetY}px)">
          <thead>
            <tr>
              <th class="status-col">Status</th>
              {#each compResult.columns as col, ci}
                <th class="source-col">
                  {col.name}
                  <span class="col-side">src</span>
                </th>
                <th class="target-col">
                  {col.name}
                  <span class="col-side">tgt</span>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each visibleRows as row}
              <tr class="row-{row.status}" style="height: {ROW_HEIGHT}px">
                <td class="status-cell">
                  <span class="status-badge {row.status}">{row.status}</span>
                </td>
                {#each compResult.columns as _, ci}
                  {@const isChanged = row.changedColumns.has(ci)}
                  <td class:changed-cell={isChanged}>
                    {row.sourceRow ? cellText(row.sourceRow[ci]) : '—'}
                  </td>
                  <td class:changed-cell={isChanged}>
                    {row.targetRow ? cellText(row.targetRow[ci]) : '—'}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <span class="text-muted">No comparison data available</span>
    </div>
  {/if}
</div>

<style>
  .result-compare-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .compare-header {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
  }

  .source-label, .target-label {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .label-badge {
    padding: 1px 6px;
    font-size: 9px;
    font-weight: 700;
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    flex-shrink: 0;
  }

  .label-badge.source {
    color: var(--accent);
    background: rgba(122, 162, 247, 0.15);
    border: 1px solid rgba(122, 162, 247, 0.3);
  }

  .label-badge.target {
    color: var(--success, #9ece6a);
    background: rgba(158, 206, 106, 0.15);
    border: 1px solid rgba(158, 206, 106, 0.3);
  }

  .sql-preview {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vs {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .stats-bar {
    display: flex;
    gap: 4px;
  }

  .stat-chip {
    padding: 2px 8px;
    font-size: 10px;
    font-family: var(--font-sans);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 10px;
  }

  .stat-chip.active {
    background: var(--bg-active);
    color: var(--text-primary);
    border-color: var(--accent);
  }

  .stat-chip.identical.active { color: var(--text-secondary); }
  .stat-chip.changed.active { color: var(--warning, #e0af68); border-color: var(--warning, #e0af68); }
  .stat-chip.source-only.active { color: var(--error); border-color: var(--error); }
  .stat-chip.target-only.active { color: var(--success, #9ece6a); border-color: var(--success, #9ece6a); }

  .match-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .match-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    margin-right: 4px;
  }

  .match-chip {
    padding: 1px 6px;
    font-size: 10px;
    font-family: var(--font-mono);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .match-chip.active {
    background: rgba(122, 162, 247, 0.15);
    color: var(--accent);
    border-color: var(--accent);
  }

  .match-hint {
    font-size: 10px;
    color: var(--text-muted);
    font-style: italic;
    margin-left: 4px;
  }

  .compare-grid {
    overflow: auto;
    flex: 1;
  }

  .scroll-spacer {
    position: relative;
  }

  .diff-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    table-layout: auto;
  }

  .diff-table th {
    padding: 6px 10px;
    text-align: left;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    position: sticky;
    top: 0;
    z-index: 1;
    white-space: nowrap;
  }

  .col-side {
    font-size: 8px;
    font-weight: 400;
    opacity: 0.5;
    margin-left: 4px;
    text-transform: lowercase;
  }

  .status-col {
    width: 80px;
  }

  .diff-table td {
    padding: 4px 10px;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-secondary);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .status-cell {
    width: 80px;
  }

  .status-badge {
    padding: 1px 5px;
    font-size: 9px;
    font-weight: 600;
    border-radius: var(--radius-sm);
  }

  .status-badge.identical { color: var(--text-muted); background: rgba(69, 71, 90, 0.2); }
  .status-badge.changed { color: var(--warning, #e0af68); background: rgba(224, 175, 104, 0.12); }
  .status-badge.source-only { color: var(--error); background: rgba(243, 139, 168, 0.12); }
  .status-badge.target-only { color: var(--success, #9ece6a); background: rgba(158, 206, 106, 0.12); }

  .changed-cell {
    background: rgba(224, 175, 104, 0.12) !important;
    color: var(--warning, #e0af68) !important;
  }

  tr.row-source-only { background: rgba(243, 139, 168, 0.04); }
  tr.row-target-only { background: rgba(158, 206, 106, 0.04); }
  tr.row-source-only td { color: var(--error); }
  tr.row-target-only td { color: var(--success, #9ece6a); }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }
</style>
