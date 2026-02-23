<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { formatDuration, formatRowCount } from '$lib/utils/formatters';
  import { updaterState, installUpdate } from '$lib/services/updaterService';

  let { executionTime = null, rowCount = null }: {
    executionTime: number | null;
    rowCount: number | null;
  } = $props();

  let appVersion = $state('');

  let activeConnection = $derived(connectionStore.activeConnection);
  let statusClass = $derived(activeConnection?.status ?? 'disconnected');
  let connectionLabel = $derived(
    activeConnection
      ? `${activeConnection.config.name} (${activeConnection.config.host}:${activeConnection.config.port})`
      : 'No connection'
  );

  onMount(async () => {
    try {
      const { getVersion } = await import('@tauri-apps/api/app');
      appVersion = await getVersion();
    } catch {
      appVersion = '0.2.0';
    }
  });
</script>

<div class="statusbar">
  <div class="statusbar-left">
    <span class="status-dot {statusClass}"></span>
    <span class="conn-label truncate">{connectionLabel}</span>
    {#if uiStore.isLoading}
      <span class="loading-indicator">
        <span class="spinner"></span>
        {uiStore.loadingMessage || 'Loading...'}
      </span>
    {/if}
  </div>

  <div class="statusbar-center">
    {#if executionTime !== null}
      <span class="execution-time">
        Execution: {formatDuration(executionTime)}
      </span>
    {/if}
  </div>

  <div class="statusbar-right">
    {#if rowCount !== null}
      <span class="row-count">
        {formatRowCount(rowCount)} {rowCount === 1 ? 'row' : 'rows'}
      </span>
    {/if}
    {#if updaterState.updateAvailable}
      <button class="update-badge" onclick={installUpdate} disabled={updaterState.updateProgress === 'downloading' || updaterState.updateProgress === 'installing'}>
        {#if updaterState.updateProgress === 'downloading'}
          Downloading...
        {:else if updaterState.updateProgress === 'installing'}
          Installing...
        {:else}
          Update v{updaterState.updateVersion} available
        {/if}
      </button>
    {/if}
    {#if appVersion}
      <span class="version-label">v{appVersion}</span>
    {/if}
  </div>
</div>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 11px;
    color: var(--text-secondary);
    user-select: none;
  }

  .statusbar-left,
  .statusbar-center,
  .statusbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .statusbar-left {
    flex: 1;
    min-width: 0;
  }

  .statusbar-center {
    flex-shrink: 0;
  }

  .statusbar-right {
    flex: 1;
    justify-content: flex-end;
    min-width: 0;
  }

  .conn-label {
    max-width: 300px;
  }

  .loading-indicator {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--accent);
    margin-left: 8px;
  }

  .spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .execution-time {
    color: var(--text-muted);
  }

  .row-count {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .update-badge {
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: 3px;
    padding: 1px 6px;
    font-size: 10px;
    font-family: var(--font-sans);
    cursor: pointer;
    white-space: nowrap;
  }

  .update-badge:hover {
    opacity: 0.85;
  }

  .update-badge:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .version-label {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
    opacity: 0.6;
    margin-left: 4px;
  }
</style>
