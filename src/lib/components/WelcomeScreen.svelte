<script lang="ts">
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import { DB_METADATA } from '$lib/types/database';
  import type { DatabaseType, ConnectionState } from '$lib/types/connection';

  let { onAddConnection }: { onAddConnection: () => void } = $props();

  const supportedDbs: DatabaseType[] = [
    'PostgreSQL', 'MySQL', 'MariaDB', 'SQLite', 'MSSQL', 'CockroachDB',
    'Redshift', 'ClickHouse', 'MongoDB', 'Redis', 'Cassandra', 'ScyllaDB',
    'Neo4j', 'DynamoDB',
  ];

  let hasConnections = $derived(connectionStore.connections.length > 0);

  // Grouped connections for dashboard
  let groups = $derived(connectionStore.groups);
  let ungrouped = $derived(connectionStore.getConnectionsByGroup(null));

  function handleCardClick(conn: ConnectionState) {
    connectionStore.setActive(conn.config.id);
    if (conn.status !== 'connected') {
      connectionService.connect(conn.config);
    }
    uiStore.dismissHome();
  }

  function handleCardDblClick(conn: ConnectionState) {
    connectionStore.setActive(conn.config.id);
    if (conn.status !== 'connected') {
      connectionService.connect(conn.config);
    }
    uiStore.dismissHome();
  }
</script>

