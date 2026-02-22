<script lang="ts">
  import { schemaStore } from '$lib/stores/schema.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import * as schemaService from '$lib/services/schemaService';
  import { DB_METADATA } from '$lib/types/database';
  import type { SchemaInfo, TableInfo, ColumnInfo, ContainerInfo, ItemInfo, FieldInfo } from '$lib/types/schema';
  import type { DatabaseCategory } from '$lib/types/connection';
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

  // SQL-specific data
  let schemas = $derived(schemaStore.getSchemas(connectionId));
  // Generic data
  let containers = $derived(schemaStore.getContainers(connectionId));

  // Track expanded state
  let expandedSchemas = $state<Set<string>>(new Set());
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

  async function handleTableExpand(schema: string, table: string, expanded: boolean) {
    const key = `${schema}.${table}`;
    if (expanded) {
      expandedTables.add(key);
      const cols = schemaStore.getColumns(connectionId, schema, table);
      if (cols.length === 0) {
        await schemaService.loadColumns(connectionId, schema, table);
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

  function getTables(schemaName: string): TableInfo[] {
    return schemaStore.getTables(connectionId, schemaName);
  }

  function getColumns(schemaName: string, tableName: string): ColumnInfo[] {
    return schemaStore.getColumns(connectionId, schemaName, tableName);
  }

  function getItems(containerName: string): ItemInfo[] {
    return schemaStore.getItems(connectionId, containerName);
  }

  function getFields(containerName: string, itemName: string): FieldInfo[] {
    return schemaStore.getFields(connectionId, containerName, itemName);
  }

  const ICON_FOLDER = '\u{1F4C1}';
  const ICON_TABLE = '\u{1F5C3}';
  const ICON_KEY = '\u{1F511}';

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
  {#if isSqlLike}
    <!-- SQL-like tree: Schemas > Tables > Columns -->
    {#if schemas.length === 0}
      <div class="empty-schemas">
        <span class="text-muted">No {meta.containerLabel.toLowerCase()}s loaded</span>
      </div>
    {:else}
      {#each schemas as schema}
        <TreeNode
          label={schema.name}
          icon={ICON_FOLDER}
          expandable={true}
          depth={0}
          onexpand={(exp) => handleSchemaExpand(schema, exp)}
        >
          {#snippet children()}
            {#each getTables(schema.name) as table}
              <TreeNode
                label={table.name}
                icon={ICON_TABLE}
                expandable={true}
                depth={1}
                onexpand={(exp) => handleTableExpand(schema.name, table.name, exp)}
                ondblclick={() => handleTableDblClick(schema.name, table.name)}
              >
                {#snippet children()}
                  {#each getColumns(schema.name, table.name) as column}
                    <TreeNode
                      label={column.name}
                      icon={column.is_primary_key ? ICON_KEY : getColumnTypeIcon(column.data_type)}
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
        <TreeNode
          label={container.name}
          icon={ICON_FOLDER}
          expandable={true}
          depth={0}
          onexpand={(exp) => handleContainerExpand(container, exp)}
        >
          {#snippet children()}
            {#each getItems(container.name) as item}
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
      {/each}
    {/if}
  {/if}
</div>

<style>
  .schema-tree {
    padding: 2px 0;
  }

  .empty-schemas {
    padding: 12px 16px;
    text-align: center;
    font-size: 11px;
  }
</style>
