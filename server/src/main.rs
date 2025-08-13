use null_talk_server::{ServerConfig, handlers::handle_client};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() {
    let config = match ServerConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {:?}", e);
            return;
        }
    };
    println!("🔧 Configuration Loaded");

    let server_address = config.get_addr();
    let listener = TcpListener::bind(&server_address).await.unwrap();
    println!("🚀 Server listening on {}", &server_address);

    loop {
        let (stream, socket_addr) = listener.accept().await.unwrap();
        println!("🔗 New connection request: {}\n", socket_addr.ip());

        // let stream = Arc::new(stream);
        let (reader, writer) = stream.into_split();
        let reader = Arc::new(Mutex::new(reader));
        let writer = Arc::new(Mutex::new(writer));
        tokio::spawn(async move { handle_client(reader, writer).await });
    }
}
