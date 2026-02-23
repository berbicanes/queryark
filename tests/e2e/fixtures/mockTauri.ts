import type { Page } from '@playwright/test';

// Default mock responses for Tauri IPC commands
const defaultMockResponses: Record<string, unknown> = {
  connect_db: 'mock-connection-id',
  disconnect_db: null,
  test_connection: true,
  ping_connection: true,
  execute_query: {
    columns: [
      { name: 'id', type_name: 'integer' },
      { name: 'name', type_name: 'text' },
    ],
    rows: [
      [1, 'Alice'],
      [2, 'Bob'],
      [3, 'Charlie'],
    ],
    row_count: 3,
    execution_time_ms: 42,
    truncated: false,
  },
  get_schemas: [{ name: 'public' }],
  get_tables: [
    { name: 'users', table_type: 'table' },
    { name: 'orders', table_type: 'table' },
  ],
  get_columns: [
    { name: 'id', data_type: 'integer', nullable: false, default_value: null, is_primary_key: true },
    { name: 'name', data_type: 'text', nullable: true, default_value: null, is_primary_key: false },
  ],
  get_indexes: [],
  get_foreign_keys: [],
  get_database_category: 'Relational',
  get_containers: [],
  get_items: [],
  get_item_fields: [],
  get_row_count: 100,
  get_table_data: {
    columns: [],
    rows: [],
    row_count: 0,
    execution_time_ms: 10,
    truncated: false,
  },
  get_table_stats: { row_count: 100, size_bytes: 8192 },
  get_routines: [],
  get_sequences: [],
  get_enums: [],
  cancel_query: true,
  begin_transaction: null,
  commit_transaction: null,
  rollback_transaction: null,
  check_keychain_available: true,
  backup_configs: 'backup_20260223_120000.json',
  list_backups: [],
  restore_backup: null,
  delete_backup: null,
};

/**
 * Set up mock Tauri IPC layer in the browser context.
 * This allows Playwright tests to run without the Rust backend.
 */
export async function setupMockTauri(page: Page, overrides?: Record<string, unknown>) {
  const responses = { ...defaultMockResponses, ...overrides };

  await page.addInitScript((mockResponses) => {
    // Mock the Tauri internals that @tauri-apps/api/core uses
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, _args?: Record<string, unknown>) => {
        if (cmd in mockResponses) {
          return mockResponses[cmd];
        }
        console.warn(`[MockTauri] Unhandled command: ${cmd}`);
        return null;
      },
      transformCallback: (callback: (response: unknown) => void) => {
        const id = Math.random();
        (window as any)[`_${id}`] = callback;
        return id;
      },
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { label: 'main' },
      },
    };

    // Mock plugin-store
    (window as any).__TAURI_PLUGIN_STORE__ = {};
  }, responses);
}

/**
 * Override a specific mock command response after initial setup.
 */
export async function setMockResponse(page: Page, command: string, response: unknown) {
  await page.evaluate(
    ({ cmd, resp }) => {
      const internals = (window as any).__TAURI_INTERNALS__;
      if (internals) {
        const originalInvoke = internals.invoke;
        internals.invoke = async (c: string, args?: Record<string, unknown>) => {
          if (c === cmd) return resp;
          return originalInvoke(c, args);
        };
      }
    },
    { cmd: command, resp: response },
  );
}
