<script lang="ts">
  import type { DiagramTable, DiagramRelationship } from '$lib/types/diagram';
  import { TABLE_WIDTH, HEADER_HEIGHT, COL_ROW_HEIGHT, computeRelationshipPath, computeBoundingBox } from '$lib/utils/diagramLayout';

  let {
    tables = [],
    relationships = [],
    ontableselect,
  }: {
    tables: DiagramTable[];
    relationships: DiagramRelationship[];
    ontableselect?: (tableId: string | null) => void;
  } = $props();

  let svgEl: SVGSVGElement;

  // Viewbox state
  let vbX = $state(0);
  let vbY = $state(0);
  let vbW = $state(1200);
  let vbH = $state(800);

  // Interaction state
  let isPanning = $state(false);
  let panStart = $state({ x: 0, y: 0, vbX: 0, vbY: 0 });
  let draggingTableId = $state<string | null>(null);
  let dragOffset = $state({ x: 0, y: 0 });
  let selectedTableId = $state<string | null>(null);

  export function getViewBox() {
    return { x: vbX, y: vbY, width: vbW, height: vbH };
  }

  export function setViewBox(x: number, y: number) {
    vbX = x;
    vbY = y;
  }

  export function zoomIn() {
    const cx = vbX + vbW / 2;
    const cy = vbY + vbH / 2;
    vbW *= 0.8;
    vbH *= 0.8;
    vbX = cx - vbW / 2;
    vbY = cy - vbH / 2;
  }

  export function zoomOut() {
    const cx = vbX + vbW / 2;
    const cy = vbY + vbH / 2;
    vbW *= 1.25;
    vbH *= 1.25;
    vbX = cx - vbW / 2;
    vbY = cy - vbH / 2;
  }

  export function fitToScreen() {
    const bb = computeBoundingBox(tables);
    vbX = bb.x;
    vbY = bb.y;
    vbW = bb.width;
    vbH = bb.height;
  }

  export function getSvgElement(): SVGSVGElement {
    return svgEl;
  }

  function screenToSvg(clientX: number, clientY: number): { x: number; y: number } {
    const rect = svgEl.getBoundingClientRect();
    return {
      x: vbX + ((clientX - rect.left) / rect.width) * vbW,
      y: vbY + ((clientY - rect.top) / rect.height) * vbH,
    };
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    // Check if clicking on a table header (handled by table mousedown)
    if ((e.target as Element).closest('.table-header')) return;
    isPanning = true;
    panStart = { x: e.clientX, y: e.clientY, vbX, vbY };
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
        tables = [...tables]; // trigger reactivity
      }
    }
  }

  function handleMouseUp() {
    isPanning = false;
    draggingTableId = null;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const pos = screenToSvg(e.clientX, e.clientY);
    const factor = e.deltaY > 0 ? 1.1 : 0.9;
    const newW = Math.max(200, Math.min(10000, vbW * factor));
    const newH = Math.max(150, Math.min(7500, vbH * factor));
    vbX = pos.x - ((pos.x - vbX) / vbW) * newW;
    vbY = pos.y - ((pos.y - vbY) / vbH) * newH;
    vbW = newW;
    vbH = newH;
  }

  function handleTableMouseDown(e: MouseEvent, tableId: string) {
    e.stopPropagation();
    const pos = screenToSvg(e.clientX, e.clientY);
    const t = tables.find(t => t.id === tableId);
    if (t) {
      draggingTableId = tableId;
      dragOffset = { x: pos.x - t.x, y: pos.y - t.y };
    }
    selectedTableId = tableId;
    ontableselect?.(tableId);
  }

  function handleCanvasClick(e: MouseEvent) {
    if (!(e.target as Element).closest('.table-group')) {
      selectedTableId = null;
      ontableselect?.(null);
    }
  }

  function getRelPath(rel: DiagramRelationship): string {
    const srcTable = tables.find(t => t.id === rel.sourceTable);
    const tgtTable = tables.find(t => t.id === rel.targetTable);
    if (!srcTable || !tgtTable) return '';
    const srcColIdx = srcTable.columns.findIndex(c => rel.sourceColumns.includes(c.name));
    const tgtColIdx = tgtTable.columns.findIndex(c => rel.targetColumns.includes(c.name));
    return computeRelationshipPath(srcTable, Math.max(0, srcColIdx), tgtTable, Math.max(0, tgtColIdx));
  }

  function isRelHighlighted(rel: DiagramRelationship): boolean {
    if (!selectedTableId) return false;
    return rel.sourceTable === selectedTableId || rel.targetTable === selectedTableId;
  }

  function isTableSelected(tableId: string): boolean {
    return tableId === selectedTableId;
  }

  function isTableConnected(tableId: string): boolean {
    if (!selectedTableId) return false;
    return relationships.some(
      r => (r.sourceTable === selectedTableId && r.targetTable === tableId) ||
           (r.targetTable === selectedTableId && r.sourceTable === tableId)
    );
  }

  function tableOpacity(tableId: string): number {
    if (!selectedTableId) return 1;
    if (isTableSelected(tableId) || isTableConnected(tableId)) return 1;
    return 0.3;
  }
