use serde::Deserialize;

/**
 * Supported symmetric encryption algorithms.
 */
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum SymmetricAlgo {
    AES256,
    ChaCha20,
}

/**
 * Supported asymmetric encryption algorithms.
 */
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum AsymmetricAlgo {
    RSA2048,
    RSA4096,
}

/**
 * Enum for the encryption method in use (symmetric or asymmetric).
 */
#[derive(Debug, Clone)]
pub enum EncryptionMethod {
    Symmetric(SymmetricAlgo),
    Asymmetric(AsymmetricAlgo),
}

/**
 * In-memory encryption configuration for a session.
 */
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub algo: SymmetricAlgo,
    pub encryption_key: Option<Vec<u8>>,
}
