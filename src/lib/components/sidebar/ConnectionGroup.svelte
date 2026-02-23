<script lang="ts">
  import type { Snippet } from 'svelte';

  let { name, count, expanded, ontoggle, children }: {
    name: string;
    count: number;
    expanded: boolean;
    ontoggle: () => void;
    children: Snippet;
  } = $props();
</script>

<div class="connection-group">
  <button class="group-header" onclick={ontoggle}>
    <svg
      width="10" height="10" viewBox="0 0 16 16" fill="none"
      class="chevron" class:expanded
    >
      <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" class="folder-icon">
      {#if expanded}
        <path d="M2 4h5l1 1h6v8H2V4z" stroke="currentColor" stroke-width="1.2" fill="none"/>
      {:else}
        <path d="M2 3h5l1 1h6v9H2V3z" fill="var(--text-muted)" opacity="0.3" stroke="currentColor" stroke-width="1.2"/>
      {/if}
    </svg>
    <span class="group-name truncate">{name}</span>
    <span class="group-count">{count}</span>
  </button>

  {#if expanded}
    <div class="group-children">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .connection-group {
    border-bottom: 1px solid var(--border-color);
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    text-align: left;
    transition: background var(--transition-fast);
    cursor: pointer;
    border: none;
    background: none;
  }

  .group-header:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .chevron {
    flex-shrink: 0;
    transition: transform var(--transition-fast);
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .folder-icon {
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .group-name {
    flex: 1;
    min-width: 0;
  }

  .group-count {
    font-size: 9px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 0 6px;
    border-radius: 9999px;
    flex-shrink: 0;
    line-height: 1.6;
    font-weight: 600;
  }

  .group-children {
    padding-left: 12px;
  }
</style>
