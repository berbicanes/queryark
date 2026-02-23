import { invoke } from '@tauri-apps/api/core';

export interface BackupEntry {
  filename: string;
  created_at: string;
  size_bytes: number;
}

export async function backupConfigs(): Promise<string> {
  return invoke<string>('backup_configs');
}

export async function listBackups(): Promise<BackupEntry[]> {
  return invoke<BackupEntry[]>('list_backups');
}

export async function restoreBackup(filename: string): Promise<void> {
  return invoke<void>('restore_backup', { filename });
}

export async function deleteBackup(filename: string): Promise<void> {
  return invoke<void>('delete_backup', { filename });
}
