<script lang="ts">
  import Hero from '$lib/components/Hero.svelte';
  import DatabaseGrid from '$lib/components/DatabaseGrid.svelte';
  import FeatureSection from '$lib/components/FeatureSection.svelte';
  import { features } from '$lib/data/features';
</script>

<svelte:head>
  <title>QueryArk — One IDE for Every Database</title>
  <meta name="description" content="A fast, lightweight database IDE supporting 16 engines. Cross-platform." />
  <link rel="canonical" href="https://queryark.com" />
  <meta property="og:title" content="QueryArk — One IDE for Every Database" />
  <meta property="og:description" content="A fast, lightweight database IDE supporting 16 engines. Cross-platform." />
  <meta property="og:url" content="https://queryark.com" />
  <meta name="twitter:title" content="QueryArk — One IDE for Every Database" />
  <meta name="twitter:description" content="A fast, lightweight database IDE supporting 16 engines. Cross-platform." />
  {@html `<script type="application/ld+json">${JSON.stringify({
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    "name": "QueryArk",
    "applicationCategory": "DeveloperApplication",
    "operatingSystem": "macOS, Windows, Linux",
    "description": "A fast, lightweight database IDE supporting 16 engines. Cross-platform.",
    "url": "https://queryark.com",
    "downloadUrl": "https://queryark.com/download",
    "softwareVersion": "0.2.0",
    "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" }
  })}</script>`}
</svelte:head>

<Hero />

<DatabaseGrid />

<section class="features section" id="features">
  <div class="container">
    <div class="features-header">
      <p class="section-label">Features</p>
      <h2 class="section-title">Everything You Need</h2>
      <p class="section-subtitle">
        A complete database toolkit — from writing queries to visualizing schemas.
      </p>
    </div>

    {#each features as feature, i}
      <FeatureSection {feature} reversed={i % 2 === 1} />
    {/each}
  </div>
</section>

<section class="cta section">
  <div class="container cta-content">
    <h2 class="cta-title">
      Built for Developers.
    </h2>
    <p class="cta-desc">
      Download QueryArk and start managing your databases faster.
    </p>
    <div class="cta-actions">
      <a href="/download" class="btn btn-primary btn-large">
        <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
          <path d="M8 2v8M4 7l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M2 12v2h12v-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        Download
      </a>
    </div>
  </div>
</section>

<style>
  .features-header {
    text-align: center;
    margin-bottom: 40px;
  }

  .cta {
    text-align: center;
    border-top: 1px solid var(--border-color);
    background: radial-gradient(ellipse 60% 50% at 50% 50%, rgba(74, 158, 255, 0.06) 0%, transparent 70%);
  }

  .cta-content {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .cta-title {
    font-size: clamp(32px, 5vw, 48px);
    font-weight: 900;
    letter-spacing: -1px;
    line-height: 1.15;
    color: var(--text-primary);
    margin-bottom: 20px;
  }

  .cta-desc {
    font-size: 18px;
    color: var(--text-secondary);
    max-width: 520px;
    line-height: 1.6;
    margin-bottom: 36px;
  }

  .cta-actions {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    justify-content: center;
  }

  @media (max-width: 768px) {
    .cta-actions {
      flex-direction: column;
      align-items: center;
    }
  }
</style>
