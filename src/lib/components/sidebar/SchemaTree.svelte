<script lang="ts">
  import { schemaStore } from '$lib/stores/schema.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import { DB_METADATA } from '$lib/types/database';
  import type { SchemaInfo, TableInfo, ColumnInfo, ContainerInfo, ItemInfo, FieldInfo, TableStats, RoutineInfo, SequenceInfo, EnumInfo } from '$lib/types/schema';
  import type { DatabaseCategory, DatabaseType } from '$lib/types/connection';
  import type { TabType } from '$lib/types/tabs';
  import TreeNode from './TreeNode.svelte';

  let { connectionId }: { connectionId: string } = $props();

  // Determine the database category for this connection
  let dbCategory = $derived.by(() => {
    const conn = connectionStore.connections.find(c => c.config.id === connectionId);
    if (!conn) return 'Relational' as DatabaseCategory;
    return DB_METADATA[conn.config.db_type].category;
  });

  let dbType = $derived.by(() => {
    const conn = connectionStore.connections.find(c => c.config.id === connectionId);
    return conn?.config.db_type ?? 'PostgreSQL';
  });

  let isSqlLike = $derived(dbCategory === 'Relational' || dbCategory === 'Analytics' || dbCategory === 'WideColumn');
  let meta = $derived(DB_METADATA[dbType]);

  // Feature flags based on database type
  let supportsRoutines = $derived(
    ['PostgreSQL', 'MySQL', 'MariaDB', 'MSSQL', 'CockroachDB'].includes(dbType)
  );
  let supportsSequences = $derived(
    ['PostgreSQL', 'CockroachDB'].includes(dbType)
  );
  let supportsEnums = $derived(
    ['PostgreSQL', 'CockroachDB'].includes(dbType)
  );

  // SQL-specific data
  let schemas = $derived(schemaStore.getSchemas(connectionId));
  // Generic data
  let containers = $derived(schemaStore.getContainers(connectionId));

  // Search
  let searchQuery = $state('');

  function isView(tableType: string): boolean {
    const t = tableType.toUpperCase();
    return t === 'VIEW' || t.includes('VIEW');
  }

  // Track expanded state
  let expandedSchemas = $state<Set<string>>(new Set());
  let expandedCategories = $state<Set<string>>(new Set());
  let expandedTables = $state<Set<string>>(new Set());
  let expandedContainers = $state<Set<string>>(new Set());
  let expandedItems = $state<Set<string>>(new Set());

  // SQL-specific handlers
  async function handleSchemaExpand(schema: SchemaInfo, expanded: boolean) {
    if (expanded) {
      expandedSchemas.add(schema.name);
      const tables = schemaStore.getTables(connectionId, schema.name);
      if (tables.length === 0) {
        await schemaService.loadTables(connectionId, schema.name);
      }
    } else {
      expandedSchemas.delete(schema.name);
    }
  }

  async function handleCategoryExpand(schemaName: string, category: string, expanded: boolean) {
    const key = `${schemaName}.${category}`;
    if (expanded) {
      expandedCategories.add(key);
      if (category === 'routines') {
        const cached = schemaStore.getRoutines(connectionId, schemaName);
        if (cached.length === 0) {
          await schemaService.loadRoutines(connectionId, schemaName);
        }
      } else if (category === 'sequences') {
        const cached = schemaStore.getSequences(connectionId, schemaName);
        if (cached.length === 0) {
          await schemaService.loadSequences(connectionId, schemaName);
        }
      } else if (category === 'enums') {
        const cached = schemaStore.getEnums(connectionId, schemaName);
        if (cached.length === 0) {
          await schemaService.loadEnums(connectionId, schemaName);
        }
      }
    } else {
      expandedCategories.delete(key);
    }
  }

  async function handleTableExpand(schema: string, table: string, expanded: boolean) {
    const key = `${schema}.${table}`;
    if (expanded) {
      expandedTables.add(key);
      const cols = schemaStore.getColumns(connectionId, schema, table);
      if (cols.length === 0) {
        await schemaService.loadColumns(connectionId, schema, table);
      }
      // Lazy-load stats
      if (!schemaStore.getTableStats(connectionId, schema, table)) {
        schemaService.loadTableStats(connectionId, schema, table);
      }
    } else {
      expandedTables.delete(key);
    }
  }

  function handleTableDblClick(schema: string, table: string) {
    tabStore.openTab({
      type: 'table',
      title: table,
      connectionId,
      schema,
      table
    });
  }

  // Generic handlers
  async function handleContainerExpand(container: ContainerInfo, expanded: boolean) {
    if (expanded) {
      expandedContainers.add(container.name);
      const items = schemaStore.getItems(connectionId, container.name);
      if (items.length === 0) {
        await schemaService.loadItems(connectionId, container.name);
      }
    } else {
      expandedContainers.delete(container.name);
    }
  }

  async function handleItemExpand(container: string, item: string, expanded: boolean) {
    const key = `${container}.${item}`;
    if (expanded) {
      expandedItems.add(key);
      const fields = schemaStore.getFields(connectionId, container, item);
      if (fields.length === 0) {
        await schemaService.loadFields(connectionId, container, item);
      }
    } else {
      expandedItems.delete(key);
    }
  }

  function getTabTypeForCategory(category: DatabaseCategory): TabType {
    switch (category) {
      case 'Document': return 'document';
      case 'KeyValue': return 'keyvalue';
      case 'Graph': return 'graph';
      default: return 'table';
    }
  }

  function handleItemDblClick(container: string, item: string) {
    const tabType = getTabTypeForCategory(dbCategory);
    tabStore.openTab({
      type: tabType,
      title: item,
      connectionId,
      container,
      item
    });
  }

  // Data getters (reactive through store)
  function getTables(schemaName: string): TableInfo[] {
    return schemaStore.getTables(connectionId, schemaName);
  }

  function getFilteredTables(schemaName: string): TableInfo[] {
    const tables = getTables(schemaName).filter(t => !isView(t.table_type));
    if (!searchQuery) return tables;
    const q = searchQuery.toLowerCase();
    return tables.filter(t => t.name.toLowerCase().includes(q));
  }

  function getFilteredViews(schemaName: string): TableInfo[] {
    const views = getTables(schemaName).filter(t => isView(t.table_type));
    if (!searchQuery) return views;
    const q = searchQuery.toLowerCase();
    return views.filter(t => t.name.toLowerCase().includes(q));
  }

  function getColumns(schemaName: string, tableName: string): ColumnInfo[] {
    return schemaStore.getColumns(connectionId, schemaName, tableName);
  }

  function getRoutines(schemaName: string): RoutineInfo[] {
    const routines = schemaStore.getRoutines(connectionId, schemaName);
    if (!searchQuery) return routines;
    const q = searchQuery.toLowerCase();
    return routines.filter(r => r.name.toLowerCase().includes(q));
  }

  function getSequences(schemaName: string): SequenceInfo[] {
    const sequences = schemaStore.getSequences(connectionId, schemaName);
    if (!searchQuery) return sequences;
    const q = searchQuery.toLowerCase();
    return sequences.filter(s => s.name.toLowerCase().includes(q));
  }

  function getEnums(schemaName: string): EnumInfo[] {
    const enums = schemaStore.getEnums(connectionId, schemaName);
    if (!searchQuery) return enums;
    const q = searchQuery.toLowerCase();
    return enums.filter(e => e.name.toLowerCase().includes(q));
  }

  function getItems(containerName: string): ItemInfo[] {
    const items = schemaStore.getItems(connectionId, containerName);
    if (!searchQuery) return items;
    const q = searchQuery.toLowerCase();
    return items.filter(i => i.name.toLowerCase().includes(q));
  }

  function getFields(containerName: string, itemName: string): FieldInfo[] {
    return schemaStore.getFields(connectionId, containerName, itemName);
  }

  function getTableTooltip(schema: string, table: string): string {
    const stats = schemaStore.getTableStats(connectionId, schema, table);
    if (!stats) return '';
    const parts: string[] = [];
    parts.push(`Rows: ${formatNumber(stats.row_count)}`);
    if (stats.size_display) parts.push(`Size: ${stats.size_display}`);
    return parts.join(' | ');
  }

  function getTableSuffix(schema: string, table: string): string {
    const stats = schemaStore.getTableStats(connectionId, schema, table);
    if (!stats) return '';
    return formatNumber(stats.row_count);
  }

  function formatNumber(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return n.toString();
  }

  function hasSearchMatch(schemaName: string): boolean {
    if (!searchQuery) return true;
    return (
      getFilteredTables(schemaName).length > 0 ||
      getFilteredViews(schemaName).length > 0 ||
      getRoutines(schemaName).length > 0 ||
      getSequences(schemaName).length > 0 ||
      getEnums(schemaName).length > 0
    );
  }

  const ICON_FOLDER = '\u{1F4C1}';
  const ICON_TABLE = '\u{1F5C3}';
  const ICON_KEY = '\u{1F511}';
  const ICON_VIEW = '\u{1F441}';
  const ICON_FUNCTION = '\u{2A}';
  const ICON_SEQUENCE = '#';
  const ICON_ENUM = '\u{2261}';

  function getColumnTypeIcon(dataType: string): string {
    const t = dataType.toLowerCase();
    if (t.includes('int') || t.includes('serial') || t.includes('numeric') || t.includes('decimal') || t.includes('float') || t.includes('double') || t.includes('real')) return '#';
    if (t.includes('bool')) return '?';
    if (t.includes('date') || t.includes('time') || t.includes('timestamp')) return '\u{1F552}';
    if (t.includes('json') || t.includes('jsonb')) return '{}';
    if (t.includes('bytea') || t.includes('blob') || t.includes('binary')) return '\u{1F4BE}';
    return 'T';
  }

  function getFieldIcon(field: FieldInfo): string {
    if (field.is_primary) return ICON_KEY;
    return getColumnTypeIcon(field.data_type);
  }
