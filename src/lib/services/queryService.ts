import * as tauri from '$lib/services/tauri';
import type { QueryResponse, MultiStatementResult } from '$lib/types/query';
import { uiStore } from '$lib/stores/ui.svelte';
import { queryHistoryStore } from '$lib/stores/queryHistory.svelte';

export async function executeQuery(connectionId: string, sql: string): Promise<QueryResponse | null> {
  uiStore.setLoading(true, 'Executing query...');
  const startTime = performance.now();
  try {
    const result = await tauri.executeQuery(connectionId, sql.trim());

    queryHistoryStore.addEntry({
      connectionId,
      sql: sql.trim(),
      executionTimeMs: result.execution_time_ms,
      rowCount: result.row_count,
    });

    return result;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    const elapsed = Math.round(performance.now() - startTime);

    queryHistoryStore.addEntry({
      connectionId,
      sql: sql.trim(),
      executionTimeMs: elapsed,
      rowCount: 0,
      error: message,
    });

    uiStore.showError(`Query error: ${message}`);
    return null;
  } finally {
    uiStore.setLoading(false);
  }
}

export async function executeStatements(
  connectionId: string,
  statements: string[]
): Promise<MultiStatementResult> {
  uiStore.setLoading(true, 'Executing queries...');
  const results: QueryResponse[] = [];

  try {
    for (let i = 0; i < statements.length; i++) {
      const stmt = statements[i].trim();
      if (!stmt) continue;

      uiStore.setLoading(true, `Executing statement ${i + 1} of ${statements.length}...`);
      const startTime = performance.now();

      try {
        const result = await tauri.executeQuery(connectionId, stmt);
        results.push(result);

        queryHistoryStore.addEntry({
          connectionId,
          sql: stmt,
          executionTimeMs: result.execution_time_ms,
          rowCount: result.row_count,
        });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        const elapsed = Math.round(performance.now() - startTime);

        queryHistoryStore.addEntry({
          connectionId,
          sql: stmt,
          executionTimeMs: elapsed,
          rowCount: 0,
          error: message,
        });

        return { results, error: { index: i, message } };
      }
    }

    return { results };
  } finally {
    uiStore.setLoading(false);
  }
}
