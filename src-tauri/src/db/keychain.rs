use log::{debug, warn};

use crate::error::AppError;

const SERVICE_NAME: &str = "com.queryark.database-ide";
const LEGACY_SERVICE_NAME: &str = "com.dataforge.database-ide";

/// All known secret keys that may be stored per connection.
#[allow(dead_code)]
const SECRET_KEYS: &[&str] = &[
    "password",
    "ssh_password",
    "ssh_passphrase",
    "aws_secret_key",
    "credentials_json",
];

/// Build the keyring username for a given connection + secret key.
/// For backward compatibility, key="password" uses the plain connection_id.
fn entry_username(connection_id: &str, key: &str) -> String {
    if key == "password" {
        connection_id.to_string()
    } else {
        format!("{}:{}", connection_id, key)
    }
}

pub fn store_password(connection_id: &str, password: &str) -> Result<(), AppError> {
    store_secret(connection_id, "password", password)
}

pub fn get_password(connection_id: &str) -> Option<String> {
    get_secret(connection_id, "password")
}

pub fn delete_password(connection_id: &str) -> Result<(), AppError> {
    delete_secret(connection_id, "password")
}

/// Store a named secret in the OS keychain.
pub fn store_secret(connection_id: &str, key: &str, value: &str) -> Result<(), AppError> {
    let username = entry_username(connection_id, key);
    let entry = keyring::Entry::new(SERVICE_NAME, &username)
        .map_err(|e| AppError::Keychain(format!("Failed to create keyring entry: {}", e)))?;
    entry
        .set_password(value)
        .map_err(|e| AppError::Keychain(format!("Failed to store secret '{}': {}", key, e)))?;
    debug!("Stored secret '{}' in keychain for '{}'", key, connection_id);
    Ok(())
}

/// Retrieve a named secret from the OS keychain.
/// Falls back to the legacy service name and migrates if found.
pub fn get_secret(connection_id: &str, key: &str) -> Option<String> {
    let username = entry_username(connection_id, key);
    let entry = keyring::Entry::new(SERVICE_NAME, &username).ok()?;
    match entry.get_password() {
        Ok(val) => {
            debug!("Retrieved secret '{}' from keychain for '{}'", key, connection_id);
            Some(val)
        }
        Err(keyring::Error::NoEntry) => {
            // Try legacy service name and migrate if found
            if let Some(legacy_entry) = keyring::Entry::new(LEGACY_SERVICE_NAME, &username).ok() {
                match legacy_entry.get_password() {
                    Ok(val) => {
                        debug!(
                            "Found secret '{}' under legacy service name for '{}', migrating",
                            key, connection_id
                        );
                        // Store under new service name
                        if let Ok(new_entry) = keyring::Entry::new(SERVICE_NAME, &username) {
                            let _ = new_entry.set_password(&val);
                        }
                        // Delete legacy entry
                        let _ = legacy_entry.delete_credential();
                        Some(val)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Err(e) => {
            warn!(
                "Failed to get keychain secret '{}' for '{}': {}",
                key, connection_id, e
            );
            None
        }
    }
}

/// Delete a single secret from the OS keychain.
fn delete_secret(connection_id: &str, key: &str) -> Result<(), AppError> {
    let username = entry_username(connection_id, key);
    let entry = keyring::Entry::new(SERVICE_NAME, &username)
        .map_err(|e| AppError::Keychain(format!("Failed to create keyring entry: {}", e)))?;
    match entry.delete_credential() {
        Ok(()) => {
            debug!("Deleted keychain secret '{}' for '{}'", key, connection_id);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keychain(format!(
            "Failed to delete secret '{}': {}",
            key, e
        ))),
    }
}

/// Delete all known secrets for a connection.
#[allow(dead_code)]
pub fn delete_secrets(connection_id: &str) -> Result<(), AppError> {
    for key in SECRET_KEYS {
        delete_secret(connection_id, key)?;
    }
    Ok(())
}

pub fn is_keychain_available() -> bool {
    let test_entry = keyring::Entry::new(SERVICE_NAME, "__queryark_probe__");
    match test_entry {
        Ok(entry) => {
            // Try a get — NoEntry is fine, other errors mean unavailable
            match entry.get_password() {
                Ok(_) | Err(keyring::Error::NoEntry) => true,
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
