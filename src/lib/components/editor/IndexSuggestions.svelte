<script lang="ts">
  import type { IndexSuggestion } from '$lib/types/query';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let {
    suggestions = [],
    connectionId = '',
  }: {
    suggestions: IndexSuggestion[];
    connectionId: string;
  } = $props();

  let collapsed = $state(false);

  function openInQueryTab(sql: string) {
    const connId = connectionId || connectionStore.activeConnectionId;
    if (!connId) return;
    const tabId = tabStore.newQueryTab(connId);
    tabStore.updateTabSql(tabId, sql);
  }

  function copyDdl(sql: string) {
    navigator.clipboard.writeText(sql).then(() => {
      uiStore.showSuccess('DDL copied to clipboard');
    });
  }
</script>

{#if suggestions.length > 0}
  <div class="index-suggestions">
    <button class="suggestions-header" onclick={() => collapsed = !collapsed}>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!collapsed}>
        <polyline points="9 18 15 12 9 6"></polyline>
      </svg>
      <span class="header-label">Index Suggestions</span>
      <span class="suggestions-count">{suggestions.length}</span>
    </button>

    {#if !collapsed}
      <div class="suggestions-list">
        {#each suggestions as suggestion}
          <div class="suggestion-item">
            <div class="suggestion-info">
              <div class="suggestion-table">
                <span class="table-name">{suggestion.table}</span>
                <span class="column-list">({suggestion.columns.join(', ')})</span>
              </div>
              <div class="suggestion-reason">{suggestion.reason}</div>
            </div>
            <div class="suggestion-actions">
              <button
                class="action-btn"
                onclick={() => copyDdl(suggestion.sql)}
                title="Copy DDL"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
              <button
                class="action-btn primary"
                onclick={() => openInQueryTab(suggestion.sql)}
                title="Open in Query Tab"
              >Create</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .index-suggestions {
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .suggestions-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 10px;
    border: none;
    background: none;
    color: var(--text-secondary);
    font-size: 11px;
    font-family: var(--font-sans);
    font-weight: 600;
    cursor: pointer;
    text-align: left;
  }

  .suggestions-header:hover {
    background: var(--bg-hover);
  }

  .suggestions-header svg {
    transition: transform 0.15s ease;
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .suggestions-header svg.rotated {
    transform: rotate(90deg);
  }

  .header-label {
    flex: 1;
  }

  .suggestions-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    font-size: 9px;
    font-weight: 700;
    color: var(--bg-primary);
    background: var(--warning, #fab387);
    border-radius: 8px;
  }

  .suggestions-list {
    padding: 4px 10px 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 160px;
    overflow-y: auto;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
  }

  .suggestion-info {
    flex: 1;
    min-width: 0;
  }

  .suggestion-table {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-family: var(--font-mono);
  }

  .table-name {
    font-weight: 600;
    color: var(--accent);
  }

  .column-list {
    color: var(--text-secondary);
  }

  .suggestion-reason {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 2px;
  }

  .suggestion-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 3px 6px;
    font-size: 10px;
    font-family: var(--font-sans);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn.primary {
    color: var(--accent);
    border-color: var(--accent);
  }

  .action-btn.primary:hover {
    background: rgba(122, 162, 247, 0.1);
  }
</style>
