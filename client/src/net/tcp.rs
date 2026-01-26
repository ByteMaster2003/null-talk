use crate::{data, net::tls::try_tls_handshake, utils::types::AsyncStream};
use tokio::net::TcpStream;

pub async fn open_tcp_stream() -> Result<Box<dyn AsyncStream>, Box<dyn std::error::Error>> {
    let config = data::CLIENT.get().unwrap().clone();
    let hostname = config.hostname.clone();

    let tcp_stream = connect(&hostname).await;
    if config.tls {
        if let Some(tls_stream) = try_tls_handshake(hostname.clone(), tcp_stream).await {
            return Ok(tls_stream);
        } else {
            let tcp_stream = connect(&hostname).await;
            return Ok(Box::new(tcp_stream));
        }
    }

    return Ok(Box::new(tcp_stream));
}

async fn connect(hostname: &str) -> TcpStream {
    match TcpStream::connect(&hostname).await {
        Ok(plain_stream) => return plain_stream,
        Err(e) => panic!("Error: {}", e.to_string()),
    }
}
