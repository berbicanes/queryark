<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import { DB_METADATA } from '$lib/types/database';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import TabBar from '$lib/components/tabs/TabBar.svelte';
  import TabContent from '$lib/components/tabs/TabContent.svelte';
  import SplitPane from '$lib/components/tabs/SplitPane.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import ConnectionModal from '$lib/components/modals/ConnectionModal.svelte';
  import ConfirmDialog from '$lib/components/modals/ConfirmDialog.svelte';
  import CreateTableModal from '$lib/components/modals/CreateTableModal.svelte';
  import AlterTableModal from '$lib/components/modals/AlterTableModal.svelte';
  import IndexModal from '$lib/components/modals/IndexModal.svelte';
  import ShortcutsModal from '$lib/components/modals/ShortcutsModal.svelte';
  import CommandPalette from '$lib/components/modals/CommandPalette.svelte';
  import SettingsModal from '$lib/components/modals/SettingsModal.svelte';

  let lastExecutionTime = $state<number | null>(null);
  let lastRowCount = $state<number | null>(null);

  onMount(() => {
    connectionStore.init();
    queryHistoryStore.init();
    savedQueriesStore.init();
    settingsStore.init();
  });

  function handleQueryResult(detail: { executionTime: number; rowCount: number }) {
    lastExecutionTime = detail.executionTime;
    lastRowCount = detail.rowCount;
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    // Don't handle shortcuts when a modal is open that captures input
    const actionId = settingsStore.matchEvent(e);
    if (!actionId) return;

    // Let editor-specific shortcuts pass through to CodeMirror
    if (['runQuery', 'formatSql', 'saveQuery'].includes(actionId)) return;

    e.preventDefault();

    switch (actionId) {
      case 'newQuery': {
        if (connectionStore.activeConnectionId) {
          tabStore.newQueryTab(connectionStore.activeConnectionId);
        } else {
          const connected = connectionStore.connectedConnections;
          if (connected.length > 0) {
            connectionStore.setActive(connected[0].config.id);
            tabStore.newQueryTab(connected[0].config.id);
          } else {
            uiStore.showError('No active connection. Connect to a database first.');
          }
        }
        break;
      }
      case 'closeTab':
        if (tabStore.activeTabId) tabStore.closeTab(tabStore.activeTabId);
        break;
      case 'nextTab': {
        const tabs = tabStore.tabs;
        if (tabs.length > 1 && tabStore.activeTabId) {
          const idx = tabs.findIndex(t => t.id === tabStore.activeTabId);
          const next = (idx + 1) % tabs.length;
          tabStore.setActive(tabs[next].id);
        }
        break;
      }
      case 'prevTab': {
        const tabs = tabStore.tabs;
        if (tabs.length > 1 && tabStore.activeTabId) {
          const idx = tabs.findIndex(t => t.id === tabStore.activeTabId);
          const prev = (idx - 1 + tabs.length) % tabs.length;
          tabStore.setActive(tabs[prev].id);
        }
        break;
      }
      case 'globalSearch':
        uiStore.showCommandPalette = !uiStore.showCommandPalette;
        break;
      case 'toggleSidebar':
        uiStore.sidebarCollapsed = !uiStore.sidebarCollapsed;
        break;
      case 'refreshSchema': {
        const connId = connectionStore.activeConnectionId;
        if (connId) {
          const conn = connectionStore.connections.find(c => c.config.id === connId);
          if (conn) {
            const cat = DB_METADATA[conn.config.db_type].category;
            const isSqlLike = cat === 'Relational' || cat === 'Analytics' || cat === 'WideColumn';
            if (isSqlLike) {
              schemaService.refreshSchema(connId);
            } else {
              schemaService.refreshContainers(connId);
            }
          }
        }
        break;
      }
      case 'toggleTheme':
        settingsStore.toggleTheme();
        break;
      case 'shortcuts':
        uiStore.showShortcutsModal = !uiStore.showShortcutsModal;
        break;
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div
  class="app-layout"
  class:sidebar-collapsed={uiStore.sidebarCollapsed}
  style="--sidebar-width: {uiStore.sidebarCollapsed ? 0 : uiStore.sidebarWidth}px"
>
  <div class="toolbar-area">
    <Toolbar />
  </div>

  <div class="sidebar-area">
    {#if !uiStore.sidebarCollapsed}
      <Sidebar />
    {/if}
  </div>

  <div class="main-area">
    {#if tabStore.tabs.length > 0}
      {#if tabStore.splitMode}
        <SplitPane onqueryresult={handleQueryResult} />
      {:else}
        <TabBar />
        <div class="tab-content-wrapper">
          <TabContent onqueryresult={handleQueryResult} />
        </div>
      {/if}
    {:else}
      <div class="empty-state">
        <div class="icon">&#x1F5C4;</div>
        <div class="message">Open a connection and start querying</div>
        <div class="hint">
          Use the sidebar to manage connections, or press
          <span class="kbd">Ctrl+N</span> to create a new query tab
          &middot;
          <span class="kbd">Ctrl+P</span> to search
        </div>
      </div>
    {/if}
  </div>

  <div class="statusbar-area">
    <StatusBar executionTime={lastExecutionTime} rowCount={lastRowCount} />
  </div>
</div>

{#if uiStore.showConnectionModal}
  <ConnectionModal />
{/if}

{#if uiStore.showConfirmDialog}
  <ConfirmDialog />
{/if}

{#if uiStore.showCreateTableModal}
  <CreateTableModal />
{/if}

{#if uiStore.showAlterTableModal}
  <AlterTableModal />
{/if}

{#if uiStore.showIndexModal}
  <IndexModal />
{/if}

{#if uiStore.showShortcutsModal}
  <ShortcutsModal />
{/if}

{#if uiStore.showCommandPalette}
  <CommandPalette />
{/if}

{#if uiStore.showSettingsModal}
  <SettingsModal />
{/if}

{#if uiStore.errorMessage}
  <div class="toast toast-error">
    {uiStore.errorMessage}
  </div>
{/if}

{#if uiStore.successMessage}
  <div class="toast toast-success">
    {uiStore.successMessage}
  </div>
{/if}

<style>
  .app-layout {
    display: grid;
    grid-template-rows: 40px 1fr 28px;
    grid-template-columns: var(--sidebar-width) 1fr;
    grid-template-areas:
      "toolbar toolbar"
      "sidebar main"
      "statusbar statusbar";
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .toolbar-area {
    grid-area: toolbar;
  }

  .sidebar-area {
    grid-area: sidebar;
    overflow: hidden;
    border-right: 1px solid var(--border-color);
    transition: width var(--transition-normal);
  }

  .sidebar-collapsed .sidebar-area {
    border-right: none;
  }

  .main-area {
    grid-area: main;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-primary);
  }

  .tab-content-wrapper {
    flex: 1;
    overflow: hidden;
  }

  .statusbar-area {
    grid-area: statusbar;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    user-select: none;
  }

  .empty-state .icon {
    font-size: 48px;
    opacity: 0.3;
  }

  .empty-state .message {
    font-size: 14px;
    color: var(--text-muted);
  }

  .empty-state .hint {
    font-size: 12px;
    color: var(--text-muted);
    opacity: 0.7;
  }
</style>
