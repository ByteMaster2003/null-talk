use crate::{
    data::CLIENT_CONFIG,
    types::{LogLevel, LogMessage},
};
use common::{
    net::{AsyncStream, HandshakePacket, StreamReader, StreamWriter},
    utils::{enc as encutils, net as netutils},
};
use rsa::RsaPrivateKey;
use rustls::{ClientConfig, pki_types::ServerName};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    time::{Duration, timeout},
};
use tokio_rustls::TlsConnector;

/// ### Perform the initial handshake with the server.
///
/// This function handles the entire handshake process, including
/// sending the initial handshake packet, receiving the server's
/// response, and completing the handshake by establishing a
/// secure session key.
pub async fn perform_handshake(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let username: String;
    let public_key: String;
    let private_key: RsaPrivateKey;

    // Get user credentials from config
    {
        let config = CLIENT_CONFIG.lock().await;
        username = match config.as_ref() {
            Some(cfg) => cfg.name.clone(),
            None => return Err("❗️Failed to get username from config".into()),
        };
        public_key = match config.as_ref() {
            Some(cfg) => encutils::to_ssh_public_key(&cfg.public_key)
                .to_openssh()
                .unwrap(),
            None => return Err("❗️Failed to get public key from config".into()),
        };
        private_key = match config.as_ref() {
            Some(cfg) => cfg.private_key.clone(),
            None => return Err("❗️Failed to get private key from config".into()),
        };
    }

    // Step: 0
    // Send handshake packet with username and public_key
    let packet = HandshakePacket {
        step: 0,
        username: Some(username),
        public_key: Some(public_key),
        nonce: None,
        signature: None,
        session_key: None,
    };
    netutils::write_packet(wt.clone(), packet).await?;

    // Step: 1
    // Receive handshake packet from server with nonce
    // Sign the nonce with private_key
    let packet: HandshakePacket = netutils::read_packet(rd.clone()).await?;
    assert!(packet.step == 1);
    let nonce = match packet.nonce {
        Some(nonce) => nonce.clone(),
        None => return Err("❗️Failed to get nonce from handshake packet".into()),
    };
    let signature = encutils::sign_nonce(&private_key, &nonce);

    // Step: 2
    // Send handshake packet with signature
    let packet = HandshakePacket {
        step: 2,
        username: None,
        public_key: None,
        nonce: None,
        signature: Some(signature),
        session_key: None,
    };
    netutils::write_packet(wt.clone(), packet).await?;

    // Step: 3
    // Receive handshake packet with session_key
    let packet: HandshakePacket = netutils::read_packet(rd.clone()).await?;
    assert!(packet.step == 3);
    let session_key = packet
        .session_key
        .ok_or("❗️Failed to get session_key from handshake packet")?;

    Ok(session_key)
}

pub async fn create_tls_connector() -> Result<TlsConnector, Box<dyn std::error::Error + Send + Sync>>
{
    // Load the root certificates from the webpki-roots crate
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    // Create the client configuration
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // Create the async TLS connector
    let connector = TlsConnector::from(Arc::new(config));
    Ok(connector)
}

pub async fn try_tls_handshake(
    host_name: String,
    stream: TcpStream,
) -> Option<Box<dyn AsyncStream>> {
    // Convert the hostname string to a ServerName
    let domain = match ServerName::try_from(host_name) {
        Ok(name) => name,
        Err(_) => return None,
    };

    let tls_connector = match create_tls_connector().await {
        Ok(connector) => connector,
        Err(e) => {
            LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to create TLS connector: {}", e),
                0,
            )
            .await;
            return None;
        }
    };

    let timeout_duration = Duration::from_secs(10); // 10-second timeout

    // The handshake future
    let handshake_future = tls_connector.connect(domain, stream);

    // Apply the timeout
    let result = timeout(timeout_duration, handshake_future).await;

    let tls = match result {
        Ok(Ok(tls)) => tls,
        _ => return None,
    };

    Some(Box::new(tls))
}
