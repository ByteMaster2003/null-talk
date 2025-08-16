use std::collections::{HashMap, hash_map::Entry};

use crate::{
    data,
    handlers::{add_group_member, create_new_group, get_session, new_connection, rm_connection},
    types::{LogLevel, LogMessage},
    utils::app::update_session,
};
use common::{
    net::{StreamReader, StreamWriter},
    utils::enc::public_key_to_user_id,
};

pub struct CommandInfo {
    pub name: String,
    pub desc: String,
    pub usage: String,
}

pub async fn process_command(cmd: &str, rd: StreamReader, wt: StreamWriter) {
    let commands = HashMap::from([
        (
            "rmc",
            CommandInfo {
                name: "rmc".into(),
                desc: "Remove a connection".into(),
                usage: "rmc <chat_id>".into(),
            },
        ),
        (
            "new",
            CommandInfo {
                name: "new".into(),
                desc: "Create a new connection".into(),
                usage: "new <file_path>".into(),
            },
        ),
        (
            "mkgp",
            CommandInfo {
                name: "mkgp".into(),
                desc: "Create a new group".into(),
                usage: "mkgp <file_path>".into(),
            },
        ),
        (
            "addgpm",
            CommandInfo {
                name: "addgpm".into(),
                desc: "Add a user to a group".into(),
                usage: "addgpm <user_id>".into(),
            },
        ),
        (
            "my-id",
            CommandInfo {
                name: "my-id".into(),
                desc: "Get your user ID".into(),
                usage: "my-id".into(),
            },
        ),
        (
            "chat",
            CommandInfo {
                name: "chat".into(),
                desc: "Start a chat".into(),
                usage: "chat <chat_id>".into(),
            },
        ),
    ]);

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.len() == 2 && parts[1] == "-h" {
        match commands.get(parts[0]) {
            Some(info) => {
                let _ = LogMessage::log(LogLevel::INFO, format!("{}: {}", info.name, info.desc), 0)
                    .await;
            }
            None => {
                let _ = LogMessage::log(LogLevel::ERROR, "Unknown command".to_string(), 5).await;
            }
        };
    }

    match parts[0] {
        "help" => {
            let cmds = commands.keys().cloned().collect::<Vec<&str>>().join(", ");
            let _ =
                LogMessage::log(LogLevel::INFO, format!("Available commands: {}", cmds), 0).await;
        }
        "rmc" => {
            if parts.len() < 2 {
                let _ = LogMessage::log(
                    LogLevel::ERROR,
                    format!(
                        "{}: {}",
                        commands.get("rmc").unwrap().name,
                        commands.get("rmc").unwrap().desc
                    ),
                    0,
                )
                .await;
                return;
            }
            rm_connection(parts[1]).await;
        }
        "new" => {
            if parts.len() < 2 {
                let _ = LogMessage::log(
                    LogLevel::ERROR,
                    format!(
                        "{}: {}",
                        commands.get("new").unwrap().name,
                        commands.get("new").unwrap().desc
                    ),
                    0,
                )
                .await;
                return;
            }
            match new_connection(parts[1], rd.clone(), wt.clone()).await {
                Some(session) => {
                    let mut s_list = data::SESSIONS.lock().await;

                    let key = session.id.clone();
                    match s_list.entry(key.clone()) {
                        Entry::Occupied(_) => (),
                        Entry::Vacant(entry) => {
                            entry.insert(session.clone());
                        }
                    };

                    let mut session_lock = data::ACTIVE_SESSION.lock().await;
                    *session_lock = Some(session.clone());

                    update_session(session.clone());
                    let _ = LogMessage::log(
                        LogLevel::INFO,
                        format!("New connection created successfully: {}", &session.id[..8]),
                        0,
                    )
                    .await;
                }
                None => (),
            };
        }
        "mkgp" => {
            if parts.len() < 2 {
                let _ = LogMessage::log(
                    LogLevel::ERROR,
                    format!(
                        "{}: {}",
                        commands.get("mkgp").unwrap().name,
                        commands.get("mkgp").unwrap().desc
                    ),
                    0,
                )
                .await;
                return;
            }
            match create_new_group(parts[1], rd.clone(), wt.clone()).await {
                Some(session) => {
                    let mut s_list = data::SESSIONS.lock().await;
                    s_list.insert(session.id.clone(), session.clone());

                    let mut session_lock = data::ACTIVE_SESSION.lock().await;
                    *session_lock = Some(session.clone());

                    update_session(session.clone());
                    let _ = LogMessage::log(
                        LogLevel::INFO,
                        format!("New group created successfully: {}", &session.id[..8]),
                        5,
                    )
                    .await;
                }
                None => {
                    let _ = LogMessage::log(
                        LogLevel::ERROR,
                        "Failed to create new group".to_string(),
                        5,
                    )
                    .await;
                    return;
                }
            };
        }
        "addgpm" => {
            if parts.len() < 2 {
                let _ = LogMessage::log(
                    LogLevel::ERROR,
                    format!(
                        "{}: {}",
                        commands.get("addgpm").unwrap().name,
                        commands.get("addgpm").unwrap().desc
                    ),
                    0,
                )
                .await;
                return;
            }
            add_group_member(parts[1], rd.clone(), wt.clone()).await;
        }
        "my-id" => {
            let config = data::CLIENT_CONFIG.lock().await;
            match config.as_ref() {
                Some(cfg) => {
                    let user_id = public_key_to_user_id(&cfg.public_key);

                    let _ =
                        LogMessage::log(LogLevel::INFO, format!("Your user_id: {}", user_id), 0)
                            .await;
                }
                None => {
                    let _ =
                        LogMessage::log(LogLevel::ERROR, format!("Failed to retrieve user_id"), 0)
                            .await;
                }
            }
        }
        "chat" => {
            if parts.len() < 2 {
                let _ = LogMessage::log(
                    LogLevel::ERROR,
                    format!(
                        "{}: {}",
                        commands.get("chat").unwrap().name,
                        commands.get("chat").unwrap().desc
                    ),
                    0,
                )
                .await;
                return;
            }
            match get_session(parts[1]).await {
                Some(session) => {
                    let mut session_lock = data::ACTIVE_SESSION.lock().await;
                    *session_lock = Some(session.clone());

                    update_session(session.clone());
                    let _ = LogMessage::log(
                        LogLevel::INFO,
                        format!("Switched to chat: {}", &session.id[..8]),
                        0,
                    )
                    .await;
                }
                None => {
                    let _ =
                        LogMessage::log(LogLevel::ERROR, format!("Session not found!"), 0).await;
                    return;
                }
            };
        }
        cmd => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Unknown command: {}", cmd), 0).await;
        }
    }
}
