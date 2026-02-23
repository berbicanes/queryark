<script lang="ts">
  import { v4 as uuidv4 } from 'uuid';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import { DB_METADATA } from '$lib/types/database';
  import type { ConnectionState } from '$lib/types/connection';
  import ConnectionGroup from './ConnectionGroup.svelte';

  let contextMenu = $state<{ x: number; y: number; connection: ConnectionState } | null>(null);
  let groupInput = $state<{ connId: string; value: string } | null>(null);

  let expandedGroups = $state(new Set<string>());
  let editingConnectionId = $state<string | null>(null);

  // Derived: ungrouped connections, then groups
  let ungrouped = $derived(connectionStore.getConnectionsByGroup(null));
  let groups = $derived(connectionStore.groups);

  function toggleGroup(name: string) {
    const next = new Set(expandedGroups);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    expandedGroups = next;
  }

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
      const connId = contextMenu.connection.config.id;
      const connTabs = tabStore.tabsForConnection(connId);
      if (settingsStore.confirmBeforeDelete && connTabs.length > 0) {
        uiStore.confirm(
          `Disconnect? ${connTabs.length} open tab${connTabs.length > 1 ? 's' : ''} will lose their connection.`,
          () => connectionService.disconnect(connId)
        );
      } else {
        connectionService.disconnect(connId);
      }
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

  function ctxDuplicate() {
    if (contextMenu) {
      const original = contextMenu.connection.config;
      const copy = {
        ...original,
        id: uuidv4(),
        name: original.name + ' (copy)',
      };
      connectionService.saveConnection(copy);
      closeContextMenu();
    }
  }

  function ctxMoveToGroup() {
    if (contextMenu) {
      groupInput = { connId: contextMenu.connection.config.id, value: contextMenu.connection.config.group ?? '' };
      closeContextMenu();
    }
  }

  function ctxRemoveFromGroup() {
    if (contextMenu) {
      const config = { ...contextMenu.connection.config, group: undefined };
      connectionStore.updateConnection(config);
      closeContextMenu();
    }
  }

  function submitGroupInput(e: KeyboardEvent) {
    if (e.key === 'Enter' && groupInput) {
      const config = connectionStore.connections.find(c => c.config.id === groupInput!.connId)?.config;
      if (config) {
        const updated = { ...config, group: groupInput.value.trim() || undefined };
        connectionStore.updateConnection(updated);
        // Auto-expand the new group
        if (updated.group) {
          expandedGroups = new Set([...expandedGroups, updated.group]);
        }
      }
      groupInput = null;
    } else if (e.key === 'Escape') {
      groupInput = null;
    }
  }
</script>

<svelte:window onclick={closeContextMenu} />

<div class="connection-list">
  {#if connectionStore.connections.length === 0}
    <div class="empty-list">
      <span class="text-muted">No connections yet</span>
    </div>
  {:else}
    <!-- Ungrouped connections -->
    {#each ungrouped as conn}
      {@const meta = DB_METADATA[conn.config.db_type]}
      <button
        class="connection-item"
        class:active={conn.config.id === connectionStore.activeConnectionId}
        style={conn.config.color ? `border-left: 3px solid ${conn.config.color}; padding-left: 9px` : ''}
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

    <!-- Grouped connections -->
    {#each groups as group}
      {@const groupConns = connectionStore.getConnectionsByGroup(group)}
      <ConnectionGroup
        name={group}
        count={groupConns.length}
        expanded={expandedGroups.has(group)}
        ontoggle={() => toggleGroup(group)}
      >
        {#each groupConns as conn}
          {@const meta = DB_METADATA[conn.config.db_type]}
          <button
            class="connection-item"
            class:active={conn.config.id === connectionStore.activeConnectionId}
            style={conn.config.color ? `border-left: 3px solid ${conn.config.color}; padding-left: 9px` : ''}
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
      </ConnectionGroup>
    {/each}
  {/if}
</div>

{#if groupInput}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="group-input-overlay" onclick={() => { groupInput = null; }} onkeydown={(e) => { if (e.key === 'Escape') groupInput = null; }}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="group-input-card" onclick={(e) => e.stopPropagation()}>
      <label class="group-input-label" for="group-name-input">Move to Group</label>
      <input
        id="group-name-input"
        type="text"
        bind:value={groupInput.value}
        onkeydown={submitGroupInput}
        placeholder="Group name"
        list="existing-groups"
      />
      <datalist id="existing-groups">
        {#each groups as g}
          <option value={g}></option>
        {/each}
      </datalist>
      <div class="group-input-hint">Press Enter to confirm, Escape to cancel</div>
    </div>
  </div>
{/if}

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
    <button class="context-item" onclick={ctxDuplicate}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <rect x="5" y="5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1.2" fill="none"/>
        <path d="M3 11V3a1 1 0 011-1h8" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
      Duplicate
    </button>
    <div class="context-divider"></div>
    <button class="context-item" onclick={ctxMoveToGroup}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M2 3h5l1 1h6v9H2V3z" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
      Move to Group
    </button>
    {#if contextMenu.connection.config.group}
      <button class="context-item" onclick={ctxRemoveFromGroup}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M4 8h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        Remove from Group
      </button>
    {/if}
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
    padding: 4px 12px;
    height: 28px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    transition: background var(--transition-subtle);
    cursor: pointer;
    border: none;
    background: none;
  }

  .connection-item:hover {
    background: var(--bg-hover);
  }

  .connection-item.active {
    background: rgba(122, 162, 247, 0.08);
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
    background: var(--bg-elevated, var(--bg-secondary));
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 4px 0;
    min-width: 160px;
    animation: ctxMenuIn 120ms var(--ease-out-expo);
  }

  @keyframes ctxMenuIn {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
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

  .group-input-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 600;
  }

  .group-input-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    padding: 16px;
    min-width: 280px;
  }

  .group-input-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .group-input-card input {
    width: 100%;
    padding: 6px 10px;
    font-size: 13px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
  }

  .group-input-card input:focus {
    border-color: var(--accent);
  }

  .group-input-hint {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 6px;
  }
</style>
