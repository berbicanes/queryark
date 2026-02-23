<script lang="ts">
  import type { QueryResponse } from '$lib/types/query';
  import { extractCellValue } from '$lib/utils/formatters';
  import {
    parsePlanTree, analyzePlan, computeTimeline, getMaxCost, getTotalTime, getTotalCost,
    type PlanNode, type ProfilingHint, type TimelineEntry,
  } from '$lib/utils/planAnalyzer';
  import { generateColors, formatAxisLabel } from '$lib/utils/chartHelpers';

  let { planData, dialect }: {
    planData: QueryResponse;
    dialect: string;
  } = $props();

  type ViewMode = 'tree' | 'timeline' | 'hints' | 'raw';
  let viewMode = $state<ViewMode>('tree');

  let planTree = $derived.by(() => {
    try { return parsePlanTree(planData, dialect); }
    catch { return null; }
  });

  let rawText = $derived.by(() => {
    if (!planData.rows.length) return '';
    return planData.rows.map(row => row.map(c => extractCellValue(c)).join('\t')).join('\n');
  });

  let maxCost = $derived(planTree ? getMaxCost(planTree) : 1);
  let totalTime = $derived(planTree ? getTotalTime(planTree) : 0);
  let totalCost = $derived(planTree ? getTotalCost(planTree) : 0);
  let hints = $derived(planTree ? analyzePlan(planTree) : []);
  let timeline = $derived(planTree ? computeTimeline(planTree) : []);
  let maxTimelineDuration = $derived(
    timeline.length > 0
      ? Math.max(...timeline.map(e => e.start + e.duration), 1)
      : 1
  );

  let colors = $derived(generateColors(timeline.length));

  function costPercent(cost: number): number {
    return maxCost > 0 ? Math.round((cost / maxCost) * 100) : 0;
  }

  function isExpensive(cost: number): boolean {
    return costPercent(cost) > 60;
  }

  function severityIcon(severity: ProfilingHint['severity']): string {
    switch (severity) {
      case 'critical': return '!!';
      case 'warning': return '!';
      case 'info': return 'i';
    }
  }
</script>

