use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rsa::rand_core::{OsRng, RngCore};

type CryptoError = Box<dyn std::error::Error + Send + Sync>;

/// Encrypts message bytes using AES-256-GCM.
/// Returns a Vec containing: [12 bytes of Nonce] + [Ciphertext]
pub fn encrypt_aes(session_key: &[u8], message: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // 1. 32 bytes for AES-256
    let key = Key::<Aes256Gcm>::from_slice(session_key);
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    // 2. Generate a random 12-byte nonce (standard for GCM)
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 3. Encrypt
    let ciphertext = cipher
        .encrypt(nonce, message)
        .map_err(|e| format!("AES encryption failed: {}", e))?;

    // 4. Return nonce + ciphertext as base64 so you can decrypt later
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(combined)
}

/// Decrypts message bytes using AES-256-GCM.
/// Expects data to be in format: [12 bytes Nonce][Ciphertext]
pub fn decrypt_aes(session_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if encrypted_data.len() < 12 {
        return Err("Ciphertext too short (missing nonce)".into());
    }

    // 1. Expand key (must match the encryption expansion logic)
    let key = Key::<Aes256Gcm>::from_slice(session_key);
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    // 2. Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 3. Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("AES decryption failed: {}. Data might be tampered.", e))?;

    Ok(plaintext)
}
