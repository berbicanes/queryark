import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { save } from '@tauri-apps/plugin-dialog';
import { captureError } from '$lib/services/sentryService';

export interface DumpResult {
  tables_dumped: number;
  rows_dumped: number;
  file_size_bytes: number;
}

export interface DumpProgress {
  schema: string;
  table: string;
  tables_done: number;
  tables_total: number;
  rows_dumped: number;
}

export async function dumpDatabase(
  connectionId: string,
  filePath: string,
  schemas: string[],
  includeData: boolean,
): Promise<DumpResult> {
  try {
    return await invoke<DumpResult>('dump_database', {
      connectionId,
      filePath,
      schemas,
      includeData,
    });
  } catch (error) {
    captureError(error, { command: 'dump_database' });
    throw error;
  }
}

export async function pickDumpFile(dbName: string): Promise<string | null> {
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  const defaultName = `${dbName}_dump_${date}.sql`;

  const filePath = await save({
    defaultPath: defaultName,
    filters: [{ name: 'SQL Files', extensions: ['sql'] }],
  });

  return filePath ?? null;
}

export function onDumpProgress(callback: (progress: DumpProgress) => void): Promise<() => void> {
  return listen<DumpProgress>('dump-progress', (event) => {
    callback(event.payload);
  });
}
