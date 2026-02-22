<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import TabBar from '$lib/components/tabs/TabBar.svelte';
  import TabContent from '$lib/components/tabs/TabContent.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import ConnectionModal from '$lib/components/modals/ConnectionModal.svelte';
  import ConfirmDialog from '$lib/components/modals/ConfirmDialog.svelte';

  let lastExecutionTime = $state<number | null>(null);
  let lastRowCount = $state<number | null>(null);

  onMount(() => {
    connectionStore.init();
    queryHistoryStore.init();
    savedQueriesStore.init();
  });

  function handleQueryResult(detail: { executionTime: number; rowCount: number }) {
    lastExecutionTime = detail.executionTime;
    lastRowCount = detail.rowCount;
  }
</script>

<div
  class="app-layout"
  style="--sidebar-width: {uiStore.sidebarWidth}px"
>
  <div class="toolbar-area">
    <Toolbar />
  </div>

  <div class="sidebar-area">
    <Sidebar />
  </div>

  <div class="main-area">
    {#if tabStore.tabs.length > 0}
      <TabBar />
      <div class="tab-content-wrapper">
        <TabContent onqueryresult={handleQueryResult} />
      </div>
    {:else}
      <div class="empty-state">
        <div class="icon">&#x1F5C4;</div>
        <div class="message">Open a connection and start querying</div>
        <div class="hint">
          Use the sidebar to manage connections, or press
          <span class="kbd">Ctrl+N</span> to create a new query
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

{#if uiStore.errorMessage}
  <div class="toast toast-error">
    {uiStore.errorMessage}
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
