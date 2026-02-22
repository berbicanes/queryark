<script lang="ts">
  import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';
  import type { QueryHistoryEntry } from '$lib/types/query';

  let { connectionId, onselect, onclose }: {
    connectionId: string;
    onselect: (sql: string) => void;
    onclose: () => void;
  } = $props();

  let searchQuery = $state('');

  let filteredEntries = $derived.by(() => {
    if (searchQuery.trim()) {
      return queryHistoryStore.search(searchQuery, connectionId);
    }
    return queryHistoryStore.getEntries(connectionId);
  });

  function formatTime(ts: number): string {
    const d = new Date(ts);
    const now = new Date();
    const isToday = d.toDateString() === now.toDateString();
    const time = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (isToday) return time;
    return `${d.toLocaleDateString([], { month: 'short', day: 'numeric' })} ${time}`;
  }

  function truncateSql(sql: string, maxLen = 80): string {
    const oneline = sql.replace(/\s+/g, ' ').trim();
    if (oneline.length <= maxLen) return oneline;
    return oneline.slice(0, maxLen) + '...';
  }

  function handleClear() {
    queryHistoryStore.clear();
  }
</script>

<div class="query-history">
  <div class="history-header">
    <span class="history-title">Query History</span>
    <div class="history-actions">
      <button class="btn-icon" onclick={handleClear} title="Clear history">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      </button>
      <button class="btn-icon" onclick={onclose} title="Close">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  </div>
  <div class="history-search">
    <input
      type="text"
      placeholder="Search queries..."
      bind:value={searchQuery}
    />
  </div>
  <div class="history-list">
    {#if filteredEntries.length === 0}
      <div class="history-empty">No queries found</div>
    {:else}
      {#each filteredEntries as entry (entry.id)}
        <button
          class="history-entry"
          class:has-error={!!entry.error}
          onclick={() => onselect(entry.sql)}
          title={entry.sql}
        >
          <div class="entry-header">
            <span class="entry-time">{formatTime(entry.executedAt)}</span>
            <span class="entry-stats">
              {#if entry.error}
                <span class="entry-error">Error</span>
              {:else}
                <span class="entry-rows">{entry.rowCount} rows</span>
              {/if}
              <span class="entry-duration">{entry.executionTimeMs}ms</span>
            </span>
          </div>
          <div class="entry-sql">{truncateSql(entry.sql)}</div>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .query-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border-color);
  }

  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .history-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .history-actions {
    display: flex;
    gap: 2px;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .btn-icon:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .history-search {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .history-search input {
    width: 100%;
    padding: 4px 8px;
    font-size: 12px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    outline: none;
    font-family: var(--font-sans);
  }

  .history-search input:focus {
    border-color: var(--accent);
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
  }

  .history-empty {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }

  .history-entry {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 8px;
    border: none;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    border-bottom: 1px solid var(--border-color);
    font-family: var(--font-sans);
  }

  .history-entry:hover {
    background: var(--bg-hover);
  }

  .entry-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
  }

  .entry-time {
    font-size: 11px;
    color: var(--text-muted);
  }

  .entry-stats {
    display: flex;
    gap: 8px;
    font-size: 11px;
  }

  .entry-rows {
    color: var(--text-muted);
  }

  .entry-error {
    color: var(--error);
  }

  .entry-duration {
    color: var(--text-muted);
  }

  .entry-sql {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .history-entry.has-error .entry-sql {
    color: var(--error);
    opacity: 0.7;
  }
</style>
