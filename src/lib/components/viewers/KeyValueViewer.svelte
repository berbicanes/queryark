<script lang="ts">
  let { keyType, value }: {
    keyType: string;
    value: string;
  } = $props();

  let parsedValue = $derived.by(() => {
    try {
      return JSON.parse(value);
    } catch {
      return null;
    }
  });

  let isJson = $derived(parsedValue !== null && typeof parsedValue === 'object');
</script>

<div class="kv-viewer">
  <div class="kv-header">
    <span class="kv-type badge">{keyType}</span>
  </div>

  <div class="kv-content">
    {#if keyType === 'list' && isJson && Array.isArray(parsedValue)}
      <div class="kv-list">
        {#each parsedValue as item, idx}
          <div class="list-item">
            <span class="list-index">{idx}</span>
            <span class="list-value">{typeof item === 'object' ? JSON.stringify(item) : String(item)}</span>
          </div>
        {/each}
      </div>
    {:else if keyType === 'set' && isJson && Array.isArray(parsedValue)}
      <div class="kv-list">
        {#each parsedValue as item}
          <div class="list-item">
            <span class="list-value">{typeof item === 'object' ? JSON.stringify(item) : String(item)}</span>
          </div>
        {/each}
      </div>
    {:else if keyType === 'hash' && isJson && !Array.isArray(parsedValue)}
      <div class="kv-hash">
        {#each Object.entries(parsedValue as Record<string, unknown>) as [field, val]}
          <div class="hash-entry">
            <span class="hash-field">{field}</span>
            <span class="hash-value">{typeof val === 'object' ? JSON.stringify(val) : String(val)}</span>
          </div>
        {/each}
      </div>
    {:else if keyType === 'zset' && isJson && Array.isArray(parsedValue)}
      <div class="kv-list">
        {#each parsedValue as item}
          <div class="list-item">
            <span class="list-value">{typeof item === 'object' ? JSON.stringify(item) : String(item)}</span>
          </div>
        {/each}
      </div>
    {:else}
      <pre class="kv-string">{value}</pre>
    {/if}
  </div>
</div>

<style>
  .kv-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .kv-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .kv-type {
    font-size: 11px;
    text-transform: uppercase;
  }

  .kv-content {
    flex: 1;
    overflow: auto;
    padding: 8px 12px;
  }

  .kv-string {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
  }

  .kv-list, .kv-hash {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .list-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 2px 4px;
    font-size: 12px;
    font-family: var(--font-mono);
    border-radius: var(--radius-sm);
  }

  .list-item:hover {
    background: var(--bg-hover);
  }

  .list-index {
    color: var(--text-muted);
    min-width: 32px;
    text-align: right;
    flex-shrink: 0;
  }

  .list-value {
    color: var(--text-primary);
    word-break: break-all;
  }

  .hash-entry {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 2px 4px;
    font-size: 12px;
    font-family: var(--font-mono);
    border-radius: var(--radius-sm);
  }

  .hash-entry:hover {
    background: var(--bg-hover);
  }

  .hash-field {
    color: #89b4fa;
    min-width: 100px;
    flex-shrink: 0;
  }

  .hash-value {
    color: var(--text-primary);
    word-break: break-all;
  }
</style>
