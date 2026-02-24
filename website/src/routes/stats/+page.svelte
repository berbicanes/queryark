<script lang="ts">
  import { onMount } from 'svelte';

  const PASSWORD_HASH = '64dd6209d8e745f608247e8732f475f229a50ff2370640946bb8da1bf4d8553a';
  const SESSION_KEY = 'queryark-stats-auth';
  const API_URL = 'https://api.github.com/repos/berbicanes/queryark/releases';

  interface Asset {
    name: string;
    download_count: number;
    size: number;
    browser_download_url: string;
  }

  interface Release {
    tag_name: string;
    name: string;
    published_at: string;
    prerelease: boolean;
    assets: Asset[];
  }

  interface PlatformStats {
    macOS: number;
    Windows: number;
    Linux: number;
    Other: number;
  }

  let authenticated = $state(false);
  let password = $state('');
  let authError = $state('');
  let releases = $state<Release[]>([]);
  let loading = $state(false);
  let fetchError = $state('');
  let lastFetched = $state<Date | null>(null);
  let expandedReleases = $state<Set<string>>(new Set());

  let totalDownloads = $derived(
    releases.reduce((sum, r) => sum + r.assets.reduce((s, a) => s + a.download_count, 0), 0)
  );

  let latestVersion = $derived(releases.length > 0 ? releases[0].tag_name : '—');

  function computePlatformStats(rels: Release[]): PlatformStats {
    const stats: PlatformStats = { macOS: 0, Windows: 0, Linux: 0, Other: 0 };
    for (const release of rels) {
      for (const asset of release.assets) {
        stats[detectPlatform(asset.name)] += asset.download_count;
      }
    }
    return stats;
  }

  let platformStats = $derived(computePlatformStats(releases));

  let maxReleaseDownloads = $derived(
    Math.max(...releases.map(releaseDownloads), 1)
  );

  function detectPlatform(filename: string): keyof PlatformStats {
    const f = filename.toLowerCase();
    if (f.includes('.dmg') || f.includes('darwin') || f.includes('macos') || f.includes('.app.tar.gz')) return 'macOS';
    if (f.includes('.exe') || f.includes('.msi') || f.includes('.nsis') || f.includes('windows')) return 'Windows';
    if (f.includes('.deb') || f.includes('.rpm') || f.includes('.appimage') || f.includes('linux')) return 'Linux';
    return 'Other';
  }

  function releaseDownloads(release: Release): number {
    return release.assets.reduce((sum, a) => sum + a.download_count, 0);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i];
  }

  function formatNumber(n: number): string {
    return n.toLocaleString('en-US');
  }

  function platformColor(platform: string): string {
    switch (platform) {
      case 'macOS': return '#a6e3a1';
      case 'Windows': return '#7aa2f7';
      case 'Linux': return '#f9e2af';
      default: return '#6c7086';
    }
  }

  async function hashPassword(pw: string): Promise<string> {
    const data = new TextEncoder().encode(pw);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    return Array.from(new Uint8Array(hashBuffer)).map(b => b.toString(16).padStart(2, '0')).join('');
  }

  async function handleLogin(e: Event) {
    e.preventDefault();
    authError = '';
    const hash = await hashPassword(password);
    if (hash === PASSWORD_HASH) {
      authenticated = true;
      sessionStorage.setItem(SESSION_KEY, hash);
      await fetchReleases();
    } else {
      authError = 'Incorrect password';
    }
  }

  async function fetchReleases() {
    loading = true;
    fetchError = '';
    try {
      const res = await fetch(API_URL);
      if (!res.ok) throw new Error(`GitHub API returned ${res.status}`);
      releases = await res.json();
      lastFetched = new Date();
    } catch (err) {
      fetchError = err instanceof Error ? err.message : 'Failed to fetch releases';
    } finally {
      loading = false;
    }
  }

  function toggleRelease(tag: string) {
    const next = new Set(expandedReleases);
    if (next.has(tag)) next.delete(tag);
    else next.add(tag);
    expandedReleases = next;
  }

  onMount(() => {
    const stored = sessionStorage.getItem(SESSION_KEY);
    if (stored === PASSWORD_HASH) {
      authenticated = true;
      fetchReleases();
    }
  });
