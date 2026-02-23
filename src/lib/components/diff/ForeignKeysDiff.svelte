<script lang="ts">
  import type { ForeignKeyDiff } from '$lib/types/diff';

  let { diffs }: { diffs: ForeignKeyDiff[] } = $props();

  function statusClass(status: string): string {
    return `status-${status}`;
  }

  function statusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }
</script>

<div class="fk-diff">
  {#if diffs.length === 0}
    <div class="empty">No foreign keys to compare</div>
  {:else}
    <table class="diff-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Source Columns</th>
          <th>Target Columns</th>
          <th>References</th>
          <th>On Delete</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {#each diffs as diff}
          <tr class={statusClass(diff.status)}>
            <td class="fk-name">{diff.name}</td>
            <td>{diff.source?.columns.join(', ') ?? '—'}</td>
            <td>{diff.target?.columns.join(', ') ?? '—'}</td>
            <td>
              {diff.source ? `${diff.source.referenced_table}(${diff.source.referenced_columns.join(', ')})` : '—'}
              /
              {diff.target ? `${diff.target.referenced_table}(${diff.target.referenced_columns.join(', ')})` : '—'}
            </td>
            <td>{diff.source?.on_delete ?? '—'} / {diff.target?.on_delete ?? '—'}</td>
            <td><span class="status-badge {statusClass(diff.status)}">{statusLabel(diff.status)}</span></td>
          </tr>
          {#if diff.changes && diff.changes.length > 0}
            <tr class="change-detail-row {statusClass(diff.status)}">
              <td colspan="6">
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
  {/if}
</div>

<style>
  .fk-diff {
    padding: 12px;
    overflow: auto;
  }

  .empty {
    padding: 20px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }

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

  .fk-name {
    font-weight: 500;
    color: var(--text-primary);
    font-family: var(--font-mono, 'JetBrains Mono', monospace);
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
