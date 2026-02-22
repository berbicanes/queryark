<script lang="ts">
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let { paneId }: { paneId?: 'left' | 'right' } = $props();

  let scrollContainer: HTMLDivElement;
  let draggedTabId = $state<string | null>(null);
  let dropTargetIndex = $state<number | null>(null);
  let contextMenu = $state<{ x: number; y: number; tabId: string } | null>(null);

  // Filter tabs by pane if in split mode, otherwise use sorted tabs (pinned first)
  let visibleTabs = $derived.by(() => {
    if (tabStore.splitMode && paneId) {
      const paneTabIds = paneId === 'left' ? tabStore.leftPaneTabs : tabStore.rightPaneTabs;
      return tabStore.tabs.filter(t => paneTabIds.includes(t.id));
    }
    return tabStore.sortedTabs;
  });

  let currentActiveTabId = $derived.by(() => {
    if (tabStore.splitMode && paneId) {
      return paneId === 'left' ? tabStore.activeLeftTabId : tabStore.activeRightTabId;
    }
    return tabStore.activeTabId;
  });

  function getConnectionColor(connectionId: string): string | undefined {
    const conn = connectionStore.connections.find(c => c.config.id === connectionId);
    return conn?.config.color;
  }

  function handleTabClick(id: string) {
    tabStore.setActive(id);
    if (paneId) tabStore.activePaneId = paneId;
  }

  function handleTabClose(e: MouseEvent, id: string) {
    e.stopPropagation();
    tabStore.closeTab(id);
  }

  function handleNewTab() {
    if (paneId) tabStore.activePaneId = paneId;
    if (connectionStore.activeConnectionId) {
      tabStore.newQueryTab(connectionStore.activeConnectionId);
    } else {
      uiStore.showError('No active connection. Connect to a database first.');
    }
  }

  function handleMiddleClick(e: MouseEvent, id: string) {
    if (e.button === 1) {
      e.preventDefault();
      tabStore.closeTab(id);
    }
  }

  function getTabIcon(type: string): string {
    return type === 'query' ? '\u{2318}' : '\u{1F5C3}';
  }

  // --- Drag and Drop ---
  function handleDragStart(e: DragEvent, tabId: string) {
    draggedTabId = tabId;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', tabId);
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dropTargetIndex = index;
  }

  function handleDragLeave() {
    dropTargetIndex = null;
  }

  function handleDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    if (!draggedTabId) return;

    const tabs = visibleTabs;
    const fromIndex = tabStore.tabs.findIndex(t => t.id === draggedTabId);
    const targetTab = tabs[targetIndex];
    const toIndex = targetTab ? tabStore.tabs.findIndex(t => t.id === targetTab.id) : tabStore.tabs.length - 1;

    if (fromIndex >= 0 && toIndex >= 0) {
      tabStore.moveTab(fromIndex, toIndex);
    }

    draggedTabId = null;
    dropTargetIndex = null;
  }

  function handleDragEnd() {
    draggedTabId = null;
    dropTargetIndex = null;
  }

  // --- Context Menu ---
  function handleContextMenu(e: MouseEvent, tabId: string) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, tabId };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function ctxClose() {
    if (contextMenu) {
      tabStore.closeTab(contextMenu.tabId, true);
      closeContextMenu();
    }
  }

  function ctxCloseOthers() {
    if (!contextMenu) return;
    tabStore.closeOthers(contextMenu.tabId);
    closeContextMenu();
  }

  function ctxCloseAll() {
    tabStore.closeAll();
    closeContextMenu();
  }

  function ctxDuplicate() {
    if (!contextMenu) return;
    tabStore.duplicateTab(contextMenu.tabId);
    closeContextMenu();
  }

  function ctxTogglePin() {
    if (!contextMenu) return;
    tabStore.togglePin(contextMenu.tabId);
    closeContextMenu();
  }

  function ctxSplitRight() {
    if (!contextMenu) return;
    tabStore.splitTab(contextMenu.tabId);
    closeContextMenu();
  }

  function isTabPinned(tabId: string): boolean {
    return tabStore.tabs.find(t => t.id === tabId)?.pinned ?? false;
  }
</script>

<svelte:window onclick={closeContextMenu} />

