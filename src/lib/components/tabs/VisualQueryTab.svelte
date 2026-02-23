<script lang="ts">
  import { v4 as uuidv4 } from 'uuid';
  import type { Tab } from '$lib/types/tabs';
  import type { VQState, VQTable, VQColumn } from '$lib/types/visualQuery';
  import type { QueryResponse } from '$lib/types/query';
  import type { DatabaseType } from '$lib/types/connection';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import * as tauri from '$lib/services/tauri';
  import { generateSQL } from '$lib/utils/visualQueryBuilder';
  import VQTablePicker from '$lib/components/visualquery/VQTablePicker.svelte';
  import VQCanvas from '$lib/components/visualquery/VQCanvas.svelte';
  import VQClausePanel from '$lib/components/visualquery/VQClausePanel.svelte';

  let { tab, onqueryresult }: {
    tab: Tab;
    onqueryresult?: (detail: { executionTime: number; rowCount: number }) => void;
  } = $props();

  let vqState = $state<VQState>({
    tables: [],
    joins: [],
    where: [],
    orderBy: [],
    groupBy: [],
    distinct: false,
    limit: null,
  });

  let queryResult = $state<QueryResponse | null>(null);
  let executing = $state(false);
  let error = $state<string | null>(null);
  let showSqlPreview = $state(true);

  let dbType = $derived.by((): DatabaseType => {
    const conn = connectionStore.connections.find(c => c.config.id === tab.connectionId);
    return conn?.config.db_type ?? 'PostgreSQL';
  });

  let generatedSQL = $derived(generateSQL(vqState, dbType));

  // Table alias counter
  let aliasCounter = $state(0);

  function generateAlias(tableName: string): string {
    const firstLetter = tableName.charAt(0).toLowerCase();
    aliasCounter++;
    return aliasCounter === 1 ? firstLetter : `${firstLetter}${aliasCounter}`;
  }

  async function handleAddTable(schema: string, tableName: string) {
    // Check if already added
    if (vqState.tables.some(t => t.schema === schema && t.name === tableName)) return;

    // Load columns
    const cols = await schemaService.loadColumns(tab.connectionId, schema, tableName);
    const fks = await schemaService.loadForeignKeys(tab.connectionId, schema, tableName);
    const fkColNames = new Set(fks.flatMap(f => f.columns));

    const columns: VQColumn[] = cols.map(c => ({
      name: c.name,
      dataType: c.data_type,
      isPK: c.is_primary_key,
      isFK: fkColNames.has(c.name),
    }));

    const alias = generateAlias(tableName);
    const x = vqState.tables.length * 250 + 20;
    const y = 20;

    const newTable: VQTable = {
      id: uuidv4(),
      schema,
      name: tableName,
      alias,
      x,
      y,
      columns,
      selectedColumns: columns.map(c => c.name), // Select all by default
    };

    vqState.tables = [...vqState.tables, newTable];
  }

  async function handleRun() {
    if (!generatedSQL.trim()) return;
    executing = true;
    error = null;
    queryResult = null;

    try {
      const result = await tauri.executeQuery(tab.connectionId, generatedSQL);
      queryResult = result;
      onqueryresult?.({ executionTime: result.execution_time_ms, rowCount: result.row_count });
    } catch (err) {
      error = String(err);
    } finally {
      executing = false;
    }
  }

  function handleClear() {
    vqState = {
      tables: [],
      joins: [],
      where: [],
      orderBy: [],
      groupBy: [],
      distinct: false,
      limit: null,
    };
    aliasCounter = 0;
    queryResult = null;
    error = null;
  }

  function handleCopySQL() {
    navigator.clipboard.writeText(generatedSQL);
  }

  function cellToString(cell: import('$lib/types/query').CellValue): string {
    switch (cell.type) {
      case 'Null': return 'NULL';
      case 'Bool': return String(cell.value);
      case 'Int': return String(cell.value);
      case 'Float': return String(cell.value);
      case 'Text': return cell.value;
      case 'Timestamp': return cell.value;
      case 'Json': return cell.value;
      case 'Binary': return `[${cell.value.length} bytes]`;
      case 'LargeText': return cell.value.preview;
      case 'LargeJson': return cell.value.preview;
      case 'LargeBinary': return `[${cell.value.full_length} bytes]`;
    }
  }
