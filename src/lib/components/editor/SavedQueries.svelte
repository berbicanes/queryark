<script lang="ts">
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import type { SavedQuery } from '$lib/types/query';

  let { connectionId, onselect, onclose }: {
    connectionId: string;
    onselect: (sql: string) => void;
    onclose: () => void;
  } = $props();

  let renamingId = $state<string | null>(null);
  let renameValue = $state('');

  let entries = $derived(savedQueriesStore.getByConnection(connectionId));

  function formatDate(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleDateString([], { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function truncateSql(sql: string, maxLen = 80): string {
    const oneline = sql.replace(/\s+/g, ' ').trim();
    if (oneline.length <= maxLen) return oneline;
    return oneline.slice(0, maxLen) + '...';
  }

  function startRename(query: SavedQuery) {
    renamingId = query.id;
    renameValue = query.name;
  }

  function finishRename() {
    if (renamingId && renameValue.trim()) {
      savedQueriesStore.rename(renamingId, renameValue.trim());
    }
    renamingId = null;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      finishRename();
    } else if (e.key === 'Escape') {
      renamingId = null;
    }
  }

  function handleDelete(e: MouseEvent, id: string) {
    e.stopPropagation();
    savedQueriesStore.remove(id);
  }
</script>

<div class="saved-queries">
  <div class="saved-header">
    <span class="saved-title">Saved Queries</span>
    <button class="btn-icon" onclick={onclose} title="Close">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>
  <div class="saved-list">
    {#if entries.length === 0}
      <div class="saved-empty">No saved queries</div>
    {:else}
      {#each entries as query (query.id)}
        <div
          class="saved-entry"
          onclick={() => onselect(query.sql)}
          onkeydown={(e) => { if (e.key === 'Enter') onselect(query.sql); }}
          role="button"
          tabindex="0"
          title={query.sql}
        >
          <div class="entry-header">
            {#if renamingId === query.id}
              <!-- svelte-ignore a11y_autofocus -->
              <input
                class="rename-input"
                bind:value={renameValue}
                onblur={finishRename}
                onkeydown={handleRenameKeydown}
                autofocus
                onclick={(e) => e.stopPropagation()}
              />
            {:else}
              <span
                class="entry-name"
                ondblclick={(e) => { e.stopPropagation(); startRename(query); }}
              >{query.name}</span>
            {/if}
            <div class="entry-actions">
              <span class="entry-date">{formatDate(query.updatedAt)}</span>
              <button
                class="btn-delete"
                onclick={(e) => handleDelete(e, query.id)}
                title="Delete"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
          </div>
          <div class="entry-sql">{truncateSql(query.sql)}</div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .saved-queries {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border-color);
  }

  .saved-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .saved-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
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

  .saved-list {
    flex: 1;
    overflow-y: auto;
  }

  .saved-empty {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }

  .saved-entry {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 8px;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    border-bottom: 1px solid var(--border-color);
    font-family: var(--font-sans);
    font-size: inherit;
  }

  .saved-entry:hover {
    background: var(--bg-hover);
  }

  .entry-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
  }

  .entry-name {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .entry-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .entry-date {
    font-size: 11px;
    color: var(--text-muted);
  }

  .btn-delete {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    opacity: 0;
  }

  .saved-entry:hover .btn-delete {
    opacity: 1;
  }

  .btn-delete:hover {
    background: var(--bg-hover);
    color: var(--error);
  }

  .rename-input {
    padding: 1px 4px;
    font-size: 12px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    outline: none;
    font-family: var(--font-sans);
    width: 150px;
  }

  .entry-sql {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
