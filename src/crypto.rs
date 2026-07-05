use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fmt::Write;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12; // AES-GCM nonce length
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

/// Derives a 32-byte AES key from a password and salt using PBKDF2
fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Encrypts plaintext using a master password.
/// Returns a hex-encoded string format: `salt:nonce:ciphertext`
pub fn encrypt_credential(plaintext: &str, master_pass: &str) -> Result<String, String> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key_bytes = derive_key(master_pass, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; 12 bytes

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = String::new();
    write!(
        &mut result,
        "{}:{}:{}",
        hex::encode(salt),
        hex::encode(nonce),
        hex::encode(ciphertext)
    )
    .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Decrypts a hex-encoded string formatted as `salt:nonce:ciphertext` using a master password.
pub fn decrypt_credential(encrypted_data: &str, master_pass: &str) -> Result<String, String> {
    let parts: Vec<&str> = encrypted_data.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid encrypted format".into());
    }

    let salt = hex::decode(parts[0]).map_err(|_| "Invalid salt hex")?;
    let nonce_bytes = hex::decode(parts[1]).map_err(|_| "Invalid nonce hex")?;
    let ciphertext = hex::decode(parts[2]).map_err(|_| "Invalid ciphertext hex")?;

    if salt.len() != SALT_LEN || nonce_bytes.len() != NONCE_LEN {
        return Err("Invalid salt or nonce length".into());
    }

    let key_bytes = derive_key(master_pass, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Decryption failed (Wrong password or corrupted data)")?;

    String::from_utf8(plaintext_bytes).map_err(|_| "Invalid UTF-8 in decrypted data".into())
}