<div class="tab-bar">
  <div class="tabs-scroll" bind:this={scrollContainer}>
    {#each visibleTabs as tab, i}
      {@const connColor = getConnectionColor(tab.connectionId)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tab"
        class:active={tab.id === currentActiveTabId}
        class:pinned={tab.pinned}
        class:drag-over={dropTargetIndex === i && draggedTabId !== tab.id}
        class:dragging={draggedTabId === tab.id}
        draggable="true"
        onclick={() => handleTabClick(tab.id)}
        onauxclick={(e) => handleMiddleClick(e, tab.id)}
        oncontextmenu={(e) => handleContextMenu(e, tab.id)}
        ondragstart={(e) => handleDragStart(e, tab.id)}
        ondragover={(e) => handleDragOver(e, i)}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDrop(e, i)}
        ondragend={handleDragEnd}
        onkeydown={(e) => { if (e.key === 'Enter') handleTabClick(tab.id); }}
        role="tab"
        tabindex="0"
        title={tab.title}
      >
        {#if connColor}
          <span class="tab-color-stripe" style="background: {connColor}"></span>
        {/if}
        {#if tab.pinned}
          <span class="pin-icon" title="Pinned">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
              <path d="M10 1L6 5l-3.5.5L5 8l-3 6 6-3 2.5 2.5L11 10l4-4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </span>
        {/if}
        <span class="tab-icon">{getTabIcon(tab.type)}</span>
        <span class="tab-title truncate">{tab.title}</span>
        {#if !tab.pinned}
          <button
            class="tab-close"
            onclick={(e) => handleTabClose(e, tab.id)}
            title="Close tab"
          >
            <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        {/if}
      </div>
    {/each}
  </div>

  <button class="new-tab-btn" onclick={handleNewTab} title="New query tab">
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
    </svg>
  </button>
</div>

{#if contextMenu}
  {@const pinned = isTabPinned(contextMenu.tabId)}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') closeContextMenu(); }}
  >
    <button class="context-item" onclick={ctxTogglePin}>
      {pinned ? 'Unpin' : 'Pin'}
    </button>
    <button class="context-item" onclick={ctxDuplicate}>Duplicate</button>
    <div class="context-divider"></div>
    <button class="context-item" onclick={ctxClose}>Close</button>
    <button class="context-item" onclick={ctxCloseOthers}>Close Others</button>
    <button class="context-item" onclick={ctxCloseAll}>Close All</button>
    {#if !tabStore.splitMode && tabStore.tabs.length >= 2}
      <div class="context-divider"></div>
      <button class="context-item" onclick={ctxSplitRight}>Split Right</button>
    {/if}
  </div>
{/if}

<style>
  .tab-bar {
    display: flex;
    align-items: stretch;
    height: 34px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .tabs-scroll {
    display: flex;
    align-items: stretch;
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }

  .tabs-scroll::-webkit-scrollbar {
    display: none;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 12px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    border: none;
    background: none;
    border-right: 1px solid var(--border-color);
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
    min-width: 100px;
    max-width: 200px;
    position: relative;
    flex-shrink: 0;
    user-select: none;
  }

  .tab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab.active {
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .tab.active::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
  }

  .tab.pinned {
    min-width: auto;
  }

  .tab.dragging {
    opacity: 0.4;
  }

  .tab.drag-over::before {
    content: '';
    position: absolute;
    left: -1px;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: var(--accent);
    border-radius: 1px;
    z-index: 5;
  }

  .tab-color-stripe {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    border-radius: 1px 1px 0 0;
  }

  .pin-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 10px;
  }

  .tab-icon {
    flex-shrink: 0;
    font-size: 11px;
    opacity: 0.6;
  }

  .tab-title {
    flex: 1;
    min-width: 0;
    text-align: left;
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    flex-shrink: 0;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
  }

  .tab:hover .tab-close,
  .tab.active .tab-close {
    opacity: 1;
  }

  .tab-close:hover {
    background: var(--bg-active);
    color: var(--text-primary);
  }

  .new-tab-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    flex-shrink: 0;
    color: var(--text-muted);
    border: none;
    background: none;
    cursor: pointer;
    transition: color var(--transition-fast), background var(--transition-fast);
    padding: 0;
    border-left: 1px solid var(--border-color);
  }

  .new-tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .context-menu {
    position: fixed;
    z-index: 500;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    padding: 4px 0;
    min-width: 140px;
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

  .context-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }
</style>
