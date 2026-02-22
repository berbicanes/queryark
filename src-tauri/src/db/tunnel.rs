use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error, info, warn};
use russh::client;
use russh_keys::key;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::AppError;
use crate::models::connection::ConnectionConfig;

struct SshTunnel {
    local_port: u16,
    task_handle: JoinHandle<()>,
}

pub struct TunnelManager {
    tunnels: Mutex<HashMap<String, SshTunnel>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: Mutex::new(HashMap::new()),
        }
    }

    /// If SSH tunneling is enabled, establishes a tunnel and returns a modified config
    /// pointing to 127.0.0.1:<local_port>. Otherwise returns config unchanged.
    pub async fn ensure_tunnel(
        &self,
        config: &ConnectionConfig,
    ) -> Result<ConnectionConfig, AppError> {
        if !config.ssh_enabled {
            return Ok(config.clone());
        }

        let ssh_host = config
            .ssh_host
            .as_deref()
            .ok_or_else(|| AppError::SshTunnel("SSH host is required".into()))?;
        let ssh_user = config
            .ssh_user
            .as_deref()
            .ok_or_else(|| AppError::SshTunnel("SSH username is required".into()))?;
        let ssh_port = config.ssh_port.unwrap_or(22);

        let remote_host = config.host_or_default().to_string();
        let remote_port = config.port_or_default();

        // Check if we already have a tunnel for this connection
        {
            let tunnels = self.tunnels.lock().await;
            if let Some(existing) = tunnels.get(&config.id) {
                if !existing.task_handle.is_finished() {
                    debug!(
                        "Reusing existing SSH tunnel for '{}' on port {}",
                        config.id, existing.local_port
                    );
                    let mut modified = config.clone();
                    modified.host = Some("127.0.0.1".to_string());
                    modified.port = Some(existing.local_port);
                    modified.ssh_enabled = false; // prevent re-tunneling
                    return Ok(modified);
                }
            }
        }

        info!(
            "Establishing SSH tunnel {}@{}:{} → {}:{}",
            ssh_user, ssh_host, ssh_port, remote_host, remote_port
        );

        // Bind local listener on random port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| AppError::SshTunnel(format!("Failed to bind local port: {}", e)))?;
        let local_addr: SocketAddr = listener
            .local_addr()
            .map_err(|e| AppError::SshTunnel(format!("Failed to get local address: {}", e)))?;
        let local_port = local_addr.port();

        info!("SSH tunnel local port: {}", local_port);

        // Connect to SSH server
        let ssh_config = Arc::new(client::Config::default());
        let ssh_addr = format!("{}:{}", ssh_host, ssh_port);

        let sh = SshHandler;
        let mut session = client::connect(ssh_config, &ssh_addr, sh)
            .await
            .map_err(|e| AppError::SshTunnel(format!("SSH connection failed: {}", e)))?;

        // Authenticate
        let authenticated = if let Some(ref key_path) = config.ssh_key_path {
            let passphrase = config.ssh_passphrase.as_deref();
            match russh_keys::load_secret_key(key_path, passphrase) {
                Ok(key_pair) => {
                    let auth_result = session
                        .authenticate_publickey(ssh_user, Arc::new(key_pair))
                        .await
                        .map_err(|e| {
                            AppError::SshTunnel(format!("SSH key auth failed: {}", e))
                        })?;
                    auth_result
                }
                Err(e) => {
                    warn!("Failed to load SSH key '{}': {}, falling back to password", key_path, e);
                    if let Some(ref pw) = config.ssh_password {
                        session
                            .authenticate_password(ssh_user, pw)
                            .await
                            .map_err(|e| {
                                AppError::SshTunnel(format!("SSH password auth failed: {}", e))
                            })?
                    } else {
                        return Err(AppError::SshTunnel(
                            "SSH key failed to load and no password provided".into(),
                        ));
                    }
                }
            }
        } else if let Some(ref pw) = config.ssh_password {
            session
                .authenticate_password(ssh_user, pw)
                .await
                .map_err(|e| AppError::SshTunnel(format!("SSH password auth failed: {}", e)))?
        } else {
            return Err(AppError::SshTunnel(
                "No SSH authentication method provided (key or password required)".into(),
            ));
        };

        if !authenticated {
            return Err(AppError::SshTunnel("SSH authentication rejected".into()));
        }

        info!("SSH authenticated successfully");

        let session = Arc::new(session);

        // Spawn forwarding task
        let task_handle = {
            let session = Arc::clone(&session);
            let remote_host = remote_host.clone();
            let conn_id = config.id.clone();

            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((mut local_stream, peer_addr)) => {
                            debug!(
                                "SSH tunnel [{}]: accepted connection from {}",
                                conn_id, peer_addr
                            );

                            let session = Arc::clone(&session);
                            let remote_host = remote_host.clone();
                            let conn_id = conn_id.clone();

                            tokio::spawn(async move {
                                match session
                                    .channel_open_direct_tcpip(
                                        &remote_host,
                                        remote_port as u32,
                                        "127.0.0.1",
                                        peer_addr.port() as u32,
                                    )
                                    .await
                                {
                                    Ok(channel) => {
                                        let mut stream = channel.into_stream();
                                        let mut local_buf = vec![0u8; 8192];
                                        let mut remote_buf = vec![0u8; 8192];

                                        loop {
                                            tokio::select! {
                                                result = local_stream.read(&mut local_buf) => {
                                                    match result {
                                                        Ok(0) => break,
                                                        Ok(n) => {
                                                            if stream.write_all(&local_buf[..n]).await.is_err() {
                                                                break;
                                                            }
                                                        }
                                                        Err(_) => break,
                                                    }
                                                }
                                                result = stream.read(&mut remote_buf) => {
                                                    match result {
                                                        Ok(0) => break,
                                                        Ok(n) => {
                                                            if local_stream.write_all(&remote_buf[..n]).await.is_err() {
                                                                break;
                                                            }
                                                        }
                                                        Err(_) => break,
                                                    }
                                                }
                                            }
                                        }
                                        debug!("SSH tunnel [{}]: connection closed", conn_id);
                                    }
                                    Err(e) => {
                                        error!(
                                            "SSH tunnel [{}]: failed to open channel: {}",
                                            conn_id, e
                                        );
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!(
                                "SSH tunnel [{}]: failed to accept connection: {}",
                                conn_id, e
                            );
                            break;
                        }
                    }
                }
            })
        };

        // Store tunnel
        {
            let mut tunnels = self.tunnels.lock().await;
            tunnels.insert(
                config.id.clone(),
                SshTunnel {
                    local_port,
                    task_handle,
                },
            );
        }

        // Return modified config pointing to the tunnel
        let mut modified = config.clone();
        modified.host = Some("127.0.0.1".to_string());
        modified.port = Some(local_port);
        modified.ssh_enabled = false; // prevent re-tunneling
        Ok(modified)
    }

    pub async fn remove_tunnel(&self, connection_id: &str) {
        let mut tunnels = self.tunnels.lock().await;
        if let Some(tunnel) = tunnels.remove(connection_id) {
            tunnel.task_handle.abort();
            info!("SSH tunnel removed for '{}'", connection_id);
        }
    }
}

/// Minimal SSH client handler — accepts all host keys.
struct SshHandler;

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all host keys (similar to SSH StrictHostKeyChecking=no)
        // In a production app, you'd want to verify against known_hosts
        Ok(true)
    }
}
