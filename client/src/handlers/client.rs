use crate::{cmd, data::ACTIVE_SESSION, utils::perform_handshake};
use common::net::{StreamReader, StreamWriter};

use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn handle_client(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let _session_key = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok(session_key) => {
            println!("✅ Handshake successful");
            session_key
        }
        Err(e) => {
            println!("❌ Handshake failed: {}", e);
            return Err("❌ Handshake failed".into());
        }
    };

    // 1. Task to receive incoming messages
    let r_clone = rd.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];

        loop {
            let mut guard = r_clone.lock().await;
            let n = match guard.read(&mut buf).await {
                Ok(0) => {
                    break; // connection closed
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    break;
                }
            };
            drop(guard); // release read lock immediately
            let msg = String::from_utf8_lossy(&buf[..n]).to_string();

            println!("\n📥 {}", msg); // Show incoming msg
            print!("{}", get_prompt());
        }
    });

    // 2. Task to read user input and send messages
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("{}", get_prompt());

        io::stdout().flush().unwrap();
        input.clear();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.starts_with('/') {
            cmd::process_command(input, rd.clone(), wt.clone()).await;
        } else {
            match ACTIVE_SESSION.lock().unwrap().as_ref() {
                Some(session) => {
                    let msg = format!("{:?}> {:?}", session.mode, input);
                    let mut writer = wt.lock().await;
                    writer.write_all(msg.as_bytes()).await.unwrap();
                }
                None => {
                    println!("❗ Not connected to any DM or Group");
                    continue;
                }
            };
        }
    }
}

fn get_prompt() -> String {
    match ACTIVE_SESSION.lock().unwrap().as_ref() {
        Some(session) => format!("{:?}> ", session.mode),
        None => format!("> "),
    }
}
