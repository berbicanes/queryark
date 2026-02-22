<script lang="ts">
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import { DB_METADATA } from '$lib/types/database';
  import type { ConnectionState } from '$lib/types/connection';

  let contextMenu = $state<{ x: number; y: number; connection: ConnectionState } | null>(null);

  function handleClick(conn: ConnectionState) {
    connectionStore.setActive(conn.config.id);
  }

  function handleDblClick(conn: ConnectionState) {
    if (conn.status === 'connected') {
      connectionService.disconnect(conn.config.id);
    } else {
      connectionService.connect(conn.config);
    }
  }

  function handleContextMenu(e: MouseEvent, conn: ConnectionState) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, connection: conn };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function ctxConnect() {
    if (contextMenu) {
      connectionService.connect(contextMenu.connection.config);
      closeContextMenu();
    }
  }

  function ctxDisconnect() {
    if (contextMenu) {
      connectionService.disconnect(contextMenu.connection.config.id);
      closeContextMenu();
    }
  }

  function ctxEdit() {
    if (contextMenu) {
      editingConnectionId = contextMenu.connection.config.id;
      uiStore.openConnectionModal();
      closeContextMenu();
    }
  }

  function ctxDelete() {
    if (contextMenu) {
      const connId = contextMenu.connection.config.id;
      const connName = contextMenu.connection.config.name;
      uiStore.confirm(
        `Delete connection "${connName}"? This cannot be undone.`,
        () => connectionService.deleteConnection(connId)
      );
      closeContextMenu();
    }
  }

  let editingConnectionId = $state<string | null>(null);
</script>

<svelte:window onclick={closeContextMenu} />

<div class="connection-list">
  {#if connectionStore.connections.length === 0}
    <div class="empty-list">
      <span class="text-muted">No connections yet</span>
    </div>
  {:else}
    {#each connectionStore.connections as conn}
      {@const meta = DB_METADATA[conn.config.db_type]}
      <button
        class="connection-item"
        class:active={conn.config.id === connectionStore.activeConnectionId}
        onclick={() => handleClick(conn)}
        ondblclick={() => handleDblClick(conn)}
        oncontextmenu={(e) => handleContextMenu(e, conn)}
      >
        <span class="status-dot {conn.status}"></span>
        <span class="conn-name truncate">{conn.config.name}</span>
        <span class="badge {meta.badgeClass}">
          {meta.badge}
        </span>
      </button>
    {/each}
  {/if}
</div>

{#if contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') closeContextMenu(); }}
  >
    {#if contextMenu.connection.status !== 'connected'}
      <button class="context-item" onclick={ctxConnect}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M4 2l10 6-10 6V2z" fill="var(--success)"/>
        </svg>
        Connect
      </button>
    {:else}
      <button class="context-item" onclick={ctxDisconnect}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <rect x="3" y="3" width="10" height="10" rx="1" fill="var(--error)"/>
        </svg>
        Disconnect
      </button>
    {/if}
    <button class="context-item" onclick={ctxEdit}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M11 2l3 3-9 9H2v-3l9-9z" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
      Edit
    </button>
    <div class="context-divider"></div>
    <button class="context-item danger" onclick={ctxDelete}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M3 4h10M6 4V3a1 1 0 011-1h2a1 1 0 011 1v1M5 4v9a1 1 0 001 1h4a1 1 0 001-1V4" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
      Delete
    </button>
  </div>
{/if}

<style>
  .connection-list {
    padding: 4px 0;
  }

  .empty-list {
    padding: 16px;
    text-align: center;
    font-size: 12px;
  }

  .connection-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    transition: background var(--transition-fast);
    cursor: pointer;
    border: none;
    background: none;
  }

  .connection-item:hover {
    background: var(--bg-hover);
  }

  .connection-item.active {
    background: var(--bg-active);
    border-left: 2px solid var(--accent);
    padding-left: 10px;
  }

  .conn-name {
    flex: 1;
    min-width: 0;
  }

  .context-menu {
    position: fixed;
    z-index: 500;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    padding: 4px 0;
    min-width: 160px;
  }

  .context-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    transition: background var(--transition-fast);
    cursor: pointer;
    border: none;
    background: none;
  }

  .context-item:hover {
    background: var(--bg-hover);
  }

  .context-item.danger {
    color: var(--error);
  }

  .context-item.danger:hover {
    background: rgba(243, 139, 168, 0.1);
  }

  .context-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }
</style>
