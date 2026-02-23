<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { detectParameters, substituteParameters } from '$lib/utils/parameterParser';

  let params = $derived(detectParameters(uiStore.parameterPromptSql));
  let values = $state<Record<string, string>>({});

  // Session-level memory for last-used parameter values
  const memory: Record<string, string> = {};

  $effect(() => {
    // Initialize values from memory when params change
    const newValues: Record<string, string> = {};
    for (const p of params) {
      newValues[p.name] = memory[p.name] ?? p.defaultValue;
    }
    values = newValues;
  });

  function handleRun() {
    // Save values to memory
    for (const [k, v] of Object.entries(values)) {
      memory[k] = v;
    }

    const substituted = substituteParameters(uiStore.parameterPromptSql, values);
    uiStore.parameterPromptCallback?.(substituted);
    close();
  }

  function close() {
    uiStore.showParameterPrompt = false;
    uiStore.parameterPromptSql = '';
    uiStore.parameterPromptCallback = null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleRun();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={handleOverlayClick} onkeydown={handleKeydown}>
  <div class="modal">
    <div class="modal-header">
      <h3>Query Parameters</h3>
      <button class="close-btn" onclick={close}>&times;</button>
    </div>

    <div class="modal-body">
      <div class="sql-preview">{uiStore.parameterPromptSql.substring(0, 200)}{uiStore.parameterPromptSql.length > 200 ? '...' : ''}</div>

      {#if params.length === 0}
        <p class="no-params">No parameters detected.</p>
      {:else}
        <div class="params-list">
          {#each params as param}
            <div class="param-row">
              <label class="param-label" for="param-{param.name}">
                <span class="param-name">{param.name}</span>
                <span class="param-style">{param.style}</span>
              </label>
              <input
                id="param-{param.name}"
                class="param-input"
                type="text"
                placeholder="Value..."
                bind:value={values[param.name]}
              />
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <span class="hint">Ctrl+Enter to run</span>
      <div class="footer-buttons">
        <button class="btn btn-secondary" onclick={close}>Cancel</button>
        <button class="btn btn-primary" onclick={handleRun}>Run</button>
      </div>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    backdrop-filter: blur(2px);
  }

  .modal {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 440px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
  }

  .sql-preview {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-primary);
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    margin-bottom: 16px;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 80px;
    overflow: hidden;
  }

  .no-params {
    color: var(--text-muted);
    font-size: 13px;
  }

  .params-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .param-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .param-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .param-name {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--accent);
  }

  .param-style {
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-primary);
    padding: 1px 5px;
    border-radius: var(--radius-sm);
  }

  .param-input {
    padding: 6px 10px;
    font-size: 13px;
    font-family: var(--font-mono);
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    outline: none;
  }

  .param-input:focus {
    border-color: var(--accent);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-top: 1px solid var(--border-color);
  }

  .hint {
    font-size: 11px;
    color: var(--text-muted);
  }

  .footer-buttons {
    display: flex;
    gap: 8px;
  }

  .btn {
    padding: 6px 14px;
    font-size: 12px;
    font-family: var(--font-sans);
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: 1px solid var(--border-color);
  }

  .btn-secondary {
    background: none;
    color: var(--text-secondary);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
    font-weight: 600;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }
</style>
