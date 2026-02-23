<script lang="ts">
  import { tick } from 'svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { DB_METADATA } from '$lib/types/database';
  import { DEFAULT_SHORTCUTS } from '$lib/types/shortcuts';
  import { fuzzyFilter, type FuzzyItem } from '$lib/utils/fuzzyMatch';

  interface PaletteItem {
    id: string;
    label: string;
    category: 'connection' | 'table' | 'query' | 'history' | 'action';
    icon: string;
    detail?: string;
    action: () => void;
  }

  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl: HTMLInputElement;

  // Build the list of all searchable items
  let allItems = $derived.by(() => {
    const items: PaletteItem[] = [];

    // Connections
    for (const conn of connectionStore.connections) {
      const meta = DB_METADATA[conn.config.db_type];
      items.push({
        id: `conn:${conn.config.id}`,
        label: conn.config.name,
        category: 'connection',
        icon: conn.status === 'connected' ? '●' : '○',
        detail: meta.badge,
        action: () => {
          connectionStore.setActive(conn.config.id);
          close();
        },
      });
    }

    // Tables from schema cache
    const connId = connectionStore.activeConnectionId;
    if (connId) {
      const schemas = schemaStore.getSchemas(connId);
      for (const schema of schemas) {
        const tables = schemaStore.getTables(connId, schema.name);
        for (const table of tables) {
          items.push({
            id: `table:${schema.name}.${table.name}`,
            label: table.name,
            category: 'table',
            icon: table.table_type === 'view' ? '◇' : '▤',
            detail: schema.name,
            action: () => {
              tabStore.openTab({
                type: 'table',
                title: table.name,
                connectionId: connId,
                schema: schema.name,
                table: table.name,
              });
              close();
            },
          });
        }
      }

      // Containers from browser cache
      const containers = schemaStore.getContainers(connId);
      for (const container of containers) {
        items.push({
          id: `container:${container.name}`,
          label: container.name,
          category: 'table',
          icon: '▤',
          action: () => {
            close();
          },
        });
      }
    }

    // Saved queries
    for (const sq of savedQueriesStore.queries) {
      items.push({
        id: `saved:${sq.id}`,
        label: sq.name,
        category: 'query',
        icon: '⌘',
        detail: sq.sql.substring(0, 50),
        action: () => {
          if (sq.connectionId) {
            connectionStore.setActive(sq.connectionId);
            const tabId = tabStore.newQueryTab(sq.connectionId);
            tabStore.updateTabSql(tabId, sq.sql);
          }
          close();
        },
      });
    }

    // Recent history (last 50)
    const recent = queryHistoryStore.entries.slice(0, 50);
    for (const entry of recent) {
      items.push({
        id: `history:${entry.id}`,
        label: entry.sql.substring(0, 80).replace(/\n/g, ' '),
        category: 'history',
        icon: '↺',
        detail: `${entry.executionTimeMs}ms`,
        action: () => {
          if (entry.connectionId) {
            connectionStore.setActive(entry.connectionId);
            const tabId = tabStore.newQueryTab(entry.connectionId);
            tabStore.updateTabSql(tabId, entry.sql);
          }
          close();
        },
      });
    }

    // Actions (shortcuts)
    const actionMap: Record<string, () => void> = {
      newQuery: () => {
        if (connectionStore.activeConnectionId) {
          tabStore.newQueryTab(connectionStore.activeConnectionId);
        }
      },
      toggleTheme: () => settingsStore.toggleTheme(),
      toggleSidebar: () => { uiStore.sidebarCollapsed = !uiStore.sidebarCollapsed; },
      shortcuts: () => { uiStore.showShortcutsModal = true; },
      refreshSchema: () => {
        window.dispatchEvent(new CustomEvent('queryark:refresh-schema'));
      },
      openDiagram: () => {
        if (connectionStore.activeConnectionId) {
          tabStore.openTab({
            type: 'diagram',
            title: 'ER Diagram',
            connectionId: connectionStore.activeConnectionId,
            diagramSchemas: [],
          });
        }
      },
      openTableDiff: () => {
        if (connectionStore.activeConnectionId) {
          tabStore.openTab({
            type: 'tablediff',
            title: 'Table Diff',
            connectionId: connectionStore.activeConnectionId,
          });
        }
      },
      openDataDiff: () => {
        if (connectionStore.activeConnectionId) {
          tabStore.openTab({
            type: 'datadiff',
            title: 'Data Diff',
            connectionId: connectionStore.activeConnectionId,
          });
        }
      },
      openVisualQuery: () => {
        if (connectionStore.activeConnectionId) {
          tabStore.openTab({
            type: 'visualquery',
            title: 'Visual Query',
            connectionId: connectionStore.activeConnectionId,
          });
        }
      },
      openSnippets: () => { uiStore.showSnippetLibrary = true; },
      openWorkspaces: () => { uiStore.showWorkspaceModal = true; },
      openBookmarks: () => { uiStore.showBookmarkList = true; },
      toggleChart: () => {
        window.dispatchEvent(new CustomEvent('queryark:toggle-chart'));
      },
      compareResults: () => {
        window.dispatchEvent(new CustomEvent('queryark:compare-results'));
      },
    };
    for (const shortcut of DEFAULT_SHORTCUTS) {
      if (actionMap[shortcut.id]) {
        items.push({
          id: `action:${shortcut.id}`,
          label: shortcut.label,
          category: 'action',
          icon: '>',
          detail: settingsStore.getBinding(shortcut.id),
          action: () => {
            actionMap[shortcut.id]();
            close();
          },
        });
      }
    }

    return items;
  });

  let filtered = $derived.by(() => {
    if (!query.trim()) {
      // Show recently used connections and actions
      return allItems.slice(0, 25).map(item => ({ item, score: 0, ranges: [] as [number, number][] }));
    }
    return fuzzyFilter(query, allItems, item => item.label).slice(0, 25);
  });

  // Reset selection when query changes
  $effect(() => {
    query; // track
    selectedIndex = 0;
  });

  // Auto-focus input
  $effect(() => {
    tick().then(() => {
      inputEl?.focus();
    });
  });

  function close() {
    uiStore.showCommandPalette = false;
    query = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        close();
        break;
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        if (filtered[selectedIndex]) {
          filtered[selectedIndex].item.action();
        }
        break;
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }

  const categoryLabels: Record<string, string> = {
    connection: 'Connection',
    table: 'Table',
    query: 'Saved Query',
    history: 'History',
    action: 'Action',
  };

  function highlightText(text: string, ranges: [number, number][]): Array<{ text: string; highlight: boolean }> {
    if (ranges.length === 0) return [{ text, highlight: false }];
    const parts: Array<{ text: string; highlight: boolean }> = [];
    let last = 0;
    for (const [start, end] of ranges) {
      if (start > last) parts.push({ text: text.slice(last, start), highlight: false });
      parts.push({ text: text.slice(start, end), highlight: true });
      last = end;
    }
    if (last < text.length) parts.push({ text: text.slice(last), highlight: false });
    return parts;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="palette-overlay" onclick={handleOverlayClick} onkeydown={handleKeydown}>
  <div class="palette-card">
    <div class="palette-input-wrapper">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" class="search-icon">
        <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" stroke-width="1.5"/>
        <path d="M10 10l4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <input
        bind:this={inputEl}
        bind:value={query}
        class="palette-input"
        placeholder="Search connections, tables, queries, actions..."
        spellcheck="false"
        autocomplete="off"
      />
      <span class="kbd palette-esc">Esc</span>
    </div>

    {#if filtered.length > 0}
      <div class="palette-results">
        {#each filtered as result, i}
          {@const item = result.item}
          <button
            class="palette-item"
            class:selected={i === selectedIndex}
            onclick={() => item.action()}
            onmouseenter={() => { selectedIndex = i; }}
          >
            <span class="item-icon">{item.icon}</span>
            <span class="item-label">
              {#each highlightText(item.label, result.ranges) as part}
                {#if part.highlight}
                  <mark>{part.text}</mark>
                {:else}
                  {part.text}
                {/if}
              {/each}
            </span>
            {#if item.detail}
              <span class="item-detail truncate">{item.detail}</span>
            {/if}
            <span class="item-category">{categoryLabels[item.category]}</span>
          </button>
        {/each}
      </div>
    {:else}
      <div class="palette-empty">No results found</div>
    {/if}
  </div>
</div>

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    justify-content: center;
    padding-top: 15vh;
    z-index: 1100;
    backdrop-filter: blur(2px);
  }

  .palette-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 560px;
    max-height: 450px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    align-self: flex-start;
  }

  .palette-input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-color);
  }

  .search-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .palette-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 14px;
    color: var(--text-primary);
    outline: none;
    padding: 0;
  }

  .palette-input::placeholder {
    color: var(--text-muted);
  }

  .palette-esc {
    flex-shrink: 0;
  }

  .palette-results {
    overflow-y: auto;
    padding: 4px 0;
  }

  .palette-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 14px;
    font-size: 13px;
    color: var(--text-primary);
    text-align: left;
    border: none;
    background: none;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .palette-item.selected {
    background: var(--bg-hover);
  }

  .palette-item:hover {
    background: var(--bg-hover);
  }

  .item-icon {
    width: 18px;
    text-align: center;
    flex-shrink: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .item-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-label :global(mark) {
    background: rgba(122, 162, 247, 0.25);
    color: var(--accent);
    border-radius: 1px;
    padding: 0 1px;
  }

  .item-detail {
    font-size: 11px;
    color: var(--text-muted);
    max-width: 150px;
  }

  .item-category {
    font-size: 10px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    flex-shrink: 0;
    opacity: 0.6;
  }

  .palette-empty {
    padding: 20px;
    text-align: center;
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
