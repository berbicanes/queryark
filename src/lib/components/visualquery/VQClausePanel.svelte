<script lang="ts">
  import { v4 as uuidv4 } from 'uuid';
  import type { VQTable, VQWhereClause, VQOrderBy, VQGroupBy } from '$lib/types/visualQuery';

  let {
    tables,
    where = $bindable([]),
    orderBy = $bindable([]),
    groupBy = $bindable([]),
    distinct = $bindable(false),
    limit = $bindable(null),
  }: {
    tables: VQTable[];
    where: VQWhereClause[];
    orderBy: VQOrderBy[];
    groupBy: VQGroupBy[];
    distinct: boolean;
    limit: number | null;
  } = $props();

  const OPERATORS = ['=', '!=', '>', '<', '>=', '<=', 'LIKE', 'NOT LIKE', 'IN', 'NOT IN', 'IS NULL', 'IS NOT NULL'];

  function allColumns(): Array<{ tableId: string; tableName: string; column: string }> {
    const result: Array<{ tableId: string; tableName: string; column: string }> = [];
    for (const t of tables) {
      for (const c of t.columns) {
        result.push({ tableId: t.id, tableName: t.alias || t.name, column: c.name });
      }
    }
    return result;
  }

  function addWhere() {
    if (tables.length === 0) return;
    const cols = allColumns();
    if (cols.length === 0) return;
    where = [...where, {
      id: uuidv4(),
      tableId: cols[0].tableId,
      column: cols[0].column,
      operator: '=',
      value: '',
      connector: 'AND',
    }];
  }

  function removeWhere(id: string) {
    where = where.filter(w => w.id !== id);
  }

  function addOrderBy() {
    const cols = allColumns();
    if (cols.length === 0) return;
    orderBy = [...orderBy, { tableId: cols[0].tableId, column: cols[0].column, direction: 'ASC' }];
  }

  function removeOrderBy(idx: number) {
    orderBy = orderBy.filter((_, i) => i !== idx);
  }

  function addGroupBy() {
    const cols = allColumns();
    if (cols.length === 0) return;
    groupBy = [...groupBy, { tableId: cols[0].tableId, column: cols[0].column }];
  }

  function removeGroupBy(idx: number) {
    groupBy = groupBy.filter((_, i) => i !== idx);
  }

  function handleWhereColumnChange(id: string, value: string) {
    const parts = value.split('.');
    if (parts.length !== 2) return;
    where = where.map(w => w.id === id ? { ...w, tableId: parts[0], column: parts[1] } : w);
  }

  function handleOrderColumnChange(idx: number, value: string) {
    const parts = value.split('.');
    if (parts.length !== 2) return;
    orderBy = orderBy.map((o, i) => i === idx ? { ...o, tableId: parts[0], column: parts[1] } : o);
  }

  function handleGroupColumnChange(idx: number, value: string) {
    const parts = value.split('.');
    if (parts.length !== 2) return;
    groupBy = groupBy.map((g, i) => i === idx ? { ...g, tableId: parts[0], column: parts[1] } : g);
  }

  let limitStr = $state(limit !== null ? String(limit) : '');

  $effect(() => {
    const num = parseInt(limitStr, 10);
    limit = isNaN(num) || num <= 0 ? null : num;
  });
</script>