</script>

<svelte:head>
  <title>Stats — QueryArk</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

{#if !authenticated}
  <!-- Password Gate -->
  <section class="stats-page">
    <div class="container">
      <div class="auth-wrapper">
        <div class="auth-card">
          <div class="auth-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
          </div>
          <h1 class="auth-title">Download Stats</h1>
          <p class="auth-subtitle">Enter the password to view download analytics.</p>
          <form onsubmit={handleLogin} class="auth-form">
            <input
              type="password"
              bind:value={password}
              placeholder="Password"
              class="auth-input"
              autocomplete="off"
            />
            {#if authError}
              <p class="auth-error">{authError}</p>
            {/if}
            <button type="submit" class="btn btn-primary auth-btn">Unlock</button>
          </form>
        </div>
      </div>
    </div>
  </section>
{:else}
  <!-- Dashboard -->
  <section class="stats-page">
    <div class="container">
      <div class="page-header">
        <p class="section-label">Analytics</p>
        <h1 class="page-title">Download Stats</h1>
        <p class="page-subtitle">
          Real-time download data from GitHub Releases.
        </p>
      </div>

      <!-- Controls -->
      <div class="controls">
        <button class="btn btn-secondary btn-refresh" onclick={fetchReleases} disabled={loading}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
          {loading ? 'Refreshing...' : 'Refresh'}
        </button>
        {#if lastFetched}
          <span class="last-updated">
            Updated {lastFetched.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })}
          </span>
        {/if}
      </div>

      {#if fetchError}
        <div class="error-banner">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          {fetchError}
        </div>
      {/if}

      {#if loading && releases.length === 0}
        <div class="loading-state">
          <div class="spinner"></div>
          <p>Loading release data...</p>
        </div>
      {:else if releases.length > 0}
        <!-- Stats Cards -->
        <div class="stats-grid">
          <div class="stat-card">
            <span class="stat-label">Total Downloads</span>
            <span class="stat-value">{formatNumber(totalDownloads)}</span>
          </div>
          <div class="stat-card">
            <span class="stat-label">Latest Release</span>
            <span class="stat-value">{latestVersion}</span>
          </div>
          <div class="stat-card">
            <span class="stat-label">Releases</span>
            <span class="stat-value">{releases.length}</span>
          </div>
        </div>

        <!-- Platform Breakdown -->
        <div class="section-block">
          <h2 class="block-title">Platform Breakdown</h2>
          <div class="platform-grid">
            {#each Object.entries(platformStats).filter(([_, v]) => v > 0) as [platform, count]}
              <div class="platform-card">
                <div class="platform-bar-track">
                  <div
                    class="platform-bar-fill"
                    style="width: {totalDownloads > 0 ? (count / totalDownloads * 100) : 0}%; background: {platformColor(platform)}"
                  ></div>
                </div>
                <div class="platform-info">
                  <span class="platform-name" style="color: {platformColor(platform)}">{platform}</span>
                  <span class="platform-count">{formatNumber(count)}</span>
                  <span class="platform-pct">{totalDownloads > 0 ? (count / totalDownloads * 100).toFixed(1) : 0}%</span>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Downloads per Release Chart -->
        <div class="section-block">
          <h2 class="block-title">Downloads per Release</h2>
          <div class="chart">
            {#each releases as release}
              {@const count = releaseDownloads(release)}
              <div class="chart-row">
                <span class="chart-label">{release.tag_name}</span>
                <div class="chart-bar-track">
                  <div
                    class="chart-bar-fill"
                    style="width: {(count / maxReleaseDownloads) * 100}%"
                  ></div>
                </div>
                <span class="chart-value">{formatNumber(count)}</span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Release Details -->
        <div class="section-block">
          <h2 class="block-title">Release Details</h2>
          <div class="releases-list">
            {#each releases as release}
              {@const count = releaseDownloads(release)}
              {@const isExpanded = expandedReleases.has(release.tag_name)}
              <div class="release-item" class:expanded={isExpanded}>
                <button class="release-header" onclick={() => toggleRelease(release.tag_name)}>
                  <div class="release-meta">
                    <span class="release-tag">{release.tag_name}</span>
                    {#if release.prerelease}
                      <span class="prerelease-badge">pre-release</span>
                    {/if}
                    <span class="release-date">
                      {new Date(release.published_at).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })}
                    </span>
                  </div>
                  <div class="release-right">
                    <span class="release-count">{formatNumber(count)} downloads</span>
                    <svg
                      class="chevron"
                      class:rotated={isExpanded}
                      width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    >
                      <polyline points="6 9 12 15 18 9"/>
                    </svg>
                  </div>
                </button>
                {#if isExpanded}
                  <div class="release-assets">
                    <table class="assets-table">
                      <thead>
                        <tr>
                          <th>Asset</th>
                          <th>Platform</th>
                          <th>Size</th>
                          <th>Downloads</th>
                        </tr>
                      </thead>
                      <tbody>
                        {#each release.assets as asset}
                          <tr>
                            <td class="asset-name">{asset.name}</td>
                            <td>
                              <span class="platform-pill" style="color: {platformColor(detectPlatform(asset.name))}; background: {platformColor(detectPlatform(asset.name))}15">
                                {detectPlatform(asset.name)}
                              </span>
                            </td>
                            <td class="asset-size">{formatBytes(asset.size)}</td>
                            <td class="asset-downloads">{formatNumber(asset.download_count)}</td>
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </section>
{/if}

<style>
  .stats-page {
    padding: 140px 0 100px;
  }

  /* ── Auth ── */

  .auth-wrapper {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 60vh;
  }

  .auth-card {
    text-align: center;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    padding: 48px 40px;
    max-width: 400px;
    width: 100%;
    box-shadow: var(--shadow-lg);
  }

  .auth-icon {
    color: var(--accent);
    margin-bottom: 20px;
  }

  .auth-title {
    font-size: 24px;
    font-weight: 800;
    color: var(--text-primary);
    margin-bottom: 8px;
  }

  .auth-subtitle {
    font-size: 14px;
    color: var(--text-secondary);
    margin-bottom: 28px;
  }

  .auth-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .auth-input {
    width: 100%;
    padding: 12px 16px;
    font-size: 15px;
    font-family: var(--font-sans);
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    outline: none;
    transition: border-color 200ms ease;
  }

  .auth-input:focus {
    border-color: var(--accent);
  }

  .auth-input::placeholder {
    color: var(--text-muted);
  }

  .auth-error {
    font-size: 13px;
    color: var(--error);
  }

  .auth-btn {
    width: 100%;
    justify-content: center;
  }

  /* ── Page Header ── */

  .page-header {
    text-align: center;
    margin-bottom: 48px;
  }

  .page-title {
    font-size: clamp(32px, 5vw, 48px);
    font-weight: 900;
    letter-spacing: -1px;
    color: var(--text-primary);
    margin-bottom: 16px;
  }

  .page-subtitle {
    font-size: 18px;
    color: var(--text-secondary);
  }

  /* ── Controls ── */

  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 32px;
  }

  .btn-refresh {
    padding: 10px 20px;
    font-size: 14px;
  }

  .btn-refresh:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  .last-updated {
    font-size: 13px;
    color: var(--text-muted);
  }

  /* ── Error Banner ── */

  .error-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 20px;
    font-size: 14px;
    color: var(--error);
    background: rgba(243, 139, 168, 0.08);
    border: 1px solid rgba(243, 139, 168, 0.2);
    border-radius: var(--radius-lg);
    margin-bottom: 32px;
  }

  /* ── Loading ── */

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 80px 0;
    color: var(--text-muted);
    font-size: 14px;
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* ── Stats Cards ── */

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
    margin-bottom: 48px;
  }

  .stat-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .stat-label {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-muted);
  }

  .stat-value {
    font-size: 32px;
    font-weight: 800;
    font-family: var(--font-mono);
    color: var(--text-primary);
    letter-spacing: -1px;
  }

  /* ── Section Block ── */

  .section-block {
    margin-bottom: 48px;
  }

  .block-title {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 20px;
  }

  /* ── Platform Breakdown ── */

  .platform-grid {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    padding: 24px;
  }

  .platform-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .platform-bar-track {
    height: 8px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    overflow: hidden;
  }

  .platform-bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 400ms ease;
  }

  .platform-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .platform-name {
    font-size: 14px;
    font-weight: 600;
    min-width: 80px;
  }

  .platform-count {
    font-size: 14px;
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .platform-pct {
    font-size: 13px;
    color: var(--text-muted);
  }

  /* ── Chart ── */

  .chart {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    padding: 24px;
  }

  .chart-row {
    display: grid;
    grid-template-columns: 80px 1fr 70px;
    align-items: center;
    gap: 16px;
  }

  .chart-label {
    font-size: 13px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    text-align: right;
  }

  .chart-bar-track {
    height: 24px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .chart-bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: var(--radius-sm);
    transition: width 400ms ease;
    min-width: 2px;
  }

  .chart-value {
    font-size: 13px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    text-align: right;
  }

  /* ── Release Details ── */

  .releases-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .release-item {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: border-color 200ms ease;
  }

  .release-item:hover {
    border-color: var(--bg-hover);
  }

  .release-item.expanded {
    border-color: var(--accent);
  }

  .release-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 16px 20px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 14px;
    text-align: left;
  }

  .release-meta {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .release-tag {
    font-weight: 700;
    font-family: var(--font-mono);
    font-size: 15px;
    color: var(--text-primary);
  }

  .prerelease-badge {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 2px 8px;
    border-radius: 10px;
    color: var(--warning);
    background: rgba(249, 226, 175, 0.1);
  }

  .release-date {
    font-size: 13px;
    color: var(--text-muted);
  }

  .release-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .release-count {
    font-size: 13px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .chevron {
    transition: transform 200ms ease;
    color: var(--text-muted);
  }

  .chevron.rotated {
    transform: rotate(180deg);
  }

  /* ── Assets Table ── */

  .release-assets {
    border-top: 1px solid var(--border-color);
    padding: 16px 20px;
    overflow-x: auto;
  }

  .assets-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .assets-table th {
    text-align: left;
    padding: 8px 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-color);
  }

  .assets-table td {
    padding: 10px 12px;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .assets-table tr:last-child td {
    border-bottom: none;
  }

  .asset-name {
    font-family: var(--font-mono);
    font-size: 12px;
    word-break: break-all;
  }

  .platform-pill {
    display: inline-block;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 10px;
    border-radius: 10px;
  }

  .asset-size {
    font-family: var(--font-mono);
    font-size: 12px;
    white-space: nowrap;
  }

  .asset-downloads {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-primary);
  }

  /* ── Responsive ── */

  @media (max-width: 768px) {
    .stats-page {
      padding: 120px 0 80px;
    }

    .stats-grid {
      grid-template-columns: 1fr;
    }

    .stat-value {
      font-size: 28px;
    }

    .chart-row {
      grid-template-columns: 60px 1fr 56px;
      gap: 8px;
    }

    .chart-label {
      font-size: 11px;
    }

    .chart-bar-track {
      height: 20px;
    }

    .release-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .release-right {
      width: 100%;
      justify-content: space-between;
    }

    .auth-card {
      padding: 36px 24px;
    }

    .controls {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
