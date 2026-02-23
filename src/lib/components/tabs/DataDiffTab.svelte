<script lang="ts">
  import type { Tab } from '$lib/types/tabs';
  import type { DataDiffResult, RowDiffStatus } from '$lib/types/diff';
  import * as schemaService from '$lib/services/schemaService';
  import * as tauri from '$lib/services/tauri';
  import { computeDataDiff } from '$lib/utils/dataDiff';
  import { quoteIdentifier } from '$lib/utils/sqlHelpers';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import DiffConnectionPicker from '$lib/components/diff/DiffConnectionPicker.svelte';
  import DataDiffGrid from '$lib/components/diff/DataDiffGrid.svelte';

  let { tab }: { tab: Tab } = $props();

  let sourceConnectionId = $state(tab.connectionId);
  let sourceSchema = $state(tab.schema ?? '');
  let sourceTable = $state(tab.table ?? '');
  let targetConnectionId = $state(tab.diffTargetConnectionId ?? '');
  let targetSchema = $state(tab.diffTargetSchema ?? '');
  let targetTable = $state(tab.diffTargetTable ?? '');

  let loading = $state(false);
  let diffResult = $state<DataDiffResult | null>(null);
  let filter = $state<'all' | RowDiffStatus>('all');
  let error = $state<string | null>(null);
  let noPK = $state(false);

  function getDbType(connId: string) {
    const conn = connectionStore.connections.find(c => c.config.id === connId);
    return conn?.config.db_type ?? 'PostgreSQL';
  }

  async function handleCompare() {
    loading = true;
    diffResult = null;
    error = null;
    noPK = false;

    try {
      // Load columns to find PK
      const columns = await schemaService.loadColumns(sourceConnectionId, sourceSchema, sourceTable);
      const pkColumns = columns.filter(c => c.is_primary_key);

      if (pkColumns.length === 0) {
        noPK = true;
        loading = false;
        return;
      }

      const pkIndices = pkColumns.map(pk => columns.findIndex(c => c.name === pk.name));
      const columnNames = columns.map(c => c.name);

      // Build ORDER BY with PK columns
      const srcDbType = getDbType(sourceConnectionId);
      const tgtDbType = getDbType(targetConnectionId);
      const srcOrderBy = pkColumns.map(c => quoteIdentifier(c.name, srcDbType)).join(', ');
      const tgtOrderBy = pkColumns.map(c => quoteIdentifier(c.name, tgtDbType)).join(', ');

      const srcQualified = `${quoteIdentifier(sourceSchema, srcDbType)}.${quoteIdentifier(sourceTable, srcDbType)}`;
      const tgtQualified = `${quoteIdentifier(targetSchema, tgtDbType)}.${quoteIdentifier(targetTable, tgtDbType)}`;

      // Fetch data from both sides (limit to 5000 rows per side)
      const [sourceData, targetData] = await Promise.all([
        tauri.executeQuery(sourceConnectionId, `SELECT * FROM ${srcQualified} ORDER BY ${srcOrderBy}`, undefined, undefined, 5000),
        tauri.executeQuery(targetConnectionId, `SELECT * FROM ${tgtQualified} ORDER BY ${tgtOrderBy}`, undefined, undefined, 5000),
      ]);

      diffResult = computeDataDiff(sourceData.rows, targetData.rows, pkIndices, columnNames);
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="datadiff-tab">
  <DiffConnectionPicker
    bind:sourceConnectionId
    bind:sourceSchema
    bind:sourceTable
    bind:targetConnectionId
    bind:targetSchema
    bind:targetTable
    oncompare={handleCompare}
    {loading}
  />

  {#if noPK}
    <div class="warning-bar">
      No primary key found on the source table. Data diff requires a primary key to match rows.
    </div>
  {/if}

  {#if error}
    <div class="error-bar">{error}</div>
  {/if}

  {#if diffResult}
    <div class="diff-header">
      <div class="diff-summary-bar">
        <span class="summary-item">Total: {diffResult.rows.length} rows</span>
        <span class="summary-item added">+{diffResult.summary.added} added</span>
        <span class="summary-item removed">-{diffResult.summary.removed} removed</span>
        <span class="summary-item changed">{diffResult.summary.changed} changed</span>
        <span class="summary-item identical">{diffResult.summary.identical} identical</span>
      </div>
      <div class="filter-btns">
        <button class="filter-btn" class:active={filter === 'all'} onclick={() => filter = 'all'}>All</button>
        <button class="filter-btn" class:active={filter === 'added'} onclick={() => filter = 'added'}>Added</button>
        <button class="filter-btn" class:active={filter === 'removed'} onclick={() => filter = 'removed'}>Removed</button>
        <button class="filter-btn" class:active={filter === 'changed'} onclick={() => filter = 'changed'}>Changed</button>
        <button class="filter-btn" class:active={filter === 'identical'} onclick={() => filter = 'identical'}>Identical</button>
      </div>
    </div>

    <DataDiffGrid result={diffResult} {filter} />
  {:else if !loading && !error && !noPK}
    <div class="empty-state">
      <span>Select source and target tables, then click Compare</span>
    </div>
  {/if}
</div>

<style>
  .datadiff-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .warning-bar {
    padding: 8px 12px;
    font-size: 12px;
    color: var(--warning, #e0af68);
    background: rgba(224, 175, 104, 0.1);
    border-bottom: 1px solid var(--border-color);
  }

  .error-bar {
    padding: 8px 12px;
    font-size: 12px;
    color: var(--error, #f38ba8);
    background: rgba(243, 139, 168, 0.1);
    border-bottom: 1px solid var(--border-color);
  }

  .diff-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    flex-wrap: wrap;
    gap: 8px;
  }

  .diff-summary-bar {
    display: flex;
    gap: 12px;
    font-size: 12px;
  }

  .summary-item { color: var(--text-muted); }
  .summary-item.added { color: var(--success, #9ece6a); }
  .summary-item.removed { color: var(--error, #f38ba8); }
  .summary-item.changed { color: var(--warning, #e0af68); }
  .summary-item.identical { color: var(--text-muted); }

  .filter-btns {
    display: flex;
    gap: 2px;
  }

  .filter-btn {
    padding: 3px 10px;
    font-size: 11px;
    color: var(--text-secondary);
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .filter-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .filter-btn.active {
    color: var(--text-primary);
    background: var(--bg-primary);
    border-color: var(--border-color);
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
