<script lang="ts">
  import { v4 as uuidv4 } from 'uuid';
  import type { VQTable, VQJoin, JoinType } from '$lib/types/visualQuery';

  let {
    tables = $bindable([]),
    joins = $bindable([]),
  }: {
    tables: VQTable[];
    joins: VQJoin[];
  } = $props();

  const TABLE_WIDTH = 200;
  const HEADER_HEIGHT = 26;
  const COL_ROW_HEIGHT = 22;

  let svgEl: SVGSVGElement;
  let vbX = $state(0);
  let vbY = $state(0);
  let vbW = $state(900);
  let vbH = $state(500);

  let isPanning = $state(false);
  let panStart = $state({ x: 0, y: 0, vbX: 0, vbY: 0 });
  let draggingTableId = $state<string | null>(null);
  let dragOffset = $state({ x: 0, y: 0 });

  // Join creation
  let joinSource = $state<{ tableId: string; column: string } | null>(null);

  // Join context menu
  let joinMenu = $state<{ x: number; y: number; joinId: string } | null>(null);

  function screenToSvg(clientX: number, clientY: number): { x: number; y: number } {
    const rect = svgEl.getBoundingClientRect();
    return {
      x: vbX + ((clientX - rect.left) / rect.width) * vbW,
      y: vbY + ((clientY - rect.top) / rect.height) * vbH,
    };
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as Element).closest('.vq-table-header')) return;
    if ((e.target as Element).closest('.vq-col-row')) return;
    isPanning = true;
    panStart = { x: e.clientX, y: e.clientY, vbX, vbY };
    joinMenu = null;
  }

  function handleMouseMove(e: MouseEvent) {
    if (isPanning) {
      const dx = e.clientX - panStart.x;
      const dy = e.clientY - panStart.y;
      const rect = svgEl.getBoundingClientRect();
      vbX = panStart.vbX - (dx / rect.width) * vbW;
      vbY = panStart.vbY - (dy / rect.height) * vbH;
    } else if (draggingTableId) {
      const pos = screenToSvg(e.clientX, e.clientY);
      const t = tables.find(t => t.id === draggingTableId);
      if (t) {
        t.x = pos.x - dragOffset.x;
        t.y = pos.y - dragOffset.y;
        tables = [...tables];
      }
    }
  }

  function handleMouseUp() {
    isPanning = false;
    draggingTableId = null;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 1.1 : 0.9;
    vbW = Math.max(200, Math.min(5000, vbW * factor));
    vbH = Math.max(150, Math.min(3750, vbH * factor));
  }

  function handleTableMouseDown(e: MouseEvent, tableId: string) {
    e.stopPropagation();
    const pos = screenToSvg(e.clientX, e.clientY);
    const t = tables.find(t => t.id === tableId);
    if (t) {
      draggingTableId = tableId;
      dragOffset = { x: pos.x - t.x, y: pos.y - t.y };
    }
  }

  function handleColumnClick(tableId: string, columnName: string) {
    if (!joinSource) {
      joinSource = { tableId, column: columnName };
    } else {
      if (joinSource.tableId !== tableId) {
        // Create join
        joins = [...joins, {
          id: uuidv4(),
          sourceTableId: joinSource.tableId,
          sourceColumn: joinSource.column,
          targetTableId: tableId,
          targetColumn: columnName,
          joinType: 'INNER JOIN' as JoinType,
        }];
      }
      joinSource = null;
    }
  }

  function handleColumnCheckbox(tableId: string, colName: string, checked: boolean) {
    const t = tables.find(t => t.id === tableId);
    if (!t) return;
    if (checked) {
      t.selectedColumns = [...t.selectedColumns, colName];
    } else {
      t.selectedColumns = t.selectedColumns.filter(c => c !== colName);
    }
    tables = [...tables];
  }

  function handleJoinContextMenu(e: MouseEvent, joinId: string) {
    e.preventDefault();
    joinMenu = { x: e.clientX, y: e.clientY, joinId };
  }

  function setJoinType(type: JoinType) {
    if (!joinMenu) return;
    joins = joins.map(j => j.id === joinMenu!.joinId ? { ...j, joinType: type } : j);
    joinMenu = null;
  }

  function deleteJoin() {
    if (!joinMenu) return;
    joins = joins.filter(j => j.id !== joinMenu!.joinId);
    joinMenu = null;
  }

  function getJoinPath(join: VQJoin): string {
    const srcTable = tables.find(t => t.id === join.sourceTableId);
    const tgtTable = tables.find(t => t.id === join.targetTableId);
    if (!srcTable || !tgtTable) return '';
    const srcIdx = srcTable.columns.findIndex(c => c.name === join.sourceColumn);
    const tgtIdx = tgtTable.columns.findIndex(c => c.name === join.targetColumn);
    const sy = srcTable.y + HEADER_HEIGHT + Math.max(0, srcIdx) * COL_ROW_HEIGHT + COL_ROW_HEIGHT / 2;
    const ty = tgtTable.y + HEADER_HEIGHT + Math.max(0, tgtIdx) * COL_ROW_HEIGHT + COL_ROW_HEIGHT / 2;
    const sx = srcTable.x + TABLE_WIDTH;
    const tx = tgtTable.x;
    const midX = (sx + tx) / 2;
    return `M ${sx} ${sy} C ${midX} ${sy}, ${midX} ${ty}, ${tx} ${ty}`;
  }

  function getJoinLabel(join: VQJoin): string {
    return join.joinType.replace(' JOIN', '');
  }

  function getJoinLabelPos(join: VQJoin): { x: number; y: number } {
    const srcTable = tables.find(t => t.id === join.sourceTableId);
    const tgtTable = tables.find(t => t.id === join.targetTableId);
    if (!srcTable || !tgtTable) return { x: 0, y: 0 };
    return {
      x: (srcTable.x + TABLE_WIDTH + tgtTable.x) / 2,
      y: (srcTable.y + tgtTable.y) / 2 + HEADER_HEIGHT,
    };
  }
