<script lang="ts">
  import { onDestroy } from 'svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { DB_METADATA } from '$lib/types/database';
  import * as tauri from '$lib/services/tauri';
  import { dumpDatabase, pickDumpFile, onDumpProgress, type DumpProgress, type DumpResult } from '$lib/services/dumpService';

  // Connected SQL databases for the picker
  let sqlConnections = $derived(
    connectionStore.connections.filter(c => {
      if (c.status !== 'connected') return false;
      const cat = DB_METADATA[c.config.db_type]?.category;
      return cat === 'Relational' || cat === 'Analytics' || cat === 'WideColumn';
    })
  );

  let selectedConnectionId = $state(uiStore.databaseBackupConnectionId ?? '');
  let connectionId = $derived(uiStore.databaseBackupConnectionId ?? (selectedConnectionId || null));
  let needsPicker = $derived(!uiStore.databaseBackupConnectionId);
  let connection = $derived(
    connectionStore.connections.find(c => c.config.id === connectionId)
  );
  let connectionName = $derived(connection?.config.name ?? 'Database');
  let dbName = $derived(connection?.config.database ?? connection?.config.name ?? 'database');

  // Schema loading
  let schemas = $state<{ name: string; checked: boolean }[]>([]);
  let loadingSchemas = $state(true);
  let schemaError = $state<string | null>(null);

  // Options
  let includeData = $state(true);
  let filePath = $state('');

  // Progress
  let isRunning = $state(false);
  let progress = $state<DumpProgress | null>(null);
  let result = $state<DumpResult | null>(null);
  let error = $state<string | null>(null);

  let unlistenProgress: (() => void) | null = null;

  let selectedCount = $derived(schemas.filter(s => s.checked).length);

  // Load schemas when connectionId changes
  $effect(() => {
    if (connectionId) {
      loadSchemas(connectionId);
    } else {
      schemas = [];
      loadingSchemas = false;
    }
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  });

  async function loadSchemas(connId: string) {
    loadingSchemas = true;
    schemaError = null;
    try {
      const schemaList = await tauri.getSchemas(connId);
      schemas = schemaList.map(s => ({ name: s.name, checked: true }));
    } catch (err) {
      schemaError = err instanceof Error ? err.message : String(err);
    } finally {
      loadingSchemas = false;
    }
  }

  async function handleBrowse() {
    const path = await pickDumpFile(dbName);
    if (path) {
      filePath = path;
    }
  }

  function toggleAll(checked: boolean) {
    schemas = schemas.map(s => ({ ...s, checked }));
  }

  async function handleStart() {
    if (!connectionId || selectedCount === 0 || !filePath.trim()) return;

    isRunning = true;
    error = null;
    result = null;
    progress = null;

    // Listen for progress events
    try {
      const unlisten = await onDumpProgress((p) => {
        progress = p;
      });
      unlistenProgress = unlisten;
    } catch {
      // Event listening may fail outside Tauri
    }

    const selectedSchemas = schemas.filter(s => s.checked).map(s => s.name);

    try {
      result = await dumpDatabase(connectionId, filePath, selectedSchemas, includeData);
      uiStore.showSuccess(`Backup complete: ${result.tables_dumped} tables, ${result.rows_dumped.toLocaleString()} rows`);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isRunning = false;
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
    }
  }

  function handleClose() {
    uiStore.showDatabaseBackupModal = false;
    uiStore.databaseBackupConnectionId = null;
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !isRunning) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && !isRunning) handleClose();
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card">
    <div class="modal-header">
      <h2>Backup Database</h2>
      <button class="close-btn" onclick={handleClose} disabled={isRunning}>&times;</button>
    </div>

    <div class="modal-body">
      <!-- Connection picker or static display -->
      {#if needsPicker}
        <div class="section">
          <span class="section-title">Connection</span>
          {#if sqlConnections.length === 0}
            <div class="empty-text">No connected SQL databases. Connect to a database first.</div>
          {:else}
            <select
              class="connection-select"
              bind:value={selectedConnectionId}
              disabled={isRunning}
            >
              <option value="">Select a database...</option>
              {#each sqlConnections as conn}
                {@const meta = DB_METADATA[conn.config.db_type]}
                <option value={conn.config.id}>
                  {conn.config.name} ({meta.badge})
                </option>
              {/each}
            </select>
          {/if}
        </div>
      {:else}
        <div class="connection-info">
          <span class="label">Connection:</span>
          <span class="value">{connectionName}</span>
        </div>
      {/if}

      <!-- Schema selection -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">Schemas</span>
          {#if !loadingSchemas && schemas.length > 0}
            <div class="schema-actions">
              <button class="link-btn" onclick={() => toggleAll(true)} disabled={isRunning}>All</button>
              <span class="separator">/</span>
              <button class="link-btn" onclick={() => toggleAll(false)} disabled={isRunning}>None</button>
            </div>
          {/if}
        </div>

        {#if loadingSchemas}
          <div class="loading-text">Loading schemas...</div>
        {:else if schemaError}
          <div class="error-text">{schemaError}</div>
        {:else if schemas.length === 0}
          <div class="empty-text">No schemas found</div>
        {:else}
          <div class="schema-list">
            {#each schemas as schema}
              <label class="schema-item">
                <input
                  type="checkbox"
                  bind:checked={schema.checked}
                  disabled={isRunning}
                />
                <span>{schema.name}</span>
              </label>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Options -->
      <div class="section">
        <label class="option-row">
          <input type="checkbox" bind:checked={includeData} disabled={isRunning} />
          <span>Include data (INSERT statements)</span>
        </label>
      </div>

      <!-- File path -->
      <div class="section">
        <span class="section-title">Output File</span>
        <div class="file-row">
          <input
            type="text"
            class="file-input"
            bind:value={filePath}
            placeholder="Select output file..."
            readonly
            disabled={isRunning}
          />
          <button class="browse-btn" onclick={handleBrowse} disabled={isRunning}>Browse</button>
        </div>
      </div>

      <!-- Progress -->
      {#if isRunning && progress}
        <div class="progress-section">
          <div class="progress-label">
            Dumping {progress.schema}.{progress.table}...
          </div>
          <div class="progress-bar-track">
            <div
              class="progress-bar-fill"
              style="width: {progress.tables_total > 0 ? (progress.tables_done / progress.tables_total) * 100 : 0}%"
            ></div>
          </div>
          <div class="progress-stats">
            <span>{progress.tables_done} / {progress.tables_total} tables</span>
            {#if includeData}
              <span>{progress.rows_dumped.toLocaleString()} rows</span>
            {/if}
          </div>
        </div>
      {:else if isRunning}
        <div class="progress-section">
          <div class="progress-label">Starting backup...</div>
        </div>
      {/if}

      <!-- Result -->
      {#if result}
        <div class="result-section">
          <div class="result-icon">Done</div>
          <div class="result-stats">
            <span>{result.tables_dumped} tables dumped</span>
            {#if result.rows_dumped > 0}
              <span>{result.rows_dumped.toLocaleString()} rows</span>
            {/if}
            <span>{formatBytes(result.file_size_bytes)}</span>
          </div>
        </div>
      {/if}

      <!-- Error -->
      {#if error}
        <div class="error-banner">{error}</div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" onclick={handleClose} disabled={isRunning}>
        {result ? 'Close' : 'Cancel'}
      </button>
      {#if !result}
        <button
          class="btn btn-primary"
          onclick={handleStart}
          disabled={isRunning || selectedCount === 0 || !filePath.trim()}
        >
          {#if isRunning}
            Backing up...
          {:else}
            Start Backup
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 480px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    font-size: 15px;
    font-weight: 600;
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 20px;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .modal-body {
    padding: 16px 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .connection-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .connection-info .label {
    color: var(--text-secondary);
  }

  .connection-info .value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .connection-select {
    width: 100%;
    padding: 6px 10px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
    cursor: pointer;
  }

  .connection-select:focus {
    border-color: var(--accent);
  }

  .connection-select:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .schema-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 11px;
    padding: 0;
  }

  .link-btn:hover {
    text-decoration: underline;
  }

  .link-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .separator {
    color: var(--text-muted);
  }

  .schema-list {
    max-height: 140px;
    overflow-y: auto;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 4px;
    background: var(--bg-primary);
  }

  .schema-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 6px;
    font-size: 12px;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .schema-item:hover {
    background: var(--bg-hover);
  }

  .option-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    cursor: pointer;
  }

  .file-row {
    display: flex;
    gap: 8px;
  }

  .file-input {
    flex: 1;
    padding: 6px 10px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
  }

  .file-input:focus {
    border-color: var(--accent);
  }

  .browse-btn {
    padding: 6px 12px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary, var(--bg-secondary));
    color: var(--text-primary);
    cursor: pointer;
    white-space: nowrap;
  }

  .browse-btn:hover {
    background: var(--bg-hover);
  }

  .browse-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .loading-text,
  .empty-text {
    font-size: 12px;
    color: var(--text-muted);
    padding: 8px 0;
  }

  .error-text {
    font-size: 12px;
    color: var(--error);
    padding: 8px 0;
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
  }

  .progress-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .progress-bar-track {
    height: 6px;
    background: var(--bg-tertiary, var(--border-color));
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .progress-stats {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-muted);
  }

  .result-section {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    background: rgba(166, 227, 161, 0.08);
    border: 1px solid rgba(166, 227, 161, 0.2);
    border-radius: var(--radius-sm);
  }

  .result-icon {
    font-size: 12px;
    font-weight: 600;
    color: var(--success);
  }

  .result-stats {
    display: flex;
    gap: 12px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .error-banner {
    padding: 8px 10px;
    font-size: 12px;
    color: var(--error);
    background: rgba(243, 139, 168, 0.08);
    border: 1px solid rgba(243, 139, 168, 0.2);
    border-radius: var(--radius-sm);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--border-color);
  }

  .btn {
    padding: 6px 16px;
    font-size: 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: none;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .btn-secondary {
    background: var(--bg-tertiary, var(--bg-secondary));
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn-primary {
    background: var(--accent);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>
