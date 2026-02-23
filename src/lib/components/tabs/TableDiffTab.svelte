<script lang="ts">
  import type { Tab } from '$lib/types/tabs';
  import type { TableDiffResult } from '$lib/types/diff';
  import type { DatabaseType } from '$lib/types/connection';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import { computeTableDiff } from '$lib/utils/schemaDiff';
  import { generateMigration } from '$lib/utils/migrationGenerator';
  import DiffConnectionPicker from '$lib/components/diff/DiffConnectionPicker.svelte';
  import ColumnsDiff from '$lib/components/diff/ColumnsDiff.svelte';
  import IndexesDiff from '$lib/components/diff/IndexesDiff.svelte';
  import ForeignKeysDiff from '$lib/components/diff/ForeignKeysDiff.svelte';

  let { tab }: { tab: Tab } = $props();

  let sourceConnectionId = $state(tab.connectionId);
  let sourceSchema = $state(tab.schema ?? '');
  let sourceTable = $state(tab.table ?? '');
  let targetConnectionId = $state(tab.diffTargetConnectionId ?? '');
  let targetSchema = $state(tab.diffTargetSchema ?? '');
  let targetTable = $state(tab.diffTargetTable ?? '');

  let loading = $state(false);
  let diffResult = $state<TableDiffResult | null>(null);
  let activeView = $state<'columns' | 'indexes' | 'fks'>('columns');
  let migrationSql = $state<string | null>(null);
  let showMigration = $state(false);

  let targetDbType = $derived.by((): DatabaseType => {
    const conn = connectionStore.connections.find(c => c.config.id === targetConnectionId);
    return conn?.config.db_type ?? 'PostgreSQL';
  });

  async function handleCompare() {
    loading = true;
    diffResult = null;
    migrationSql = null;
    showMigration = false;

    try {
      const [sourceCols, targetCols, sourceIndexes, targetIndexes, sourceFKs, targetFKs] = await Promise.all([
        schemaService.loadColumns(sourceConnectionId, sourceSchema, sourceTable),
        schemaService.loadColumns(targetConnectionId, targetSchema, targetTable),
        schemaService.loadIndexes(sourceConnectionId, sourceSchema, sourceTable),
        schemaService.loadIndexes(targetConnectionId, targetSchema, targetTable),
        schemaService.loadForeignKeys(sourceConnectionId, sourceSchema, sourceTable),
        schemaService.loadForeignKeys(targetConnectionId, targetSchema, targetTable),
      ]);

      diffResult = computeTableDiff(
        sourceCols, targetCols,
        sourceIndexes, targetIndexes,
        sourceFKs, targetFKs,
        `${sourceSchema}.${sourceTable}`,
        `${targetSchema}.${targetTable}`
      );
    } catch (err) {
      console.error('Diff failed:', err);
    } finally {
      loading = false;
    }
  }

  function handleGenerateMigration() {
    if (!diffResult) return;
    migrationSql = generateMigration(diffResult, targetSchema, targetTable, targetDbType);
    showMigration = true;
  }

  function handleOpenInQuery() {
    if (!migrationSql) return;
    tabStore.openTab({
      type: 'query',
      title: `Migration: ${targetTable}`,
      connectionId: targetConnectionId,
      sql: migrationSql,
    });
  }

  function handleCopyMigration() {
    if (migrationSql) {
      navigator.clipboard.writeText(migrationSql);
    }
  }
</script>

<div class="tablediff-tab">
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

  {#if diffResult}
    <div class="diff-header">
      <div class="diff-summary-bar">
        <span class="summary-item added">+{diffResult.summary.added} added</span>
        <span class="summary-item removed">-{diffResult.summary.removed} removed</span>
        <span class="summary-item changed">{diffResult.summary.changed} changed</span>
        <span class="summary-item unchanged">{diffResult.summary.unchanged} unchanged</span>
      </div>
      <div class="diff-tabs">
        <button
          class="diff-tab-btn"
          class:active={activeView === 'columns'}
          onclick={() => activeView = 'columns'}
        >
          Columns ({diffResult.columns.length})
        </button>
        <button
          class="diff-tab-btn"
          class:active={activeView === 'indexes'}
          onclick={() => activeView = 'indexes'}
        >
          Indexes ({diffResult.indexes.length})
        </button>
        <button
          class="diff-tab-btn"
          class:active={activeView === 'fks'}
          onclick={() => activeView = 'fks'}
        >
          Foreign Keys ({diffResult.foreignKeys.length})
        </button>
      </div>
    </div>

    <div class="diff-content">
      {#if activeView === 'columns'}
        <ColumnsDiff diffs={diffResult.columns} />
      {:else if activeView === 'indexes'}
        <IndexesDiff diffs={diffResult.indexes} />
      {:else if activeView === 'fks'}
        <ForeignKeysDiff diffs={diffResult.foreignKeys} />
      {/if}
    </div>

    {#if diffResult.summary.added > 0 || diffResult.summary.removed > 0 || diffResult.summary.changed > 0}
      <div class="migration-section">
        {#if !showMigration}
          <button class="migration-btn" onclick={handleGenerateMigration}>
            Generate Migration SQL
          </button>
        {:else if migrationSql}
          <div class="migration-actions">
            <button class="migration-btn secondary" onclick={() => showMigration = false}>Hide</button>
            <button class="migration-btn" onclick={handleOpenInQuery}>Open in Query Tab</button>
            <button class="migration-btn secondary" onclick={handleCopyMigration}>Copy SQL</button>
          </div>
          <pre class="migration-preview">{migrationSql}</pre>
        {/if}
      </div>
    {/if}
  {:else if !loading}
    <div class="empty-state">
      <span>Select source and target tables, then click Compare</span>
    </div>
  {/if}
</div>

<style>
  .tablediff-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .diff-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .diff-summary-bar {
    display: flex;
    gap: 12px;
    font-size: 12px;
  }

  .summary-item {
    font-weight: 500;
  }
  .summary-item.added { color: var(--success, #9ece6a); }
  .summary-item.removed { color: var(--error, #f38ba8); }
  .summary-item.changed { color: var(--warning, #e0af68); }
  .summary-item.unchanged { color: var(--text-muted); }

  .diff-tabs {
    display: flex;
    gap: 2px;
  }

  .diff-tab-btn {
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .diff-tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .diff-tab-btn.active {
    color: var(--text-primary);
    background: var(--bg-primary);
    border-color: var(--border-color);
  }

  .diff-content {
    flex: 1;
    overflow: auto;
  }

  .migration-section {
    border-top: 1px solid var(--border-color);
    padding: 10px 12px;
    background: var(--bg-secondary);
  }

  .migration-btn {
    padding: 5px 14px;
    font-size: 12px;
    font-weight: 500;
    color: var(--bg-primary);
    background: var(--accent);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .migration-btn:hover { opacity: 0.9; }

  .migration-btn.secondary {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .migration-actions {
    display: flex;
    gap: 6px;
    margin-bottom: 8px;
  }

  .migration-preview {
    padding: 10px;
    font-size: 12px;
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    overflow: auto;
    max-height: 200px;
    color: var(--text-primary);
    white-space: pre-wrap;
    margin: 0;
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
