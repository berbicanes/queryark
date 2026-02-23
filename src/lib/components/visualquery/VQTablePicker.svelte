<script lang="ts">
  import { schemaStore } from '$lib/stores/schema.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import type { SchemaInfo, TableInfo } from '$lib/types/schema';

  let {
    connectionId,
    onaddtable,
  }: {
    connectionId: string;
    onaddtable: (schema: string, table: string) => void;
  } = $props();

  let schemas = $derived(schemaStore.getSchemas(connectionId));
  let selectedSchema = $state('');
  let tables = $state<TableInfo[]>([]);
  let searchQuery = $state('');

  $effect(() => {
    if (schemas.length > 0 && !selectedSchema) {
      selectedSchema = schemas[0].name;
    }
  });

  $effect(() => {
    if (selectedSchema) {
      loadTables();
    }
  });

  async function loadTables() {
    tables = await schemaService.loadTables(connectionId, selectedSchema);
  }

  let filteredTables = $derived(
    searchQuery
      ? tables.filter(t => t.name.toLowerCase().includes(searchQuery.toLowerCase()))
      : tables
  );

  function handleDblClick(table: TableInfo) {
    onaddtable(selectedSchema, table.name);
  }
</script>

<div class="vq-table-picker">
  <div class="picker-header">
    <select class="schema-select" bind:value={selectedSchema}>
      {#each schemas as s}
        <option value={s.name}>{s.name}</option>
      {/each}
    </select>
    <input
      class="table-search"
      type="text"
      placeholder="Filter tables..."
      bind:value={searchQuery}
    />
  </div>
  <div class="table-list">
    {#each filteredTables as table}
      <button
        class="table-item"
        ondblclick={() => handleDblClick(table)}
        title="Double-click to add"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <rect x="2" y="2" width="12" height="12" rx="2" stroke="currentColor" stroke-width="1.2" fill="none"/>
          <path d="M2 6h12" stroke="currentColor" stroke-width="1"/>
        </svg>
        <span class="table-name">{table.name}</span>
      </button>
    {/each}
    {#if filteredTables.length === 0}
      <div class="empty-list">No tables found</div>
    {/if}
  </div>
</div>

<style>
  .vq-table-picker {
    width: 180px;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-color);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .picker-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .schema-select {
    width: 100%;
    padding: 4px 6px;
    font-size: 11px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .table-search {
    width: 100%;
    padding: 4px 6px;
    font-size: 11px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .table-search:focus, .schema-select:focus {
    border-color: var(--accent);
  }

  .table-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .table-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--text-secondary);
    background: none;
    border: none;
    text-align: left;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .table-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .table-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-list {
    padding: 12px;
    text-align: center;
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
