<script lang="ts">
  import { v4 as uuidv4 } from 'uuid';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import type { ConnectionConfig, DatabaseType } from '$lib/types/connection';
  import { DB_METADATA, DB_GROUPS } from '$lib/types/database';

  let editConnection = $state<ConnectionConfig | null>(null);

  // Form state
  let name = $state('');
  let dbType = $state<DatabaseType>('PostgreSQL');
  let host = $state('localhost');
  let port = $state<number | undefined>(5432);
  let username = $state('');
  let password = $state('');
  let database = $state('');
  let useSsl = $state(false);
  // SQLite
  let filePath = $state('');
  // Oracle
  let oracleSid = $state('');
  let oracleServiceName = $state('');
  // Snowflake
  let snowflakeAccount = $state('');
  let snowflakeWarehouse = $state('');
  let snowflakeRole = $state('');
  // Neo4j
  let boltUrl = $state('');
  // AWS
  let awsRegion = $state('us-east-1');
  let awsAccessKey = $state('');
  let awsSecretKey = $state('');

  let isTesting = $state(false);
  let testResult = $state<'success' | 'fail' | null>(null);
  let isSaving = $state(false);

  let isEditing = $derived(editConnection !== null);
  let meta = $derived(DB_METADATA[dbType]);

  // Auto-switch port when database type changes
  $effect(() => {
    if (!isEditing) {
      const defaultPort = DB_METADATA[dbType].defaultPort;
      port = defaultPort ?? undefined;
    }
  });

  function buildConfig(): ConnectionConfig {
    const config: ConnectionConfig = {
      id: editConnection?.id ?? uuidv4(),
      name,
      db_type: dbType,
      use_ssl: useSsl,
    };

    if (meta.requiresHost) {
      config.host = host;
      config.port = port;
      config.username = username;
      config.password = password;
    }

    if (meta.requiresFilePath) {
      config.file_path = filePath;
    }

    if (database) config.database = database;

    // Oracle-specific
    if (dbType === 'Oracle') {
      if (oracleSid) config.oracle_sid = oracleSid;
      if (oracleServiceName) config.oracle_service_name = oracleServiceName;
    }

    // Snowflake-specific
    if (dbType === 'Snowflake') {
      config.snowflake_account = snowflakeAccount;
      if (snowflakeWarehouse) config.snowflake_warehouse = snowflakeWarehouse;
      if (snowflakeRole) config.snowflake_role = snowflakeRole;
    }

    // Neo4j-specific
    if (dbType === 'Neo4j') {
      if (boltUrl) config.bolt_url = boltUrl;
    }

    // DynamoDB-specific
    if (dbType === 'DynamoDB') {
      config.aws_region = awsRegion;
      if (awsAccessKey && awsSecretKey) {
        config.cloud_auth = {
          AwsCredentials: {
            access_key: awsAccessKey,
            secret_key: awsSecretKey,
            region: awsRegion,
          }
        };
      }
    }

    // BigQuery-specific
    if (dbType === 'BigQuery') {
      // credentials_json would be pasted or loaded from file
      if (database) config.database = database; // project ID
    }

    return config;
  }

  async function handleTestConnection() {
    isTesting = true;
    testResult = null;
    try {
      const config = buildConfig();
      const success = await connectionService.testConnectionConfig(config);
      testResult = success ? 'success' : 'fail';
    } catch {
      testResult = 'fail';
    } finally {
      isTesting = false;
    }
  }

  async function handleSave() {
    if (!name.trim()) {
      uiStore.showError('Connection name is required');
      return;
    }
    if (meta.requiresHost && !host.trim()) {
      uiStore.showError('Host is required');
      return;
    }
    if (meta.requiresFilePath && !filePath.trim()) {
      uiStore.showError('File path is required');
      return;
    }
    if (dbType === 'Snowflake' && !snowflakeAccount.trim()) {
      uiStore.showError('Snowflake account is required');
      return;
    }

    isSaving = true;
    try {
      const config = buildConfig();
      await connectionService.saveConnection(config);
      uiStore.closeConnectionModal();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      uiStore.showError(`Failed to save connection: ${message}`);
    } finally {
      isSaving = false;
    }
  }

  function handleCancel() {
    uiStore.closeConnectionModal();
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleCancel();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick}>
  <div class="modal-card connection-modal">
    <div class="modal-header">
      <h2>{isEditing ? 'Edit Connection' : 'New Connection'}</h2>
      <button class="close-btn" onclick={handleCancel} title="Close" aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="form-group">
        <label for="conn-name">Connection Name</label>
        <input
          id="conn-name"
          type="text"
          bind:value={name}
          placeholder="My Database"
        />
      </div>

      <div class="form-row">
        <div class="form-group" style="flex: 2">
          <label for="conn-type">Database Type</label>
          <select id="conn-type" bind:value={dbType}>
            {#each DB_GROUPS as group}
              <optgroup label={group.name}>
                {#each group.types as type}
                  <option value={type}>{DB_METADATA[type].label}</option>
                {/each}
              </optgroup>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label for="conn-ssl">SSL</label>
          <div class="toggle-wrapper">
            <label class="toggle">
              <input type="checkbox" bind:checked={useSsl} />
              <span class="slider"></span>
            </label>
            <span class="toggle-label">{useSsl ? 'Enabled' : 'Disabled'}</span>
          </div>
        </div>
      </div>

      {#if meta.requiresHost}
        <div class="form-row">
          <div class="form-group" style="flex: 2">
            <label for="conn-host">Host</label>
            <input
              id="conn-host"
              type="text"
              bind:value={host}
              placeholder="localhost"
            />
          </div>
          {#if meta.defaultPort !== null}
            <div class="form-group" style="flex: 1">
              <label for="conn-port">Port</label>
              <input
                id="conn-port"
                type="number"
                bind:value={port}
                min="1"
                max="65535"
              />
            </div>
          {/if}
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="conn-user">Username</label>
            <input
              id="conn-user"
              type="text"
              bind:value={username}
              placeholder="user"
            />
          </div>
          <div class="form-group">
            <label for="conn-pass">Password</label>
            <input
              id="conn-pass"
              type="password"
              bind:value={password}
              placeholder="********"
            />
          </div>
        </div>
      {/if}

      {#if meta.requiresFilePath}
        <div class="form-group">
          <label for="conn-filepath">File Path</label>
          <input
            id="conn-filepath"
            type="text"
            bind:value={filePath}
            placeholder="/path/to/database.db"
          />
        </div>
      {/if}

      {#if dbType === 'Oracle'}
        <div class="form-row">
          <div class="form-group">
            <label for="conn-oracle-sid">SID</label>
            <input
              id="conn-oracle-sid"
              type="text"
              bind:value={oracleSid}
              placeholder="ORCL"
            />
          </div>
          <div class="form-group">
            <label for="conn-oracle-service">Service Name</label>
            <input
              id="conn-oracle-service"
              type="text"
              bind:value={oracleServiceName}
              placeholder="orcl.example.com"
            />
          </div>
        </div>
      {/if}

      {#if dbType === 'Snowflake'}
        <div class="form-group">
          <label for="conn-sf-account">Account</label>
          <input
            id="conn-sf-account"
            type="text"
            bind:value={snowflakeAccount}
            placeholder="xy12345.us-east-1"
          />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label for="conn-sf-warehouse">Warehouse</label>
            <input
              id="conn-sf-warehouse"
              type="text"
              bind:value={snowflakeWarehouse}
              placeholder="COMPUTE_WH"
            />
          </div>
          <div class="form-group">
            <label for="conn-sf-role">Role</label>
            <input
              id="conn-sf-role"
              type="text"
              bind:value={snowflakeRole}
              placeholder="SYSADMIN"
            />
          </div>
        </div>
      {/if}

      {#if dbType === 'Neo4j'}
        <div class="form-group">
          <label for="conn-bolt-url">Bolt URL (optional)</label>
          <input
            id="conn-bolt-url"
            type="text"
            bind:value={boltUrl}
            placeholder="bolt://localhost:7687"
          />
        </div>
      {/if}

      {#if dbType === 'DynamoDB'}
        <div class="form-group">
          <label for="conn-aws-region">AWS Region</label>
          <input
            id="conn-aws-region"
            type="text"
            bind:value={awsRegion}
            placeholder="us-east-1"
          />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label for="conn-aws-key">Access Key</label>
            <input
              id="conn-aws-key"
              type="text"
              bind:value={awsAccessKey}
              placeholder="AKIA..."
            />
          </div>
          <div class="form-group">
            <label for="conn-aws-secret">Secret Key</label>
            <input
              id="conn-aws-secret"
              type="password"
              bind:value={awsSecretKey}
              placeholder="********"
            />
          </div>
        </div>
      {/if}

      {#if !meta.requiresFilePath && dbType !== 'DynamoDB'}
        <div class="form-group">
          <label for="conn-db">
            {dbType === 'BigQuery' ? 'Project ID' : 'Database'}
          </label>
          <input
            id="conn-db"
            type="text"
            bind:value={database}
            placeholder={dbType === 'BigQuery' ? 'my-project-id' : 'mydb'}
          />
        </div>
      {/if}

      {#if testResult !== null}
        <div class="test-result" class:success={testResult === 'success'} class:fail={testResult === 'fail'}>
          {#if testResult === 'success'}
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M3 8l3 3 7-7" stroke="var(--success)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            Connection successful!
          {:else}
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="var(--error)" stroke-width="2" stroke-linecap="round"/>
            </svg>
            Connection failed
          {/if}
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <button
        class="btn test-btn"
        onclick={handleTestConnection}
        disabled={isTesting}
      >
        {#if isTesting}
          <span class="spinner"></span>
          Testing...
        {:else}
          Test Connection
        {/if}
      </button>

      <div class="footer-spacer"></div>

      <button class="btn cancel-btn" onclick={handleCancel}>
        Cancel
      </button>
      <button
        class="btn btn-primary save-btn"
        onclick={handleSave}
        disabled={isSaving}
      >
        {#if isSaving}
          Saving...
        {:else}
          {isEditing ? 'Update' : 'Save'}
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .connection-modal {
    width: 520px;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    border: none;
    background: none;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
    padding: 0;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .toggle-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 2px;
  }

  .toggle-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .test-result {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    margin-top: 4px;
  }

  .test-result.success {
    background: rgba(166, 227, 161, 0.1);
    color: var(--success);
    border: 1px solid rgba(166, 227, 161, 0.3);
  }

  .test-result.fail {
    background: rgba(243, 139, 168, 0.1);
    color: var(--error);
    border: 1px solid rgba(243, 139, 168, 0.3);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border-color);
  }

  .footer-spacer {
    flex: 1;
  }

  .test-btn {
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
  }

  .test-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
    background: transparent;
  }

  .test-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .cancel-btn {
    color: var(--text-secondary);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .save-btn {
    padding: 6px 16px;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
