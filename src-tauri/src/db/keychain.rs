use log::{debug, warn};

use crate::error::AppError;

const SERVICE_NAME: &str = "com.dataforge.database-ide";

pub fn store_password(connection_id: &str, password: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, connection_id)
        .map_err(|e| AppError::Keychain(format!("Failed to create keyring entry: {}", e)))?;
    entry
        .set_password(password)
        .map_err(|e| AppError::Keychain(format!("Failed to store password: {}", e)))?;
    debug!("Stored password in keychain for '{}'", connection_id);
    Ok(())
}

pub fn get_password(connection_id: &str) -> Option<String> {
    let entry = keyring::Entry::new(SERVICE_NAME, connection_id).ok()?;
    match entry.get_password() {
        Ok(pw) => {
            debug!("Retrieved password from keychain for '{}'", connection_id);
            Some(pw)
        }
        Err(keyring::Error::NoEntry) => None,
        Err(e) => {
            warn!(
                "Failed to get keychain password for '{}': {}",
                connection_id, e
            );
            None
        }
    }
}

pub fn delete_password(connection_id: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, connection_id)
        .map_err(|e| AppError::Keychain(format!("Failed to create keyring entry: {}", e)))?;
    match entry.delete_credential() {
        Ok(()) => {
            debug!("Deleted keychain password for '{}'", connection_id);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keychain(format!(
            "Failed to delete password: {}",
            e
        ))),
    }
}

pub fn is_keychain_available() -> bool {
    let test_entry = keyring::Entry::new(SERVICE_NAME, "__dataforge_probe__");
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
