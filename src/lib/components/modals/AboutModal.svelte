<script lang="ts">
  import { onMount } from 'svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let appVersion = $state('0.0.0');

  onMount(async () => {
    try {
      const { getVersion } = await import('@tauri-apps/api/app');
      appVersion = await getVersion();
    } catch {
      // Fallback if not running in Tauri
      appVersion = '0.2.0';
    }
  });

  function handleClose() {
    uiStore.showAboutModal = false;
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  function openChangelog() {
    handleClose();
    uiStore.showWhatsNewModal = true;
  }

  async function openLink(url: string) {
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(url);
    } catch {
      window.open(url, '_blank');
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card about-modal">
    <div class="modal-header">
      <h2>About QueryArk</h2>
      <button class="close-btn" onclick={handleClose} title="Close" aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="about-hero">
        <div class="about-logo">
          <svg width="40" height="40" viewBox="0 0 48 48" fill="none">
            <rect x="4" y="8" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5"/>
            <rect x="4" y="20" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5"/>
            <rect x="4" y="32" width="40" height="8" rx="4" stroke="var(--accent)" stroke-width="2.5"/>
            <circle cx="10" cy="12" r="1.5" fill="var(--accent)"/>
            <circle cx="10" cy="24" r="1.5" fill="var(--accent)"/>
            <circle cx="10" cy="36" r="1.5" fill="var(--accent)"/>
          </svg>
        </div>
        <div class="about-title">QueryArk</div>
        <div class="about-version">v{appVersion}</div>
      </div>

      <p class="about-description">
        A fast, lightweight database IDE supporting 17 database engines.
        Built with Tauri, SvelteKit, and Rust.
      </p>

      <div class="about-section">
        <div class="about-row">
          <span class="about-label">License</span>
          <span class="about-value">MIT</span>
        </div>
        <div class="about-row">
          <span class="about-label">Runtime</span>
          <span class="about-value">Tauri 2 + SvelteKit 5</span>
        </div>
        <div class="about-row">
          <span class="about-label">Backend</span>
          <span class="about-value">Rust (sqlx, tiberius, mongodb, redis)</span>
        </div>
        <div class="about-row">
          <span class="about-label">Editor</span>
          <span class="about-value">CodeMirror 6</span>
        </div>
      </div>

      <div class="about-links">
        <button class="link-btn" onclick={() => openLink('https://github.com/berbicanes/queryark')}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 00-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0020 4.77 5.07 5.07 0 0019.91 1S18.73.65 16 2.48a13.38 13.38 0 00-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 005 4.77a5.44 5.44 0 00-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 009 18.13V22"/>
          </svg>
          GitHub
        </button>
        <button class="link-btn" onclick={openChangelog}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
            <line x1="16" y1="13" x2="8" y2="13"/>
            <line x1="16" y1="17" x2="8" y2="17"/>
            <polyline points="10 9 9 9 8 9"/>
          </svg>
          View Changelog
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .about-modal {
    width: 400px;
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

  .about-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
  }

  .about-logo {
    opacity: 0.9;
  }

  .about-title {
    font-size: 20px;
    font-weight: 700;
    color: var(--accent);
  }

  .about-version {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    padding: 2px 10px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
  }

  .about-description {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.6;
    text-align: center;
    margin: 0 0 16px;
  }

  .about-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 0;
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
  }

  .about-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }

  .about-label {
    color: var(--text-muted);
  }

  .about-value {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .about-links {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-top: 16px;
  }

  .link-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    font-size: 12px;
    font-family: var(--font-sans);
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast), border-color var(--transition-fast);
  }

  .link-btn:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
  }
</style>
