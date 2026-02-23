<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  function handleClose() {
    uiStore.showSettingsModal = false;
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card settings-modal">
    <div class="modal-header">
      <h2>Settings</h2>
      <button class="close-btn" onclick={handleClose} title="Close" aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="settings-section">
        <h3 class="section-title">Appearance</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Theme</span>
          </div>
          <div class="setting-control">
            <select
              value={settingsStore.theme}
              onchange={(e) => settingsStore.setTheme(e.currentTarget.value as 'dark' | 'light')}
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Editor Font Size</span>
            <span class="label-hint">Code editor text size (10-24px)</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              min="10"
              max="24"
              value={settingsStore.editorFontSize}
              onchange={(e) => settingsStore.setEditorFontSize(Number(e.currentTarget.value))}
            />
            <span class="unit">px</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Grid Font Size</span>
            <span class="label-hint">Data grid text size (10-24px)</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              min="10"
              max="24"
              value={settingsStore.gridFontSize}
              onchange={(e) => settingsStore.setGridFontSize(Number(e.currentTarget.value))}
            />
            <span class="unit">px</span>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">Data</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Default Page Size</span>
            <span class="label-hint">Rows per page in data grid (10-10000)</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              min="10"
              max="10000"
              step="10"
              value={settingsStore.defaultPageSize}
              onchange={(e) => settingsStore.setDefaultPageSize(Number(e.currentTarget.value))}
            />
            <span class="unit">rows</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Max Query Result Rows</span>
            <span class="label-hint">Maximum rows returned per query statement (100-100000)</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              min="100"
              max="100000"
              step="100"
              value={settingsStore.maxQueryRows}
              onchange={(e) => settingsStore.setMaxQueryRows(Number(e.currentTarget.value))}
            />
            <span class="unit">rows</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Max Cell Preview Size</span>
            <span class="label-hint">Truncate large text/JSON cells beyond this length (64-10000 chars)</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              min="64"
              max="10000"
              step="64"
              value={settingsStore.maxCellSize}
              onchange={(e) => settingsStore.setMaxCellSize(Number(e.currentTarget.value))}
            />
            <span class="unit">chars</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Confirm Before Delete</span>
            <span class="label-hint">Show confirmation dialog for destructive actions</span>
          </div>
          <div class="setting-control">
            <label class="toggle">
              <input
                type="checkbox"
                checked={settingsStore.confirmBeforeDelete}
                onchange={(e) => settingsStore.setConfirmBeforeDelete(e.currentTarget.checked)}
              />
              <span class="slider"></span>
            </label>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">Session</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">Restore Tabs on Startup</span>
            <span class="label-hint">Reopen last active tabs and reconnect on launch</span>
          </div>
          <div class="setting-control">
            <label class="toggle">
              <input
                type="checkbox"
                checked={settingsStore.restoreSession}
                onchange={(e) => settingsStore.setRestoreSession(e.currentTarget.checked)}
              />
              <span class="slider"></span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .settings-modal {
    width: 480px;
    max-height: 80vh;
    overflow-y: auto;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    border: none;
    background: none;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
    padding: 0;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-section {
    padding: 0 0 12px;
  }

  .settings-section + .settings-section {
    border-top: 1px solid var(--border-color);
    padding-top: 12px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    gap: 16px;
  }

  .setting-label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .label-text {
    font-size: 13px;
    color: var(--text-primary);
  }

  .label-hint {
    font-size: 11px;
    color: var(--text-muted);
  }

  .setting-control {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .setting-control input[type="number"] {
    width: 72px;
    padding: 4px 8px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
    text-align: right;
  }

  .setting-control input[type="number"]:focus {
    border-color: var(--accent);
  }

  .setting-control select {
    padding: 4px 8px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    outline: none;
  }

  .setting-control select:focus {
    border-color: var(--accent);
  }

  .unit {
    font-size: 11px;
    color: var(--text-muted);
    min-width: 28px;
  }
</style>
