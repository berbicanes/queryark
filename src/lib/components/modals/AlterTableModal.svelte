<script lang="ts">
  import { onMount } from 'svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import * as tauri from '$lib/services/tauri';
  import { generateAddColumn, generateDropColumn, getCommonDataTypes, type ColumnSpec } from '$lib/utils/ddlGenerator';
  import type { ColumnInfo } from '$lib/types/schema';

  let ctx = $derived(uiStore.alterTableContext);
  let dbType = $derived(ctx?.dbType ?? 'PostgreSQL');
  let dataTypes = $derived(getCommonDataTypes(dbType));

  let activeTab = $state<'add' | 'drop'>('add');
  let existingColumns = $state<ColumnInfo[]>([]);
  let isExecuting = $state(false);
  let error = $state<string | null>(null);

  // Add column state
  let newColumn = $state<ColumnSpec>({
    name: '', dataType: 'TEXT', nullable: true, defaultValue: '', isPrimaryKey: false,
  });

  // Drop column state
  let dropColumnName = $state('');

  let addDdl = $derived.by(() => {
    if (!ctx || !newColumn.name.trim()) return '';
    return generateAddColumn(dbType, ctx.schema, ctx.table, { ...newColumn, dataType: newColumn.dataType || dataTypes[0] });
  });

  let dropDdl = $derived.by(() => {
    if (!ctx || !dropColumnName) return '';
    return generateDropColumn(dbType, ctx.schema, ctx.table, dropColumnName);
  });

  let isSQLite = $derived(dbType === 'SQLite');

  async function loadColumns() {
    if (!ctx) return;
    try {
      existingColumns = await tauri.getColumns(ctx.connectionId, ctx.schema, ctx.table);
    } catch {
      existingColumns = [];
    }
  }

  async function handleExecute() {
    if (!ctx) return;
    const sql = activeTab === 'add' ? addDdl : dropDdl;
    if (!sql) return;

    isExecuting = true;
    error = null;
    try {
      await tauri.executeQuery(ctx.connectionId, sql);
      uiStore.showSuccess(activeTab === 'add' ? 'Column added' : 'Column dropped');
      schemaStore.clearConnection(ctx.connectionId);
      await loadColumns();
      if (activeTab === 'add') {
        newColumn = { name: '', dataType: dataTypes[0] ?? 'TEXT', nullable: true, defaultValue: '', isPrimaryKey: false };
      } else {
        dropColumnName = '';
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isExecuting = false;
    }
  }

  function handleClose() {
    uiStore.showAlterTableModal = false;
    uiStore.alterTableContext = null;
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  onMount(() => {
    loadColumns();
  });

  // Reset datatype when dbType changes
  $effect(() => {
    const types = dataTypes;
    if (types.length > 0 && !types.includes(newColumn.dataType)) {
      newColumn.dataType = types[0];
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card alter-table-modal">
    <div class="modal-header">
      <h2>Alter Table: {ctx?.schema}.{ctx?.table}</h2>
      <button class="close-btn" onclick={handleClose}>&times;</button>
    </div>

    <div class="modal-body">
      <div class="tab-bar">
        <button class="tab-btn" class:active={activeTab === 'add'} onclick={() => { activeTab = 'add'; error = null; }}>Add Column</button>
        <button
          class="tab-btn"
          class:active={activeTab === 'drop'}
          onclick={() => { activeTab = 'drop'; error = null; }}
          disabled={isSQLite}
          title={isSQLite ? 'SQLite does not support DROP COLUMN' : ''}
        >Drop Column</button>
      </div>

      {#if activeTab === 'add'}
        <div class="form-section">
          <div class="form-row">
            <label>Name <input type="text" class="form-input" bind:value={newColumn.name} placeholder="column_name" /></label>
          </div>
          <div class="form-row">
            <label>Type <select class="form-input" bind:value={newColumn.dataType}>
              {#each dataTypes as dt}
                <option value={dt}>{dt}</option>
              {/each}
            </select></label>
          </div>
          <div class="form-row">
            <label>Nullable <input type="checkbox" bind:checked={newColumn.nullable} /></label>
          </div>
          <div class="form-row">
            <label>Default <input type="text" class="form-input" bind:value={newColumn.defaultValue} placeholder="optional" /></label>
          </div>
          {#if addDdl}
            <pre class="ddl-preview">{addDdl}</pre>
          {/if}
        </div>
      {:else}
        <div class="form-section">
          <div class="form-row">
            <label>Column <select class="form-input" bind:value={dropColumnName}>
              <option value="">-- Select column --</option>
              {#each existingColumns as col}
                <option value={col.name}>{col.name} ({col.data_type})</option>
              {/each}
            </select></label>
          </div>
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
        disabled={isExecuting || (activeTab === 'add' ? !addDdl : !dropDdl)}
      >
        {isExecuting ? 'Executing...' : 'Execute'}
      </button>
    </div>
  </div>
</div>

<style>
  .alter-table-modal {
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
  .tab-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
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
    width: 70px;
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
