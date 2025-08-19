use crate::TLSConfig;
use common::{
    net::{HandshakePacket, StreamReader, StreamWriter},
    utils::{
        enc::{generate_session_data, parse_public_key, verify_nonce_signature},
        net::{close_connection, read_packet, write_packet},
    },
};
use rsa::RsaPublicKey;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{fs::File, io::BufReader, sync::Arc};
use tokio_rustls::TlsAcceptor;

/// Perform the handshake process with the client
pub async fn perform_handshake(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<(String, Vec<u8>, RsaPublicKey), Box<dyn std::error::Error + Send + Sync>> {
    println!("Trying to receive handshake packet...");
    // Step: 0
    // Receive handshake packet from client with username and public_key
    let packet: HandshakePacket = read_packet(rd.clone()).await?;
    println!("\nReceived handshake packet: {:?}\n", packet);
    if packet.step != 0 {
        eprintln!("❗️ Invalid handshake step: expected 0, got {}", packet.step);
        let _ = close_connection(wt.clone(), "Invalid handshake step").await;
        return Err("Invalid handshake step".into());
    }
    let user_name = match packet.username {
        Some(name) => name,
        None => return Err("❗️Missing username".into()),
    };
    let public_key = match packet.public_key {
        Some(key) => parse_public_key(&key).map_err(|_| "❗️Failed to parse public key")?,
        None => return Err("❗️Missing Public Key".into()),
    };

    println!("Received handshake packet from client: {}", user_name);
    // Step: 1
    // Generate session data (nonce & session_key)
    // Send handshake packet from server with nonce
    let (session_key, nonce) = generate_session_data();

    let packet = HandshakePacket {
        step: 1,
        username: None,
        public_key: None,
        nonce: Some(nonce.clone()),
        signature: None,
        session_key: None,
    };
    println!("\nSending handshake packet with nonce: {:?}\n", nonce);
    write_packet(wt.clone(), packet).await?;

    // Step: 2
    // Receive signature and verify it
    let packet: HandshakePacket = read_packet(rd.clone()).await?;
    if packet.step != 2 {
        eprintln!("❗️ Invalid handshake step: expected 2, got {}", packet.step);
        let _ = close_connection(wt.clone(), "Invalid handshake step").await;
        return Err("Invalid handshake step".into());
    }
    let signature = match packet.signature {
        Some(signature) => signature,
        None => return Err("❗️Missing signature".into()),
    };

    if !verify_nonce_signature(&public_key, &nonce, &signature) {
        eprintln!("❗️ Invalid signature!");
        let _ = close_connection(wt.clone(), "Invalid signature!").await;
        return Err("Invalid signature!".into());
    }

    // Step: 3
    // Send Session Key After Successful Verification
    let packet = HandshakePacket {
        step: 3,
        username: None,
        public_key: None,
        nonce: None,
        signature: None,
        session_key: Some(session_key.clone()),
    };
    write_packet(wt.clone(), packet).await?;

    Ok((user_name, session_key, public_key))
}

pub async fn create_tls_acceptor(
    tls_config: &TLSConfig,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    // Load the certificate chain from `fullchain.pem`
    let cert_file = match File::open(&tls_config.cert_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open certificate file: {}", e).into()),
    };
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
        .filter_map(|r| r.ok())
        .collect();

    // Load the private key from `privkey.pem`
    let key_file = match File::open(&tls_config.key_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open private key file: {}", e).into()),
    };
    let mut key_reader = BufReader::new(key_file);
    let key_pkcs8: PrivateKeyDer = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .next()
        .expect("Failed to read private key")
        .expect("Failed to parse private key")
        .into();

    // Create the server configuration
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key_pkcs8)
        .expect("Failed to create server config");

    // Create the async TLS acceptor
    let acceptor = TlsAcceptor::from(Arc::new(config));

    Ok(acceptor)
}
