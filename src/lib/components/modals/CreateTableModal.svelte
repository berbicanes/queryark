<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import * as tauri from '$lib/services/tauri';
  import { generateCreateTable, getCommonDataTypes, type ColumnSpec } from '$lib/utils/ddlGenerator';

  let ctx = $derived(uiStore.createTableContext);
  let dbType = $derived(ctx?.dbType ?? 'PostgreSQL');
  let dataTypes = $derived(getCommonDataTypes(dbType));

  let tableName = $state('');
  let columns = $state<ColumnSpec[]>([
    { name: '', dataType: 'TEXT', nullable: true, defaultValue: '', isPrimaryKey: false },
  ]);

  // Update default type when dataTypes changes
  $effect(() => {
    if (dataTypes.length > 0 && columns.length === 1 && !columns[0].name) {
      columns[0].dataType = dataTypes[0];
    }
  });
  let isExecuting = $state(false);
  let error = $state<string | null>(null);

  let ddlPreview = $derived.by(() => {
    if (!ctx || !tableName.trim() || columns.length === 0) return '';
    const validCols = columns.filter(c => c.name.trim());
    if (validCols.length === 0) return '';
    return generateCreateTable(dbType, ctx.schema, tableName.trim(), validCols);
  });

  function addColumn() {
    const defaultType = getCommonDataTypes(dbType)[0] ?? 'TEXT';
    columns = [...columns, { name: '', dataType: defaultType, nullable: true, defaultValue: '', isPrimaryKey: false }];
  }

  function removeColumn(index: number) {
    columns = columns.filter((_, i) => i !== index);
  }

  async function handleCreate() {
    if (!ctx || !ddlPreview) return;
    isExecuting = true;
    error = null;
    try {
      await tauri.executeQuery(ctx.connectionId, ddlPreview);
      uiStore.showSuccess(`Table "${tableName}" created successfully`);
      // Refresh schema tree
      schemaStore.clearConnection(ctx.connectionId);
      handleClose();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isExecuting = false;
    }
  }

  function handleClose() {
    uiStore.showCreateTableModal = false;
    uiStore.createTableContext = null;
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card create-table-modal">
    <div class="modal-header">
      <h2>Create Table</h2>
      <button class="close-btn" onclick={handleClose}>&times;</button>
    </div>

    <div class="modal-body">
      <div class="form-row">
        <label>Schema <input type="text" class="form-input" value={ctx?.schema ?? ''} disabled /></label>
      </div>
      <div class="form-row">
        <label>Table Name <input type="text" class="form-input" bind:value={tableName} placeholder="new_table" /></label>
      </div>

      <div class="columns-section">
        <div class="columns-header">
          <span>Columns</span>
          <button class="add-btn" onclick={addColumn}>+ Add Column</button>
        </div>
        <div class="columns-list">
          {#each columns as col, i}
            <div class="column-row">
              <input type="text" class="col-input name" bind:value={col.name} placeholder="column_name" />
              <select class="col-input type" bind:value={col.dataType}>
                {#each dataTypes as dt}
                  <option value={dt}>{dt}</option>
                {/each}
              </select>
              <label class="col-check">
                <input type="checkbox" bind:checked={col.nullable} /> NULL
              </label>
              <label class="col-check">
                <input type="checkbox" bind:checked={col.isPrimaryKey} /> PK
              </label>
              <input type="text" class="col-input default" bind:value={col.defaultValue} placeholder="default" />
              <button class="remove-btn" onclick={() => removeColumn(i)} title="Remove column">&times;</button>
            </div>
          {/each}
        </div>
      </div>

      {#if ddlPreview}
        <div class="preview-section">
          <span class="preview-label">DDL Preview</span>
          <pre class="ddl-preview">{ddlPreview}</pre>
        </div>
      {/if}

      {#if error}
        <div class="error-msg">{error}</div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn" onclick={handleClose}>Cancel</button>
      <button class="btn btn-primary" onclick={handleCreate} disabled={isExecuting || !ddlPreview}>
        {isExecuting ? 'Creating...' : 'Create Table'}
      </button>
    </div>
  </div>
</div>

<style>
  .create-table-modal {
    width: 640px;
    max-height: 80vh;
    overflow-y: auto;
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
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

  .form-input:disabled {
    opacity: 0.5;
  }

  .columns-section {
    margin-top: 12px;
  }

  .columns-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .add-btn {
    padding: 2px 8px;
    font-size: 11px;
    border: 1px solid var(--border-color);
    background: none;
    color: var(--accent);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .add-btn:hover {
    background: var(--bg-hover);
  }

  .columns-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 240px;
    overflow-y: auto;
  }

  .column-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .col-input {
    padding: 4px 6px;
    font-size: 11px;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
  }

  .col-input.name {
    width: 130px;
  }

  .col-input.type {
    width: 140px;
  }

  .col-input.default {
    width: 80px;
  }

  .col-check {
    display: flex;
    align-items: center;
    gap: 2px;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    cursor: pointer;
  }

  .col-check input {
    accent-color: var(--accent);
  }

  .remove-btn {
    padding: 2px 6px;
    border: none;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 14px;
  }

  .remove-btn:hover {
    color: var(--error);
  }

  .preview-section {
    margin-top: 12px;
  }

  .preview-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    margin-bottom: 4px;
    display: block;
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
    max-height: 120px;
    overflow-y: auto;
    margin: 0;
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
