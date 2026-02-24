<script lang="ts">
  import DeviceMockup from './DeviceMockup.svelte';
  import type { Feature } from '$lib/data/features';

  let { feature, reversed = false }: { feature: Feature; reversed?: boolean } = $props();
</script>

<div class="feature" class:reversed>
  <div class="feature-text">
    <span class="feature-number">{feature.number}</span>
    <h3 class="feature-title">{feature.title}</h3>
    <p class="feature-desc">{feature.description}</p>
    <ul class="feature-highlights">
      {#each feature.highlights as highlight}
        <li>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 8l3 3 5-6" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          {highlight}
        </li>
      {/each}
    </ul>
  </div>

  <div class="feature-mockup">
    <DeviceMockup type={feature.mockupType} />
  </div>
</div>

<style>
  .feature {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 64px;
    align-items: center;
    padding: 60px 0;
  }

  .feature.reversed {
    direction: rtl;
  }

  .feature.reversed > * {
    direction: ltr;
  }

  .feature-text {
    display: flex;
    flex-direction: column;
  }

  .feature-number {
    font-size: 48px;
    font-weight: 900;
    background: linear-gradient(135deg, #4a9eff 0%, rgba(255, 255, 255, 0.7) 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    letter-spacing: -2px;
    line-height: 1;
    margin-bottom: 8px;
  }

  .feature-title {
    font-size: clamp(24px, 3vw, 32px);
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.5px;
    margin-bottom: 16px;
  }

  .feature-desc {
    font-size: 16px;
    color: var(--text-secondary);
    line-height: 1.6;
    margin-bottom: 24px;
  }

  .feature-highlights {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .feature-highlights li {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: var(--text-secondary);
  }

  .feature-highlights li svg {
    flex-shrink: 0;
  }

  .feature-mockup {
    display: flex;
    justify-content: center;
    align-items: center;
  }

  @media (max-width: 900px) {
    .feature {
      grid-template-columns: 1fr;
      gap: 40px;
      padding: 40px 0;
    }

    .feature.reversed {
      direction: ltr;
    }

    .feature-mockup {
      order: -1;
    }
  }
</style>
