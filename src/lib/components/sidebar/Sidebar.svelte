<script lang="ts">
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { DB_METADATA } from '$lib/types/database';
  import * as schemaService from '$lib/services/schemaService';
  import { executeQuery } from '$lib/services/tauri';
  import ConnectionList from './ConnectionList.svelte';
  import SchemaTree from './SchemaTree.svelte';

  let isResizing = $state(false);
  let startX = $state(0);
  let startWidth = $state(0);
  let scrollContainer: HTMLElement | undefined = $state();

  let activeConnection = $derived(connectionStore.activeConnection);
  let isConnected = $derived(activeConnection?.status === 'connected');

  // Schema search path display
  let searchPath = $state<string | null>(null);

  $effect(() => {
    searchPath = null;
    if (!isConnected || !activeConnection) return;
    const connId = activeConnection.config.id;
    const dbType = activeConnection.config.db_type;
    const cat = DB_METADATA[dbType].category;
    const isSqlLike = cat === 'Relational' || cat === 'Analytics' || cat === 'WideColumn';
    if (!isSqlLike) return;

    let query: string | null = null;
    switch (dbType) {
      case 'PostgreSQL':
      case 'CockroachDB':
      case 'Redshift':
        query = 'SHOW search_path';
        break;
      case 'MySQL':
      case 'MariaDB':
        query = 'SELECT DATABASE()';
        break;
      case 'MSSQL':
        query = 'SELECT SCHEMA_NAME()';
        break;
      default:
        return; // SQLite, ClickHouse, Cassandra — no meaningful search path
    }

    executeQuery(connId, query).then(result => {
      if (result.rows && result.rows.length > 0) {
        const cell = result.rows[0][0];
        if (cell && typeof cell === 'object' && 'type' in cell && cell.type === 'Text') {
          searchPath = (cell as { type: 'Text'; value: string }).value;
        }
      }
    }).catch(() => {
      // Silently ignore — don't block UI
    });
  });

  let refreshing = $state(false);

  function handleAddConnection() {
    uiStore.openConnectionModal();
  }

  async function handleRefresh() {
    const connId = connectionStore.activeConnectionId;
    if (!connId) return;
    refreshing = true;
    try {
      const conn = connectionStore.connections.find(c => c.config.id === connId);
      if (!conn) return;
      const cat = DB_METADATA[conn.config.db_type].category;
      const isSqlLike = cat === 'Relational' || cat === 'Analytics' || cat === 'WideColumn';
      if (isSqlLike) {
        await schemaService.refreshSchema(connId);
      } else {
        await schemaService.refreshContainers(connId);
      }
    } finally {
      refreshing = false;
    }
  }

  function onMouseDown(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    startWidth = uiStore.sidebarWidth;
    e.preventDefault();
  }

  function onMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    const delta = e.clientX - startX;
    const newWidth = Math.max(180, Math.min(500, startWidth + delta));
    uiStore.sidebarWidth = newWidth;
  }

  function onMouseUp() {
    isResizing = false;
  }
</script>

<svelte:window
  onmousemove={onMouseMove}
  onmouseup={onMouseUp}
/>

<div class="sidebar" class:resizing={isResizing}>
  <div class="sidebar-header">
    <span class="sidebar-title">Connections</span>
    <button class="btn btn-sm add-btn" onclick={handleAddConnection} title="Add connection">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div class="sidebar-content" bind:this={scrollContainer}>
    <ConnectionList />

    {#if isConnected && connectionStore.activeConnectionId}
      <div class="schema-section">
        <div class="section-header">
          <span class="section-title">Schema</span>
          <button
            class="btn btn-sm refresh-btn"
            class:spinning={refreshing}
            onclick={handleRefresh}
            title="Refresh schema"
            disabled={refreshing}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <path d="M8 0l3 2-3 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>
        {#if searchPath}
          {@const dbType = activeConnection?.config.db_type}
          {@const pathLabel = dbType === 'MySQL' || dbType === 'MariaDB' ? 'database' : dbType === 'MSSQL' ? 'schema' : 'search_path'}
          <div class="search-path" title="{pathLabel}: {searchPath}">
            {pathLabel}: {searchPath}
          </div>
        {/if}
        <SchemaTree connectionId={connectionStore.activeConnectionId} {scrollContainer} />
      </div>
    {/if}
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="resize-handle"
    onmousedown={onMouseDown}
    role="separator"
    aria-orientation="vertical"
    tabindex="-1"
  ></div>
</div>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    position: relative;
    user-select: none;
  }

  .sidebar.resizing {
    user-select: none;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .sidebar-title {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-muted);
  }

  .add-btn {
    color: var(--text-secondary);
    padding: 2px;
    border-radius: var(--radius-sm);
    transition: color var(--transition-subtle), background var(--transition-subtle);
  }

  .add-btn:hover {
    color: var(--accent);
    background: rgba(122, 162, 247, 0.1);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .schema-section {
    border-top: 1px solid var(--border-color);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
  }

  .section-title {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-muted);
  }

  .refresh-btn {
    color: var(--text-secondary);
    padding: 2px;
    border-radius: var(--radius-sm);
  }

  .refresh-btn:hover {
    color: var(--accent);
    background: var(--bg-hover);
  }

  .search-path {
    padding: 2px 12px 4px;
    font-size: 10px;
    font-style: italic;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0.8;
    font-family: var(--font-mono);
  }

  .refresh-btn.spinning svg {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .resize-handle {
    position: absolute;
    top: 0;
    right: -3px;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 10;
  }

  .resize-handle:hover,
  .sidebar.resizing .resize-handle {
    background: var(--accent);
    opacity: 0.3;
  }
</style>
