<script lang="ts">
  let {
    onzoomin,
    onzoomout,
    onfit,
    onreset,
    onexport,
    schemas,
    enabledSchemas,
    ontoggleschema,
  }: {
    onzoomin: () => void;
    onzoomout: () => void;
    onfit: () => void;
    onreset: () => void;
    onexport: () => void;
    schemas: string[];
    enabledSchemas: Set<string>;
    ontoggleschema: (schema: string) => void;
  } = $props();

  let filterOpen = $state(false);
</script>

<div class="diagram-toolbar">
  <div class="toolbar-group">
    <button class="toolbar-btn" onclick={onzoomin} title="Zoom In">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
        <path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="M5 7h4M7 5v4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>
    <button class="toolbar-btn" onclick={onzoomout} title="Zoom Out">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
        <path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="M5 7h4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>
    <button class="toolbar-btn" onclick={onfit} title="Fit to Screen">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M2 5V2h3M11 2h3v3M14 11v3h-3M5 14H2v-3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <button class="toolbar-btn" onclick={onreset} title="Reset Layout">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M2 8a6 6 0 1110.5-4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
        <path d="M12 1v3.5h-3.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
  </div>

  <div class="toolbar-group">
    <button class="toolbar-btn" onclick={onexport} title="Export SVG">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M8 2v8M5 7l3 3 3-3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M2 11v2a1 1 0 001 1h10a1 1 0 001-1v-2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
      </svg>
      <span>SVG</span>
    </button>
  </div>

  {#if schemas.length > 1}
    <div class="toolbar-group filter-group">
      <button class="toolbar-btn" onclick={() => filterOpen = !filterOpen} title="Filter Schemas">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M2 3h12L9 8.5V12l-2 1V8.5L2 3z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/>
        </svg>
        <span>Schemas</span>
      </button>
      {#if filterOpen}
        <div class="filter-dropdown">
          {#each schemas as schema}
            <label class="filter-option">
              <input
                type="checkbox"
                checked={enabledSchemas.has(schema)}
                onchange={() => ontoggleschema(schema)}
              />
              <span>{schema}</span>
            </label>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .diagram-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--text-secondary);
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .toolbar-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-color);
  }

  .filter-group {
    position: relative;
    margin-left: auto;
  }

  .filter-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 50;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    padding: 4px 0;
    min-width: 150px;
    max-height: 200px;
    overflow-y: auto;
  }

  .filter-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .filter-option:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .filter-option input[type="checkbox"] {
    width: 13px;
    height: 13px;
    margin: 0;
    accent-color: var(--accent);
  }
</style>