</script>

<svelte:window onclick={() => { joinMenu = null; }} />

<svg
  bind:this={svgEl}
  class="vq-canvas"
  viewBox="{vbX} {vbY} {vbW} {vbH}"
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseUp}
  onwheel={handleWheel}
>
  <!-- Join lines -->
  {#each joins as join}
    {@const path = getJoinPath(join)}
    {@const labelPos = getJoinLabelPos(join)}
    {#if path}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <path
        d={path}
        fill="none"
        stroke="var(--accent)"
        stroke-width="2"
        style="cursor: context-menu"
        oncontextmenu={(e) => handleJoinContextMenu(e, join.id)}
      />
      <text
        x={labelPos.x} y={labelPos.y}
        font-size="9"
        fill="var(--accent)"
        text-anchor="middle"
        style="pointer-events: none"
      >
        {getJoinLabel(join)}
      </text>
    {/if}
  {/each}

  <!-- Tables -->
  {#each tables as table}
    {@const tableH = HEADER_HEIGHT + table.columns.length * COL_ROW_HEIGHT}
    <g transform="translate({table.x}, {table.y})">
      <!-- Background -->
      <rect width={TABLE_WIDTH} height={tableH} rx="4" fill="var(--bg-secondary)" stroke="var(--border-color)" stroke-width="1" />
      <!-- Header -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect
        class="vq-table-header"
        width={TABLE_WIDTH} height={HEADER_HEIGHT} rx="4"
        fill="var(--bg-tertiary, var(--bg-hover))"
        onmousedown={(e) => handleTableMouseDown(e, table.id)}
        style="cursor: grab"
      />
      <rect x="0" y={HEADER_HEIGHT - 4} width={TABLE_WIDTH} height="4" fill="var(--bg-tertiary, var(--bg-hover))" />
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <text
        class="vq-table-header"
        x="8" y={HEADER_HEIGHT / 2 + 4}
        font-size="11" font-weight="600" fill="var(--text-primary)"
        onmousedown={(e) => handleTableMouseDown(e, table.id)}
        style="cursor: grab; pointer-events: all"
      >
        {table.alias || table.name}
      </text>

      <!-- Columns -->
      {#each table.columns as col, i}
        {@const cy = HEADER_HEIGHT + i * COL_ROW_HEIGHT}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <g class="vq-col-row" style="cursor: pointer" onclick={() => handleColumnClick(table.id, col.name)}>
          <rect x="0" y={cy} width={TABLE_WIDTH} height={COL_ROW_HEIGHT} fill="transparent" />
          {#if i > 0}
            <line x1="0" y1={cy} x2={TABLE_WIDTH} y2={cy} stroke="var(--border-color)" stroke-width="0.5" opacity="0.3" />
          {/if}
          <!-- Checkbox for SELECT -->
          <foreignObject x="4" y={cy + 3} width="16" height="16">
            <input
              type="checkbox"
              checked={table.selectedColumns.includes(col.name)}
              onchange={(e) => handleColumnCheckbox(table.id, col.name, (e.target as HTMLInputElement).checked)}
              onclick={(e) => e.stopPropagation()}
              style="width: 13px; height: 13px; margin: 0; accent-color: var(--accent)"
            />
          </foreignObject>
          <!-- Column name -->
          <text x="24" y={cy + COL_ROW_HEIGHT / 2 + 3} font-size="10" fill="var(--text-secondary)">
            {col.name}
          </text>
          <!-- Type indicator -->
          <text x={TABLE_WIDTH - 6} y={cy + COL_ROW_HEIGHT / 2 + 3} font-size="9" fill="var(--text-muted)" text-anchor="end">
            {#if col.isPK}PK{:else if col.isFK}FK{/if}
          </text>
        </g>
      {/each}
    </g>
  {/each}

  <!-- Join source indicator -->
  {#if joinSource}
    {@const srcTable = tables.find(t => t.id === joinSource?.tableId)}
    {#if srcTable}
      {@const srcIdx = srcTable.columns.findIndex(c => c.name === joinSource?.column)}
      {@const sy = srcTable.y + HEADER_HEIGHT + Math.max(0, srcIdx) * COL_ROW_HEIGHT + COL_ROW_HEIGHT / 2}
      <circle cx={srcTable.x + TABLE_WIDTH} cy={sy} r="4" fill="var(--accent)" />
    {/if}
  {/if}
</svg>

<!-- Join context menu -->
{#if joinMenu}
  <div
    class="join-context-menu"
    style="left: {joinMenu.x}px; top: {joinMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') joinMenu = null; }}
    role="menu"
    tabindex="-1"
  >
    <button class="ctx-item" onclick={() => setJoinType('INNER JOIN')}>INNER JOIN</button>
    <button class="ctx-item" onclick={() => setJoinType('LEFT JOIN')}>LEFT JOIN</button>
    <button class="ctx-item" onclick={() => setJoinType('RIGHT JOIN')}>RIGHT JOIN</button>
    <button class="ctx-item" onclick={() => setJoinType('FULL JOIN')}>FULL JOIN</button>
    <button class="ctx-item" onclick={() => setJoinType('CROSS JOIN')}>CROSS JOIN</button>
    <div class="ctx-divider"></div>
    <button class="ctx-item danger" onclick={deleteJoin}>Delete Join</button>
  </div>
{/if}

<style>
  .vq-canvas {
    width: 100%;
    height: 100%;
    background: var(--bg-primary);
    cursor: grab;
    user-select: none;
  }

  .vq-canvas:active { cursor: grabbing; }

  .vq-canvas text {
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
    user-select: none;
  }

  .join-context-menu {
    position: fixed;
    z-index: 500;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 4px 0;
    min-width: 140px;
  }

  .ctx-item {
    display: block;
    width: 100%;
    padding: 5px 12px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .ctx-item:hover { background: var(--bg-hover); }
  .ctx-item.danger { color: var(--error, #f38ba8); }
  .ctx-item.danger:hover { background: rgba(243, 139, 168, 0.1); }

  .ctx-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }
</style>
