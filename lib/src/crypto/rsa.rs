use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
    pkcs1v15::{Signature, SigningKey, VerifyingKey},
    pkcs8::EncodePublicKey,
    rand_core::{OsRng, RngCore},
    sha2::{Digest, Sha256},
    signature::{SignatureEncoding, Signer, Verifier},
};
use ssh_key::{PrivateKey, PublicKey};
use std::{fs, path::Path};

/// Parse Private Key (with optional Passphrase support)
pub fn parse_private_key(path: &Path) -> Result<RsaPrivateKey, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut ssh_sk = PrivateKey::from_openssh(&content)?;

    // Check if the key is encrypted
    if ssh_sk.is_encrypted() {
        let prompt = format!("Enter passphrase for key {:?}: ", path);
        let passphrase = rpassword::prompt_password(prompt)?;
        match ssh_sk.decrypt(passphrase) {
            Ok(key) => {
                ssh_sk = key;
            }
            Err(_) => (),
        };
    }

    let rsa_data = ssh_sk.key_data().rsa().ok_or("Not an RSA private key")?;

    let n = rsa::BigUint::from_bytes_be(rsa_data.public.n.as_bytes());
    let e = rsa::BigUint::from_bytes_be(rsa_data.public.e.as_bytes());
    let d = rsa::BigUint::from_bytes_be(rsa_data.private.d.as_bytes());
    let primes = vec![
        rsa::BigUint::from_bytes_be(rsa_data.private.p.as_bytes()),
        rsa::BigUint::from_bytes_be(rsa_data.private.q.as_bytes()),
    ];

    Ok(RsaPrivateKey::from_components(n, e, d, primes)?)
}

/// Parse Public Key from file
pub fn parse_public_key(path: &Path) -> Result<(RsaPublicKey, String), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let public_key = parse_public_key_from_str(content.clone())?;

    Ok((public_key, content))
}

pub fn parse_public_key_from_str(
    content: String,
) -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    let ssh_pk = PublicKey::from_openssh(&content)?;
    let rsa_data = ssh_pk.key_data().rsa().ok_or("Not an RSA public key")?;

    let n = rsa::BigUint::from_bytes_be(rsa_data.n.as_bytes());
    let e = rsa::BigUint::from_bytes_be(rsa_data.e.as_bytes());

    Ok(RsaPublicKey::new(n, e)?)
}

/// Converts an RSA public key to a user ID.
pub fn public_key_to_user_id(pub_key: &RsaPublicKey) -> String {
    // Convert public key to DER format
    let der_bytes = pub_key.to_public_key_der().unwrap(); // or pkcs8_der if using PKCS#8
    let der_bytes = der_bytes.as_bytes();

    // Hash the DER bytes
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(der_bytes);
    let hash = hasher.finalize();

    // Convert to hex string
    hex::encode(hash)
}

/// Sign bytes with Private Key
pub fn sign_bytes(priv_key: &RsaPrivateKey, data: &[u8]) -> Vec<u8> {
    let hashed_data = hash_bytes(&data);
    let signing_key = SigningKey::<Sha256>::new(priv_key.clone());
    signing_key.sign(&hashed_data).to_vec()
}

/// Verify signature with Public Key
pub fn verify_signature(pub_key: &RsaPublicKey, data: &[u8], signature_bytes: &[u8]) -> bool {
    let verifying_key = VerifyingKey::<Sha256>::new(pub_key.clone());
    let signature = match Signature::try_from(signature_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let hashed_data = hash_bytes(&data);
    verifying_key.verify(&hashed_data, &signature).is_ok()
}

/// Encrypt bytes (session key) with Public Key
pub fn encrypt_bytes(pub_key: &RsaPublicKey, data: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .expect("Encryption failed")
}

/// Decrypt bytes with Private Key
pub fn decrypt_bytes(
    priv_key: &RsaPrivateKey,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decrypted = priv_key.decrypt(Pkcs1v15Encrypt, encrypted_data)?;
    Ok(decrypted)
}

/// Hashes a string using SHA-256.
/// Returns the hash as a hexadecimal string.
pub fn hash_string(input: &str) -> String {
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(input);

    let result = hasher.finalize();
    hex::encode(result)
}

pub fn hash_bytes(input: &[u8]) -> Vec<u8> {
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(input);

    let result = hasher.finalize();
    result.to_vec()
}

pub fn generate_session_data() -> (Vec<u8>, Vec<u8>) {
    let mut session_key = vec![0u8; 32]; // 256-bit key
    let mut nonce = vec![0u8; 12]; // 96-bit nonce

    OsRng.fill_bytes(&mut session_key);
    OsRng.fill_bytes(&mut nonce);

    (session_key, nonce)
}
