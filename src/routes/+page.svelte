<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { favoritesStore } from '$lib/stores/favorites.svelte';
  import { snippetsStore } from '$lib/stores/snippets.svelte';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';
  import { workspacesStore } from '$lib/stores/workspaces.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import * as schemaService from '$lib/services/schemaService';
  import { checkForUpdates } from '$lib/services/updaterService.svelte';
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
  import SnippetModal from '$lib/components/modals/SnippetModal.svelte';
  import SnippetLibrary from '$lib/components/modals/SnippetLibrary.svelte';
  import SnippetVariablePrompt from '$lib/components/modals/SnippetVariablePrompt.svelte';
  import BookmarkList from '$lib/components/modals/BookmarkList.svelte';
  import WorkspaceModal from '$lib/components/modals/WorkspaceModal.svelte';
  import ParameterPrompt from '$lib/components/modals/ParameterPrompt.svelte';

  let lastExecutionTime = $state<number | null>(null);
  let lastRowCount = $state<number | null>(null);
  let startupComplete = $state(false);

  function handleSchemaRefresh() {
    const connId = connectionStore.activeConnectionId;
    if (!connId) return;
    const conn = connectionStore.activeConnection;
    const dbType = conn?.config.db_type;
    const category = dbType ? DB_METADATA[dbType]?.category : null;
    if (category === 'Relational' || category === 'Analytics' || category === 'WideColumn') {
      schemaService.refreshSchema(connId);
    } else {
      schemaService.refreshContainers(connId);
    }
  }

  onDestroy(() => {
    window.removeEventListener('queryark:refresh-schema', handleSchemaRefresh);
  });

  async function restoreWindowGeometry() {
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
  }

  async function restoreSession() {
    if (!settingsStore.restoreSession) return;
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

  onMount(async () => {
    // Phase 1: Initialize all stores in parallel
    await Promise.all([
      connectionStore.init(),
      settingsStore.init(),
      queryHistoryStore.init(),
      savedQueriesStore.init(),
      favoritesStore.init(),
      snippetsStore.init(),
      bookmarksStore.init(),
      workspacesStore.init(),
    ]);

    // Phase 2: Apply settings (depends on settingsStore)
    uiStore.sidebarWidth = settingsStore.sidebarWidth;
    uiStore.sidebarCollapsed = true;

    // Phase 3: Window geometry + session restore in parallel
    await Promise.all([
      restoreWindowGeometry(),
      restoreSession(),
    ]);

    // Phase 4: Event listeners, update check, mark startup complete
    window.addEventListener('queryark:refresh-schema', handleSchemaRefresh);
    setTimeout(() => checkForUpdates(), 5000);
    startupComplete = true;
  });

  // Dismiss home screen when a tab becomes active (after startup)
  $effect(() => {
    if (!startupComplete) return;
    if (tabStore.activeTabId && tabStore.tabs.length > 0 && uiStore.showHome) {
      uiStore.dismissHome();
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
    {#if uiStore.showHome}
      <WelcomeScreen onAddConnection={() => uiStore.openConnectionModal()} />
    {:else if tabStore.tabs.length === 0}
      <div class="empty-main">
        <span class="text-muted">No tabs open</span>
      </div>
    {:else if tabStore.splitMode}
      <SplitPane onqueryresult={handleQueryResult} />
    {:else}
      <TabBar />
      <div class="tab-content-wrapper">
        <TabContent onqueryresult={handleQueryResult} />
      </div>
    {/if}
  </div>

  {#if !uiStore.showHome}
    <div class="statusbar-area">
      <StatusBar executionTime={lastExecutionTime} rowCount={lastRowCount} />
    </div>
  {/if}
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

{#if uiStore.showSnippetModal}
  <SnippetModal />
{/if}

{#if uiStore.showSnippetLibrary}
  <SnippetLibrary />
{/if}

{#if uiStore.showSnippetVariablePrompt}
  <SnippetVariablePrompt />
{/if}

{#if uiStore.showBookmarkList}
  <BookmarkList />
{/if}

{#if uiStore.showWorkspaceModal}
  <WorkspaceModal />
{/if}

{#if uiStore.showParameterPrompt}
  <ParameterPrompt />
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
    grid-template-rows: 44px 1fr auto;
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

  .empty-main {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }

</style>
