<script lang="ts">
  import { v4 as uuidv4 } from 'uuid';
  import { open } from '@tauri-apps/plugin-dialog';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import * as connectionService from '$lib/services/connectionService';
  import { storeKeychainPassword, getKeychainPassword, checkKeychainAvailable } from '$lib/services/tauri';
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
  // BigQuery
  let bigqueryCredentials = $state('');
  // Group
  let group = $state('');
  // Color
  let color = $state('');
  // SSH tunneling
  let sshEnabled = $state(false);
  let sshHost = $state('');
  let sshPort = $state<number>(22);
  let sshUser = $state('');
  let sshPassword = $state('');
  let sshKeyPath = $state('');
  let sshPassphrase = $state('');
  // SSL certificates
  let sslCaCert = $state('');
  let sslClientCert = $state('');
  let sslClientKey = $state('');
  // Keychain
  let useKeychain = $state(false);
  let keychainAvailable = $state(false);
  // Connection URL
  let connectionUrl = $state('');
  // Advanced pool settings
  let showAdvanced = $state(false);
  let poolMaxConnections = $state(5);
  let poolIdleTimeout = $state(300);
  let poolAcquireTimeout = $state(10);

  const COLOR_PALETTE = [
    '#ef4444', // red
    '#f97316', // orange
    '#eab308', // yellow
    '#22c55e', // green
    '#06b6d4', // cyan
    '#3b82f6', // blue
    '#8b5cf6', // violet
    '#ec4899', // pink
  ];

  const SSL_CERT_DRIVERS: DatabaseType[] = [
    'PostgreSQL', 'MySQL', 'MariaDB', 'CockroachDB', 'Redshift', 'MongoDB',
  ];

  let isTesting = $state(false);
  let testResult = $state<'success' | 'fail' | null>(null);
  let isSaving = $state(false);

  let isEditing = $derived(editConnection !== null);
  let meta = $derived(DB_METADATA[dbType]);
  let showSslCerts = $derived(useSsl && SSL_CERT_DRIVERS.includes(dbType));

  // Check keychain availability on mount
  $effect(() => {
    checkKeychainAvailable().then(available => {
      keychainAvailable = available;
    }).catch(() => {
      keychainAvailable = false;
    });
  });

  // Auto-switch port when database type changes
  $effect(() => {
    if (!isEditing) {
      const defaultPort = DB_METADATA[dbType].defaultPort;
      port = defaultPort ?? undefined;
    }
  });

  function parseConnectionUrl(url: string) {
    try {
      const trimmed = url.trim();
      if (!trimmed) return;

      // SQLite special case
      if (trimmed.startsWith('sqlite:')) {
        dbType = 'SQLite';
        filePath = trimmed.replace(/^sqlite:/, '');
        return;
      }

      // Detect scheme and set dbType
      const schemeMatch = trimmed.match(/^([a-z][a-z0-9+.-]*):\/\//i);
      if (!schemeMatch) return;

      const scheme = schemeMatch[1].toLowerCase();

      if (scheme === 'postgres' || scheme === 'postgresql') {
        dbType = 'PostgreSQL';
      } else if (scheme === 'mysql') {
        dbType = 'MySQL';
      } else if (scheme === 'mongodb' || scheme === 'mongodb+srv') {
        dbType = 'MongoDB';
      } else if (scheme === 'redis' || scheme === 'rediss') {
        dbType = 'Redis';
        if (scheme === 'rediss') useSsl = true;
      } else if (scheme === 'bolt' || scheme === 'neo4j' || scheme === 'neo4j+s' || scheme === 'neo4j+ssc') {
        dbType = 'Neo4j';
        boltUrl = trimmed;
      } else {
        return;
      }

      // For Neo4j we just set the bolt URL; for redis, use custom parsing
      if (scheme === 'bolt' || scheme === 'neo4j' || scheme === 'neo4j+s' || scheme === 'neo4j+ssc') {
        // Also try to extract host/port for the form
        try {
          const parsed = new URL(trimmed);
          host = parsed.hostname || 'localhost';
          port = parsed.port ? parseInt(parsed.port) : 7687;
          if (parsed.username) username = decodeURIComponent(parsed.username);
          if (parsed.password) password = decodeURIComponent(parsed.password);
        } catch { /* boltUrl is enough */ }
        return;
      }

      // Parse with URL API
      // Redis URLs use `:password@` instead of `user:password@`, handle that
      if (scheme === 'redis' || scheme === 'rediss') {
        try {
          const parsed = new URL(trimmed);
          host = parsed.hostname || 'localhost';
          port = parsed.port ? parseInt(parsed.port) : 6379;
          // Redis: password is in the "username" slot if format is redis://:pass@host
          if (parsed.password) {
            password = decodeURIComponent(parsed.password);
          } else if (parsed.username) {
            password = decodeURIComponent(parsed.username);
          }
          // Database number from path
          const dbNum = parsed.pathname.replace(/^\//, '');
          if (dbNum) database = dbNum;
        } catch { /* ignore parse errors */ }
        return;
      }

      const parsed = new URL(trimmed);
      host = parsed.hostname || 'localhost';
      if (parsed.port) port = parseInt(parsed.port);
      if (parsed.username) username = decodeURIComponent(parsed.username);
      if (parsed.password) password = decodeURIComponent(parsed.password);

      // Database from pathname
      const dbPath = parsed.pathname.replace(/^\//, '');
      if (dbPath) database = dbPath;

      // Check query params for SSL
      const sslMode = parsed.searchParams.get('sslmode') || parsed.searchParams.get('ssl-mode');
      if (sslMode && sslMode !== 'disable') {
        useSsl = true;
      }

      // Auto-generate name if empty
      if (!name) {
        name = `${dbType} - ${host}`;
      }
    } catch {
      // Gracefully ignore parse errors
    }
  }

  function buildConfig(): ConnectionConfig {
    const config: ConnectionConfig = {
      id: editConnection?.id ?? uuidv4(),
      name,
      db_type: dbType,
      use_ssl: useSsl,
    };

    if (group.trim()) config.group = group.trim();
    if (color) config.color = color;

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
      config.username = username;
      config.password = password;
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
      if (database) config.database = database;
      if (bigqueryCredentials.trim()) {
        config.cloud_auth = {
          GcpServiceAccount: {
            credentials_json: bigqueryCredentials.trim(),
          },
        };
      }
    }

    // SSH tunneling
    if (sshEnabled && meta.requiresHost) {
      config.ssh_enabled = true;
      config.ssh_host = sshHost;
      config.ssh_port = sshPort;
      config.ssh_user = sshUser;
      if (sshPassword) config.ssh_password = sshPassword;
      if (sshKeyPath) config.ssh_key_path = sshKeyPath;
      if (sshPassphrase) config.ssh_passphrase = sshPassphrase;
    }

    // SSL certificates
    if (showSslCerts) {
      if (sslCaCert) config.ssl_ca_cert = sslCaCert;
      if (sslClientCert) config.ssl_client_cert = sslClientCert;
      if (sslClientKey) config.ssl_client_key = sslClientKey;
    }

    // Keychain
    if (useKeychain && keychainAvailable) {
      config.use_keychain = true;
    }

    // Pool tuning — only set if non-default
    if (poolMaxConnections !== 5) config.pool_max_connections = poolMaxConnections;
    if (poolIdleTimeout !== 300) config.pool_idle_timeout_secs = poolIdleTimeout;
    if (poolAcquireTimeout !== 10) config.pool_acquire_timeout_secs = poolAcquireTimeout;

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
    if (dbType === 'BigQuery' && !database.trim()) {
      uiStore.showError('BigQuery project ID is required');
      return;
    }
    if (dbType === 'BigQuery' && !bigqueryCredentials.trim()) {
      uiStore.showError('BigQuery service account credentials are required');
      return;
    }

    isSaving = true;
    try {
      const config = buildConfig();

      // Store password in keychain if enabled
      if (useKeychain && keychainAvailable && password) {
        await storeKeychainPassword(config.id, password);
        config.password = undefined; // Don't persist in JSON
      }

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

  const certFileFilters = [{ name: 'Certificates', extensions: ['pem', 'crt', 'cer', 'key', 'p12', 'pfx'] }];
  const keyFileFilters = [{ name: 'Key Files', extensions: ['pem', 'key', 'ppk', 'pub'] }];

  async function browseSslFile(setter: (v: string) => void) {
    const selected = await open({
      multiple: false,
      filters: certFileFilters,
    });
    if (selected) setter(selected as string);
  }

  async function browseSshKey() {
    const selected = await open({
      multiple: false,
      filters: keyFileFilters,
    });
    if (selected) sshKeyPath = selected as string;
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
      <!-- Connection URL Input -->
      <div class="form-group">
        <label for="conn-url">Connection URL (paste to auto-fill)</label>
        <input
          id="conn-url"
          type="text"
          bind:value={connectionUrl}
          oninput={() => parseConnectionUrl(connectionUrl)}
          placeholder="postgres://user:pass@host:5432/db"
        />
      </div>

      <div class="section-divider"></div>

      <div class="form-group">
        <label for="conn-name">Connection Name</label>
        <input
          id="conn-name"
          type="text"
          bind:value={name}
          placeholder="My Database"
        />
      </div>

      <div class="form-group">
        <label for="conn-group">Group (optional)</label>
        <input
          id="conn-group"
          type="text"
          bind:value={group}
          placeholder="e.g. Production, Staging"
          list="conn-groups"
        />
        <datalist id="conn-groups">
          {#each connectionStore.groups as g}
            <option value={g}></option>
          {/each}
        </datalist>
      </div>

      <div class="form-group">
        <span>Color (optional)</span>
        <div class="color-palette" role="group" aria-label="Connection color">
          <button
            class="color-swatch no-color"
            class:selected={!color}
            onclick={() => { color = ''; }}
            title="No color"
            type="button"
          >
            <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
              <path d="M3 3l10 10M13 3L3 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
          {#each COLOR_PALETTE as c}
            <button
              class="color-swatch"
              class:selected={color === c}
              style="background: {c}"
              onclick={() => { color = c; }}
              title={c}
              type="button"
            ></button>
          {/each}
        </div>
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
            <div class="password-row">
              <input
                id="conn-pass"
                type="password"
                bind:value={password}
                placeholder="********"
              />
              {#if keychainAvailable}
                <label class="keychain-toggle" title="Store password in OS keychain">
                  <input type="checkbox" bind:checked={useKeychain} />
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                    <rect x="3" y="7" width="10" height="7" rx="1.5" stroke="currentColor" stroke-width="1.2"/>
                    <path d="M5 7V5a3 3 0 016 0v2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
                  </svg>
                </label>
              {/if}
            </div>
            {#if useKeychain}
              <span class="field-hint">Password stored in OS keychain</span>
            {/if}
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
            <label for="conn-sf-user">Username</label>
            <input
              id="conn-sf-user"
              type="text"
              bind:value={username}
              placeholder="user"
            />
          </div>
          <div class="form-group">
            <label for="conn-sf-pass">Password</label>
            <div class="password-row">
              <input
                id="conn-sf-pass"
                type="password"
                bind:value={password}
                placeholder="********"
              />
              {#if keychainAvailable}
                <label class="keychain-toggle" title="Store password in OS keychain">
                  <input type="checkbox" bind:checked={useKeychain} />
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                    <rect x="3" y="7" width="10" height="7" rx="1.5" stroke="currentColor" stroke-width="1.2"/>
                    <path d="M5 7V5a3 3 0 016 0v2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
                  </svg>
                </label>
              {/if}
            </div>
            {#if useKeychain}
              <span class="field-hint">Password stored in OS keychain</span>
            {/if}
          </div>
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

      {#if dbType === 'BigQuery'}
        <div class="form-group">
          <label for="conn-bq-creds">Service Account JSON</label>
          <textarea
            id="conn-bq-creds"
            bind:value={bigqueryCredentials}
            placeholder='Paste service account JSON key...'
            rows="5"
            class="credentials-textarea"
          ></textarea>
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

      <!-- SSL Certificate Configuration -->
      {#if showSslCerts}
        <div class="section-divider"></div>
        <div class="section-header">SSL Certificates</div>

        <div class="form-group">
          <label for="ssl-ca">CA Certificate</label>
          <div class="file-picker-row">
            <input
              id="ssl-ca"
              type="text"
              bind:value={sslCaCert}
              placeholder="Path to CA certificate (.pem, .crt)"
              readonly
            />
            <button class="btn browse-btn" type="button" onclick={() => browseSslFile((v) => { sslCaCert = v; })}>Browse</button>
          </div>
        </div>

        <div class="form-group">
          <label for="ssl-cert">Client Certificate</label>
          <div class="file-picker-row">
            <input
              id="ssl-cert"
              type="text"
              bind:value={sslClientCert}
              placeholder="Path to client certificate (.pem, .crt)"
              readonly
            />
            <button class="btn browse-btn" type="button" onclick={() => browseSslFile((v) => { sslClientCert = v; })}>Browse</button>
          </div>
        </div>

        <div class="form-group">
          <label for="ssl-key">Client Key</label>
          <div class="file-picker-row">
            <input
              id="ssl-key"
              type="text"
              bind:value={sslClientKey}
              placeholder="Path to client key (.pem, .key)"
              readonly
            />
            <button class="btn browse-btn" type="button" onclick={() => browseSslFile((v) => { sslClientKey = v; })}>Browse</button>
          </div>
        </div>
      {/if}

      <!-- Advanced Pool Settings -->
      <div class="section-divider"></div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="section-header clickable" onclick={() => { showAdvanced = !showAdvanced; }}>
        <span>Advanced</span>
        <svg
          class="chevron"
          class:expanded={showAdvanced}
          width="12"
          height="12"
          viewBox="0 0 16 16"
          fill="none"
        >
          <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>

      {#if showAdvanced}
        <div class="form-row">
          <div class="form-group">
            <label for="pool-max">Max Connections</label>
            <input
              id="pool-max"
              type="number"
              min="1"
              max="50"
              bind:value={poolMaxConnections}
            />
          </div>
          <div class="form-group">
            <label for="pool-idle">Idle Timeout (s)</label>
            <input
              id="pool-idle"
              type="number"
              min="10"
              max="3600"
              bind:value={poolIdleTimeout}
            />
          </div>
          <div class="form-group">
            <label for="pool-acquire">Acquire Timeout (s)</label>
            <input
              id="pool-acquire"
              type="number"
              min="5"
              max="60"
              bind:value={poolAcquireTimeout}
            />
          </div>
        </div>
      {/if}

      <!-- SSH Tunnel Configuration -->
      {#if meta.requiresHost}
        <div class="section-divider"></div>
        <div class="form-row">
          <div class="form-group" style="flex: 1">
            <label for="ssh-toggle">SSH Tunnel</label>
            <div class="toggle-wrapper">
              <label class="toggle">
                <input type="checkbox" bind:checked={sshEnabled} />
                <span class="slider"></span>
              </label>
              <span class="toggle-label">{sshEnabled ? 'Enabled' : 'Disabled'}</span>
            </div>
          </div>
        </div>

        {#if sshEnabled}
          <div class="form-row">
            <div class="form-group" style="flex: 2">
              <label for="ssh-host">SSH Host</label>
              <input
                id="ssh-host"
                type="text"
                bind:value={sshHost}
                placeholder="bastion.example.com"
              />
            </div>
            <div class="form-group" style="flex: 1">
              <label for="ssh-port">SSH Port</label>
              <input
                id="ssh-port"
                type="number"
                bind:value={sshPort}
                min="1"
                max="65535"
              />
            </div>
          </div>

          <div class="form-group">
            <label for="ssh-user">SSH Username</label>
            <input
              id="ssh-user"
              type="text"
              bind:value={sshUser}
              placeholder="ubuntu"
            />
          </div>

          <div class="form-group">
            <label for="ssh-password">SSH Password</label>
            <input
              id="ssh-password"
              type="password"
              bind:value={sshPassword}
              placeholder="Leave empty if using key"
            />
          </div>

          <div class="form-group">
            <label for="ssh-key">Private Key File</label>
            <div class="file-picker-row">
              <input
                id="ssh-key"
                type="text"
                bind:value={sshKeyPath}
                placeholder="Path to SSH private key"
                readonly
              />
              <button class="btn browse-btn" type="button" onclick={browseSshKey}>Browse</button>
            </div>
          </div>

          {#if sshKeyPath}
            <div class="form-group">
              <label for="ssh-passphrase">Key Passphrase</label>
              <input
                id="ssh-passphrase"
                type="password"
                bind:value={sshPassphrase}
                placeholder="Leave empty if key is not encrypted"
              />
            </div>
          {/if}
        {/if}
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
    width: 560px;
    max-height: 85vh;
  }

  .section-divider {
    height: 1px;
    background: var(--border-color);
    margin: 12px 0 8px;
  }

  .section-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .section-header.clickable {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    user-select: none;
  }

  .section-header.clickable:hover {
    color: var(--text-secondary);
  }

  .chevron {
    transition: transform var(--transition-fast);
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .color-palette {
    display: flex;
    gap: 6px;
    align-items: center;
    padding-top: 2px;
  }

  .color-swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: border-color var(--transition-fast), transform var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .color-swatch:hover {
    transform: scale(1.15);
  }

  .color-swatch.selected {
    border-color: var(--text-primary);
  }

  .color-swatch.no-color {
    background: var(--bg-tertiary);
    border-color: var(--border-color);
    color: var(--text-muted);
  }

  .color-swatch.no-color.selected {
    border-color: var(--text-primary);
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

  .password-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .password-row input {
    flex: 1;
  }

  .keychain-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--text-muted);
    transition: color var(--transition-fast), background var(--transition-fast);
    flex-shrink: 0;
  }

  .keychain-toggle:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .keychain-toggle input {
    display: none;
  }

  .keychain-toggle:has(input:checked) {
    color: var(--accent);
  }

  .field-hint {
    font-size: 11px;
    color: var(--accent);
    margin-top: 2px;
  }

  .file-picker-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .file-picker-row input {
    flex: 1;
  }

  .browse-btn {
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .browse-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: transparent;
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

  .credentials-textarea {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
    resize: vertical;
    min-height: 80px;
  }
</style>
