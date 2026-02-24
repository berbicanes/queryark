<script lang="ts">
  import DeviceMockup from './DeviceMockup.svelte';
</script>

<section class="hero">
  <div class="hero-content container">
    <div class="hero-text">
      <h1 class="hero-title">
        <span class="hero-line">One IDE for</span>
        <span class="hero-line accent">Every Database</span>
      </h1>
      <p class="hero-subtitle">
        A fast, lightweight database management tool for developers. Connect to PostgreSQL, MySQL, MongoDB, Redis, and 12 more engines &mdash; all in one app.
      </p>
      <div class="hero-actions">
        <a href="/download" class="btn btn-primary btn-large">
          <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
            <path d="M8 2v8M4 7l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M2 12v2h12v-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          Download
        </a>
      </div>
      <p class="hero-meta">macOS, Windows, Linux</p>
    </div>

    <div class="hero-mockup">
      <DeviceMockup type="laptop" />
    </div>
  </div>

  <div class="hero-glow"></div>
</section>

<style>
  .hero {
    position: relative;
    padding: 160px 0 100px;
    overflow: hidden;
    background: rgba(15, 17, 23, 0.92);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .hero-glow {
    position: absolute;
    top: -200px;
    left: 50%;
    transform: translateX(-50%);
    width: 1000px;
    height: 600px;
    background: radial-gradient(ellipse, rgba(74, 158, 255, 0.08) 0%, transparent 70%);
    pointer-events: none;
  }

  .hero-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 60px;
  }

  .hero-text {
    max-width: 700px;
  }

  .hero-title {
    font-size: clamp(40px, 8vw, 72px);
    font-weight: 900;
    line-height: 1.05;
    letter-spacing: -2px;
    margin-bottom: 24px;
  }

  .hero-line {
    display: block;
    color: var(--text-primary);
  }

  .hero-line.accent {
    color: var(--accent);
  }

  .hero-subtitle {
    font-size: clamp(16px, 2.5vw, 20px);
    color: var(--text-secondary);
    line-height: 1.6;
    margin-bottom: 36px;
  }

  .hero-actions {
    display: flex;
    gap: 16px;
    justify-content: center;
    flex-wrap: wrap;
  }

  .hero-meta {
    margin-top: 16px;
    font-size: 15px;
    color: var(--text-secondary);
    letter-spacing: 1px;
  }

  .hero-mockup {
    width: 100%;
    max-width: 760px;
    display: flex;
    justify-content: center;
  }

  @media (max-width: 768px) {
    .hero {
      padding: 120px 0 60px;
    }

    .hero-content {
      gap: 40px;
    }

    .hero-title {
      letter-spacing: -1px;
    }

    .hero-actions {
      flex-direction: column;
      align-items: center;
    }
  }
</style>
