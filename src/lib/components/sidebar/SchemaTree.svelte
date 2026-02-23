<script lang="ts">
  import { schemaStore } from '$lib/stores/schema.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import { executeQuery } from '$lib/services/tauri';
  import { quoteIdentifier } from '$lib/utils/sqlHelpers';
  import { DB_METADATA } from '$lib/types/database';
  import type { SchemaInfo, TableInfo, ColumnInfo, ContainerInfo, ItemInfo, FieldInfo, TableStats, RoutineInfo, SequenceInfo, EnumInfo } from '$lib/types/schema';
  import type { DatabaseCategory, DatabaseType } from '$lib/types/connection';
  import type { TabType } from '$lib/types/tabs';
  import TreeNode from './TreeNode.svelte';
  import VirtualTreeList from './VirtualTreeList.svelte';

  let { connectionId, scrollContainer }: { connectionId: string; scrollContainer?: HTMLElement } = $props();

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

  // Schema visibility
  const SYSTEM_SCHEMAS: Record<string, string[]> = {
    PostgreSQL: ['pg_catalog', 'information_schema', 'pg_toast'],
    CockroachDB: ['pg_catalog', 'information_schema', 'pg_extension', 'crdb_internal'],
    Redshift: ['pg_catalog', 'information_schema', 'pg_toast'],
    MySQL: ['mysql', 'sys', 'performance_schema', 'information_schema'],
    MariaDB: ['mysql', 'sys', 'performance_schema', 'information_schema'],
  };

  function getSystemSchemas(): string[] {
    return SYSTEM_SCHEMAS[dbType] ?? ['information_schema'];
  }

  let schemaDropdownOpen = $state(false);
  let visibilityInitialized = $state(false);

  // Initialize visibility from persisted settings or apply defaults when schemas load
  $effect(() => {
    if (schemas.length < 2 || visibilityInitialized) return;
    const allNames = schemas.map(s => s.name);
    const saved = settingsStore.getSchemaVisibility(connectionId);
    if (saved) {
      // Restore saved — filter to only schemas that still exist
      const valid = saved.filter(s => allNames.includes(s));
      schemaStore.setVisibleSchemas(connectionId, valid.length > 0 ? valid : null);
    } else {
      // First time: hide system schemas
      const sysSchemas = getSystemSchemas();
      const visible = allNames.filter(s => !sysSchemas.includes(s));
      if (visible.length > 0 && visible.length < allNames.length) {
        schemaStore.setVisibleSchemas(connectionId, visible);
        settingsStore.setSchemaVisibility(connectionId, visible);
      }
    }
    visibilityInitialized = true;
  });

  // Auto-expand when only 1 schema is visible
  $effect(() => {
    if (!visibilityInitialized) return;
    const visible = schemaStore.getVisibleSchemas(connectionId);
    if (visible && visible.length === 1) {
      const schemaName = visible[0];
      const schema = schemas.find(s => s.name === schemaName);
      if (schema && !expandedSchemas.has(schemaName)) {
        handleSchemaExpand(schema, true);
      }
    }
  });

  let allSchemaNames = $derived(schemas.map(s => s.name));
  let showSchemaSelector = $derived(isSqlLike && schemas.length >= 2);

  let filteredSchemas = $derived.by(() => {
    const visible = schemaStore.getVisibleSchemas(connectionId);
    if (!visible) return schemas;
    return schemas.filter(s => visible.includes(s.name));
  });

  let selectorLabel = $derived.by(() => {
    const visible = schemaStore.getVisibleSchemas(connectionId);
    if (!visible) return 'All schemas';
    if (visible.length === 1) return visible[0];
    return `${visible.length} schemas`;
  });

  let allSchemasVisible = $derived(schemaStore.getVisibleSchemas(connectionId) === null);

  function toggleAllSchemas() {
    if (allSchemasVisible) {
      // Switch to showing only non-system schemas
      const sysSchemas = getSystemSchemas();
      const visible = allSchemaNames.filter(s => !sysSchemas.includes(s));
      if (visible.length > 0 && visible.length < allSchemaNames.length) {
        schemaStore.setVisibleSchemas(connectionId, visible);
        settingsStore.setSchemaVisibility(connectionId, visible);
      }
    } else {
      schemaStore.setVisibleSchemas(connectionId, null);
      settingsStore.setSchemaVisibility(connectionId, null);
    }
  }

  function toggleSchema(schemaName: string) {
    schemaStore.toggleSchemaVisibility(connectionId, schemaName, allSchemaNames);
    const updated = schemaStore.getVisibleSchemas(connectionId);
    settingsStore.setSchemaVisibility(connectionId, updated);
  }

  function isSchemaChecked(schemaName: string): boolean {
    return schemaStore.isSchemaVisible(connectionId, schemaName);
  }

  function closeDropdown() {
    schemaDropdownOpen = false;
  }

  function handleDropdownKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closeDropdown();
    }
  }

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
    const activeSchema = schemaStore.getActiveSchema(connectionId, dbType);
    const title = (activeSchema && schema !== activeSchema) ? `${schema}.${table}` : table;
    tabStore.openTab({
      type: 'table',
      title,
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

  // --- Schema context menu ---
  let schemaContextMenu = $state<{ x: number; y: number; schemaName: string } | null>(null);
  let createSchemaInput = $state<{ value: string } | null>(null);

  function handleSchemaContextMenu(e: MouseEvent, schemaName: string) {
    e.preventDefault();
    schemaContextMenu = { x: e.clientX, y: e.clientY, schemaName };
  }

  function closeSchemaContextMenu() {
    schemaContextMenu = null;
  }

  function ctxNewQueryHere() {
    if (!schemaContextMenu) return;
    const schema = schemaContextMenu.schemaName;
    closeSchemaContextMenu();

    let prefix = '';
    switch (dbType) {
      case 'PostgreSQL':
      case 'CockroachDB':
      case 'Redshift':
        prefix = `SET search_path TO ${quoteIdentifier(schema, dbType)};\n\n`;
        break;
      case 'MySQL':
      case 'MariaDB':
        prefix = `USE ${quoteIdentifier(schema, dbType)};\n\n`;
        break;
      case 'MSSQL':
        // MSSQL doesn't have USE for schemas, just qualify tables
        prefix = `-- Schema: ${schema}\n\n`;
        break;
      default:
        prefix = `-- Schema: ${schema}\n\n`;
    }

    const queryCount = tabStore.tabs.filter(t => t.type === 'query').length + 1;
    tabStore.openTab({
      type: 'query',
      title: `Query ${queryCount}`,
      connectionId,
      sql: prefix
    });
  }

  function ctxCreateSchema() {
    closeSchemaContextMenu();
    createSchemaInput = { value: '' };
    // Focus the input after render
    setTimeout(() => {
      const el = document.querySelector('.create-schema-input input') as HTMLInputElement;
      el?.focus();
    }, 0);
  }

  async function submitCreateSchema() {
    if (!createSchemaInput || !createSchemaInput.value.trim()) {
      createSchemaInput = null;
      return;
    }
    const name = createSchemaInput.value.trim();
    createSchemaInput = null;
    const ddl = `CREATE SCHEMA ${quoteIdentifier(name, dbType)}`;
    try {
      await executeQuery(connectionId, ddl);
      await schemaService.refreshSchema(connectionId);
      uiStore.showSuccess(`Schema "${name}" created`);
    } catch (err) {
      uiStore.showError(`Failed to create schema: ${err}`);
    }
  }

  function handleCreateSchemaKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      submitCreateSchema();
    } else if (e.key === 'Escape') {
      createSchemaInput = null;
    }
  }

  function ctxDropSchema() {
    if (!schemaContextMenu) return;
    const schema = schemaContextMenu.schemaName;
    closeSchemaContextMenu();

    let ddl: string;
    switch (dbType) {
      case 'MySQL':
      case 'MariaDB':
        ddl = `DROP SCHEMA ${quoteIdentifier(schema, dbType)}`;
        break;
      case 'MSSQL':
        ddl = `DROP SCHEMA ${quoteIdentifier(schema, dbType)}`;
        break;
      default:
        ddl = `DROP SCHEMA ${quoteIdentifier(schema, dbType)} CASCADE`;
    }

    uiStore.confirm(
      `Drop schema "${schema}"? This will permanently delete all objects within it.`,
      async () => {
        try {
          await executeQuery(connectionId, ddl);
          await schemaService.refreshSchema(connectionId);
          uiStore.showSuccess(`Schema "${schema}" dropped`);
        } catch (err) {
          uiStore.showError(`Failed to drop schema: ${err}`);
        }
      }
    );
  }

  // Databases that support CREATE/DROP SCHEMA
  let supportsSchemaManagement = $derived(
    ['PostgreSQL', 'MySQL', 'MariaDB', 'MSSQL', 'CockroachDB', 'Redshift'].includes(dbType)
  );
