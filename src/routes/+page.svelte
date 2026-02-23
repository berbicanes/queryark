<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import * as schemaService from '$lib/services/schemaService';
  import { checkForUpdates } from '$lib/services/updaterService';
  import { DB_METADATA } from '$lib/types/database';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import TabBar from '$lib/components/tabs/TabBar.svelte';
  import TabContent from '$lib/components/tabs/TabContent.svelte';
  import SplitPane from '$lib/components/tabs/SplitPane.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import WelcomeScreen from '$lib/components/WelcomeScreen.svelte';
  import ConnectionModal from '$lib/components/modals/ConnectionModal.svelte';
  import ConfirmDialog from '$lib/components/modals/ConfirmDialog.svelte';
  import CreateTableModal from '$lib/components/modals/CreateTableModal.svelte';
  import AlterTableModal from '$lib/components/modals/AlterTableModal.svelte';
  import IndexModal from '$lib/components/modals/IndexModal.svelte';
  import ShortcutsModal from '$lib/components/modals/ShortcutsModal.svelte';
  import CommandPalette from '$lib/components/modals/CommandPalette.svelte';
  import SettingsModal from '$lib/components/modals/SettingsModal.svelte';
  import AboutModal from '$lib/components/modals/AboutModal.svelte';

  let lastExecutionTime = $state<number | null>(null);
  let lastRowCount = $state<number | null>(null);

  onMount(async () => {
    await connectionStore.init();
    queryHistoryStore.init();
    savedQueriesStore.init();
    await settingsStore.init();

    // Restore sidebar layout from persisted settings
    uiStore.sidebarWidth = settingsStore.sidebarWidth;
    uiStore.sidebarCollapsed = settingsStore.sidebarCollapsed;

    // Restore window geometry
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const { LogicalSize, LogicalPosition } = await import('@tauri-apps/api/dpi');
      const appWindow = getCurrentWindow();

      if (settingsStore.windowMaximized) {
        await appWindow.maximize();
      } else {
        if (settingsStore.windowWidth && settingsStore.windowHeight) {
          await appWindow.setSize(new LogicalSize(settingsStore.windowWidth, settingsStore.windowHeight));
        }
        if (settingsStore.windowX !== null && settingsStore.windowY !== null) {
          await appWindow.setPosition(new LogicalPosition(settingsStore.windowX, settingsStore.windowY));
        }
      }

      // Save state on close
      appWindow.onCloseRequested(async () => {
        try {
          const isMaximized = await appWindow.isMaximized();
          if (!isMaximized) {
            const size = await appWindow.innerSize();
            const pos = await appWindow.outerPosition();
            settingsStore.setWindowState(size.width, size.height, pos.x, pos.y, false);
          } else {
            settingsStore.setWindowState(
              settingsStore.windowWidth, settingsStore.windowHeight,
              settingsStore.windowX ?? 0, settingsStore.windowY ?? 0, true
            );
          }
          settingsStore.setSidebarLayout(uiStore.sidebarWidth, uiStore.sidebarCollapsed);
        } catch {
          // Best-effort save
        }
      });
    } catch {
      // Not running in Tauri (e.g. dev server in browser)
    }

    // Check for updates after a short delay
    setTimeout(() => checkForUpdates(), 5000);

    // Session restore: reopen tabs and reconnect
    if (settingsStore.restoreSession) {
      await tabStore.init();

      if (settingsStore.lastActiveConnectionId) {
        const conn = connectionStore.connections.find(
          c => c.config.id === settingsStore.lastActiveConnectionId
        );
        if (conn && conn.status !== 'connected') {
          // Silently attempt reconnection
          connectionService.connect(conn.config).catch(() => {});
        }
      }
    }
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
        if (tabStore.activeTabId) {
          const id = tabStore.activeTabId;
          if (settingsStore.confirmBeforeDelete && tabStore.hasContent(id)) {
            uiStore.confirm('This tab has unsaved content. Close it anyway?', () => tabStore.closeTab(id));
          } else {
            tabStore.closeTab(id);
          }
        }
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
      <WelcomeScreen onAddConnection={() => uiStore.openConnectionModal()} />
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

{#if uiStore.showAboutModal}
  <AboutModal />
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

</style>
