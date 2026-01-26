use crate::utils::types::AsyncStream;
use rustls::{ClientConfig, pki_types::ServerName};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpStream, time::timeout};
use tokio_rustls::TlsConnector;

pub fn create_tls_connector() -> TlsConnector {
    // Load the root certificates from the webpki-roots crate
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    // Create the client configuration
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // Create the async TLS connector
    TlsConnector::from(Arc::new(config))
}

pub async fn try_tls_handshake(
    hostname: String,
    stream: TcpStream,
) -> Option<Box<dyn AsyncStream>> {
    // Convert the hostname string to a ServerName
    let domain = match ServerName::try_from(hostname) {
        Ok(name) => name,
        Err(_) => return None,
    };

    let tls_connector = create_tls_connector();
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