</script>

<svelte:window onclick={closeSchemaContextMenu} />

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

  {#if showSchemaSelector}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="schema-selector" onkeydown={handleDropdownKeydown}>
      <button
        class="schema-selector-btn"
        onclick={() => schemaDropdownOpen = !schemaDropdownOpen}
        aria-expanded={schemaDropdownOpen}
        aria-haspopup="listbox"
      >
        <svg class="schema-icon" width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <span class="schema-selector-label">{selectorLabel}</span>
        <svg class="schema-chevron" class:open={schemaDropdownOpen} width="10" height="10" viewBox="0 0 16 16" fill="none">
          <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
      {#if schemaDropdownOpen}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="schema-dropdown-backdrop" onclick={closeDropdown} onkeydown={handleDropdownKeydown}></div>
        <div class="schema-dropdown" role="listbox">
          <label class="schema-option schema-option-all">
            <input
              type="checkbox"
              checked={allSchemasVisible}
              onchange={toggleAllSchemas}
            />
            <span>All schemas</span>
          </label>
          <div class="schema-dropdown-divider"></div>
          {#each allSchemaNames as name}
            <label class="schema-option">
              <input
                type="checkbox"
                checked={isSchemaChecked(name)}
                onchange={() => toggleSchema(name)}
              />
              <span>{name}</span>
            </label>
          {/each}
        </div>
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
      {#each filteredSchemas as schema}
        {#if hasSearchMatch(schema.name) || !searchQuery}
          <TreeNode
            label={schema.name}
            icon={ICON_FOLDER}
            expandable={true}
            depth={0}
            onexpand={(exp) => handleSchemaExpand(schema, exp)}
            oncontextmenu={supportsSchemaManagement ? (e) => handleSchemaContextMenu(e, schema.name) : undefined}
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
                    <VirtualTreeList items={filteredTables} {scrollContainer}>
                      {#snippet children(table: TableInfo)}
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
                      {/snippet}
                    </VirtualTreeList>
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
                    <VirtualTreeList items={filteredViews} {scrollContainer}>
                      {#snippet children(view: TableInfo)}
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
                      {/snippet}
                    </VirtualTreeList>
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
                      <VirtualTreeList items={routines} {scrollContainer}>
                        {#snippet children(routine: RoutineInfo)}
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
                        {/snippet}
                      </VirtualTreeList>
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
                      <VirtualTreeList items={sequences} {scrollContainer}>
                        {#snippet children(seq: SequenceInfo)}
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
                        {/snippet}
                      </VirtualTreeList>
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
                      <VirtualTreeList items={enums} {scrollContainer}>
                        {#snippet children(en: EnumInfo)}
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
                        {/snippet}
                      </VirtualTreeList>
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
              <VirtualTreeList items={filteredItems} {scrollContainer}>
                {#snippet children(item: ItemInfo)}
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
                {/snippet}
              </VirtualTreeList>
            {/snippet}
          </TreeNode>
        {/if}
      {/each}
    {/if}
  {/if}

  {#if createSchemaInput}
    <div class="create-schema-input">
      <input
        type="text"
        bind:value={createSchemaInput.value}
        onkeydown={handleCreateSchemaKeydown}
        onblur={() => createSchemaInput = null}
        placeholder="New schema name..."
      />
    </div>
  {/if}
</div>

{#if schemaContextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="schema-context-menu"
    style="left: {schemaContextMenu.x}px; top: {schemaContextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') closeSchemaContextMenu(); }}
  >
    <button class="context-item" onclick={ctxNewQueryHere}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M3 2h7l3 3v9H3V2z" stroke="currentColor" stroke-width="1.2" fill="none"/>
        <path d="M6 8h4M6 11h4" stroke="currentColor" stroke-width="1" stroke-linecap="round"/>
      </svg>
      New Query Here
    </button>
    <div class="context-divider"></div>
    <button class="context-item" onclick={ctxCreateSchema}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      Create Schema...
    </button>
    <button class="context-item danger" onclick={ctxDropSchema}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M3 4h10M6 4V3a1 1 0 011-1h2a1 1 0 011 1v1M5 4v9a1 1 0 001 1h4a1 1 0 001-1V4" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
      Drop Schema
    </button>
  </div>
{/if}

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

  .schema-selector {
    position: relative;
    padding: 2px 8px 4px;
  }

  .schema-selector-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    font-size: 11px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    text-align: left;
  }

  .schema-selector-btn:hover {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .schema-icon {
    flex-shrink: 0;
    opacity: 0.6;
  }

  .schema-selector-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .schema-chevron {
    flex-shrink: 0;
    transition: transform 0.15s ease;
  }

  .schema-chevron.open {
    transform: rotate(180deg);
  }

  .schema-dropdown-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .schema-dropdown {
    position: absolute;
    top: 100%;
    left: 8px;
    right: 8px;
    z-index: 100;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    max-height: 240px;
    overflow-y: auto;
    padding: 4px 0;
  }

  .schema-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
    user-select: none;
  }

  .schema-option:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .schema-option input[type="checkbox"] {
    width: 13px;
    height: 13px;
    margin: 0;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .schema-option-all {
    font-weight: 500;
    color: var(--text-primary);
  }

  .schema-dropdown-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }

  /* Schema context menu */
  .schema-context-menu {
    position: fixed;
    z-index: 500;
    background: var(--bg-elevated, var(--bg-secondary));
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 4px 0;
    min-width: 160px;
    animation: ctxMenuIn 120ms var(--ease-out-expo);
  }

  @keyframes ctxMenuIn {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }

  .schema-context-menu :global(.context-item) {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    transition: background var(--transition-fast);
    cursor: pointer;
    border: none;
    background: none;
  }

  .context-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    transition: background var(--transition-fast);
    cursor: pointer;
    border: none;
    background: none;
  }

  .context-item:hover {
    background: var(--bg-hover);
  }

  .context-item.danger {
    color: var(--error);
  }

  .context-item.danger:hover {
    background: rgba(243, 139, 168, 0.1);
  }

  .context-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }

  /* Create schema input */
  .create-schema-input {
    padding: 4px 8px;
  }

  .create-schema-input input {
    width: 100%;
    padding: 4px 8px;
    font-size: 11px;
    background: var(--bg-primary);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
    box-shadow: 0 0 0 3px rgba(122, 162, 247, 0.1);
  }

  .create-schema-input input::placeholder {
    color: var(--text-muted);
    font-style: italic;
  }
</style>
