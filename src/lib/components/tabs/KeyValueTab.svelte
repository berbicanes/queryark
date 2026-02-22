<script lang="ts">
  import { onMount } from 'svelte';
  import type { Tab } from '$lib/types/tabs';
  import * as kvService from '$lib/services/keyvalueService';
  import KeyValueViewer from '$lib/components/viewers/KeyValueViewer.svelte';

  let { tab }: { tab: Tab } = $props();

  let keys = $state<string[]>([]);
  let isLoading = $state(false);
  let pattern = $state('*');
  let selectedKey = $state<string | null>(null);
  let selectedValue = $state<string | null>(null);
  let selectedType = $state<string>('string');

  // Set value dialog
  let showSetValue = $state(false);
  let newKey = $state('');
  let newValue = $state('');
  let isSaving = $state(false);

  async function loadKeys() {
    isLoading = true;
    try {
      keys = await kvService.scanKeys(tab.connectionId, pattern, 200);
    } finally {
      isLoading = false;
    }
  }

  async function selectKey(key: string) {
    selectedKey = key;
    const [type, value] = await Promise.all([
      kvService.getKeyType(tab.connectionId, key),
      kvService.getValue(tab.connectionId, key),
    ]);
    selectedType = type;
    selectedValue = value;
  }

  async function handleSetValue() {
    if (!newKey.trim()) return;
    isSaving = true;
    try {
      const ok = await kvService.setValue(tab.connectionId, newKey, newValue);
      if (ok) {
        showSetValue = false;
        newKey = '';
        newValue = '';
        await loadKeys();
      }
    } finally {
      isSaving = false;
    }
  }

  async function handleDeleteKey() {
    if (!selectedKey) return;
    const deleted = await kvService.deleteKeys(tab.connectionId, [selectedKey]);
    if (deleted > 0) {
      selectedKey = null;
      selectedValue = null;
      await loadKeys();
    }
  }

  function handleSearch() {
    loadKeys();
  }

  onMount(() => {
    loadKeys();
  });
</script>

<div class="kv-tab">
  <div class="kv-toolbar">
    <input
      class="search-input"
      type="text"
      bind:value={pattern}
      placeholder="Search pattern (e.g. user:*)"
      onkeydown={(e) => { if (e.key === 'Enter') handleSearch(); }}
    />
    <button class="btn btn-sm" onclick={handleSearch}>Search</button>
    <div class="toolbar-spacer"></div>
    <button class="btn btn-sm" onclick={() => showSetValue = !showSetValue}>+ Set Key</button>
    {#if selectedKey}
      <button class="btn btn-sm btn-danger" onclick={handleDeleteKey}>Delete</button>
    {/if}
    <button class="btn btn-sm" onclick={loadKeys}>Refresh</button>
  </div>

  {#if showSetValue}
    <div class="set-panel">
      <div class="set-row">
        <input class="set-input" type="text" bind:value={newKey} placeholder="Key" />
        <input class="set-input" type="text" bind:value={newValue} placeholder="Value" style="flex: 2" />
        <button class="btn btn-sm btn-primary" onclick={handleSetValue} disabled={isSaving}>
          {isSaving ? 'Saving...' : 'Set'}
        </button>
        <button class="btn btn-sm" onclick={() => showSetValue = false}>Cancel</button>
      </div>
    </div>
  {/if}

  <div class="kv-content">
    <div class="key-list">
      {#if isLoading}
        <div class="loading-state">
          <span class="spinner"></span>
          Loading...
        </div>
      {:else if keys.length === 0}
        <div class="empty-state">
          <span class="text-muted">No keys found</span>
        </div>
      {:else}
        {#each keys as key}
          <button
            class="key-item"
            class:selected={selectedKey === key}
            onclick={() => selectKey(key)}
          >
            <span class="key-name truncate text-mono">{key}</span>
          </button>
        {/each}
      {/if}
    </div>

    <div class="value-panel">
      {#if selectedKey && selectedValue !== null}
        <KeyValueViewer keyType={selectedType} value={selectedValue} />
      {:else}
        <div class="empty-state">
          <span class="text-muted">Select a key to view its value</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="kv-status">
    <span class="text-muted">{keys.length} keys</span>
    {#if selectedKey}
      <span class="text-muted">| {selectedKey} ({selectedType})</span>
    {/if}
  </div>
</div>

<style>
  .kv-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .kv-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .search-input {
    width: 240px;
    font-size: 12px;
    padding: 4px 8px;
    font-family: var(--font-mono);
  }

  .toolbar-spacer {
    flex: 1;
  }

  .set-panel {
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .set-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .set-input {
    flex: 1;
    font-size: 12px;
    padding: 4px 8px;
    font-family: var(--font-mono);
  }

  .kv-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .key-list {
    width: 300px;
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
  }

  .value-panel {
    flex: 1;
    overflow: hidden;
  }

  .key-item {
    display: block;
    width: 100%;
    padding: 4px 12px;
    text-align: left;
    border: none;
    background: none;
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
    border-bottom: 1px solid var(--border-color);
    transition: background var(--transition-fast);
  }

  .key-item:hover {
    background: var(--bg-hover);
  }

  .key-item.selected {
    background: var(--bg-active);
    border-left: 2px solid var(--accent);
  }

  .key-name {
    display: block;
  }

  .kv-status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 11px;
    flex-shrink: 0;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }
</style>
