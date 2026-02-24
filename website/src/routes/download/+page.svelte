<script lang="ts">
  import DownloadCard from '$lib/components/DownloadCard.svelte';

  let { data } = $props();
  let version = $derived(data.version);
  let platforms = $derived(data.platforms);
</script>

<svelte:head>
  <title>Download QueryArk — Database IDE for macOS, Windows, Linux</title>
  <meta name="description" content="Download QueryArk for macOS, Windows, or Linux. A fast, lightweight database IDE supporting 16 engines." />
  <link rel="canonical" href="https://queryark.com/download" />
  <meta property="og:title" content="Download QueryArk — Database IDE for macOS, Windows, Linux" />
  <meta property="og:description" content="Download QueryArk for macOS, Windows, or Linux. A fast, lightweight database IDE supporting 16 engines." />
  <meta property="og:url" content="https://queryark.com/download" />
  <meta name="twitter:title" content="Download QueryArk" />
  <meta name="twitter:description" content="Download QueryArk for macOS, Windows, or Linux. A fast, lightweight database IDE supporting 16 engines." />
</svelte:head>

<section class="download-page">
  <div class="container">
    <div class="page-header">
      <span class="version-badge">v{version}</span>
      <h1 class="page-title">Download QueryArk</h1>
      <p class="page-subtitle">
        Available for macOS, Windows, and Linux.
      </p>
    </div>

    <div class="platforms-grid">
      {#each platforms as platform}
        <DownloadCard {platform} />
      {/each}
    </div>

    <div class="extra-info">
      <div class="info-section">
        <h3>System Requirements</h3>
        <ul>
          <li><strong>macOS:</strong> 10.15 (Catalina) or later, Apple Silicon or Intel</li>
          <li><strong>Windows:</strong> Windows 10 or later, 64-bit</li>
          <li><strong>Linux:</strong> Ubuntu 22.04+, Fedora 38+, or any distro with WebKitGTK 4.1</li>
        </ul>
      </div>

    </div>
  </div>
</section>

<style>
  .download-page {
    padding: 140px 0 100px;
    background: rgba(15, 17, 23, 0.92);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .page-header {
    text-align: center;
    margin-bottom: 64px;
  }

  .version-badge {
    display: inline-block;
    padding: 4px 14px;
    font-size: 13px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--accent);
    background: rgba(74, 158, 255, 0.1);
    border: 1px solid rgba(74, 158, 255, 0.2);
    border-radius: 20px;
    margin-bottom: 20px;
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

  .platforms-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 24px;
    margin-bottom: 80px;
  }

  .extra-info {
    display: flex;
    flex-direction: column;
    gap: 48px;
    max-width: 680px;
    margin: 0 auto;
  }

  .info-section h3 {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 12px;
  }

  .info-section ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .info-section li {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.5;
    padding-left: 16px;
    position: relative;
  }

  .info-section li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 8px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .info-section li strong {
    color: var(--text-primary);
  }

  @media (max-width: 900px) {
    .platforms-grid {
      grid-template-columns: 1fr;
      max-width: 480px;
      margin-left: auto;
      margin-right: auto;
      margin-bottom: 60px;
    }
  }
</style>