<div class="clause-panel">
  <!-- WHERE -->
  <div class="clause-section">
    <div class="section-header">
      <span class="section-title">WHERE</span>
      <button class="add-btn" onclick={addWhere} title="Add condition">+</button>
    </div>
    {#each where as w, i}
      <div class="clause-row">
        {#if i > 0}
          <select class="connector-select" bind:value={w.connector}>
            <option>AND</option>
            <option>OR</option>
          </select>
        {:else}
          <span class="connector-placeholder"></span>
        {/if}
        <select
          class="col-select"
          value="{w.tableId}.{w.column}"
          onchange={(e) => handleWhereColumnChange(w.id, (e.target as HTMLSelectElement).value)}
        >
          {#each allColumns() as col}
            <option value="{col.tableId}.{col.column}">{col.tableName}.{col.column}</option>
          {/each}
        </select>
        <select class="op-select" bind:value={w.operator}>
          {#each OPERATORS as op}
            <option>{op}</option>
          {/each}
        </select>
        {#if w.operator !== 'IS NULL' && w.operator !== 'IS NOT NULL'}
          <input class="value-input" bind:value={w.value} placeholder="value" />
        {/if}
        <button class="remove-btn" onclick={() => removeWhere(w.id)}>x</button>
      </div>
    {/each}
  </div>

  <!-- GROUP BY -->
  <div class="clause-section">
    <div class="section-header">
      <span class="section-title">GROUP BY</span>
      <button class="add-btn" onclick={addGroupBy} title="Add group by">+</button>
    </div>
    {#each groupBy as g, i}
      <div class="clause-row">
        <select
          class="col-select wide"
          value="{g.tableId}.{g.column}"
          onchange={(e) => handleGroupColumnChange(i, (e.target as HTMLSelectElement).value)}
        >
          {#each allColumns() as col}
            <option value="{col.tableId}.{col.column}">{col.tableName}.{col.column}</option>
          {/each}
        </select>
        <button class="remove-btn" onclick={() => removeGroupBy(i)}>x</button>
      </div>
    {/each}
  </div>

  <!-- ORDER BY -->
  <div class="clause-section">
    <div class="section-header">
      <span class="section-title">ORDER BY</span>
      <button class="add-btn" onclick={addOrderBy} title="Add order by">+</button>
    </div>
    {#each orderBy as o, i}
      <div class="clause-row">
        <select
          class="col-select"
          value="{o.tableId}.{o.column}"
          onchange={(e) => handleOrderColumnChange(i, (e.target as HTMLSelectElement).value)}
        >
          {#each allColumns() as col}
            <option value="{col.tableId}.{col.column}">{col.tableName}.{col.column}</option>
          {/each}
        </select>
        <select class="dir-select" bind:value={o.direction}>
          <option>ASC</option>
          <option>DESC</option>
        </select>
        <button class="remove-btn" onclick={() => removeOrderBy(i)}>x</button>
      </div>
    {/each}
  </div>

  <!-- Options -->
  <div class="clause-section">
    <div class="section-header">
      <span class="section-title">Options</span>
    </div>
    <div class="options-row">
      <label class="option-label">
        <input type="checkbox" bind:checked={distinct} />
        DISTINCT
      </label>
      <label class="option-label">
        LIMIT
        <input class="limit-input" type="text" bind:value={limitStr} placeholder="none" />
      </label>
    </div>
  </div>
</div>

<style>
  .clause-panel {
    width: 260px;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--border-color);
    background: var(--bg-secondary);
    overflow-y: auto;
    flex-shrink: 0;
  }

  .clause-section {
    border-bottom: 1px solid var(--border-color);
    padding: 8px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .section-title {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .add-btn {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    color: var(--accent);
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .add-btn:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
  }

  .clause-row {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 4px;
    flex-wrap: wrap;
  }

  .connector-select {
    width: 45px;
    padding: 2px 2px;
    font-size: 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .connector-placeholder {
    width: 45px;
  }

  .col-select {
    flex: 1;
    min-width: 80px;
    padding: 3px 4px;
    font-size: 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .col-select.wide { flex: 2; }

  .op-select {
    width: 65px;
    padding: 3px 2px;
    font-size: 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .dir-select {
    width: 50px;
    padding: 3px 2px;
    font-size: 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .value-input {
    flex: 1;
    min-width: 50px;
    padding: 3px 4px;
    font-size: 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .remove-btn {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--text-muted);
    background: none;
    border: none;
    cursor: pointer;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .remove-btn:hover {
    color: var(--error, #f38ba8);
    background: rgba(243, 139, 168, 0.1);
  }

  .options-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .option-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .option-label input[type="checkbox"] {
    width: 13px;
    height: 13px;
    accent-color: var(--accent);
  }

  .limit-input {
    width: 60px;
    padding: 3px 4px;
    font-size: 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  select:focus, input:focus {
    border-color: var(--accent);
  }
</style>
