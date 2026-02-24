<footer class="footer">
  <div class="footer-inner container">
    <div class="footer-brand">
      <div class="brand-row">
        <svg width="24" height="24" viewBox="0 0 36 36" fill="none">
          <ellipse cx="18" cy="10" rx="12" ry="5" stroke="var(--accent)" stroke-width="2" fill="rgba(74, 158, 255, 0.12)"/>
          <path d="M6 10v8c0 2.76 5.37 5 12 5s12-2.24 12-5v-8" stroke="var(--accent)" stroke-width="2" fill="none"/>
          <path d="M6 18v8c0 2.76 5.37 5 12 5s12-2.24 12-5v-8" stroke="var(--accent)" stroke-width="2" fill="none" opacity="0.6"/>
        </svg>
        <span class="brand-name">QueryArk</span>
      </div>
      <p class="brand-tagline">A fast, lightweight database IDE for developers.</p>
    </div>

    <div class="footer-links">
      <div class="link-group">
        <h4>Product</h4>
        <a href="/#features">Features</a>
        <a href="/#databases">Databases</a>
        <a href="/download">Download</a>
        <a href="/docs">Docs</a>
        <a href="/changelog">Changelog</a>
      </div>
      <div class="link-group">
        <h4>Built With</h4>
        <span>Tauri 2 + Rust</span>
        <span>SvelteKit 5</span>
        <span>TypeScript</span>
      </div>
    </div>
  </div>

  <div class="footer-bottom container">
    <span>&copy; {new Date().getFullYear()} QueryArk. All rights reserved.</span>
  </div>
</footer>

<style>
  .footer {
    border-top: 1px solid var(--border-color);
    padding: 64px 0 0;
    background: rgba(15, 17, 23, 0.95);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .footer-inner {
    display: flex;
    justify-content: space-between;
    gap: 48px;
    flex-wrap: wrap;
  }

  .footer-brand {
    max-width: 280px;
  }

  .brand-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .brand-name {
    font-size: 18px;
    font-weight: 800;
    color: var(--accent);
    letter-spacing: -0.5px;
  }

  .brand-tagline {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .footer-links {
    display: flex;
    gap: 64px;
  }

  .link-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .link-group h4 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  .link-group a,
  .link-group span {
    font-size: 14px;
    color: var(--text-secondary);
    text-decoration: none;
    transition: color 200ms ease;
  }

  .link-group a:hover {
    color: var(--text-primary);
  }

  .footer-bottom {
    margin-top: 48px;
    padding: 20px 24px;
    border-top: 1px solid var(--border-color);
    font-size: 13px;
    color: var(--text-muted);
  }

  @media (max-width: 768px) {
    .footer-inner {
      flex-direction: column;
      gap: 40px;
    }

    .footer-links {
      gap: 40px;
      flex-wrap: wrap;
    }

    .footer-brand {
      max-width: 100%;
    }
  }
</style>
