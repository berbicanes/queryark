<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { CHANGELOG, type ChangelogEntry } from '$lib/types/changelog';

  let appVersion = $state('0.2.0');

  // Show entries newer than lastSeenVersion
  const newEntries = $derived(() => {
    const lastSeen = settingsStore.lastSeenVersion;
    if (!lastSeen) return CHANGELOG;
    return CHANGELOG.filter(entry => compareVersions(entry.version, lastSeen) > 0);
  });

  function compareVersions(a: string, b: string): number {
    const pa = a.split('.').map(Number);
    const pb = b.split('.').map(Number);
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
      const na = pa[i] ?? 0;
      const nb = pb[i] ?? 0;
      if (na > nb) return 1;
      if (na < nb) return -1;
    }
    return 0;
  }

  function categoryIcon(category: ChangelogEntry['category']): string {
    switch (category) {
      case 'feature': return '+';
      case 'fix': return '~';
      case 'improvement': return '^';
    }
  }

  function categoryLabel(category: ChangelogEntry['category']): string {
    switch (category) {
      case 'feature': return 'New Features';
      case 'fix': return 'Bug Fixes';
      case 'improvement': return 'Improvements';
    }
  }

  function handleClose() {
    uiStore.showWhatsNewModal = false;
    settingsStore.setLastSeenVersion(appVersion);
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  import { onMount } from 'svelte';

  onMount(async () => {
    try {
      const { getVersion } = await import('@tauri-apps/api/app');
      appVersion = await getVersion();
    } catch {
      appVersion = '0.2.0';
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card whatsnew-modal">
    <div class="modal-header">
      <h2>What's New</h2>
      <button class="close-btn" onclick={handleClose} title="Close" aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#each newEntries() as entry}
        <div class="version-block">
          <div class="version-header">
            <span class="version-badge">v{entry.version}</span>
            <span class="version-date">{entry.date}</span>
            <span class="category-badge category-{entry.category}">
              {categoryIcon(entry.category)} {categoryLabel(entry.category)}
            </span>
          </div>
          <ul class="highlights">
            {#each entry.highlights as highlight}
              <li>{highlight}</li>
            {/each}
          </ul>
        </div>
      {/each}

      {#if newEntries().length === 0}
        <p class="no-updates">You're up to date!</p>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn-primary" onclick={handleClose}>Got it</button>
    </div>
  </div>
</div>

<style>
  .whatsnew-modal {
    width: 500px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-body {
    overflow-y: auto;
    flex: 1;
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

  .version-block {
    padding: 12px 0;
  }

  .version-block + .version-block {
    border-top: 1px solid var(--border-color);
  }

  .version-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .version-badge {
    font-size: 13px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--accent);
    padding: 2px 8px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
  }

  .version-date {
    font-size: 11px;
    color: var(--text-muted);
  }

  .category-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    margin-left: auto;
  }

  .category-feature {
    color: #22c55e;
    background: rgba(34, 197, 94, 0.12);
  }

  .category-fix {
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.12);
  }

  .category-improvement {
    color: #3b82f6;
    background: rgba(59, 130, 246, 0.12);
  }

  .highlights {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .highlights li {
    font-size: 12px;
    color: var(--text-secondary);
    padding-left: 16px;
    position: relative;
    line-height: 1.5;
  }

  .highlights li::before {
    content: '';
    position: absolute;
    left: 4px;
    top: 7px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .no-updates {
    text-align: center;
    font-size: 13px;
    color: var(--text-muted);
    padding: 24px 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 12px 16px;
    border-top: 1px solid var(--border-color);
  }

  .btn-primary {
    padding: 6px 20px;
    font-size: 12px;
    font-family: var(--font-sans);
    font-weight: 500;
    color: #fff;
    background: var(--accent);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .btn-primary:hover {
    opacity: 0.9;
  }
</style>