</script>

<div class="vq-tab">
  <!-- Toolbar -->
  <div class="vq-toolbar">
    <button class="vq-btn primary" onclick={handleRun} disabled={executing || !generatedSQL.trim()}>
      {#if executing}
        <span class="btn-spinner"></span> Running...
      {:else}
        Run
      {/if}
    </button>
    <button class="vq-btn" onclick={handleClear}>Clear</button>
    <button class="vq-btn" onclick={handleCopySQL} disabled={!generatedSQL.trim()}>Copy SQL</button>
    <button class="vq-btn" onclick={() => showSqlPreview = !showSqlPreview}>
      {showSqlPreview ? 'Hide SQL' : 'Show SQL'}
    </button>
  </div>

  <!-- Main area -->
  <div class="vq-main">
    <VQTablePicker connectionId={tab.connectionId} onaddtable={handleAddTable} />

    <div class="vq-center">
      <div class="vq-canvas-area">
        {#if vqState.tables.length === 0}
          <div class="canvas-empty">
            Double-click a table from the left panel to add it to the canvas
          </div>
        {:else}
          <VQCanvas bind:tables={vqState.tables} bind:joins={vqState.joins} />
        {/if}
      </div>

      {#if showSqlPreview}
        <div class="sql-preview-panel">
          <div class="sql-preview-header">Generated SQL</div>
          <pre class="sql-preview">{generatedSQL || '-- Add tables and select columns to generate SQL'}</pre>
        </div>
      {/if}

      {#if error}
        <div class="error-bar">{error}</div>
      {/if}

      {#if queryResult}
        <div class="results-panel">
          <div class="results-header">
            Results: {queryResult.row_count} rows ({queryResult.execution_time_ms}ms)
          </div>
          <div class="results-grid">
            <table class="result-table">
              <thead>
                <tr>
                  {#each queryResult.columns as col}
                    <th>{col.name}</th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each queryResult.rows.slice(0, 100) as row}
                  <tr>
                    {#each row as cell}
                      <td>{cellToString(cell)}</td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}
    </div>

    <VQClausePanel
      tables={vqState.tables}
      bind:where={vqState.where}
      bind:orderBy={vqState.orderBy}
      bind:groupBy={vqState.groupBy}
      bind:distinct={vqState.distinct}
      bind:limit={vqState.limit}
    />
  </div>
</div>

<style>
  .vq-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .vq-toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .vq-btn {
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .vq-btn:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--accent);
  }

  .vq-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .vq-btn.primary {
    color: var(--bg-primary);
    background: var(--accent);
    border-color: var(--accent);
  }

  .vq-btn.primary:hover:not(:disabled) {
    opacity: 0.9;
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

  .vq-main {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .vq-center {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .vq-canvas-area {
    flex: 1;
    min-height: 200px;
    position: relative;
  }

  .canvas-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 13px;
  }

  .sql-preview-panel {
    border-top: 1px solid var(--border-color);
    max-height: 120px;
    overflow: auto;
    flex-shrink: 0;
  }

  .sql-preview-header {
    padding: 4px 10px;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    position: sticky;
    top: 0;
  }

  .sql-preview {
    padding: 8px 10px;
    font-size: 11px;
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
    color: var(--text-primary);
    background: var(--bg-primary);
    white-space: pre-wrap;
    margin: 0;
  }

  .error-bar {
    padding: 6px 10px;
    font-size: 12px;
    color: var(--error, #f38ba8);
    background: rgba(243, 139, 168, 0.1);
    border-top: 1px solid var(--border-color);
  }

  .results-panel {
    border-top: 1px solid var(--border-color);
    max-height: 250px;
    overflow: auto;
    flex-shrink: 0;
  }

  .results-header {
    padding: 4px 10px;
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .results-grid {
    overflow: auto;
  }

  .result-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }

  .result-table th {
    padding: 4px 8px;
    text-align: left;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    position: sticky;
    top: 26px;
    z-index: 1;
  }

  .result-table td {
    padding: 3px 8px;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-secondary);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
  }
</style>
