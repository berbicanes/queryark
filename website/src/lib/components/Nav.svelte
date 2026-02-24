<script lang="ts">
  let scrolled = $state(false);
  let mobileOpen = $state(false);

  $effect(() => {
    function onScroll() {
      scrolled = window.scrollY > 20;
    }
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  });

  function closeMobile() {
    mobileOpen = false;
  }
</script>

<nav class="nav" class:scrolled>
  <div class="nav-inner container">
    <a href="/" class="nav-brand" onclick={closeMobile}>
      <svg class="brand-icon" width="28" height="28" viewBox="0 0 36 36" fill="none">
        <ellipse cx="18" cy="10" rx="12" ry="5" stroke="var(--accent)" stroke-width="2" fill="rgba(74, 158, 255, 0.12)"/>
        <path d="M6 10v8c0 2.76 5.37 5 12 5s12-2.24 12-5v-8" stroke="var(--accent)" stroke-width="2" fill="none"/>
        <path d="M6 18v8c0 2.76 5.37 5 12 5s12-2.24 12-5v-8" stroke="var(--accent)" stroke-width="2" fill="none" opacity="0.6"/>
      </svg>
      <span class="brand-text">QueryArk</span>
    </a>

    <div class="nav-links" class:open={mobileOpen}>
      <a href="/#databases" class="nav-link" onclick={closeMobile}>Databases</a>
      <a href="/#features" class="nav-link" onclick={closeMobile}>Features</a>
      <a href="/download" class="nav-link" onclick={closeMobile}>Download</a>
      <a href="/docs" class="nav-link" onclick={closeMobile}>Docs</a>
      <a href="/changelog" class="nav-link" onclick={closeMobile}>Changelog</a>
    </div>

    <button class="hamburger" onclick={() => mobileOpen = !mobileOpen} aria-label="Toggle menu">
      <span class="bar" class:open={mobileOpen}></span>
      <span class="bar" class:open={mobileOpen}></span>
      <span class="bar" class:open={mobileOpen}></span>
    </button>
  </div>
</nav>

<style>
  .nav {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
    padding: 16px 0;
    transition: background 300ms ease, backdrop-filter 300ms ease, padding 300ms ease;
  }

  .nav.scrolled {
    background: rgba(15, 17, 23, 0.85);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    padding: 12px 0;
    border-bottom: 1px solid var(--border-color);
  }

  .nav-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .nav-brand {
    display: flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    color: var(--text-primary);
  }

  .brand-icon {
    filter: drop-shadow(0 0 8px rgba(74, 158, 255, 0.3));
  }

  .brand-text {
    font-size: 20px;
    font-weight: 800;
    color: var(--accent);
    letter-spacing: -0.5px;
  }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 32px;
  }

  .nav-link {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-secondary);
    text-decoration: none;
    transition: color 200ms ease;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .nav-link:hover {
    color: var(--text-primary);
  }

  .hamburger {
    display: none;
    flex-direction: column;
    gap: 5px;
    padding: 4px;
    background: none;
    border: none;
    cursor: pointer;
  }

  .bar {
    width: 24px;
    height: 2px;
    background: var(--text-secondary);
    border-radius: 2px;
    transition: transform 300ms ease, opacity 300ms ease;
  }

  .bar.open:nth-child(1) {
    transform: translateY(7px) rotate(45deg);
  }

  .bar.open:nth-child(2) {
    opacity: 0;
  }

  .bar.open:nth-child(3) {
    transform: translateY(-7px) rotate(-45deg);
  }

  @media (max-width: 768px) {
    .hamburger {
      display: flex;
    }

    .nav-links {
      display: none;
      position: absolute;
      top: 100%;
      left: 0;
      right: 0;
      flex-direction: column;
      gap: 0;
      background: rgba(15, 17, 23, 0.95);
      backdrop-filter: blur(16px);
      -webkit-backdrop-filter: blur(16px);
      border-bottom: 1px solid var(--border-color);
      padding: 8px 0;
    }

    .nav-links.open {
      display: flex;
    }

    .nav-link {
      padding: 14px 24px;
      width: 100%;
      font-size: 15px;
    }

    .nav-link:hover {
      background: var(--bg-hover);
    }
  }
</style>
