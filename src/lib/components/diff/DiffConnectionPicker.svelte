<script lang="ts">
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import type { SchemaInfo, TableInfo } from '$lib/types/schema';

  let {
    sourceConnectionId = $bindable(''),
    sourceSchema = $bindable(''),
    sourceTable = $bindable(''),
    targetConnectionId = $bindable(''),
    targetSchema = $bindable(''),
    targetTable = $bindable(''),
    oncompare,
    loading = false,
  }: {
    sourceConnectionId: string;
    sourceSchema: string;
    sourceTable: string;
    targetConnectionId: string;
    targetSchema: string;
    targetTable: string;
    oncompare: () => void;
    loading?: boolean;
  } = $props();

  let sourceSchemas = $state<SchemaInfo[]>([]);
  let sourceTables = $state<TableInfo[]>([]);
  let targetSchemas = $state<SchemaInfo[]>([]);
  let targetTables = $state<TableInfo[]>([]);

  let connectedConns = $derived(connectionStore.connections.filter(c => c.status === 'connected'));

  let canCompare = $derived(
    sourceConnectionId && sourceSchema && sourceTable &&
    targetConnectionId && targetSchema && targetTable && !loading
  );

  async function loadSourceSchemas() {
    if (!sourceConnectionId) { sourceSchemas = []; return; }
    sourceSchemas = schemaStore.getSchemas(sourceConnectionId);
    if (sourceSchemas.length === 0) {
      sourceSchemas = await schemaService.loadSchemas(sourceConnectionId);
    }
    sourceSchema = '';
    sourceTable = '';
    sourceTables = [];
  }

  async function loadSourceTables() {
    if (!sourceConnectionId || !sourceSchema) { sourceTables = []; return; }
    sourceTables = await schemaService.loadTables(sourceConnectionId, sourceSchema);
    sourceTable = '';
  }

  async function loadTargetSchemas() {
    if (!targetConnectionId) { targetSchemas = []; return; }
    targetSchemas = schemaStore.getSchemas(targetConnectionId);
    if (targetSchemas.length === 0) {
      targetSchemas = await schemaService.loadSchemas(targetConnectionId);
    }
    targetSchema = '';
    targetTable = '';
    targetTables = [];
  }

  async function loadTargetTables() {
    if (!targetConnectionId || !targetSchema) { targetTables = []; return; }
    targetTables = await schemaService.loadTables(targetConnectionId, targetSchema);
    targetTable = '';
  }
</script>

<div class="diff-picker">
  <div class="picker-row">
    <span class="picker-label">Source</span>
    <select
      class="picker-select"
      bind:value={sourceConnectionId}
      onchange={loadSourceSchemas}
    >
      <option value="">Connection...</option>
      {#each connectedConns as conn}
        <option value={conn.config.id}>{conn.config.name}</option>
      {/each}
    </select>
    <select
      class="picker-select"
      bind:value={sourceSchema}
      onchange={loadSourceTables}
      disabled={sourceSchemas.length === 0}
    >
      <option value="">Schema...</option>
      {#each sourceSchemas as s}
        <option value={s.name}>{s.name}</option>
      {/each}
    </select>
    <select
      class="picker-select"
      bind:value={sourceTable}
      disabled={sourceTables.length === 0}
    >
      <option value="">Table...</option>
      {#each sourceTables as t}
        <option value={t.name}>{t.name}</option>
      {/each}
    </select>
  </div>

  <div class="picker-row">
    <span class="picker-label">Target</span>
    <select
      class="picker-select"
      bind:value={targetConnectionId}
      onchange={loadTargetSchemas}
    >
      <option value="">Connection...</option>
      {#each connectedConns as conn}
        <option value={conn.config.id}>{conn.config.name}</option>
      {/each}
    </select>
    <select
      class="picker-select"
      bind:value={targetSchema}
      onchange={loadTargetTables}
      disabled={targetSchemas.length === 0}
    >
      <option value="">Schema...</option>
      {#each targetSchemas as s}
        <option value={s.name}>{s.name}</option>
      {/each}
    </select>
    <select
      class="picker-select"
      bind:value={targetTable}
      disabled={targetTables.length === 0}
    >
      <option value="">Table...</option>
      {#each targetTables as t}
        <option value={t.name}>{t.name}</option>
      {/each}
    </select>
  </div>

  <div class="picker-actions">
    <button
      class="compare-btn"
      disabled={!canCompare}
      onclick={oncompare}
    >
      {#if loading}
        <span class="btn-spinner"></span> Comparing...
      {:else}
        Compare
      {/if}
    </button>
  </div>
</div>

<style>
  .diff-picker {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .picker-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .picker-label {
    width: 50px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    flex-shrink: 0;
  }

  .picker-select {
    flex: 1;
    padding: 5px 8px;
    font-size: 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .picker-select:focus {
    border-color: var(--accent);
  }

  .picker-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .picker-actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 4px;
  }

  .compare-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 16px;
    font-size: 12px;
    font-weight: 500;
    color: var(--bg-primary);
    background: var(--accent);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .compare-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .compare-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
