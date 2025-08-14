use crate::{cmd, data::ACTIVE_SESSION, utils::perform_handshake};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::ChatMode,
    utils::{
        enc::{decrypt_message, encrypt_message},
        net::{read_packet, write_packet},
    },
};

use std::io::{self, Write};

pub async fn handle_client(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let _ = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok(session_key) => session_key,
        Err(e) => {
            println!("❌ Handshake failed: {}", e);
            return Err("❌ Handshake failed".into());
        }
    };

    // 1. Task to receive incoming messages
    let mut read_handler = start_reader_thread(rd.clone()).await;

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
            read_handler.abort();
            cmd::process_command(input, rd.clone(), wt.clone()).await;
            read_handler = start_reader_thread(rd.clone()).await;
        } else {
            send_message(wt.clone(), input).await;
        }
    }
}

fn get_prompt() -> String {
    match ACTIVE_SESSION.lock().unwrap().as_ref() {
        Some(session) => format!("{:?}> ", session.mode),
        None => format!("> "),
    }
}

async fn send_message(wt: StreamWriter, input: &str) {
    let session = ACTIVE_SESSION.lock().unwrap();

    match session.as_ref() {
        Some(session) => {
            match encrypt_message(input, session.encryption.clone()) {
                Ok(payload) => {
                    let kind = match session.mode.clone() {
                        ChatMode::Dm(_) => ChatMessageKind::DirectMessage(session.id.clone()),
                        ChatMode::Group(_) => ChatMessageKind::GroupMessage(session.id.clone()),
                    };
                    let packet = Packet { kind, payload };
                    let _ = write_packet::<Packet>(wt.clone(), packet).await;
                }
                Err(err) => {
                    eprintln!("❗️Failed to encrypt message: {}", err);
                }
            };
        }
        None => {
            eprintln!("❗Not connected to any DM or Group");
        }
    };
}

async fn start_reader_thread(rd: StreamReader) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let packet = match read_packet::<Packet>(rd.clone()).await {
                Ok(packet) => packet,
                Err(_) => break,
            };

            match packet.kind {
                ChatMessageKind::DirectMessage(id) => {
                    print_message(id.clone(), packet.payload.clone());
                }
                ChatMessageKind::GroupMessage(id) => {
                    print_message(id.clone(), packet.payload.clone());
                }
                _ => {
                    eprintln!("Unknown packet kind");
                }
            }
            print!("{}", get_prompt());
        }
    })
}

fn print_message(id: String, payload: Vec<u8>) {
    match ACTIVE_SESSION.lock().unwrap().as_ref() {
        Some(session) => {
            let decrypted_msg = match decrypt_message(&payload, session.encryption.clone()) {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("\n{:?}", err);
                    return;
                }
            };
            if session.id == id {
                println!("\n{:?}>{}", session.mode, decrypted_msg);
            }
        }
        None => return,
    };
}