</script>

<div class="schema-tree">
  {#if isSqlLike || containers.length > 0 || schemas.length > 0}
    <div class="search-bar">
      <input
        type="text"
        class="search-input"
        placeholder="Filter..."
        bind:value={searchQuery}
      />
      {#if searchQuery}
        <button class="search-clear" onclick={() => searchQuery = ''} aria-label="Clear search">
          <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      {/if}
    </div>
  {/if}

  {#if isSqlLike}
    <!-- SQL-like tree: Schemas > (Tables/Views/Functions/Sequences/Types) > Columns -->
    {#if schemas.length === 0}
      <div class="empty-schemas">
        <span class="text-muted">No {meta.containerLabel.toLowerCase()}s loaded</span>
      </div>
    {:else}
      {#each schemas as schema}
        {#if hasSearchMatch(schema.name) || !searchQuery}
          <TreeNode
            label={schema.name}
            icon={ICON_FOLDER}
            expandable={true}
            depth={0}
            onexpand={(exp) => handleSchemaExpand(schema, exp)}
          >
            {#snippet children()}
              <!-- Tables category -->
              {@const filteredTables = getFilteredTables(schema.name)}
              {#if filteredTables.length > 0 || !searchQuery}
                <TreeNode
                  label="Tables"
                  icon={ICON_TABLE}
                  expandable={true}
                  depth={1}
                  suffix={filteredTables.length > 0 ? `(${filteredTables.length})` : ''}
                  onexpand={(exp) => handleCategoryExpand(schema.name, 'tables', exp)}
                >
                  {#snippet children()}
                    {#each filteredTables as table}
                      <TreeNode
                        label={table.name}
                        icon={ICON_TABLE}
                        expandable={true}
                        depth={2}
                        tooltip={getTableTooltip(schema.name, table.name)}
                        suffix={getTableSuffix(schema.name, table.name)}
                        onexpand={(exp) => handleTableExpand(schema.name, table.name, exp)}
                        ondblclick={() => handleTableDblClick(schema.name, table.name)}
                      >
                        {#snippet children()}
                          {#each getColumns(schema.name, table.name) as column}
                            <TreeNode
                              label={column.name}
                              icon={column.is_primary_key ? ICON_KEY : getColumnTypeIcon(column.data_type)}
                              expandable={false}
                              depth={3}
                              tooltip={column.data_type}
                            >
                              {#snippet children()}
                                <!-- leaf node -->
                              {/snippet}
                            </TreeNode>
                          {/each}
                        {/snippet}
                      </TreeNode>
                    {/each}
                  {/snippet}
                </TreeNode>
              {/if}

              <!-- Views category -->
              {@const filteredViews = getFilteredViews(schema.name)}
              {#if filteredViews.length > 0}
                <TreeNode
                  label="Views"
                  icon={ICON_VIEW}
                  expandable={true}
                  depth={1}
                  suffix={`(${filteredViews.length})`}
                  onexpand={(exp) => handleCategoryExpand(schema.name, 'views', exp)}
                >
                  {#snippet children()}
                    {#each filteredViews as view}
                      <TreeNode
                        label={view.name}
                        icon={ICON_VIEW}
                        expandable={true}
                        depth={2}
                        onexpand={(exp) => handleTableExpand(schema.name, view.name, exp)}
                        ondblclick={() => handleTableDblClick(schema.name, view.name)}
                      >
                        {#snippet children()}
                          {#each getColumns(schema.name, view.name) as column}
                            <TreeNode
                              label={column.name}
                              icon={column.is_primary_key ? ICON_KEY : getColumnTypeIcon(column.data_type)}
                              expandable={false}
                              depth={3}
                              tooltip={column.data_type}
                            >
                              {#snippet children()}
                                <!-- leaf node -->
                              {/snippet}
                            </TreeNode>
                          {/each}
                        {/snippet}
                      </TreeNode>
                    {/each}
                  {/snippet}
                </TreeNode>
              {/if}

              <!-- Functions category -->
              {#if supportsRoutines}
                {@const routines = getRoutines(schema.name)}
                {#if routines.length > 0 || expandedCategories.has(`${schema.name}.routines`)}
                  <TreeNode
                    label="Functions"
                    icon={ICON_FUNCTION}
                    expandable={true}
                    depth={1}
                    suffix={routines.length > 0 ? `(${routines.length})` : ''}
                    onexpand={(exp) => handleCategoryExpand(schema.name, 'routines', exp)}
                  >
                    {#snippet children()}
                      {#each routines as routine}
                        <TreeNode
                          label={routine.name}
                          icon={routine.routine_type === 'PROCEDURE' ? 'P' : 'f'}
                          expandable={false}
                          depth={2}
                          tooltip={`${routine.routine_type}${routine.return_type ? ' → ' + routine.return_type : ''}`}
                        >
                          {#snippet children()}
                            <!-- leaf node -->
                          {/snippet}
                        </TreeNode>
                      {/each}
                    {/snippet}
                  </TreeNode>
                {/if}
              {/if}

              <!-- Sequences category -->
              {#if supportsSequences}
                {@const sequences = getSequences(schema.name)}
                {#if sequences.length > 0 || expandedCategories.has(`${schema.name}.sequences`)}
                  <TreeNode
                    label="Sequences"
                    icon={ICON_SEQUENCE}
                    expandable={true}
                    depth={1}
                    suffix={sequences.length > 0 ? `(${sequences.length})` : ''}
                    onexpand={(exp) => handleCategoryExpand(schema.name, 'sequences', exp)}
                  >
                    {#snippet children()}
                      {#each sequences as seq}
                        <TreeNode
                          label={seq.name}
                          icon={ICON_SEQUENCE}
                          expandable={false}
                          depth={2}
                          tooltip={seq.data_type ?? ''}
                        >
                          {#snippet children()}
                            <!-- leaf node -->
                          {/snippet}
                        </TreeNode>
                      {/each}
                    {/snippet}
                  </TreeNode>
                {/if}
              {/if}

              <!-- Types/Enums category -->
              {#if supportsEnums}
                {@const enums = getEnums(schema.name)}
                {#if enums.length > 0 || expandedCategories.has(`${schema.name}.enums`)}
                  <TreeNode
                    label="Types"
                    icon={ICON_ENUM}
                    expandable={true}
                    depth={1}
                    suffix={enums.length > 0 ? `(${enums.length})` : ''}
                    onexpand={(exp) => handleCategoryExpand(schema.name, 'enums', exp)}
                  >
                    {#snippet children()}
                      {#each enums as en}
                        <TreeNode
                          label={en.name}
                          icon={ICON_ENUM}
                          expandable={false}
                          depth={2}
                          tooltip={en.variants.join(', ')}
                        >
                          {#snippet children()}
                            <!-- leaf node -->
                          {/snippet}
                        </TreeNode>
                      {/each}
                    {/snippet}
                  </TreeNode>
                {/if}
              {/if}
            {/snippet}
          </TreeNode>
        {/if}
      {/each}
    {/if}
  {:else}
    <!-- Generic tree: Containers > Items > Fields -->
    {#if containers.length === 0}
      <div class="empty-schemas">
        <span class="text-muted">No {meta.containerLabel.toLowerCase()}s loaded</span>
      </div>
    {:else}
      {#each containers as container}
        {@const filteredItems = getItems(container.name)}
        {#if filteredItems.length > 0 || !searchQuery}
          <TreeNode
            label={container.name}
            icon={ICON_FOLDER}
            expandable={true}
            depth={0}
            onexpand={(exp) => handleContainerExpand(container, exp)}
          >
            {#snippet children()}
              {#each filteredItems as item}
                <TreeNode
                  label={item.name}
                  icon={ICON_TABLE}
                  expandable={true}
                  depth={1}
                  onexpand={(exp) => handleItemExpand(container.name, item.name, exp)}
                  ondblclick={() => handleItemDblClick(container.name, item.name)}
                >
                  {#snippet children()}
                    {#each getFields(container.name, item.name) as field}
                      <TreeNode
                        label={field.name}
                        icon={getFieldIcon(field)}
                        expandable={false}
                        depth={2}
                      >
                        {#snippet children()}
                          <!-- leaf node -->
                        {/snippet}
                      </TreeNode>
                    {/each}
                  {/snippet}
                </TreeNode>
              {/each}
            {/snippet}
          </TreeNode>
        {/if}
      {/each}
    {/if}
  {/if}
</div>

<style>
  .schema-tree {
    padding: 2px 0;
  }

  .search-bar {
    position: relative;
    padding: 4px 8px;
  }

  .search-input {
    width: 100%;
    padding: 4px 24px 4px 8px;
    font-size: 11px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .search-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(122, 162, 247, 0.1);
  }

  .search-input::placeholder {
    color: var(--text-muted);
    font-style: italic;
  }

  .search-clear {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    display: flex;
    align-items: center;
  }

  .search-clear:hover {
    color: var(--text-primary);
  }

  .empty-schemas {
    padding: 12px 16px;
    text-align: center;
    font-size: 11px;
  }
</style>
