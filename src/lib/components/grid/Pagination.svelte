<script lang="ts">
  import { formatRowCount } from '$lib/utils/formatters';

  let { currentPage, totalRows, pageSize, onPageChange, onPageSizeChange }: {
    currentPage: number;
    totalRows: number;
    pageSize: number;
    onPageChange: (page: number) => void;
    onPageSizeChange: (size: number) => void;
  } = $props();

  let totalPages = $derived(Math.max(1, Math.ceil(totalRows / pageSize)));
  let startRow = $derived((currentPage - 1) * pageSize + 1);
  let endRow = $derived(Math.min(currentPage * pageSize, totalRows));
  let canPrevious = $derived(currentPage > 1);
  let canNext = $derived(currentPage < totalPages);

  const pageSizes = [25, 50, 100, 250];

  function handlePrevious() {
    if (canPrevious) {
      onPageChange(currentPage - 1);
    }
  }

  function handleNext() {
    if (canNext) {
      onPageChange(currentPage + 1);
    }
  }

  function handleFirst() {
    if (canPrevious) {
      onPageChange(1);
    }
  }

  function handleLast() {
    if (canNext) {
      onPageChange(totalPages);
    }
  }

  function handlePageSizeChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    onPageSizeChange(parseInt(target.value));
  }
</script>

<div class="pagination">
  <div class="pagination-left">
    <span class="page-size-label">Rows per page:</span>
    <select
      class="page-size-select"
      value={pageSize}
      onchange={handlePageSizeChange}
    >
      {#each pageSizes as size}
        <option value={size}>{size}</option>
      {/each}
    </select>
  </div>

  <div class="pagination-center">
    <span class="range-info">
      {#if totalRows > 0}
        {startRow.toLocaleString()}-{endRow.toLocaleString()} of {formatRowCount(totalRows)}
      {:else}
        0 rows
      {/if}
    </span>
  </div>

  <div class="pagination-right">
    <button
      class="page-btn"
      onclick={handleFirst}
      disabled={!canPrevious}
      title="First page"
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M11 12L7 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M7 12L3 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <button
      class="page-btn"
      onclick={handlePrevious}
      disabled={!canPrevious}
      title="Previous page"
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M10 12L6 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>

    <span class="page-indicator">
      Page {currentPage} of {totalPages}
    </span>

    <button
      class="page-btn"
      onclick={handleNext}
      disabled={!canNext}
      title="Next page"
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <button
      class="page-btn"
      onclick={handleLast}
      disabled={!canNext}
      title="Last page"
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M5 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M9 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 11px;
    color: var(--text-secondary);
    user-select: none;
    flex-shrink: 0;
  }

  .pagination-left,
  .pagination-center,
  .pagination-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .page-size-label {
    color: var(--text-muted);
  }

  .page-size-select {
    padding: 2px 4px;
    font-size: 11px;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .range-info {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.3px;
  }

  .page-indicator {
    font-size: 11px;
    color: var(--text-muted);
    padding: 0 4px;
  }

  .page-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    background: none;
    border: none;
    cursor: pointer;
    transition: background var(--transition-micro, 80ms ease), color var(--transition-micro, 80ms ease);
    padding: 0;
  }

  .page-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .page-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }
</style>
