<script lang="ts">
  import { onMount } from 'svelte';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, CellValue } from '$lib/types/query';
  import * as graphService from '$lib/services/graphService';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';

  let { tab }: { tab: Tab } = $props();

  let labels = $state<string[]>([]);
  let relationshipTypes = $state<string[]>([]);
  let selectedLabel = $state<string | null>(null);
  let nodeResult = $state<QueryResponse | null>(null);
  let isLoading = $state(false);
  let page = $state(0);
  let pageSize = $state(50);

  let container = $derived(tab.container ?? '');

  async function loadLabels() {
    const [lbls, rels] = await Promise.all([
      graphService.getLabels(tab.connectionId),
      graphService.getRelationshipTypes(tab.connectionId),
    ]);
    labels = lbls;
    relationshipTypes = rels;

    // Auto-select first label or the item from the tab
    if (tab.item && labels.includes(tab.item)) {
      selectLabel(tab.item);
    } else if (labels.length > 0) {
      selectLabel(labels[0]);
    }
  }

  async function selectLabel(label: string) {
    selectedLabel = label;
    page = 0;
    await loadNodes();
  }

  async function loadNodes() {
    if (!selectedLabel) return;
    isLoading = true;
    try {
      nodeResult = await graphService.getNodes(tab.connectionId, selectedLabel, pageSize, page * pageSize);
    } finally {
      isLoading = false;
    }
  }

  function handlePrevPage() {
    if (page > 0) {
      page--;
      loadNodes();
    }
  }

  function handleNextPage() {
    if (nodeResult && nodeResult.rows.length >= pageSize) {
      page++;
      loadNodes();
    }
  }

  onMount(() => {
    loadLabels();
  });
</script>

<div class="graph-tab">
  <div class="graph-sidebar">
    <div class="sidebar-section">
      <div class="section-header">Labels</div>
      {#each labels as label}
        <button
          class="sidebar-item"
          class:selected={selectedLabel === label}
          onclick={() => selectLabel(label)}
        >
          <span class="label-dot"></span>
          {label}
        </button>
      {/each}
    </div>

    {#if relationshipTypes.length > 0}
      <div class="sidebar-section">
        <div class="section-header">Relationships</div>
        {#each relationshipTypes as rel}
          <div class="sidebar-item rel-item">
            <span class="rel-arrow">&rarr;</span>
            {rel}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="graph-content">
    {#if isLoading}
      <div class="loading-state">
        <span class="spinner"></span>
        Loading nodes...
      </div>
    {:else if nodeResult && nodeResult.rows.length > 0}
      <DataGrid columns={nodeResult.columns} rows={nodeResult.rows} />
      <div class="graph-pagination">
        <button class="btn btn-sm" onclick={handlePrevPage} disabled={page === 0}>Prev</button>
        <span class="text-muted">Page {page + 1}</span>
        <button class="btn btn-sm" onclick={handleNextPage} disabled={nodeResult.rows.length < pageSize}>Next</button>
      </div>
    {:else if selectedLabel}
      <div class="empty-state">
        <span class="text-muted">No nodes found for label "{selectedLabel}"</span>
      </div>
    {:else}
      <div class="empty-state">
        <span class="text-muted">Select a label to view nodes</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .graph-tab {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .graph-sidebar {
    width: 200px;
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .sidebar-section {
    padding: 4px 0;
  }

  .section-header {
    padding: 6px 12px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-color);
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 5px 12px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    border: none;
    background: none;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .sidebar-item:hover {
    background: var(--bg-hover);
  }

  .sidebar-item.selected {
    background: var(--bg-active);
    border-left: 2px solid var(--accent);
    padding-left: 10px;
  }

  .rel-item {
    cursor: default;
    color: var(--text-secondary);
  }

  .label-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
  }

  .rel-arrow {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .graph-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .graph-pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 12px;
    flex-shrink: 0;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
