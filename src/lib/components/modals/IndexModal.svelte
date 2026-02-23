<script lang="ts">
  import { onMount } from 'svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import * as tauri from '$lib/services/tauri';
  import { generateCreateIndex, generateDropIndex } from '$lib/utils/ddlGenerator';
  import type { ColumnInfo, IndexInfo } from '$lib/types/schema';

  let ctx = $derived(uiStore.indexModalContext);
  let dbType = $derived(ctx?.dbType ?? 'PostgreSQL');

  let activeTab = $state<'create' | 'drop'>('create');
  let existingColumns = $state<ColumnInfo[]>([]);
  let existingIndexes = $state<IndexInfo[]>([]);
  let isExecuting = $state(false);
  let error = $state<string | null>(null);

  // Create index state
  let indexName = $state('');
  let selectedColumns = $state<string[]>([]);
  let isUnique = $state(false);

  // Drop index state
  let dropIndexName = $state('');

  let createDdl = $derived.by(() => {
    if (!ctx || !indexName.trim() || selectedColumns.length === 0) return '';
    return generateCreateIndex(dbType, ctx.schema, ctx.table, indexName.trim(), selectedColumns, isUnique);
  });

  let dropDdl = $derived.by(() => {
    if (!ctx || !dropIndexName) return '';
    return generateDropIndex(dbType, ctx.schema, dropIndexName, ctx.table);
  });

  async function loadMeta() {
    if (!ctx) return;
    try {
      existingColumns = await tauri.getColumns(ctx.connectionId, ctx.schema, ctx.table);
      existingIndexes = await tauri.getIndexes(ctx.connectionId, ctx.schema, ctx.table);
    } catch {
      existingColumns = [];
      existingIndexes = [];
    }
  }

  function toggleColumn(colName: string) {
    if (selectedColumns.includes(colName)) {
      selectedColumns = selectedColumns.filter(c => c !== colName);
    } else {
      selectedColumns = [...selectedColumns, colName];
    }
  }

  async function handleExecute() {
    if (!ctx) return;
    const sql = activeTab === 'create' ? createDdl : dropDdl;
    if (!sql) return;

    isExecuting = true;
    error = null;
    try {
      await tauri.executeQuery(ctx.connectionId, sql);
      uiStore.showSuccess(activeTab === 'create' ? 'Index created' : 'Index dropped');
      schemaStore.clearConnection(ctx.connectionId);
      await loadMeta();
      if (activeTab === 'create') {
        indexName = '';
        selectedColumns = [];
        isUnique = false;
      } else {
        dropIndexName = '';
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isExecuting = false;
    }
  }

  function handleClose() {
    uiStore.showIndexModal = false;
    uiStore.indexModalContext = null;
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  onMount(() => {
    loadMeta();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card index-modal">
    <div class="modal-header">
      <h2>Indexes: {ctx?.schema}.{ctx?.table}</h2>
      <button class="close-btn" onclick={handleClose}>&times;</button>
    </div>

    <div class="modal-body">
      <div class="tab-bar">
        <button class="tab-btn" class:active={activeTab === 'create'} onclick={() => { activeTab = 'create'; error = null; }}>Create Index</button>
        <button class="tab-btn" class:active={activeTab === 'drop'} onclick={() => { activeTab = 'drop'; error = null; }}>Drop Index</button>
      </div>

      {#if activeTab === 'create'}
        <div class="form-section">
          <div class="form-row">
            <label>Index Name <input type="text" class="form-input" bind:value={indexName} placeholder="idx_table_column" /></label>
          </div>
          <div class="form-row">
            <label>Unique <input type="checkbox" bind:checked={isUnique} /></label>
          </div>
          <div class="column-picker">
            <span class="picker-label">Columns (select in order):</span>
            <div class="column-chips">
              {#each existingColumns as col}
                {@const isSelected = selectedColumns.includes(col.name)}
                {@const order = selectedColumns.indexOf(col.name) + 1}
                <button
                  class="column-chip"
                  class:selected={isSelected}
                  onclick={() => toggleColumn(col.name)}
                >
                  {#if isSelected}<span class="chip-order">{order}</span>{/if}
                  {col.name}
                </button>
              {/each}
            </div>
          </div>
          {#if createDdl}
            <pre class="ddl-preview">{createDdl}</pre>
          {/if}
        </div>
      {:else}
        <div class="form-section">
          {#if existingIndexes.length === 0}
            <p class="no-indexes">No indexes found on this table.</p>
          {:else}
            <div class="form-row">
              <label>Index <select class="form-input" bind:value={dropIndexName}>
                <option value="">-- Select index --</option>
                {#each existingIndexes as idx}
                  <option value={idx.name}>
                    {idx.name} ({idx.columns.join(', ')}) {idx.is_unique ? '[unique]' : ''} {idx.is_primary ? '[PK]' : ''}
                  </option>
                {/each}
              </select></label>
            </div>
          {/if}
          {#if dropDdl}
            <pre class="ddl-preview">{dropDdl}</pre>
          {/if}
        </div>
      {/if}

      {#if error}
        <div class="error-msg">{error}</div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn" onclick={handleClose}>Cancel</button>
      <button
        class="btn btn-primary"
        onclick={handleExecute}
        disabled={isExecuting || (activeTab === 'create' ? !createDdl : !dropDdl)}
      >
        {isExecuting ? 'Executing...' : 'Execute'}
      </button>
    </div>
  </div>
</div>

<style>
  .index-modal {
    width: 520px;
  }

  .tab-bar {
    display: flex;
    gap: 0;
    margin-bottom: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .tab-btn {
    padding: 6px 16px;
    font-size: 12px;
    border: none;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    position: relative;
  }

  .tab-btn:hover { color: var(--text-primary); }
  .tab-btn.active { color: var(--accent); }
  .tab-btn.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--accent);
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .form-row label {
    width: 90px;
    font-size: 12px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .form-input {
    flex: 1;
    padding: 6px 8px;
    font-size: 12px;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
  }

  .column-picker {
    margin-top: 4px;
  }

  .picker-label {
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 4px;
    display: block;
  }

  .column-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .column-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    font-size: 11px;
    font-family: var(--font-mono);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .column-chip:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .column-chip.selected {
    background: rgba(137, 180, 250, 0.1);
    border-color: var(--accent);
    color: var(--accent);
  }

  .chip-order {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    font-size: 9px;
    font-weight: 700;
    background: var(--accent);
    color: var(--bg-primary);
    border-radius: 50%;
  }

  .no-indexes {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0;
  }

  .ddl-preview {
    padding: 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    white-space: pre-wrap;
    margin: 4px 0 0 0;
  }

  .error-msg {
    margin-top: 8px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--error);
    background: rgba(243, 139, 168, 0.05);
    border-radius: var(--radius-sm);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    padding: 0 4px;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }
</style>
