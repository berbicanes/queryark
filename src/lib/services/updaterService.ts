import type { Update } from '@tauri-apps/plugin-updater';

let updateAvailable = $state(false);
let updateVersion = $state('');
let updateProgress = $state<'idle' | 'checking' | 'downloading' | 'installing'>('idle');

let pendingUpdate: Update | null = null;

export const updaterState = {
  get updateAvailable() { return updateAvailable; },
  get updateVersion() { return updateVersion; },
  get updateProgress() { return updateProgress; },
};

export async function checkForUpdates(): Promise<void> {
  try {
    updateProgress = 'checking';
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check();

    if (update) {
      pendingUpdate = update;
      updateAvailable = true;
      updateVersion = update.version;
    }
  } catch {
    // Updater not configured or network error — silent no-op
  } finally {
    if (updateProgress === 'checking') {
      updateProgress = 'idle';
    }
  }
}

export async function installUpdate(): Promise<void> {
  if (!pendingUpdate) return;

  try {
    updateProgress = 'downloading';
    await pendingUpdate.downloadAndInstall();

    updateProgress = 'installing';
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  } catch {
    updateProgress = 'idle';
  }
}
