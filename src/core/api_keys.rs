use zeroize::Zeroize;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use dirs_next::config_dir;
use std::fs;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use rand::{RngCore, rngs::OsRng};
use base64::Engine;

/// Return the base configuration directory (respects `AEONMI_CONFIG_DIR` env override).
fn config_base_dir() -> PathBuf {
    if let Ok(base) = std::env::var("AEONMI_CONFIG_DIR") {
        return PathBuf::from(base);
    }
    config_dir().unwrap_or_else(std::env::temp_dir).join("aeonmi")
}

/// Path to the per-installation random entropy file used to strengthen key derivation.
/// Storing a random salt on disk means the encryption key is not reconstructable
/// from username/hostname alone — an attacker must also read this file.
fn salt_path() -> PathBuf {
    config_base_dir().join("keystore.salt")
}

/// Load the 32-byte per-installation random salt, or generate and persist it on
/// first use.  If the file cannot be created (e.g. read-only filesystem) the
/// function returns a fresh random salt for this process invocation; keys stored
/// in that case will be unrecoverable after the process exits (the user will need
/// to re-enter them), but we never panic.
fn load_or_create_install_salt() -> [u8; 32] {
    let path = salt_path();
    // Ensure the parent directory exists.
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    // Try to read an existing salt.
    if let Ok(bytes) = fs::read(&path) {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            return arr;
        }
    }
    // Generate a fresh 32-byte cryptographically-random salt.
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    // Persist it so the same salt is used across invocations.
    let _ = fs::write(&path, &salt);
    salt
}

fn key_material() -> [u8; 32] {
    // Mix in a random per-installation salt so the key is not reconstructable
    // from username + hostname alone (both are non-secret public values).
    let install_salt = load_or_create_install_salt();
    let user = whoami::username();
    // Prefer fallible hostname API; fall back to placeholder without using deprecated call.
    let host = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
    // Static domain-separation constant — version suffix changed from v1 to v2
    // because key derivation now includes `install_salt`.
    let salt_static = b"AEONMI_API_KEY_SALT_v2";
    #[cfg(feature = "kdf-argon2")]
    {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        // Combine install_salt with user+host+static for a 32-byte argon2 salt.
        let mut hasher = Sha256::new();
        hasher.update(user.as_bytes());
        hasher.update(host.as_bytes());
        hasher.update(salt_static);
        hasher.update(&install_salt);
        let full = hasher.finalize();
        let salt_bytes = &full[..16];
        // `encode_b64` only fails if the input is empty; our 16-byte slice is
        // always valid.  If it does fail (environment issue), fall back to the
        // SHA256 path to avoid a panic.
        let salt = match SaltString::encode_b64(salt_bytes) {
            Ok(s) => s,
            Err(_) => return fallback_sha256(&user, &host, salt_static, &install_salt),
        };
        let argon = Argon2::default();
        let mut key = [0u8; 32];
        let password = format!("{}:{}", user, host);
        if argon.hash_password_into(password.as_bytes(), salt.as_salt(), &mut key).is_err() {
            return fallback_sha256(&user, &host, salt_static, &install_salt);
        }
        return key;
    }
    #[cfg(not(feature = "kdf-argon2"))]
    {
        fallback_sha256(&user, &host, salt_static, &install_salt)
    }
}

fn fallback_sha256(user: &str, host: &str, static_salt: &[u8], install_salt: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_bytes());
    hasher.update(host.as_bytes());
    hasher.update(static_salt);
    // Include the per-installation random salt so the key is not derivable
    // from username + hostname + the known static constant alone.
    hasher.update(install_salt);
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out[..32]);
    arr
}

fn storage_path() -> PathBuf {
    config_base_dir().join("keys.json")
}

const KEY_FORMAT_VERSION: u32 = 1; // increment if structure changes

pub fn set_api_key(provider: &str, key: &str) -> Result<(), String> {
    let mut data = load_all_raw();
    let key_bytes = key_material();
    let mut nonce = [0u8;12]; OsRng.fill_bytes(&mut nonce);
    let mut cipher = ChaCha20::new((&key_bytes).into(), (&nonce).into());
    let mut buf = key.as_bytes().to_vec();
    cipher.apply_keystream(&mut buf);
    let mut stored = Vec::with_capacity(12+buf.len());
    stored.extend_from_slice(&nonce); stored.extend_from_slice(&buf);
    buf.zeroize();
    // Store as JSON object (string) embedding version marker so future migrations possible
    let entry = serde_json::json!({
        "v": KEY_FORMAT_VERSION,
        "alg": "ChaCha20",
    "nonce_ct_b64": base64::engine::general_purpose::STANDARD.encode(&stored)
    });
    data.insert(provider.to_string(), entry.to_string());
    save_all_raw(&data)
}

pub fn get_api_key(provider: &str) -> Option<String> {
    let data = load_all_raw(); let raw = data.get(provider)?;
    // Backwards compatibility: either plain base64 (legacy) or JSON object
    let (b64, _ver) = if raw.trim_start().starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
            if let Some(nc) = val.get("nonce_ct_b64").and_then(|v| v.as_str()) { (nc.to_string(), val.get("v").and_then(|v| v.as_u64()).unwrap_or(0) as u32) } else { ("".to_string(), 0) }
        } else { ("".to_string(), 0) }
    } else { (raw.clone(), 0) };
    if b64.is_empty() { return None; }
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?; if bytes.len()<13 { return None; }
    let mut nonce=[0u8;12]; nonce.copy_from_slice(&bytes[..12]);
    let mut ct = bytes[12..].to_vec();
    let key_bytes = key_material();
    let mut cipher = ChaCha20::new((&key_bytes).into(), (&nonce).into());
    cipher.apply_keystream(&mut ct);
    let s = String::from_utf8_lossy(&ct).to_string(); ct.zeroize(); Some(s)
}

pub fn delete_api_key(provider: &str) -> Result<(), String> { let mut data = load_all_raw(); data.remove(provider); save_all_raw(&data) }

pub fn list_providers() -> Vec<String> { let data = load_all_raw(); data.keys().cloned().collect() }

fn load_all_raw() -> std::collections::HashMap<String,String> {
    let path = storage_path();
    if let Ok(txt) = fs::read_to_string(path) { serde_json::from_str(&txt).unwrap_or_default() } else { Default::default() }
}
fn save_all_raw(map: &std::collections::HashMap<String, String>) -> Result<(), String> {
    let path = storage_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(map).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// Re-encrypt all stored provider keys with current key_material (e.g., after enabling new KDF feature)
pub struct RotationReport { pub attempted: usize, pub rotated: usize, pub errors: Vec<(String,String)> }

pub fn rotate_all_keys() -> Result<RotationReport, String> {
    let data = load_all_raw();
    let providers: Vec<String> = data.keys().cloned().collect();
    let mut rotated = 0usize; let mut errors = Vec::new();
    for prov in providers.iter() {
        match get_api_key(prov) {
            Some(plain) => {
                if let Err(e) = set_api_key(prov, &plain) { errors.push((prov.clone(), e)); } else { rotated += 1; }
            },
            None => { errors.push((prov.clone(), "decrypt_failed".to_string())); }
        }
    }
    Ok(RotationReport { attempted: providers.len(), rotated, errors })
}
