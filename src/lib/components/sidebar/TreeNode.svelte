<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    icon = '',
    expandable = false,
    expanded = $bindable(false),
    depth = 0,
    tooltip = '',
    suffix = '',
    onclick,
    ondblclick,
    oncontextmenu,
    onexpand,
    children
  }: {
    label: string;
    icon?: string;
    expandable?: boolean;
    expanded?: boolean;
    depth?: number;
    tooltip?: string;
    suffix?: string;
    onclick?: () => void;
    ondblclick?: () => void;
    oncontextmenu?: (e: MouseEvent) => void;
    onexpand?: (expanded: boolean) => void;
    children?: Snippet;
  } = $props();

  let loading = $state(false);
  let clickTimer: ReturnType<typeof setTimeout> | null = null;

  function handleClick() {
    // If a dblclick handler exists, delay expand to avoid double-toggle on double-click
    if (ondblclick) {
      if (clickTimer) {
        clearTimeout(clickTimer);
        clickTimer = null;
        return; // second click of double-click, skip
      }
      clickTimer = setTimeout(() => {
        clickTimer = null;
        performExpand();
      }, 200);
    } else {
      performExpand();
    }
  }

  async function performExpand() {
    if (expandable) {
      expanded = !expanded;
      if (onexpand) {
        loading = true;
        try {
          await onexpand(expanded);
        } finally {
          loading = false;
        }
      }
    }
    onclick?.();
  }

  function handleDblClick() {
    if (clickTimer) {
      clearTimeout(clickTimer);
      clickTimer = null;
    }
    ondblclick?.();
  }
</script>

<div class="tree-node">
  <button
    class="node-row"
    style="padding-left: {8 + depth * 16}px"
    title={tooltip || undefined}
    onclick={handleClick}
    ondblclick={handleDblClick}
    oncontextmenu={oncontextmenu}
  >
    {#if expandable}
      <svg
        width="12" height="12" viewBox="0 0 16 16" fill="none"
        class="chevron"
        class:expanded
        class:loading
      >
        <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    {:else}
      <span class="chevron-spacer"></span>
    {/if}

    {#if icon}
      <span class="node-icon">{icon}</span>
    {/if}

    <span class="node-label truncate">{label}</span>

    {#if suffix}
      <span class="node-suffix">{suffix}</span>
    {/if}

    {#if loading}
      <span class="node-spinner"></span>
    {/if}
  </button>

  {#if expanded && children}
    <div class="node-children">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .tree-node {
    width: 100%;
  }

  .node-row {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 2px 8px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    border: none;
    background: none;
    transition: background var(--transition-micro);
    min-height: 26px;
  }

  .node-row:hover {
    background: var(--bg-hover);
  }

  .chevron {
    flex-shrink: 0;
    width: 10px;
    height: 10px;
    transition: transform var(--transition-micro);
    color: var(--text-muted);
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .chevron.loading {
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .chevron-spacer {
    display: inline-block;
    width: 12px;
    flex-shrink: 0;
  }

  .node-icon {
    flex-shrink: 0;
    font-size: 13px;
    width: 16px;
    text-align: center;
  }

  .node-label {
    flex: 1;
    min-width: 0;
  }

  .node-suffix {
    flex-shrink: 0;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    margin-left: 4px;
    opacity: 0.7;
  }

  .node-spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .node-children {
    width: 100%;
    position: relative;
  }

  .node-children::before {
    content: '';
    position: absolute;
    left: 20px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--border-color);
    opacity: 0.4;
  }
</style>
