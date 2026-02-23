<script lang="ts">
  import type { ColumnDiff } from '$lib/types/diff';

  let { diffs }: { diffs: ColumnDiff[] } = $props();

  let summary = $derived({
    added: diffs.filter(d => d.status === 'added').length,
    removed: diffs.filter(d => d.status === 'removed').length,
    changed: diffs.filter(d => d.status === 'changed').length,
    unchanged: diffs.filter(d => d.status === 'unchanged').length,
  });

  function statusClass(status: string): string {
    return `status-${status}`;
  }

  function statusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }
</script>

<div class="columns-diff">
  <div class="diff-summary">
    {#if summary.added > 0}<span class="badge badge-added">+{summary.added} added</span>{/if}
    {#if summary.removed > 0}<span class="badge badge-removed">-{summary.removed} removed</span>{/if}
    {#if summary.changed > 0}<span class="badge badge-changed">{summary.changed} changed</span>{/if}
    <span class="badge badge-unchanged">{summary.unchanged} unchanged</span>
  </div>

  <table class="diff-table">
    <thead>
      <tr>
        <th>Name</th>
        <th>Source Type</th>
        <th>Target Type</th>
        <th>Nullable</th>
        <th>Default</th>
        <th>PK</th>
        <th>Status</th>
      </tr>
    </thead>
    <tbody>
      {#each diffs as diff}
        <tr class={statusClass(diff.status)}>
          <td class="col-name">{diff.name}</td>
          <td>{diff.source?.data_type ?? '—'}</td>
          <td>{diff.target?.data_type ?? '—'}</td>
          <td>{diff.source?.is_nullable ?? '—'} / {diff.target?.is_nullable ?? '—'}</td>
          <td class="col-default">{diff.source?.column_default ?? '—'} / {diff.target?.column_default ?? '—'}</td>
          <td>{diff.source?.is_primary_key ?? '—'} / {diff.target?.is_primary_key ?? '—'}</td>
          <td>
            <span class="status-badge {statusClass(diff.status)}">{statusLabel(diff.status)}</span>
          </td>
        </tr>
        {#if diff.changes && diff.changes.length > 0}
          <tr class="change-detail-row {statusClass(diff.status)}">
            <td colspan="7">
              <div class="change-details">
                {#each diff.changes as change}
                  <span class="change-item">{change}</span>
                {/each}
              </div>
            </td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>

<style>
  .columns-diff {
    padding: 12px;
    overflow: auto;
  }

  .diff-summary {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  .badge {
    padding: 2px 8px;
    font-size: 11px;
    border-radius: 10px;
    font-weight: 500;
  }

  .badge-added { background: rgba(158, 206, 106, 0.15); color: var(--success, #9ece6a); }
  .badge-removed { background: rgba(243, 139, 168, 0.15); color: var(--error, #f38ba8); }
  .badge-changed { background: rgba(224, 175, 104, 0.15); color: var(--warning, #e0af68); }
  .badge-unchanged { background: rgba(150, 150, 150, 0.1); color: var(--text-muted); }

  .diff-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  .diff-table th {
    padding: 6px 10px;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    position: sticky;
    top: 0;
  }

  .diff-table td {
    padding: 5px 10px;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-secondary);
  }

  .col-name {
    font-weight: 500;
    color: var(--text-primary);
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
  }

  .col-default {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  tr.status-added { background: rgba(158, 206, 106, 0.06); }
  tr.status-removed { background: rgba(243, 139, 168, 0.06); }
  tr.status-changed { background: rgba(224, 175, 104, 0.06); }

  .status-badge {
    padding: 1px 6px;
    font-size: 10px;
    border-radius: 8px;
    font-weight: 500;
  }
  .status-badge.status-added { background: rgba(158, 206, 106, 0.15); color: var(--success, #9ece6a); }
  .status-badge.status-removed { background: rgba(243, 139, 168, 0.15); color: var(--error, #f38ba8); }
  .status-badge.status-changed { background: rgba(224, 175, 104, 0.15); color: var(--warning, #e0af68); }
  .status-badge.status-unchanged { background: rgba(150, 150, 150, 0.1); color: var(--text-muted); }

  .change-detail-row td {
    padding: 2px 10px 6px 20px;
    border-bottom: 1px solid var(--border-color);
  }

  .change-details {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .change-item {
    font-size: 10px;
    color: var(--warning, #e0af68);
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
    background: rgba(224, 175, 104, 0.08);
    padding: 1px 6px;
    border-radius: 3px;
  }
</style>