<div class="welcome-screen">
  {#if !hasConnections}
    <div class="welcome-hero">
      <div class="logo-icon">
        <svg width="64" height="64" viewBox="0 0 48 48" fill="none">
          <rect x="4" y="8" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5" fill="rgba(122, 162, 247, 0.1)"/>
          <rect x="4" y="20" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5" fill="rgba(122, 162, 247, 0.07)"/>
          <rect x="4" y="32" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5" fill="rgba(122, 162, 247, 0.04)"/>
          <circle cx="10" cy="12" r="2" fill="var(--accent)"/>
          <circle cx="10" cy="24" r="2" fill="var(--accent)"/>
          <circle cx="10" cy="36" r="2" fill="var(--accent)"/>
        </svg>
      </div>
      <h1 class="welcome-title">Welcome to DataForge</h1>
      <p class="welcome-subtitle">A fast, lightweight database IDE for developers</p>

      <button class="cta-btn" onclick={onAddConnection}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        Add Your First Connection
      </button>

      <div class="db-badges">
        {#each supportedDbs as db}
          {@const meta = DB_METADATA[db]}
          <span class="badge {meta.badgeClass}" title={meta.label}>{meta.badge}</span>
        {/each}
      </div>
    </div>
  {:else}
    <div class="dashboard">
      <div class="dashboard-header">
        <div class="dashboard-brand">
          <svg width="32" height="32" viewBox="0 0 48 48" fill="none">
            <rect x="4" y="8" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5" fill="rgba(122, 162, 247, 0.1)"/>
            <rect x="4" y="20" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5" fill="rgba(122, 162, 247, 0.07)"/>
            <rect x="4" y="32" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5" fill="rgba(122, 162, 247, 0.04)"/>
            <circle cx="10" cy="12" r="2" fill="var(--accent)"/>
            <circle cx="10" cy="24" r="2" fill="var(--accent)"/>
            <circle cx="10" cy="36" r="2" fill="var(--accent)"/>
          </svg>
          <span class="dashboard-title">DataForge</span>
          <span class="dashboard-subtitle">Your Connections</span>
        </div>
      </div>

      <div class="cards-area">
        {#if ungrouped.length > 0}
          <div class="cards-grid">
            {#each ungrouped as conn}
              {@const meta = DB_METADATA[conn.config.db_type]}
              <button
                class="conn-card"
                style={conn.config.color ? `--card-accent: ${conn.config.color}` : ''}
                onclick={() => handleCardClick(conn)}
                ondblclick={() => handleCardDblClick(conn)}
              >
                {#if conn.config.color}
                  <span class="card-stripe" style="background: {conn.config.color}"></span>
                {/if}
                <div class="card-top">
                  <span class="badge {meta.badgeClass}">{meta.badge}</span>
                  <span class="status-dot {conn.status}"></span>
                </div>
                <div class="card-name">{conn.config.name}</div>
                <div class="card-host">
                  {#if meta.requiresHost}
                    {conn.config.host || 'localhost'}{conn.config.port ? `:${conn.config.port}` : ''}
                  {:else if meta.requiresFilePath}
                    {conn.config.database || 'local'}
                  {:else}
                    {conn.config.database || meta.label}
                  {/if}
                </div>
              </button>
            {/each}

            <button class="conn-card add-card" onclick={onAddConnection}>
              <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
                <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
              <span>Add Connection</span>
            </button>
          </div>
        {/if}

        {#each groups as group}
          {@const groupConns = connectionStore.getConnectionsByGroup(group)}
          <div class="group-label">{group}</div>
          <div class="cards-grid">
            {#each groupConns as conn}
              {@const meta = DB_METADATA[conn.config.db_type]}
              <button
                class="conn-card"
                style={conn.config.color ? `--card-accent: ${conn.config.color}` : ''}
                onclick={() => handleCardClick(conn)}
                ondblclick={() => handleCardDblClick(conn)}
              >
                {#if conn.config.color}
                  <span class="card-stripe" style="background: {conn.config.color}"></span>
                {/if}
                <div class="card-top">
                  <span class="badge {meta.badgeClass}">{meta.badge}</span>
                  <span class="status-dot {conn.status}"></span>
                </div>
                <div class="card-name">{conn.config.name}</div>
                <div class="card-host">
                  {#if meta.requiresHost}
                    {conn.config.host || 'localhost'}{conn.config.port ? `:${conn.config.port}` : ''}
                  {:else if meta.requiresFilePath}
                    {conn.config.database || 'local'}
                  {:else}
                    {conn.config.database || meta.label}
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/each}

        {#if ungrouped.length === 0}
          <div class="cards-grid">
            <button class="conn-card add-card" onclick={onAddConnection}>
              <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
                <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
              <span>Add Connection</span>
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <div class="shortcuts-hint">
    <span class="kbd">Ctrl+N</span> New query
    <span class="sep"></span>
    <span class="kbd">Ctrl+P</span> Command palette
    <span class="sep"></span>
    <span class="kbd">Ctrl+B</span> Toggle sidebar
  </div>
</div>

<style>
  .welcome-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    user-select: none;
    background:
      radial-gradient(ellipse 80% 60% at 50% 40%, rgba(122, 162, 247, 0.06) 0%, transparent 70%),
      var(--bg-primary);
  }

  .welcome-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .logo-icon {
    margin-bottom: 8px;
    filter: drop-shadow(0 0 12px rgba(122, 162, 247, 0.3));
  }

  .welcome-title {
    font-size: 28px;
    font-weight: 800;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.5px;
  }

  .welcome-subtitle {
    font-size: 15px;
    color: var(--text-secondary);
    margin: 0;
  }

  .cta-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 28px;
    margin-top: 12px;
    font-size: 14px;
    font-weight: 600;
    font-family: var(--font-sans);
    color: #fff;
    background: var(--accent);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: box-shadow var(--transition-subtle, 150ms ease), transform var(--transition-subtle, 150ms ease);
    box-shadow: 0 2px 8px rgba(122, 162, 247, 0.25);
  }

  .cta-btn:hover {
    box-shadow: 0 4px 20px rgba(122, 162, 247, 0.45);
    transform: translateY(-2px);
  }

  .db-badges {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 6px;
    max-width: 420px;
    margin-top: 20px;
  }

  /* Dashboard — shown when connections exist */
  .dashboard {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 840px;
    gap: 28px;
    padding-top: 24px;
  }

  .dashboard-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .dashboard-brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .dashboard-title {
    font-size: 22px;
    font-weight: 800;
    color: var(--accent);
    letter-spacing: -0.5px;
  }

  .dashboard-subtitle {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    border-left: 1px solid var(--border-color);
    padding-left: 12px;
    margin-left: 4px;
  }

  .cards-area {
    width: 100%;
    padding: 0 32px;
  }

  .group-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-secondary);
    padding: 16px 0 8px;
    border-bottom: 1px solid var(--border-color);
    margin-bottom: 12px;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
    margin-bottom: 12px;
  }

  .conn-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px 18px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    cursor: pointer;
    text-align: left;
    transition: transform var(--transition-subtle, 150ms ease), box-shadow var(--transition-subtle, 150ms ease), border-color var(--transition-subtle, 150ms ease);
    position: relative;
    overflow: hidden;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 13px;
  }

  .conn-card:hover {
    transform: translateY(-3px);
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.35), 0 0 0 1px var(--card-accent, var(--accent));
    border-color: var(--card-accent, var(--accent));
  }

  .card-stripe {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
  }

  .card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
    opacity: 0.4;
    flex-shrink: 0;
  }

  .status-dot.connected {
    background: var(--success);
    opacity: 1;
    box-shadow: 0 0 6px rgba(166, 227, 161, 0.5);
  }

  .status-dot.connecting {
    background: var(--warning);
    opacity: 1;
  }

  .status-dot.error {
    background: var(--error);
    opacity: 1;
  }

  .card-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-host {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .add-card {
    border-style: dashed;
    border-width: 2px;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    background: transparent;
    gap: 8px;
    min-height: 100px;
    font-size: 13px;
    font-weight: 500;
  }

  .add-card:hover {
    color: var(--accent);
    border-color: var(--accent);
    background: rgba(122, 162, 247, 0.08);
    transform: translateY(-2px);
    box-shadow: 0 4px 16px rgba(122, 162, 247, 0.15);
  }

  .shortcuts-hint {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 32px;
    font-size: 11px;
    color: var(--text-secondary);
    opacity: 0.7;
  }

  .sep {
    width: 1px;
    height: 12px;
    background: var(--border-color);
  }
</style>
