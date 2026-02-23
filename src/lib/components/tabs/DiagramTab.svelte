<script lang="ts">
  import type { Tab } from '$lib/types/tabs';
  import type { DiagramTable, DiagramRelationship, DiagramColumn } from '$lib/types/diagram';
  import type { ColumnInfo, ForeignKeyInfo } from '$lib/types/schema';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import { computeGridLayout } from '$lib/utils/diagramLayout';
  import ERDiagramCanvas from '$lib/components/diagram/ERDiagramCanvas.svelte';
  import DiagramToolbar from '$lib/components/diagram/DiagramToolbar.svelte';
  import DiagramMinimap from '$lib/components/diagram/DiagramMinimap.svelte';

  let { tab }: { tab: Tab } = $props();

  let diagramTables = $state<DiagramTable[]>([]);
  let relationships = $state<DiagramRelationship[]>([]);
  let loading = $state(true);
  let progress = $state(0);
  let totalTables = $state(0);
  let loadedTables = $state(0);
  let selectedTable = $state<string | null>(null);

  let allSchemas = $state<string[]>([]);
  let enabledSchemas = $state<Set<string>>(new Set());

  let canvas = $state<ERDiagramCanvas | undefined>(undefined);

  // Provide a static viewBox for the minimap; the minimap gets updated from table positions
  let viewBox = $derived.by(() => {
    // Attempt to read from canvas if available; otherwise use defaults
    if (canvas) {
      return canvas.getViewBox();
    }
    return { x: 0, y: 0, width: 1200, height: 800 };
  });

  // Filter tables by enabled schemas
  let visibleTables = $derived(
    diagramTables.filter(t => enabledSchemas.has(t.schema))
  );
  let visibleRelationships = $derived(
    relationships.filter(r => {
      const src = diagramTables.find(t => t.id === r.sourceTable);
      const tgt = diagramTables.find(t => t.id === r.targetTable);
      return src && tgt && enabledSchemas.has(src.schema) && enabledSchemas.has(tgt.schema);
    })
  );

  $effect(() => {
    loadDiagram();
  });

  async function loadDiagram() {
    loading = true;
    progress = 0;
    loadedTables = 0;

    const connId = tab.connectionId;
    const targetSchemas = tab.diagramSchemas ?? [];

    // Load schemas if needed
    let schemas = schemaStore.getSchemas(connId);
    if (schemas.length === 0) {
      schemas = await schemaService.loadSchemas(connId);
    }

    const schemaNames = targetSchemas.length > 0
      ? targetSchemas
      : schemas.map(s => s.name);

    allSchemas = schemaNames;
    enabledSchemas = new Set(schemaNames);

    // Load all tables for each schema
    const allTableInfos: { schema: string; name: string }[] = [];
    for (const schema of schemaNames) {
      const tables = await schemaService.loadTables(connId, schema);
      for (const t of tables) {
        if (t.table_type.toUpperCase() !== 'VIEW' && !t.table_type.toUpperCase().includes('VIEW')) {
          allTableInfos.push({ schema, name: t.name });
        }
      }
    }

    totalTables = allTableInfos.length;

    // Batch-load columns and FKs with concurrency limit
    const CONCURRENCY = 10;
    const builtTables: DiagramTable[] = [];
    const allFKs: ForeignKeyInfo[] = [];

    for (let i = 0; i < allTableInfos.length; i += CONCURRENCY) {
      const batch = allTableInfos.slice(i, i + CONCURRENCY);
      const results = await Promise.all(
        batch.map(async ({ schema, name }) => {
          const [cols, fks] = await Promise.all([
            schemaService.loadColumns(connId, schema, name),
            schemaService.loadForeignKeys(connId, schema, name),
          ]);
          return { schema, name, cols, fks };
        })
      );

      for (const { schema, name, cols, fks } of results) {
        const fkColNames = new Set(fks.flatMap(f => f.columns));
        const columns: DiagramColumn[] = cols.map(c => ({
          name: c.name,
          dataType: c.data_type,
          isPK: c.is_primary_key,
          isFK: fkColNames.has(c.name),
          isNullable: c.is_nullable,
        }));
        builtTables.push({
          id: `${schema}.${name}`,
          schema,
          name,
          x: 0,
          y: 0,
          columns,
        });
        allFKs.push(...fks.map(f => ({ ...f, _sourceSchema: schema, _sourceTable: name } as ForeignKeyInfo & { _sourceSchema: string; _sourceTable: string })));
      }

      loadedTables += batch.length;
      progress = Math.round((loadedTables / totalTables) * 100);
    }

    // Build relationships from FKs
    const rels: DiagramRelationship[] = [];
    for (const fk of allFKs as (ForeignKeyInfo & { _sourceSchema?: string; _sourceTable?: string })[]) {
      const sourceId = `${fk._sourceSchema}.${fk._sourceTable}`;
      const targetId = `${fk.referenced_schema}.${fk.referenced_table}`;
      // Only include if both tables are in the diagram
      if (builtTables.some(t => t.id === sourceId) && builtTables.some(t => t.id === targetId)) {
        rels.push({
          id: `${sourceId}.${fk.name}`,
          sourceTable: sourceId,
          sourceColumns: fk.columns,
          targetTable: targetId,
          targetColumns: fk.referenced_columns,
          fkName: fk.name,
          onDelete: fk.on_delete,
          onUpdate: fk.on_update,
        });
      }
    }

    // Layout
    diagramTables = computeGridLayout(builtTables, rels);
    relationships = rels;
    loading = false;

    // Fit to screen after layout
    requestAnimationFrame(() => {
      canvas?.fitToScreen();
    });
  }

  function handleToggleSchema(schema: string) {
    const next = new Set(enabledSchemas);
    if (next.has(schema)) {
      if (next.size > 1) next.delete(schema);
    } else {
      next.add(schema);
    }
    enabledSchemas = next;
  }

  function handleExport() {
    const svgEl = canvas?.getSvgElement();
    if (!svgEl) return;
    const serializer = new XMLSerializer();
    const svgString = serializer.serializeToString(svgEl);
    const blob = new Blob([svgString], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `er-diagram-${tab.title.replace(/\s+/g, '-')}.svg`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleReset() {
    diagramTables = computeGridLayout([...diagramTables], relationships);
    requestAnimationFrame(() => canvas?.fitToScreen());
  }

  function handleNavigate(x: number, y: number) {
    canvas?.setViewBox(x, y);
  }
</script>

<div class="diagram-tab">
  <DiagramToolbar
    onzoomin={() => canvas?.zoomIn()}
    onzoomout={() => canvas?.zoomOut()}
    onfit={() => canvas?.fitToScreen()}
    onreset={handleReset}
    onexport={handleExport}
    schemas={allSchemas}
    {enabledSchemas}
    ontoggleschema={handleToggleSchema}
  />

  <div class="diagram-content">
    {#if loading}
      <div class="loading-overlay">
        <div class="loading-info">
          <div class="spinner"></div>
          <span>Loading tables... {loadedTables}/{totalTables}</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {progress}%"></div>
        </div>
      </div>
    {:else if visibleTables.length === 0}
      <div class="empty-state">
        <span>No tables found in the selected schema(s)</span>
      </div>
    {:else}
      <ERDiagramCanvas
        bind:this={canvas}
        tables={visibleTables}
        relationships={visibleRelationships}
        ontableselect={(id) => selectedTable = id}
      />
      <DiagramMinimap
        tables={visibleTables}
        {viewBox}
        onnavigate={handleNavigate}
      />
      {#if selectedTable}
        {@const t = diagramTables.find(t => t.id === selectedTable)}
        {#if t}
          <div class="selection-info">
            <strong>{t.schema}.{t.name}</strong>
            <span class="col-count">{t.columns.length} columns</span>
            <span class="rel-count">
              {relationships.filter(r => r.sourceTable === t.id || r.targetTable === t.id).length} relationships
            </span>
          </div>
        {/if}
      {/if}
    {/if}
  </div>
</div>

<style>
  .diagram-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .diagram-content {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .loading-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
  }

  .loading-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .progress-bar {
    width: 200px;
    height: 3px;
    background: var(--border-color);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 13px;
  }

  .selection-info {
    position: absolute;
    bottom: 12px;
    left: 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--text-primary);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .col-count, .rel-count {
    color: var(--text-muted);
    font-size: 11px;
  }
</style>
