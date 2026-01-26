use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{fs::File, io::BufReader, sync::Arc};
use tokio_rustls::TlsAcceptor;

pub async fn create_tls_acceptor(
    cert_path: &str,
    key_path: &str,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    // Load the certificate chain from `fullchain.pem`
    let cert_file = match File::open(cert_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open certificate file: {}", e).into()),
    };
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
        .filter_map(|r| r.ok())
        .collect();

    // Load the private key from `privkey.pem`
    let key_file = match File::open(key_path) {
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
