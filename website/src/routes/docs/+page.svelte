<script lang="ts">
  import { shortcuts, shortcutCategories } from '$lib/data/shortcuts';
  import { connectionGuides } from '$lib/data/connections';

  let expandedDb = $state<string | null>(null);

  function toggleDb(name: string) {
    expandedDb = expandedDb === name ? null : name;
  }
</script>

<svelte:head>
  <title>Documentation — QueryArk</title>
  <meta name="description" content="QueryArk documentation: getting started, database connection guides, keyboard shortcuts, and feature overview." />
  <link rel="canonical" href="https://queryark.com/docs" />
  <meta property="og:title" content="Documentation — QueryArk" />
  <meta property="og:description" content="Getting started, database connection guides, keyboard shortcuts, and feature overview." />
  <meta property="og:url" content="https://queryark.com/docs" />
  <meta name="twitter:title" content="Documentation — QueryArk" />
  <meta name="twitter:description" content="Getting started, database connection guides, keyboard shortcuts, and feature overview." />
</svelte:head>

<section class="docs-page">
  <div class="container docs-layout">
    <aside class="docs-sidebar">
      <nav class="sidebar-nav">
        <a href="#getting-started" class="sidebar-link">Getting Started</a>
        <a href="#connecting" class="sidebar-link">Connecting to Databases</a>
        <a href="#shortcuts" class="sidebar-link">Keyboard Shortcuts</a>
        <a href="#features" class="sidebar-link">Features Overview</a>
      </nav>
    </aside>

    <div class="docs-content">
      <!-- Getting Started -->
      <section id="getting-started" class="docs-section">
        <h2 class="section-heading">Getting Started</h2>
        <p class="section-intro">Get up and running with QueryArk in four steps.</p>

        <div class="steps">
          <div class="step">
            <span class="step-number">1</span>
            <div class="step-body">
              <h3>Download & Install</h3>
              <p>
                Grab the latest release from the <a href="/download">download page</a>.
                Open the installer for your platform — DMG on macOS, EXE/MSI on Windows, or AppImage/deb/rpm on Linux.
              </p>
            </div>
          </div>

          <div class="step">
            <span class="step-number">2</span>
            <div class="step-body">
              <h3>Create a Connection</h3>
              <p>
                Click <strong>"New Connection"</strong> in the sidebar. Pick your database type, fill in host, port, credentials,
                and click <strong>Test</strong> to verify. You can also paste a connection URL to auto-fill the form.
              </p>
            </div>
          </div>

          <div class="step">
            <span class="step-number">3</span>
            <div class="step-body">
              <h3>Browse Your Schema</h3>
              <p>
                Once connected, expand the schema tree in the sidebar. Click a table to open it in a data tab,
                or right-click for options like viewing structure, DDL, or opening in the ER diagram.
              </p>
            </div>
          </div>

          <div class="step">
            <span class="step-number">4</span>
            <div class="step-body">
              <h3>Run Your First Query</h3>
              <p>
                Press <kbd>Ctrl+N</kbd> to open a new query tab. Write SQL in the editor with autocomplete,
                then press <kbd>Ctrl+Enter</kbd> to execute. Results appear in the data grid below.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Connecting to Databases -->
      <section id="connecting" class="docs-section">
        <h2 class="section-heading">Connecting to Databases</h2>
        <p class="section-intro">QueryArk supports 16 database engines. Select one below for connection details.</p>

        <div class="db-cards">
          {#each connectionGuides as db}
            <div class="db-card" class:expanded={expandedDb === db.name}>
              <button class="db-card-header" onclick={() => toggleDb(db.name)}>
                <span class="db-badge" style="color: {db.color}; background: {db.color}1a; border-color: {db.color}33">{db.badge}</span>
                <span class="db-name">{db.name}</span>
                <svg class="chevron" width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
              {#if expandedDb === db.name}
                <div class="db-card-body">
                  <div class="db-field">
                    <span class="field-label">Default Port</span>
                    <span class="field-value">{db.defaultPort ?? 'N/A'}</span>
                  </div>
                  <div class="db-field">
                    <span class="field-label">Required Fields</span>
                    <span class="field-value">{db.requiredFields.join(', ')}</span>
                  </div>
                  <div class="db-field">
                    <span class="field-label">Example URL</span>
                    <code class="field-code">{db.exampleUrl}</code>
                  </div>
                  <div class="db-field">
                    <span class="field-label">Notes</span>
                    <span class="field-value">{db.notes}</span>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </section>

      <!-- Keyboard Shortcuts -->
      <section id="shortcuts" class="docs-section">
        <h2 class="section-heading">Keyboard Shortcuts</h2>
        <p class="section-intro">Default keyboard shortcuts. All shortcuts are customizable in-app via <kbd>Ctrl+K</kbd>.</p>

        {#each shortcutCategories as category}
          <div class="shortcut-group">
            <h3 class="group-title">{category}</h3>
            <div class="shortcut-table">
              {#each shortcuts.filter(s => s.category === category) as shortcut}
                <div class="shortcut-row">
                  <span class="shortcut-label">{shortcut.label}</span>
                  <kbd class="shortcut-key">{shortcut.key}</kbd>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </section>

      <!-- Features Overview -->
      <section id="features" class="docs-section">
        <h2 class="section-heading">Features Overview</h2>
        <p class="section-intro">A quick look at what QueryArk offers.</p>

        <div class="feature-cards">
          <div class="feature-card">
            <h3>Query Editor</h3>
            <p>
              CodeMirror 6-powered SQL editor with schema-aware autocomplete, multi-dialect highlighting
              (PostgreSQL, MySQL, SQLite, MSSQL, Cassandra), multi-statement execution, query formatting,
              saved queries, and query history.
            </p>
          </div>

          <div class="feature-card">
            <h3>Data Grid</h3>
            <p>
              Spreadsheet-like grid with inline editing, multi-cell selection, drag-fill, column sorting/filtering/resizing,
              find &amp; replace, foreign key dropdowns, and export to CSV, JSON, SQL, or Markdown.
            </p>
          </div>

          <div class="feature-card">
            <h3>Schema Browser</h3>
            <p>
              Expandable tree with schemas, tables, views, functions, sequences, and types.
              Virtual scrolling handles thousands of tables. Includes table stats, DDL viewer,
              favorite tables, and multi-schema visibility.
            </p>
          </div>

          <div class="feature-card">
            <h3>Visual Tools</h3>
            <p>
              Interactive ER diagrams with zoom/pan, visual query builder for JOINs and GROUP BY,
              table structure diff and data diff across environments, and automatic migration script generation.
            </p>
          </div>

          <div class="feature-card">
            <h3>Security</h3>
            <p>
              SSH tunneling through bastion hosts, SSL certificate configuration, OS keychain integration
              for passwords (macOS Keychain, Windows Credential Manager, Linux Secret Service),
              and connection URL parsing.
            </p>
          </div>

          <div class="feature-card">
            <h3>Productivity</h3>
            <p>
              Query result charts (bar, line, pie), parameterized query execution, query profiling
              with optimization hints, auto-suggested indexes, workspace profiles, result bookmarks,
              and query snippet templates.
            </p>
          </div>
        </div>
      </section>
    </div>
  </div>
</section>

<style>
  .docs-page {
    padding: 120px 0 100px;
    background: rgba(15, 17, 23, 0.92);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .docs-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 48px;
    align-items: start;
  }

  /* Sidebar */
  .docs-sidebar {
    position: sticky;
    top: 100px;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-left: 2px solid var(--bg-hover);
    padding-left: 16px;
  }

  .sidebar-link {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
    text-decoration: none;
    padding: 6px 0;
    transition: color 200ms ease;
  }

  .sidebar-link:hover {
    color: var(--text-primary);
  }

  /* Content */
  .docs-content {
    min-width: 0;
  }

  .docs-section {
    margin-bottom: 80px;
  }

  .section-heading {
    font-size: 28px;
    font-weight: 800;
    letter-spacing: -0.5px;
    color: var(--text-primary);
    margin-bottom: 12px;
  }

  .section-intro {
    font-size: 16px;
    color: var(--text-secondary);
    margin-bottom: 32px;
    line-height: 1.6;
  }

  /* Getting Started Steps */
  .steps {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .step {
    display: flex;
    gap: 20px;
    align-items: flex-start;
  }

  .step-number {
    flex-shrink: 0;
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    font-weight: 700;
    color: var(--accent);
    background: rgba(74, 158, 255, 0.1);
    border: 1px solid rgba(74, 158, 255, 0.2);
    border-radius: 50%;
  }

  .step-body h3 {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .step-body p {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .step-body kbd {
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
  }

  /* Database Cards */
  .db-cards {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .db-card {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: border-color 200ms ease;
  }

  .db-card.expanded {
    border-color: var(--accent);
  }

  .db-card-header {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 14px 16px;
    background: var(--bg-secondary);
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 600;
    text-align: left;
    transition: background 200ms ease;
  }

  .db-card-header:hover {
    background: var(--bg-tertiary);
  }

  .db-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 22px;
    font-size: 11px;
    font-weight: 700;
    font-family: var(--font-mono);
    border-radius: var(--radius-sm);
    border: 1px solid;
    flex-shrink: 0;
  }

  .db-name {
    flex: 1;
  }

  .chevron {
    color: var(--text-muted);
    transition: transform 200ms ease;
    flex-shrink: 0;
  }

  .db-card.expanded .chevron {
    transform: rotate(180deg);
  }

  .db-card-body {
    padding: 16px 16px 20px;
    background: var(--bg-primary);
    display: flex;
    flex-direction: column;
    gap: 12px;
    border-top: 1px solid var(--border-color);
  }

  .db-field {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
  }

  .field-value {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .field-code {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--accent);
    background: var(--bg-secondary);
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
    word-break: break-all;
  }

  /* Keyboard Shortcuts */
  .shortcut-group {
    margin-bottom: 28px;
  }

  .group-title {
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-muted);
    margin-bottom: 12px;
  }

  .shortcut-table {
    display: flex;
    flex-direction: column;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .shortcut-row:last-child {
    border-bottom: none;
  }

  .shortcut-label {
    font-size: 14px;
    color: var(--text-secondary);
  }

  .shortcut-key {
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 3px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    white-space: nowrap;
  }

  /* Feature Cards */
  .feature-cards {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .feature-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    padding: 24px;
  }

  .feature-card h3 {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 10px;
  }

  .feature-card p {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  /* Responsive */
  @media (max-width: 900px) {
    .docs-page {
      padding: 100px 0 80px;
    }

    .docs-layout {
      grid-template-columns: 1fr;
      gap: 0;
    }

    .docs-sidebar {
      position: static;
      margin-bottom: 40px;
    }

    .sidebar-nav {
      flex-direction: row;
      flex-wrap: wrap;
      border-left: none;
      padding-left: 0;
      gap: 8px;
    }

    .sidebar-link {
      padding: 6px 12px;
      background: var(--bg-secondary);
      border: 1px solid var(--border-color);
      border-radius: var(--radius-lg);
      font-size: 12px;
    }

    .feature-cards {
      grid-template-columns: 1fr;
    }

    .docs-section {
      margin-bottom: 56px;
    }

    .section-heading {
      font-size: 22px;
    }
  }

  @media (max-width: 480px) {
    .shortcut-row {
      padding: 10px 8px;
      gap: 8px;
    }

    .shortcut-label {
      font-size: 13px;
    }

    .db-card-header {
      padding: 12px;
    }

    .db-card-body {
      padding: 12px 12px 16px;
    }

    .field-code {
      font-size: 12px;
    }

    .feature-card {
      padding: 20px;
    }
  }
</style>
