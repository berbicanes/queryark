<script lang="ts">
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { DB_METADATA } from '$lib/types/database';

  let showConnectionDropdown = $state(false);

  let activeConnection = $derived(connectionStore.activeConnection);
  let connectedConnections = $derived(connectionStore.connectedConnections);

  function handleNewQuery() {
    if (connectionStore.activeConnectionId) {
      tabStore.newQueryTab(connectionStore.activeConnectionId);
    } else if (connectedConnections.length > 0) {
      const first = connectedConnections[0];
      connectionStore.setActive(first.config.id);
      tabStore.newQueryTab(first.config.id);
    } else {
      uiStore.showError('No active connection. Connect to a database first.');
    }
  }

  function handleRun() {
    window.dispatchEvent(new CustomEvent('dataforge:execute-query'));
  }

  function selectConnection(id: string) {
    connectionStore.setActive(id);
    showConnectionDropdown = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      showConnectionDropdown = false;
    }
  }

  function handleWindowClick() {
    showConnectionDropdown = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} onclick={handleWindowClick} />

<div class="toolbar">
  <div class="toolbar-left">
    <span class="branding">DataForge</span>

    <div class="toolbar-divider"></div>

    <button class="btn toolbar-btn" onclick={handleNewQuery}>
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      New Query
    </button>

    <button class="btn toolbar-btn" onclick={handleRun}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M4 2l10 6-10 6V2z" fill="var(--success)"/>
      </svg>
      Run
      <span class="kbd">Ctrl+Enter</span>
    </button>
  </div>

  <div class="toolbar-right">
    <button
      class="btn toolbar-btn"
      onclick={() => settingsStore.toggleTheme()}
      title="Toggle theme (Ctrl+Shift+T)"
    >
      {#if settingsStore.theme === 'dark'}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="3.5" stroke="currentColor" stroke-width="1.3"/>
          <path d="M8 1.5v1.5M8 13v1.5M1.5 8H3M13 8h1.5M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
        </svg>
      {:else}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M13.5 9.5a5.5 5.5 0 01-7-7 5.5 5.5 0 107 7z" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      {/if}
    </button>

    <button
      class="btn toolbar-btn"
      onclick={() => { uiStore.showShortcutsModal = true; }}
      title="Keyboard shortcuts (Ctrl+K)"
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <rect x="1" y="4" width="14" height="9" rx="1.5" stroke="currentColor" stroke-width="1.2"/>
        <path d="M4 7h1M7 7h2M11 7h1M4 10h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>

    <button
      class="btn toolbar-btn"
      onclick={() => { uiStore.showSettingsModal = true; }}
      title="Settings"
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="2.5" stroke="currentColor" stroke-width="1.2"/>
        <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.05 3.05l1.41 1.41M11.54 11.54l1.41 1.41M3.05 12.95l1.41-1.41M11.54 4.46l1.41-1.41" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>

    <div class="toolbar-divider"></div>
    <div class="connection-selector">
      <button
        class="btn selector-btn"
        onclick={(e) => { e.stopPropagation(); showConnectionDropdown = !showConnectionDropdown; }}
      >
        {#if activeConnection}
          {@const meta = DB_METADATA[activeConnection.config.db_type]}
          <span class="status-dot {activeConnection.status}"></span>
          <span class="conn-name">{activeConnection.config.name}</span>
          <span class="badge {meta.badgeClass}">
            {meta.badge}
          </span>
        {:else}
          <span class="text-muted">No connection</span>
        {/if}
        <svg width="10" height="10" viewBox="0 0 16 16" fill="none" class="chevron" class:open={showConnectionDropdown}>
          <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>

      {#if showConnectionDropdown}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="dropdown" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') showConnectionDropdown = false; }}>
          {#if connectedConnections.length === 0}
            <div class="dropdown-empty">No connected databases</div>
          {:else}
            {#each connectedConnections as conn}
              {@const connMeta = DB_METADATA[conn.config.db_type]}
              <button
                class="dropdown-item"
                class:active={conn.config.id === connectionStore.activeConnectionId}
                onclick={() => selectConnection(conn.config.id)}
              >
                <span class="status-dot {conn.status}"></span>
                <span class="truncate">{conn.config.name}</span>
                <span class="badge {connMeta.badgeClass}">
                  {connMeta.badge}
                </span>
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    padding: 0 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    user-select: none;
    -webkit-app-region: drag;
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
    -webkit-app-region: no-drag;
  }

  .branding {
    font-size: 14px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: -0.3px;
    padding-right: 4px;
  }

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: var(--border-color);
    margin: 0 6px;
  }

  .toolbar-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    transition: background var(--transition-fast);
  }

  .toolbar-btn:hover {
    background: var(--bg-hover);
  }

  .toolbar-btn:active {
    background: var(--bg-active);
  }

  .connection-selector {
    position: relative;
  }

  .selector-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    min-width: 160px;
    transition: border-color var(--transition-fast);
  }

  .selector-btn:hover {
    border-color: var(--accent);
  }

  .conn-name {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chevron {
    transition: transform var(--transition-fast);
    flex-shrink: 0;
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 200px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: 100;
    overflow: hidden;
  }

  .dropdown-empty {
    padding: 12px 16px;
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    font-size: 12px;
    color: var(--text-primary);
    transition: background var(--transition-fast);
    text-align: left;
  }

  .dropdown-item:hover {
    background: var(--bg-hover);
  }

  .dropdown-item.active {
    background: var(--bg-active);
  }
</style>
