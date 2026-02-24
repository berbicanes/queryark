<script lang="ts">
  let { type = 'laptop' }: { type: 'laptop' | 'tablet' | 'phone' } = $props();
</script>

{#if type === 'laptop'}
  <div class="device laptop">
    <div class="laptop-lid">
      <div class="laptop-bezel">
        <div class="laptop-camera"></div>
        <div class="laptop-screen">
          <div class="app-chrome">
            <div class="traffic-lights">
              <span class="dot red"></span>
              <span class="dot yellow"></span>
              <span class="dot green"></span>
            </div>
            <span class="app-title">QueryArk</span>
          </div>
          <div class="app-body">
            <div class="sidebar">
              <div class="sidebar-header">
                <span class="conn-badge">PG</span>
                <span class="conn-name">Production PG</span>
                <span class="conn-status"></span>
              </div>
              <div class="tree">
                <div class="tree-item schema">
                  <span class="arrow">&#9662;</span> public
                </div>
                <div class="tree-item table active">
                  <span class="icon">&#9638;</span> users
                </div>
                <div class="tree-item table">
                  <span class="icon">&#9638;</span> orders
                </div>
                <div class="tree-item table">
                  <span class="icon">&#9638;</span> products
                </div>
                <div class="tree-item table">
                  <span class="icon">&#9638;</span> payments
                </div>
              </div>
            </div>
            <div class="main-area">
              <div class="tab-bar">
                <span class="tab active">
                  <span class="tab-badge">PG</span>
                  Query 1
                </span>
                <span class="tab">users</span>
              </div>
              <div class="editor">
                <div class="editor-line">
                  <span class="ln">1</span>
                  <span class="kw">SELECT</span> u.id, u.name, u.email, u.role
                </div>
                <div class="editor-line">
                  <span class="ln">2</span>
                  <span class="kw">FROM</span> users u
                </div>
                <div class="editor-line">
                  <span class="ln">3</span>
                  <span class="kw">JOIN</span> orders o <span class="kw">ON</span> o.user_id = u.id
                </div>
                <div class="editor-line">
                  <span class="ln">4</span>
                  <span class="kw">WHERE</span> u.active = <span class="bool">true</span>
                </div>
                <div class="editor-line">
                  <span class="ln">5</span>
                  <span class="kw">ORDER BY</span> u.name <span class="kw">ASC</span>;
                </div>
              </div>
              <div class="grid">
                <div class="grid-header">
                  <span class="gh">id</span>
                  <span class="gh">name</span>
                  <span class="gh">email</span>
                  <span class="gh">role</span>
                </div>
                <div class="grid-row">
                  <span class="gc num">1</span>
                  <span class="gc">Alice Chen</span>
                  <span class="gc str">alice@example.com</span>
                  <span class="gc">admin</span>
                </div>
                <div class="grid-row">
                  <span class="gc num">2</span>
                  <span class="gc">Bob Smith</span>
                  <span class="gc str">bob@example.com</span>
                  <span class="gc">editor</span>
                </div>
                <div class="grid-row">
                  <span class="gc num">3</span>
                  <span class="gc">Carol Diaz</span>
                  <span class="gc str">carol@example.com</span>
                  <span class="gc">viewer</span>
                </div>
                <div class="grid-row">
                  <span class="gc num">4</span>
                  <span class="gc">Dave Kim</span>
                  <span class="gc str">dave@example.com</span>
                  <span class="gc">editor</span>
                </div>
              </div>
              <div class="status-bar">
                <span class="status-connected">&#9679; Connected</span>
                <span>localhost:5432</span>
                <span class="status-right">4 rows &middot; 12ms</span>
              </div>
            </div>
          </div>
        </div>
        <div class="screen-reflection"></div>
      </div>
    </div>
    <div class="laptop-hinge"></div>
    <div class="laptop-base">
      <div class="laptop-keyboard">
        <div class="keyboard-rows">
          <div class="key-row">{#each Array(13) as _}<span class="key"></span>{/each}</div>
          <div class="key-row">{#each Array(12) as _}<span class="key"></span>{/each}</div>
          <div class="key-row">{#each Array(11) as _}<span class="key"></span>{/each}</div>
          <div class="key-row"><span class="key spacebar"></span></div>
        </div>
      </div>
      <div class="laptop-trackpad"></div>
    </div>
  </div>
{:else if type === 'tablet'}
  <div class="device tablet">
    <div class="tablet-screen">
      <div class="app-chrome tablet-chrome">
        <div class="traffic-lights">
          <span class="dot red"></span>
          <span class="dot yellow"></span>
          <span class="dot green"></span>
        </div>
        <span class="app-title">QueryArk</span>
      </div>
      <div class="tablet-grid">
        <div class="grid-header">
          <span class="gh">id</span>
          <span class="gh">product</span>
          <span class="gh">price</span>
          <span class="gh">stock</span>
          <span class="gh">category</span>
          <span class="gh">rating</span>
        </div>
        <div class="grid-row">
          <span class="gc num">101</span>
          <span class="gc">Wireless Mouse</span>
          <span class="gc num">29.99</span>
          <span class="gc num">142</span>
          <span class="gc">Electronics</span>
          <span class="gc num">4.5</span>
        </div>
        <div class="grid-row">
          <span class="gc num">102</span>
          <span class="gc">USB-C Hub</span>
          <span class="gc num">49.99</span>
          <span class="gc num">87</span>
          <span class="gc">Electronics</span>
          <span class="gc num">4.2</span>
        </div>
        <div class="grid-row">
          <span class="gc num">103</span>
          <span class="gc">Desk Lamp</span>
          <span class="gc num">34.50</span>
          <span class="gc num">56</span>
          <span class="gc">Furniture</span>
          <span class="gc num">4.8</span>
        </div>
        <div class="grid-row">
          <span class="gc num">104</span>
          <span class="gc">Notebook Set</span>
          <span class="gc num">12.99</span>
          <span class="gc num">230</span>
          <span class="gc">Stationery</span>
          <span class="gc num">4.1</span>
        </div>
        <div class="grid-row">
          <span class="gc num">105</span>
          <span class="gc">Monitor Stand</span>
          <span class="gc num">79.00</span>
          <span class="gc num">34</span>
          <span class="gc">Furniture</span>
          <span class="gc num">4.6</span>
        </div>
        <div class="grid-row last">
          <span class="gc num">106</span>
          <span class="gc">Webcam HD</span>
          <span class="gc num">59.99</span>
          <span class="gc num">71</span>
          <span class="gc">Electronics</span>
          <span class="gc num">4.3</span>
        </div>
        <div class="tablet-pagination">
          <span>Page 1 of 12</span>
          <span class="page-btns">
            <span class="page-btn disabled">&lsaquo;</span>
            <span class="page-btn active">1</span>
            <span class="page-btn">2</span>
            <span class="page-btn">3</span>
            <span class="page-btn">&rsaquo;</span>
          </span>
        </div>
      </div>
    </div>
  </div>
{:else}
  <div class="device phone">
    <div class="phone-screen">
      <div class="phone-status-bar">
        <span>9:41</span>
        <span class="phone-icons">
          <span>&#9679;&#9679;&#9679;</span>
        </span>
      </div>
      <div class="phone-header">
        <span class="phone-title">Connections</span>
      </div>
      <div class="phone-connections">
        <div class="phone-card" style="--card-color: #4a9eff">
          <span class="phone-badge" style="background: rgba(74,158,255,0.2); color: #4a9eff">PG</span>
          <div class="phone-card-info">
            <span class="phone-card-name">Production PG</span>
            <span class="phone-card-host">db.example.com:5432</span>
          </div>
          <span class="phone-status connected"></span>
        </div>
        <div class="phone-card" style="--card-color: #fbbf24">
          <span class="phone-badge" style="background: rgba(251,191,36,0.2); color: #fbbf24">MY</span>
          <div class="phone-card-info">
            <span class="phone-card-name">Staging MySQL</span>
            <span class="phone-card-host">staging.internal:3306</span>
          </div>
          <span class="phone-status"></span>
        </div>
        <div class="phone-card" style="--card-color: #f87171">
          <span class="phone-badge" style="background: rgba(248,113,113,0.2); color: #f87171">RD</span>
          <div class="phone-card-info">
            <span class="phone-card-name">Cache Redis</span>
            <span class="phone-card-host">redis.internal:6379</span>
          </div>
          <span class="phone-status connected"></span>
        </div>
        <div class="phone-card" style="--card-color: #74c7ec">
          <span class="phone-badge" style="background: rgba(116,199,236,0.2); color: #74c7ec">BQ</span>
          <div class="phone-card-info">
            <span class="phone-card-name">Analytics BQ</span>
            <span class="phone-card-host">my-project.analytics</span>
          </div>
          <span class="phone-status"></span>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* === Shared === */
  .device {
    font-family: var(--font-sans);
    user-select: none;
  }

  /* === Laptop === */
  .laptop {
    width: 100%;
    max-width: 760px;
    perspective: 1200px;
  }

  .laptop-lid {
    position: relative;
  }

  .laptop-bezel {
    position: relative;
    background: linear-gradient(180deg, #2a2c32 0%, #1c1e24 100%);
    padding: 20px 16px 16px;
    border-radius: 14px 14px 0 0;
    box-shadow:
      0 -1px 0 rgba(255, 255, 255, 0.06) inset,
      0 20px 60px rgba(0, 0, 0, 0.6),
      0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .laptop-camera {
    position: absolute;
    top: 8px;
    left: 50%;
    transform: translateX(-50%);
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #1a1c22;
    box-shadow: 0 0 0 1px #333640, 0 0 2px 1px rgba(74, 158, 255, 0.15);
  }

  .laptop-screen {
    background: #0f1117;
    border-radius: 4px;
    overflow: hidden;
    position: relative;
  }

  .screen-reflection {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      165deg,
      rgba(255, 255, 255, 0.03) 0%,
      transparent 40%,
      transparent 100%
    );
    pointer-events: none;
    border-radius: 4px;
  }

  .laptop-hinge {
    height: 8px;
    background: linear-gradient(180deg, #1a1c22 0%, #2a2c32 40%, #1a1c22 100%);
    border-radius: 0;
    margin: 0 2%;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
  }

  .laptop-base {
    background: linear-gradient(180deg, #2a2c32 0%, #22242a 60%, #1e2026 100%);
    height: 72px;
    border-radius: 0 0 8px 8px;
    margin: 0 -2%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    box-shadow:
      0 1px 0 rgba(255, 255, 255, 0.04) inset,
      0 4px 12px rgba(0, 0, 0, 0.3);
    position: relative;
    overflow: hidden;
  }

  .laptop-keyboard {
    width: 82%;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .keyboard-rows {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .key-row {
    display: flex;
    gap: 2px;
    justify-content: center;
  }

  .key {
    flex: 1;
    height: 7px;
    background: linear-gradient(180deg, #383a42 0%, #2e3038 100%);
    border-radius: 1.5px;
    max-width: 28px;
    box-shadow: 0 0.5px 0 rgba(0, 0, 0, 0.4);
  }

  .key.spacebar {
    flex: 4;
    max-width: 140px;
  }

  .laptop-trackpad {
    width: 30%;
    height: 10px;
    background: linear-gradient(180deg, #2e3038 0%, #282a30 100%);
    border-radius: 3px;
    border: 1px solid rgba(255, 255, 255, 0.04);
  }

  .app-chrome {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: #161821;
    border-bottom: 1px solid #2a2d3a;
  }

  .traffic-lights {
    display: flex;
    gap: 6px;
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .dot.red { background: #f87171; }
  .dot.yellow { background: #fbbf24; }
  .dot.green { background: #4ade80; }

  .app-title {
    font-size: 11px;
    font-weight: 600;
    color: #5c6070;
  }

  .app-body {
    display: flex;
    height: 280px;
  }

  /* Sidebar */
  .sidebar {
    width: 160px;
    background: #161821;
    border-right: 1px solid #2a2d3a;
    flex-shrink: 0;
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    border-bottom: 1px solid #2a2d3a;
  }

  .conn-badge {
    font-size: 8px;
    font-weight: 700;
    padding: 2px 5px;
    border-radius: 3px;
    background: rgba(74, 158, 255, 0.2);
    color: #4a9eff;
  }

  .conn-name {
    font-size: 10px;
    font-weight: 600;
    color: #e4e6ef;
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .conn-status {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #4ade80;
    box-shadow: 0 0 4px rgba(74, 222, 128, 0.5);
    flex-shrink: 0;
  }

  .tree {
    padding: 6px 0;
  }

  .tree-item {
    padding: 3px 10px 3px 12px;
    font-size: 10px;
    color: #9ca0b0;
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .tree-item.schema {
    color: #e4e6ef;
    font-weight: 600;
  }

  .tree-item.table {
    padding-left: 24px;
  }

  .tree-item.active {
    background: rgba(74, 158, 255, 0.1);
    color: #4a9eff;
  }

  .arrow {
    font-size: 7px;
    color: #5c6070;
  }

  .icon {
    font-size: 6px;
    color: #5c6070;
  }

  /* Main area */
  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .tab-bar {
    display: flex;
    gap: 0;
    background: #161821;
    border-bottom: 1px solid #2a2d3a;
    padding: 0 4px;
  }

  .tab {
    padding: 5px 12px;
    font-size: 9px;
    color: #5c6070;
    border-bottom: 2px solid transparent;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tab.active {
    color: #e4e6ef;
    border-bottom-color: #4a9eff;
  }

  .tab-badge {
    font-size: 7px;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: 2px;
    background: rgba(74, 158, 255, 0.2);
    color: #4a9eff;
  }

  /* Editor */
  .editor {
    padding: 8px 0;
    background: #0f1117;
    flex-shrink: 0;
    border-bottom: 1px solid #2a2d3a;
  }

  .editor-line {
    padding: 1px 10px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    line-height: 1.6;
    color: #e4e6ef;
    white-space: nowrap;
    overflow: hidden;
  }

  .ln {
    display: inline-block;
    width: 16px;
    text-align: right;
    color: #2a2d3a;
    margin-right: 10px;
    font-size: 8px;
  }

  .kw { color: #4a9eff; }
  .str { color: #4ade80; }
  .num { color: #fbbf24; }
  .bool { color: #fbbf24; }

  /* Grid */
  .grid {
    flex: 1;
    overflow: hidden;
    font-size: 9px;
  }

  .grid-header {
    display: grid;
    grid-template-columns: 40px 1fr 1fr 80px;
    padding: 4px 10px;
    background: #161821;
    border-bottom: 1px solid #2a2d3a;
    font-weight: 600;
    color: #9ca0b0;
  }

  .gh {
    padding: 2px 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .grid-row {
    display: grid;
    grid-template-columns: 40px 1fr 1fr 80px;
    padding: 3px 10px;
    border-bottom: 1px solid rgba(42, 45, 58, 0.5);
    color: #e4e6ef;
  }

  .grid-row:hover {
    background: rgba(74, 158, 255, 0.05);
  }

  .gc {
    padding: 2px 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .gc.num { color: #fbbf24; }
  .gc.str { color: #4ade80; }

  /* Status bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 10px;
    background: #161821;
    border-top: 1px solid #2a2d3a;
    font-size: 8px;
    color: #5c6070;
  }

  .status-connected {
    color: #4ade80;
  }

  .status-right {
    margin-left: auto;
  }

  /* === Tablet === */
  .tablet {
    width: 100%;
    max-width: 540px;
  }

  .tablet-screen {
    background: #0f1117;
    border: 2px solid #2a2d3a;
    border-radius: 16px;
    overflow: hidden;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    padding: 0;
  }

  .tablet-chrome {
    border-radius: 0;
  }

  .tablet-grid {
    font-size: 9px;
  }

  .tablet-grid .grid-header {
    grid-template-columns: 50px 1fr 70px 60px 90px 60px;
  }

  .tablet-grid .grid-row {
    grid-template-columns: 50px 1fr 70px 60px 90px 60px;
  }

  .tablet-grid .grid-row.last {
    border-bottom: none;
  }

  .tablet-pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: #161821;
    border-top: 1px solid #2a2d3a;
    font-size: 8px;
    color: #5c6070;
  }

  .page-btns {
    display: flex;
    gap: 3px;
  }

  .page-btn {
    padding: 2px 6px;
    background: #1e2028;
    border-radius: 3px;
    color: #9ca0b0;
    font-size: 8px;
  }

  .page-btn.active {
    background: #4a9eff;
    color: #fff;
  }

  .page-btn.disabled {
    opacity: 0.3;
  }

  /* === Phone === */
  .phone {
    width: 100%;
    max-width: 280px;
  }

  .phone-screen {
    background: #0f1117;
    border: 2px solid #2a2d3a;
    border-radius: 24px;
    overflow: hidden;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    padding: 0;
  }

  .phone-status-bar {
    display: flex;
    justify-content: space-between;
    padding: 8px 16px 4px;
    font-size: 10px;
    font-weight: 600;
    color: #e4e6ef;
  }

  .phone-icons {
    font-size: 8px;
    color: #5c6070;
    letter-spacing: -1px;
  }

  .phone-header {
    padding: 8px 16px 12px;
  }

  .phone-title {
    font-size: 18px;
    font-weight: 700;
    color: #e4e6ef;
  }

  .phone-connections {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0 12px 16px;
  }

  .phone-card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    background: #161821;
    border: 1px solid #2a2d3a;
    border-radius: 10px;
    border-left: 3px solid var(--card-color, #4a9eff);
  }

  .phone-badge {
    font-size: 9px;
    font-weight: 700;
    padding: 3px 6px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .phone-card-info {
    flex: 1;
    min-width: 0;
  }

  .phone-card-name {
    display: block;
    font-size: 11px;
    font-weight: 600;
    color: #e4e6ef;
  }

  .phone-card-host {
    display: block;
    font-size: 9px;
    font-family: var(--font-mono);
    color: #5c6070;
    margin-top: 2px;
  }

  .phone-status {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #2a2d3a;
    flex-shrink: 0;
  }

  .phone-status.connected {
    background: #4ade80;
    box-shadow: 0 0 6px rgba(74, 222, 128, 0.5);
  }

  /* === Responsive === */
  @media (max-width: 768px) {
    .app-body {
      height: 180px;
    }

    .sidebar {
      width: 100px;
    }

    .editor-line {
      font-size: 7px;
    }

    .laptop {
      max-width: 100%;
    }

    .laptop-bezel {
      padding: 14px 10px 10px;
      border-radius: 10px 10px 0 0;
    }

    .laptop-camera {
      top: 5px;
      width: 4px;
      height: 4px;
    }

    .laptop-hinge {
      height: 5px;
    }

    .laptop-base {
      height: 40px;
    }

    .key {
      height: 5px;
    }

    .laptop-trackpad {
      height: 7px;
    }

    .conn-name {
      font-size: 8px;
    }

    .conn-badge {
      font-size: 7px;
      padding: 1px 3px;
    }

    .tree-item {
      font-size: 9px;
    }

    .tab {
      font-size: 8px;
    }

    .grid-header,
    .grid-row {
      font-size: 8px;
    }

    .status-bar {
      font-size: 7px;
    }

    .tablet {
      max-width: 100%;
    }

    .phone {
      max-width: 240px;
    }
  }
</style>