<div class="profiler">
  <!-- Stats summary -->
  {#if planTree}
    <div class="stats-row">
      <div class="stat-item">
        <span class="stat-label">Total Cost</span>
        <span class="stat-value">{formatAxisLabel(totalCost)}</span>
      </div>
      {#if totalTime > 0}
        <div class="stat-item">
          <span class="stat-label">Execution Time</span>
          <span class="stat-value">{totalTime.toFixed(2)}ms</span>
        </div>
      {/if}
      {#if planTree.actualRows !== undefined}
        <div class="stat-item">
          <span class="stat-label">Rows Returned</span>
          <span class="stat-value">{planTree.actualRows.toLocaleString()}</span>
        </div>
      {/if}
      {#if hints.length > 0}
        <div class="stat-item hints-count">
          <span class="stat-label">Hints</span>
          <span class="stat-value">{hints.length}</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- View mode tabs -->
  <div class="profiler-toolbar">
    <button class="prof-tab" class:active={viewMode === 'tree'} onclick={() => viewMode = 'tree'}>Tree</button>
    <button class="prof-tab" class:active={viewMode === 'timeline'} onclick={() => viewMode = 'timeline'}>Timeline</button>
    <button class="prof-tab" class:active={viewMode === 'hints'} onclick={() => viewMode = 'hints'}>
      Hints
      {#if hints.length > 0}
        <span class="hint-badge">{hints.length}</span>
      {/if}
    </button>
    <button class="prof-tab" class:active={viewMode === 'raw'} onclick={() => viewMode = 'raw'}>Raw</button>
  </div>

  <div class="profiler-body">
    {#if viewMode === 'tree' && planTree}
      <div class="plan-tree">
        {#snippet nodeSnippet(node: PlanNode, depth: number)}
          <div class="plan-node" style="padding-left: {depth * 20 + 8}px;">
            <div class="node-header" class:expensive={isExpensive(node.cost)}>
              <span class="node-type">{node.type}</span>
              {#if node.table}
                <span class="node-relation">on {node.table}</span>
              {/if}
              {#if node.cost > 0}
                <span class="node-cost">cost: {node.cost.toFixed(2)}</span>
              {/if}
              {#if node.actualRows !== undefined}
                <span class="node-rows">rows: {node.actualRows}</span>
              {:else if node.rows > 0}
                <span class="node-rows">est: {node.rows}</span>
              {/if}
              {#if node.actualTime !== undefined}
                <span class="node-time">{node.actualTime.toFixed(3)}ms</span>
              {/if}
              {#if node.loops !== undefined && node.loops > 1}
                <span class="node-loops">x{node.loops}</span>
              {/if}
            </div>
            {#if node.cost > 0}
              <div class="cost-bar-wrapper" style="margin-left: {depth * 20 + 8}px;">
                <div class="cost-bar" class:expensive={isExpensive(node.cost)} style="width: {costPercent(node.cost)}%;"></div>
              </div>
            {/if}
            {#each node.children as child}
              {@render nodeSnippet(child, depth + 1)}
            {/each}
          </div>
        {/snippet}
        {@render nodeSnippet(planTree, 0)}
      </div>

    {:else if viewMode === 'timeline'}
      <div class="timeline-view">
        {#if timeline.length === 0}
          <div class="timeline-empty">
            <span class="text-muted">No timing data available. Use EXPLAIN ANALYZE for timing info.</span>
          </div>
        {:else}
          <div class="timeline-chart">
            {#each timeline as entry, i}
              <div class="timeline-row">
                <div class="timeline-label" title={entry.node}>
                  {entry.node.length > 30 ? entry.node.substring(0, 30) + '...' : entry.node}
                </div>
                <div class="timeline-bar-area">
                  <div
                    class="timeline-bar"
                    style="left: {(entry.start / maxTimelineDuration) * 100}%; width: {Math.max(1, (entry.duration / maxTimelineDuration) * 100)}%;"
                    style:background={colors[i]}
                    title="{entry.node}: {entry.duration.toFixed(2)}ms"
                  ></div>
                </div>
                <div class="timeline-duration">{entry.duration.toFixed(2)}ms</div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else if viewMode === 'hints'}
      <div class="hints-view">
        {#if hints.length === 0}
          <div class="hints-empty">
            <span class="text-muted">No optimization hints — the query plan looks good.</span>
          </div>
        {:else}
          {#each hints as hint}
            <div class="hint-card {hint.severity}">
              <div class="hint-header">
                <span class="severity-icon {hint.severity}">{severityIcon(hint.severity)}</span>
                <span class="hint-node">{hint.node}</span>
              </div>
              <div class="hint-message">{hint.message}</div>
              <div class="hint-suggestion">{hint.suggestion}</div>
            </div>
          {/each}
        {/if}
      </div>

    {:else if viewMode === 'raw'}
      <pre class="plan-raw">{rawText}</pre>

    {:else}
      <pre class="plan-raw">{rawText}</pre>
    {/if}
  </div>
</div>

<style>
  .profiler {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .stats-row {
    display: flex;
    gap: 16px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .stat-label {
    font-size: 9px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .stat-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .hints-count .stat-value {
    color: var(--warning, #fab387);
  }

  .profiler-toolbar {
    display: flex;
    gap: 2px;
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .prof-tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 10px;
    font-size: 11px;
    font-family: var(--font-sans);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .prof-tab.active {
    background: var(--bg-active);
    color: var(--accent);
    border-color: var(--accent);
  }

  .hint-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    font-size: 9px;
    font-weight: 700;
    color: var(--bg-primary);
    background: var(--warning, #fab387);
    border-radius: 7px;
  }

  .profiler-body {
    flex: 1;
    overflow: auto;
  }

  /* Tree view */
  .plan-tree {
    padding: 8px 0;
  }

  .plan-node { margin-bottom: 2px; }

  .node-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 8px;
    font-size: 12px;
    font-family: var(--font-mono);
    border-radius: var(--radius-sm);
  }

  .node-header:hover { background: var(--bg-hover); }
  .node-header.expensive { background: rgba(250, 179, 135, 0.08); }

  .node-type { font-weight: 600; color: var(--text-primary); }
  .node-relation { color: var(--accent); }
  .node-cost, .node-rows { color: var(--text-muted); font-size: 11px; }
  .node-time { color: var(--text-secondary); font-size: 11px; }
  .node-loops { color: var(--warning, #fab387); font-size: 10px; font-weight: 600; }

  .cost-bar-wrapper {
    height: 3px;
    margin-right: 8px;
    margin-bottom: 2px;
    background: rgba(69, 71, 90, 0.2);
    border-radius: 2px;
    overflow: hidden;
  }

  .cost-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s;
  }

  .cost-bar.expensive { background: var(--warning, #fab387); }

  /* Timeline view */
  .timeline-view {
    padding: 8px 12px;
  }

  .timeline-chart {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .timeline-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 26px;
  }

  .timeline-label {
    width: 200px;
    flex-shrink: 0;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .timeline-bar-area {
    flex: 1;
    height: 16px;
    background: rgba(69, 71, 90, 0.1);
    border-radius: var(--radius-sm);
    position: relative;
  }

  .timeline-bar {
    position: absolute;
    height: 100%;
    border-radius: var(--radius-sm);
    opacity: 0.8;
    min-width: 2px;
  }

  .timeline-duration {
    width: 70px;
    flex-shrink: 0;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    text-align: right;
  }

  .timeline-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    font-size: 13px;
  }

  /* Hints view */
  .hints-view {
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hints-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    font-size: 13px;
  }

  .hint-card {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border-left: 3px solid;
  }

  .hint-card.info {
    border-color: var(--accent);
    background: rgba(122, 162, 247, 0.04);
  }

  .hint-card.warning {
    border-color: var(--warning, #fab387);
    background: rgba(250, 179, 135, 0.04);
  }

  .hint-card.critical {
    border-color: var(--error);
    background: rgba(243, 139, 168, 0.04);
  }

  .hint-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .severity-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    font-size: 10px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .severity-icon.info { background: var(--accent); color: var(--bg-primary); }
  .severity-icon.warning { background: var(--warning, #fab387); color: var(--bg-primary); }
  .severity-icon.critical { background: var(--error); color: var(--bg-primary); }

  .hint-node {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .hint-message {
    font-size: 12px;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .hint-suggestion {
    font-size: 11px;
    color: var(--text-muted);
    font-style: italic;
  }

  /* Raw view */
  .plan-raw {
    flex: 1;
    overflow: auto;
    padding: 12px;
    margin: 0;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
