<script lang="ts">
  import type { DataDiffResult, RowDiff, RowDiffStatus } from '$lib/types/diff';
  import type { CellValue } from '$lib/types/query';

  let {
    result,
    filter = 'all',
  }: {
    result: DataDiffResult;
    filter: 'all' | RowDiffStatus;
  } = $props();

  const ROW_HEIGHT = 32;
  const VISIBLE_ROWS = 20;

  let scrollTop = $state(0);
  let containerEl: HTMLDivElement;

  let filteredRows = $derived(
    filter === 'all' ? result.rows : result.rows.filter(r => r.status === filter)
  );

  let startIdx = $derived(Math.floor(scrollTop / ROW_HEIGHT));
  let visibleRows = $derived(filteredRows.slice(startIdx, startIdx + VISIBLE_ROWS + 2));
  let totalHeight = $derived(filteredRows.length * ROW_HEIGHT);
  let offsetY = $derived(startIdx * ROW_HEIGHT);

  // Non-PK column indices
  let dataColIndices = $derived(
    result.columns
      .map((_, i) => i)
      .filter(i => !result.pkColumns.includes(result.columns[i]))
  );

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

  function rowClass(row: RowDiff): string {
    return `row-${row.status}`;
  }

  function handleScroll() {
    scrollTop = containerEl.scrollTop;
  }
</script>

<div class="data-diff-grid" bind:this={containerEl} onscroll={handleScroll}>
  <div class="scroll-spacer" style="height: {totalHeight}px">
    <table class="diff-table" style="transform: translateY({offsetY}px)">
      <thead>
        <tr>
          {#each result.pkColumns as pk}
            <th class="pk-col">{pk} (PK)</th>
          {/each}
          {#each dataColIndices as colIdx}
            <th class="source-col">Source: {result.columns[colIdx]}</th>
            <th class="target-col">Target: {result.columns[colIdx]}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each visibleRows as row}
          <tr class={rowClass(row)} style="height: {ROW_HEIGHT}px">
            {#each row.pkValues as pk}
              <td class="pk-cell">{pk}</td>
            {/each}
            {#each dataColIndices as colIdx}
              {@const isChanged = row.changedColumns.includes(colIdx)}
              <td class:changed-cell={isChanged}>
                {row.sourceRow ? cellText(row.sourceRow[colIdx]) : '—'}
              </td>
              <td class:changed-cell={isChanged}>
                {row.targetRow ? cellText(row.targetRow[colIdx]) : '—'}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .data-diff-grid {
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

  .pk-col {
    background: var(--bg-tertiary, var(--bg-hover)) !important;
  }

  .source-col { color: var(--text-secondary) !important; }
  .target-col { color: var(--text-secondary) !important; }

  .diff-table td {
    padding: 4px 10px;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-secondary);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
    font-size: 11px;
  }

  .pk-cell {
    font-weight: 500;
    color: var(--text-primary) !important;
    background: rgba(122, 162, 247, 0.04);
  }

  .changed-cell {
    background: rgba(224, 175, 104, 0.12) !important;
    color: var(--warning, #e0af68) !important;
  }

  tr.row-added { background: rgba(158, 206, 106, 0.06); }
  tr.row-removed { background: rgba(243, 139, 168, 0.06); }
  tr.row-changed { background: transparent; }
  tr.row-identical { background: transparent; }

  tr.row-added td { color: var(--success, #9ece6a); }
  tr.row-removed td { color: var(--error, #f38ba8); }
</style>
