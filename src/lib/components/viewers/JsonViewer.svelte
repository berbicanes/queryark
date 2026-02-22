<script lang="ts">
  import JsonViewer from './JsonViewer.svelte';

  let { data, depth = 0, expanded = true }: {
    data: unknown;
    depth?: number;
    expanded?: boolean;
  } = $props();

  let initialExpanded = depth < 2;
  let isExpanded = $state(initialExpanded);

  function toggle() {
    isExpanded = !isExpanded;
  }

  function isObject(val: unknown): val is Record<string, unknown> {
    return val !== null && typeof val === 'object' && !Array.isArray(val);
  }

  function isArray(val: unknown): val is unknown[] {
    return Array.isArray(val);
  }

  function getEntries(val: unknown): [string, unknown][] {
    if (isObject(val)) return Object.entries(val);
    if (isArray(val)) return val.map((v, i) => [String(i), v]);
    return [];
  }

  function formatValue(val: unknown): string {
    if (val === null) return 'null';
    if (val === undefined) return 'undefined';
    if (typeof val === 'string') return `"${val}"`;
    if (typeof val === 'boolean') return val ? 'true' : 'false';
    return String(val);
  }

  function getPreview(val: unknown): string {
    if (isObject(val)) {
      const keys = Object.keys(val);
      if (keys.length === 0) return '{}';
      return `{ ${keys.length} keys }`;
    }
    if (isArray(val)) {
      return `[ ${val.length} items ]`;
    }
    return formatValue(val);
  }

  function isPrimitive(val: unknown): boolean {
    return !isObject(val) && !isArray(val);
  }

  function getValueClass(val: unknown): string {
    if (val === null || val === undefined) return 'json-null';
    if (typeof val === 'string') return 'json-string';
    if (typeof val === 'number') return 'json-number';
    if (typeof val === 'boolean') return 'json-boolean';
    return '';
  }
</script>

<div class="json-viewer" style="padding-left: {depth * 16}px">
  {#if isPrimitive(data)}
    <span class={getValueClass(data)}>{formatValue(data)}</span>
  {:else}
    <button class="toggle-btn" onclick={toggle}>
      <span class="arrow" class:expanded={isExpanded}>{isExpanded ? '\u25BC' : '\u25B6'}</span>
      {#if !isExpanded}
        <span class="preview">{getPreview(data)}</span>
      {:else}
        <span class="bracket">{isArray(data) ? '[' : '{'}</span>
      {/if}
    </button>

    {#if isExpanded}
      {#each getEntries(data) as [key, value]}
        <div class="json-entry">
          <span class="json-key" style="padding-left: {(depth + 1) * 16}px">
            {isArray(data) ? key : `"${key}"`}:
          </span>
          {#if isPrimitive(value)}
            <span class={getValueClass(value)}>{formatValue(value)}</span>
          {:else}
            <JsonViewer data={value} depth={depth + 1} expanded={depth < 1} />
          {/if}
        </div>
      {/each}
      <span class="bracket" style="padding-left: {depth * 16}px">{isArray(data) ? ']' : '}'}</span>
    {/if}
  {/if}
</div>

<style>
  .json-viewer {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
  }

  .toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 0;
  }

  .toggle-btn:hover {
    color: var(--accent);
  }

  .arrow {
    font-size: 8px;
    width: 12px;
    text-align: center;
    color: var(--text-muted);
  }

  .preview {
    color: var(--text-muted);
    font-style: italic;
  }

  .bracket {
    color: var(--text-muted);
  }

  .json-entry {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .json-key {
    color: #89b4fa;
    flex-shrink: 0;
  }

  .json-string {
    color: #a6e3a1;
  }

  .json-number {
    color: #fab387;
  }

  .json-boolean {
    color: #f38ba8;
  }

  .json-null {
    color: var(--text-muted);
    font-style: italic;
  }
</style>
