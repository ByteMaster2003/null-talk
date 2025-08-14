use crate::{
    data::{ACTIVE_SESSION, CLIENT_CONFIG, SESSIONS},
    handlers::{
        add_group_member, create_new_group, get_session, list_connections, new_connection,
        rm_connection,
    },
};
use common::{
    net::{StreamReader, StreamWriter},
    utils::enc::public_key_to_user_id,
};

pub async fn process_command(cmd: &str, rd: StreamReader, wt: StreamWriter) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts[0] {
        "/rmc" => {
            if parts.len() < 2 {
                println!("Usage: /rmc <chat_id>");
                return;
            }
            rm_connection(parts[1]);
        }
        "/ls" => {
            list_connections();
        }
        "/new" => {
            if parts.len() < 2 {
                println!("Usage: /new file_path");
                return;
            }
            match new_connection(parts[1], rd.clone(), wt.clone()).await {
                Some(session) => {
                    let mut s_list = SESSIONS.lock().unwrap();
                    s_list.insert(session.id.clone(), session.clone());

                    let mut session_lock = ACTIVE_SESSION.lock().unwrap();
                    *session_lock = Some(session);

                    println!();
                }
                None => return,
            };
        }
        "/mkgp" => {
            if parts.len() < 2 {
                println!("Usage: /newgroup filepath");
                return;
            }
            match create_new_group(parts[1], rd.clone(), wt.clone()).await {
                Some(session) => {
                    let mut s_list = SESSIONS.lock().unwrap();
                    s_list.insert(session.id.clone(), session.clone());

                    let mut session_lock = ACTIVE_SESSION.lock().unwrap();
                    *session_lock = Some(session.clone());
                }
                None => {
                    println!("Failed to create new group");
                    return;
                }
            };
        }
        "/addgpm" => {
            if parts.len() < 2 {
                println!("Usage: /addgpm <user_id>");
                return;
            }
            add_group_member(parts[1], rd.clone(), wt.clone()).await;
        }
        "/my-id" => {
            let config = CLIENT_CONFIG.lock().unwrap();
            match config.as_ref() {
                Some(cfg) => {
                    let user_id = public_key_to_user_id(&cfg.public_key);
                    println!("Your user ID is: {}", user_id);
                }
                None => {
                    println!("❗️Something went wrong!");
                }
            }
        }
        "/chat" => {
            if parts.len() < 2 {
                println!("Usage: /chat <chat_id>");
                return;
            }
            match get_session(parts[1]) {
                Some(session) => {
                    let mut session_lock = ACTIVE_SESSION.lock().unwrap();
                    *session_lock = Some(session);
                    println!();
                }
                None => {
                    println!("Session not found!");
                    return;
                }
            };
        }
        "/exit" => {
            println!("👋 See you soon! Exiting...");
            std::process::exit(0);
        }
        _ => {
            println!("❌ Unknown command.");
        }
    }
}
