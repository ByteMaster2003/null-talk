use common::utils::{
    enc::{parse_private_key, parse_public_key},
    file::read_file_contents,
};
use null_talk_client::{
    data,
    handlers::handle_client,
    types::ConnectionConfig,
    utils::{parse_client_config, take_file_input, take_user_input},
};
use std::{env, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    configure_client(&args);

    let config_lock = data::CLIENT_CONFIG.lock().unwrap();
    let config = match config_lock.as_ref() {
        Some(cfg) => cfg,
        None => {
            eprintln!("❌ Something went wrong!");
            return;
        }
    };
    let addr = format!("{}:{}", &config.hostname, &config.port);
    drop(config_lock);

    // Try connecting
    match TcpStream::connect(&addr).await {
        Ok(stream) => {
            println!("✅ Successfully connected to {}", addr);

            let (reader, writer) = stream.into_split();
            let reader = Arc::new(Mutex::new(reader));
            let writer = Arc::new(Mutex::new(writer));
            match handle_client(reader, writer).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("❌ Failed to handle client: {}", e);
                    return;
                }
            };
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to {}: {}", addr, e);
            return;
        }
    };
}

fn configure_client(args: &[String]) {
    let mut config = data::CLIENT_CONFIG.lock().unwrap();
    if args.len() == 2 {
        let config_path = &args[1];
        match parse_client_config(config_path) {
            Some(cfg) => *config = Some(cfg),
            None => {
                eprintln!("❌ Failed to load configuration from {}", config_path);
                return;
            }
        };
    } else {
        // Ask connection config
        let hostname = take_user_input("Enter server hostname: ");
        let port = take_user_input("Enter port: ");
        let name = take_user_input("Enter username: ");

        let public_key = take_file_input("Enter public key path: ");
        let rsa_public_key = match read_file_contents(&public_key) {
            Ok(contents) => match parse_public_key(&contents) {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("❌ Failed to parse public key: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("❌ Failed to read public key file: {}", e);
                return;
            }
        };
        let private_key = take_file_input("Enter private key path: ");
        let rsa_private_key = match read_file_contents(&private_key) {
            Ok(contents) => match parse_private_key(&contents) {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("❌ Failed to parse private key: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("❌ Failed to read private key file: {}", e);
                return;
            }
        };

        *config = Some(ConnectionConfig {
            hostname,
            port,
            name,
            public_key: rsa_public_key,
            private_key: rsa_private_key,
        });
    }
}
