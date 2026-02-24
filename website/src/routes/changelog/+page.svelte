<script lang="ts">
  let { data } = $props();
  let changelog = $derived(data.changelog);
</script>

<svelte:head>
  <title>Changelog — QueryArk</title>
  <meta name="description" content="QueryArk version history and release notes." />
  <link rel="canonical" href="https://queryark.com/changelog" />
  <meta property="og:title" content="Changelog — QueryArk" />
  <meta property="og:description" content="QueryArk version history and release notes." />
  <meta property="og:url" content="https://queryark.com/changelog" />
  <meta name="twitter:title" content="Changelog — QueryArk" />
  <meta name="twitter:description" content="QueryArk version history and release notes." />
</svelte:head>

<section class="changelog-page">
  <div class="container">
    <div class="page-header">
      <p class="section-label">Changelog</p>
      <h1 class="page-title">What's New</h1>
      <p class="page-subtitle">
        Version history and release highlights for QueryArk.
      </p>
    </div>

    <div class="timeline">
      {#each changelog as entry}
        <div class="timeline-entry">
          <div class="timeline-marker">
            <span class="version-badge">v{entry.version}</span>
            <span class="date">{new Date(entry.date).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}</span>
          </div>
          <div class="timeline-content">
            <div class="entry-header">
              <h2 class="entry-version">v{entry.version}</h2>
              <span class="category-tag {entry.category}">{entry.category}</span>
            </div>
            <ul class="highlights">
              {#each entry.highlights as highlight}
                <li>{highlight}</li>
              {/each}
            </ul>
          </div>
        </div>
      {/each}
    </div>

  </div>
</section>

<style>
  .changelog-page {
    padding: 140px 0 100px;
    background: rgba(15, 17, 23, 0.92);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .page-header {
    text-align: center;
    margin-bottom: 72px;
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

  .timeline {
    max-width: 720px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 48px;
    position: relative;
  }

  .timeline::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--border-color);
  }

  .timeline-entry {
    padding-left: 36px;
    position: relative;
  }

  .timeline-entry::before {
    content: '';
    position: absolute;
    left: -5px;
    top: 4px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg-primary);
  }

  .timeline-marker {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .version-badge {
    display: inline-block;
    padding: 3px 12px;
    font-size: 13px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--accent);
    background: rgba(74, 158, 255, 0.1);
    border: 1px solid rgba(74, 158, 255, 0.2);
    border-radius: 20px;
  }

  .date {
    font-size: 13px;
    color: var(--text-muted);
  }

  .timeline-content {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    padding: 24px;
  }

  .entry-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .entry-version {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .category-tag {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 2px 10px;
    border-radius: 12px;
  }

  .category-tag.feature {
    color: var(--accent);
    background: rgba(74, 158, 255, 0.1);
  }

  .category-tag.fix {
    color: var(--success);
    background: rgba(74, 222, 128, 0.1);
  }

  .category-tag.improvement {
    color: var(--warning);
    background: rgba(251, 191, 36, 0.1);
  }

  .highlights {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .highlights li {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.5;
    padding-left: 20px;
    position: relative;
  }

  .highlights li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 8px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    opacity: 0.6;
  }

  @media (max-width: 768px) {
    .changelog-page {
      padding: 120px 0 80px;
    }

    .timeline::before {
      display: none;
    }

    .timeline-entry {
      padding-left: 0;
    }

    .timeline-entry::before {
      display: none;
    }

    .timeline-content {
      padding: 20px;
    }
  }
</style>
