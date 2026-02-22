<script lang="ts">
  import { onMount } from 'svelte';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, CellValue } from '$lib/types/query';
  import * as documentService from '$lib/services/documentService';
  import { uiStore } from '$lib/stores/ui.svelte';
  import JsonViewer from '$lib/components/viewers/JsonViewer.svelte';

  let { tab }: { tab: Tab } = $props();

  let result = $state<QueryResponse | null>(null);
  let isLoading = $state(false);
  let totalCount = $state(0);
  let page = $state(0);
  let pageSize = $state(50);
  let selectedDocIndex = $state<number | null>(null);

  // Insert dialog
  let showInsert = $state(false);
  let insertJson = $state('{\n  \n}');
  let isInserting = $state(false);

  // Delete dialog
  let deleteFilter = $state('');

  let container = $derived(tab.container ?? '');
  let item = $derived(tab.item ?? '');

  async function loadData() {
    if (!container || !item) return;
    isLoading = true;
    try {
      const [data, count] = await Promise.all([
        documentService.loadDocuments(tab.connectionId, container, item, pageSize, page * pageSize),
        documentService.getDocumentCount(tab.connectionId, container, item),
      ]);
      result = data;
      totalCount = count;
    } finally {
      isLoading = false;
    }
  }

  function cellToJson(cell: CellValue): unknown {
    if (cell.type === 'Null') return null;
    if (cell.type === 'Json') {
      try { return JSON.parse(cell.value); } catch { return cell.value; }
    }
    if ('value' in cell) return cell.value;
    return null;
  }

  function getDocumentPreview(row: CellValue[]): string {
    // Show first few fields as preview
    if (!result || row.length === 0) return '{}';
    const obj: Record<string, unknown> = {};
    result.columns.forEach((col, i) => {
      if (i < 4) obj[col.name] = cellToJson(row[i]);
    });
    return JSON.stringify(obj).slice(0, 120) + (Object.keys(obj).length < result.columns.length ? '...' : '');
  }

  function getSelectedDoc(): unknown | null {
    if (selectedDocIndex === null || !result) return null;
    const row = result.rows[selectedDocIndex];
    if (!row) return null;
    const obj: Record<string, unknown> = {};
    result.columns.forEach((col, i) => {
      obj[col.name] = cellToJson(row[i]);
    });
    return obj;
  }

  async function handleInsert() {
    if (!insertJson.trim()) return;
    isInserting = true;
    try {
      const id = await documentService.insertDocument(tab.connectionId, container, item, insertJson);
      if (id) {
        showInsert = false;
        insertJson = '{\n  \n}';
        await loadData();
      }
    } finally {
      isInserting = false;
    }
  }

  function handlePrevPage() {
    if (page > 0) {
      page--;
      loadData();
    }
  }

  function handleNextPage() {
    if ((page + 1) * pageSize < totalCount) {
      page++;
      loadData();
    }
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="document-tab">
  <div class="doc-toolbar">
    <span class="doc-title">{item}</span>
    <span class="text-muted">({totalCount} documents)</span>
    <div class="toolbar-spacer"></div>
    <button class="btn btn-sm" onclick={() => showInsert = !showInsert}>
      + Insert
    </button>
    <button class="btn btn-sm" onclick={loadData}>
      Refresh
    </button>
  </div>

  {#if showInsert}
    <div class="insert-panel">
      <textarea
        class="insert-editor"
        bind:value={insertJson}
        placeholder="key: value"
        rows="6"
      ></textarea>
      <div class="insert-actions">
        <button class="btn btn-sm" onclick={() => showInsert = false}>Cancel</button>
        <button class="btn btn-sm btn-primary" onclick={handleInsert} disabled={isInserting}>
          {isInserting ? 'Inserting...' : 'Insert Document'}
        </button>
      </div>
    </div>
  {/if}

  <div class="doc-content">
    <div class="doc-list">
      {#if isLoading}
        <div class="loading-state">
          <span class="spinner"></span>
          Loading...
        </div>
      {:else if result && result.rows.length > 0}
        {#each result.rows as row, idx}
          <button
            class="doc-item"
            class:selected={selectedDocIndex === idx}
            onclick={() => selectedDocIndex = idx}
          >
            <span class="doc-preview text-mono">{getDocumentPreview(row)}</span>
          </button>
        {/each}
      {:else}
        <div class="empty-state">
          <span class="text-muted">No documents found</span>
        </div>
      {/if}
    </div>

    <div class="doc-detail">
      {#if selectedDocIndex !== null}
        {@const doc = getSelectedDoc()}
        {#if doc}
          <JsonViewer data={doc} />
        {/if}
      {:else}
        <div class="empty-state">
          <span class="text-muted">Select a document to view</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="doc-pagination">
    <button class="btn btn-sm" onclick={handlePrevPage} disabled={page === 0}>Prev</button>
    <span class="text-muted">
      Page {page + 1} of {Math.max(1, Math.ceil(totalCount / pageSize))}
    </span>
    <button class="btn btn-sm" onclick={handleNextPage} disabled={(page + 1) * pageSize >= totalCount}>Next</button>
  </div>
</div>

<style>
  .document-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .doc-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .doc-title {
    font-weight: 600;
    font-size: 13px;
  }

  .toolbar-spacer {
    flex: 1;
  }

  .insert-panel {
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .insert-editor {
    width: 100%;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 8px;
    resize: vertical;
  }

  .insert-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }

  .doc-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .doc-list {
    width: 40%;
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
  }

  .doc-detail {
    flex: 1;
    overflow: auto;
    padding: 8px 12px;
  }

  .doc-item {
    display: block;
    width: 100%;
    padding: 6px 12px;
    text-align: left;
    border: none;
    background: none;
    color: var(--text-primary);
    font-size: 11px;
    cursor: pointer;
    border-bottom: 1px solid var(--border-color);
    transition: background var(--transition-fast);
  }

  .doc-item:hover {
    background: var(--bg-hover);
  }

  .doc-item.selected {
    background: var(--bg-active);
    border-left: 2px solid var(--accent);
  }

  .doc-preview {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .doc-pagination {
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
  }
</style>