</script>

<svg
  bind:this={svgEl}
  class="er-canvas"
  viewBox="{vbX} {vbY} {vbW} {vbH}"
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseUp}
  onwheel={handleWheel}
  onclick={handleCanvasClick}
>
  <defs>
    <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
      <path d="M0,0 L8,3 L0,6" fill="var(--text-muted)" />
    </marker>
    <marker id="arrowhead-accent" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
      <path d="M0,0 L8,3 L0,6" fill="var(--accent)" />
    </marker>
  </defs>

  <!-- Relationship lines -->
  {#each relationships as rel}
    {@const path = getRelPath(rel)}
    {@const highlighted = isRelHighlighted(rel)}
    {#if path}
      <path
        d={path}
        fill="none"
        stroke={highlighted ? 'var(--accent)' : 'var(--text-muted)'}
        stroke-width={highlighted ? 2 : 1}
        stroke-dasharray={highlighted ? '' : '4,3'}
        marker-end={highlighted ? 'url(#arrowhead-accent)' : 'url(#arrowhead)'}
        opacity={selectedTableId && !highlighted ? 0.15 : 0.7}
      />
    {/if}
  {/each}

  <!-- Table groups -->
  {#each tables as table}
    {@const tableH = HEADER_HEIGHT + table.columns.length * COL_ROW_HEIGHT}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <g
      class="table-group"
      opacity={tableOpacity(table.id)}
      transform="translate({table.x}, {table.y})"
    >
      <!-- Shadow -->
      <rect
        x="2" y="2"
        width={TABLE_WIDTH} height={tableH}
        rx="4"
        fill="rgba(0,0,0,0.2)"
      />
      <!-- Background -->
      <rect
        width={TABLE_WIDTH} height={tableH}
        rx="4"
        fill="var(--bg-secondary)"
        stroke={isTableSelected(table.id) ? 'var(--accent)' : 'var(--border-color)'}
        stroke-width={isTableSelected(table.id) ? 2 : 1}
      />
      <!-- Header -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect
        class="table-header"
        width={TABLE_WIDTH} height={HEADER_HEIGHT}
        rx="4"
        fill={isTableSelected(table.id) ? 'var(--accent)' : 'var(--bg-tertiary, var(--bg-hover))'}
        onmousedown={(e) => handleTableMouseDown(e, table.id)}
        style="cursor: grab"
      />
      <!-- Header bottom corners (square off the rounded bottom) -->
      <rect
        x="0" y={HEADER_HEIGHT - 4}
        width={TABLE_WIDTH} height="4"
        fill={isTableSelected(table.id) ? 'var(--accent)' : 'var(--bg-tertiary, var(--bg-hover))'}
      />
      <!-- Table name -->
      <text
        x="10" y={HEADER_HEIGHT / 2 + 4}
        font-size="12"
        font-weight="600"
        fill={isTableSelected(table.id) ? 'var(--bg-primary)' : 'var(--text-primary)'}
        class="table-header"
        onmousedown={(e) => handleTableMouseDown(e, table.id)}
        style="cursor: grab; pointer-events: all"
      >
        {table.name}
      </text>

      <!-- Columns -->
      {#each table.columns as col, i}
        {@const cy = HEADER_HEIGHT + i * COL_ROW_HEIGHT}
        <rect
          x="0" y={cy}
          width={TABLE_WIDTH} height={COL_ROW_HEIGHT}
          fill="transparent"
        />
        {#if i > 0}
          <line
            x1="0" y1={cy}
            x2={TABLE_WIDTH} y2={cy}
            stroke="var(--border-color)" stroke-width="0.5" opacity="0.3"
          />
        {/if}
        <!-- PK/FK indicator -->
        {#if col.isPK}
          <text x="8" y={cy + COL_ROW_HEIGHT / 2 + 3} font-size="9" fill="var(--warning, #e0af68)">PK</text>
        {:else if col.isFK}
          <text x="8" y={cy + COL_ROW_HEIGHT / 2 + 3} font-size="9" fill="var(--accent)">FK</text>
        {/if}
        <!-- Column name -->
        <text
          x="30" y={cy + COL_ROW_HEIGHT / 2 + 3}
          font-size="11"
          fill={col.isNullable ? 'var(--text-secondary)' : 'var(--text-primary)'}
        >
          {col.name}
        </text>
        <!-- Data type -->
        <text
          x={TABLE_WIDTH - 8} y={cy + COL_ROW_HEIGHT / 2 + 3}
          font-size="10"
          fill="var(--text-muted)"
          text-anchor="end"
        >
          {col.dataType}
        </text>
      {/each}
    </g>
  {/each}
</svg>

<style>
  .er-canvas {
    width: 100%;
    height: 100%;
    background: var(--bg-primary);
    cursor: grab;
    user-select: none;
  }

  .er-canvas:active {
    cursor: grabbing;
  }

  .table-group {
    transition: opacity 0.2s ease;
  }

  .er-canvas text {
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
    user-select: none;
  }
</style>
