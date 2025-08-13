use crate::{cmd, data::ACTIVE_SESSION, utils::perform_handshake};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    utils::net::read_packet,
};

use std::io::{self, Write};
use tokio::io::AsyncWriteExt;

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
        loop {
            let packet = match read_packet::<Packet>(r_clone.clone()).await {
                Ok(packet) => packet,
                Err(e) => {
                    eprintln!("Failed to read packet: {}", e);
                    break;
                }
            };

            match packet.kind {
                ChatMessageKind::DirectMessage(_) => {
                    println!("Received a dm {}", String::from_utf8_lossy(&packet.payload));
                }
                ChatMessageKind::GroupMessage(_) => {
                    println!("Received a gm {}", String::from_utf8_lossy(&packet.payload));
                }
                _ => {
                    eprintln!("Unknown packet kind");
                }
            }
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
