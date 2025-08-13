use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use pem::parse as parse_pem;
use rsa::{
    BigUint, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::DecodePrivateKey,
    rand_core::{OsRng, RngCore},
    sha2::{Digest, Sha256},
    traits::PublicKeyParts,
};
use ssh_key::{
    PrivateKey as SshPrivateKey, PublicKey as SshPublicKey,
    public::{KeyData, RsaPublicKey as SshRsaPublicKey},
};
use std::error::Error;

use crate::types::{EncryptionConfig, SymmetricAlgo};

pub fn encrypt_message(
    message: &str,
    enc_config: EncryptionConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match enc_config.algo {
        SymmetricAlgo::AES256 => {
            let encryption_key = &enc_config.encryption_key.expect("❗️Missing encryption key");
            let key = Key::<Aes256Gcm>::from_slice(&encryption_key); // Normally random
            let cipher = Aes256Gcm::new(key);

            // Generate a random 96-bit nonce (12 bytes)
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            // Encrypt
            let ciphertext = match cipher.encrypt(nonce, message.as_bytes()) {
                Ok(ct) => ct,
                Err(_) => return Err("❗️Encryption failed".into()),
            };

            // Return nonce + ciphertext as base64 so you can decrypt later
            let mut combined = nonce_bytes.to_vec();
            combined.extend(ciphertext);

            Ok(combined)
        }
        SymmetricAlgo::ChaCha20 => {
            let encryption_key = enc_config.encryption_key.expect("❗️Missing encryption key");
            let key = ChaChaKey::from_slice(&encryption_key);
            let cipher = ChaCha20Poly1305::new(key);

            // Generate a random 96-bit nonce (12 bytes)
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);

            // Encrypt
            let ciphertext = match cipher.encrypt(nonce, message.as_bytes()) {
                Ok(ct) => ct,
                Err(_) => return Err("❗️Encryption failed".into()),
            };

            // Return nonce + ciphertext as base64 so you can decrypt later
            let mut combined = nonce_bytes.to_vec();
            combined.extend(ciphertext);

            Ok(combined)
        }
    }
}

/**
 * Attempts to parse an RSA private key from various formats.
 *
 * Supports:
 * - PKCS#1 PEM: "-----BEGIN RSA PRIVATE KEY-----"
 * - PKCS#8 PEM: "-----BEGIN PRIVATE KEY-----"
 * - Base64 DER (PKCS#1 or PKCS#8)
 * - OpenSSH private key: "-----BEGIN OPENSSH PRIVATE KEY-----"
 */
pub fn parse_private_key(input: &str) -> Result<RsaPrivateKey, Box<dyn Error>> {
    let trimmed = input.trim();

    if trimmed.contains("BEGIN RSA PRIVATE KEY") {
        // PKCS#1 PEM
        let pem = parse_pem(trimmed)?;
        let der_bytes = pem.contents();
        let private_key = RsaPrivateKey::from_pkcs1_der(der_bytes)?;

        Ok(private_key)
    } else if trimmed.contains("BEGIN PRIVATE KEY") {
        // PKCS#8 PEM
        let pem = parse_pem(trimmed)?;
        let der_bytes = pem.contents();
        let private_key = RsaPrivateKey::from_pkcs8_der(der_bytes)?;

        Ok(private_key)
    } else if trimmed.contains("BEGIN OPENSSH PRIVATE KEY") {
        // OpenSSH proprietary format
        let ssh_key = SshPrivateKey::from_openssh(trimmed)?;
        match ssh_key.key_data().rsa() {
            Some(rkp) => {
                let public = &rkp.public;
                let e = BigUint::from_bytes_be(public.e.as_bytes());
                let n = BigUint::from_bytes_be(public.n.as_bytes());

                let private = &rkp.private;
                let d = BigUint::from_bytes_be(private.d.as_bytes());
                let p = BigUint::from_bytes_be(private.p.as_bytes());
                let q = BigUint::from_bytes_be(private.q.as_bytes());

                let rsa_key = match RsaPrivateKey::from_components(n, e, d, vec![p, q]) {
                    Ok(key) => key,
                    Err(_) => return Err("Failed to parse RSA private key".into()),
                };
                Ok(rsa_key)
            }
            None => return Err("OpenSSH key is not RSA".into()),
        }
    } else {
        // Try raw Base64 DER → PKCS#1 first, then PKCS#8
        let der = b64.decode(trimmed)?;
        if let Ok(key) = RsaPrivateKey::from_pkcs1_der(&der) {
            return Ok(key);
        }
        if let Ok(key) = RsaPrivateKey::from_pkcs8_der(&der) {
            return Ok(key);
        }
        return Err("Unsupported private key format".into());
    }
}

pub fn parse_public_key(input: &str) -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    let ssh_pub = input.parse::<SshPublicKey>()?;

    match ssh_pub.key_data().rsa() {
        Some(rsa_pub) => {
            let e = BigUint::from_bytes_be(rsa_pub.e.as_bytes());
            let n = BigUint::from_bytes_be(rsa_pub.n.as_bytes());

            Ok(RsaPublicKey::new(n.clone(), e.clone())?)
        }
        None => Err("Not an RSA public key".into()),
    }
}

pub fn public_key_to_user_id(pub_key: &RsaPublicKey) -> String {
    // Convert public key to DER format
    let der_bytes = pub_key.to_pkcs1_der().unwrap(); // or pkcs8_der if using PKCS#8
    let der_bytes = der_bytes.as_bytes();

    // Hash the DER bytes
    let mut hasher =<Sha256 as Digest>::new();
    hasher.update(der_bytes);
    let hash = hasher.finalize();

    // Convert to hex string
    hex::encode(hash)
}

pub fn to_ssh_public_key(pubkey: &RsaPublicKey) -> SshPublicKey {
    let e = ssh_key::Mpint::from_bytes(pubkey.e().to_bytes_be().as_slice())
        .expect("Failed to convert e");
    let n = ssh_key::Mpint::from_bytes(pubkey.n().to_bytes_be().as_slice())
        .expect("Failed to convert n");

    let ssh_pub = SshRsaPublicKey { n, e };

    SshPublicKey::new(KeyData::Rsa(ssh_pub), "comment".to_string())
}

pub fn sign_nonce(private_key: &RsaPrivateKey, nonce: &[u8]) -> Vec<u8> {
    // Hash the nonce first
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(nonce);
    let hashed_nonce = hasher.finalize();

    // Sign the hashed nonce using PKCS#1 v1.5
    private_key
        .sign(Pkcs1v15Sign::new::<Sha256>(), &hashed_nonce)
        .expect("Failed to sign nonce")
}

pub fn verify_nonce_signature(public_key: &RsaPublicKey, nonce: &[u8], signature: &[u8]) -> bool {
    // Hash the nonce first (must match client's hashing)
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(nonce);
    let hashed_nonce = hasher.finalize();

    // Verify the signature
    public_key
        .verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_nonce, signature)
        .is_ok()
}

pub fn generate_session_data() -> (Vec<u8>, Vec<u8>) {
    let mut session_key = vec![0u8; 32]; // 256-bit key
    let mut nonce = vec![0u8; 12]; // 96-bit nonce

    OsRng.fill_bytes(&mut session_key);
    OsRng.fill_bytes(&mut nonce);

    (session_key, nonce)
}

pub fn hash_string(input: &str) -> String {
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(input);

    let result = hasher.finalize();
    hex::encode(result)
}
