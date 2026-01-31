use crate::{
    data,
    net::dm,
    types::{AppState, LogLevel, LogMessage},
};
use std::{collections::HashMap, sync::MutexGuard};

/// Information about a command.
pub struct CommandInfo {
    pub name: String,
    pub desc: String,
    pub usage: String,
}

/// Processes a command.
pub async fn process_command(cmd: &str, app: &mut MutexGuard<'_, AppState>) {
    let commands = HashMap::from([
        (
            "dm",
            CommandInfo {
                name: "dm".into(),
                desc: "New Direct Message".into(),
                usage: "dm <user_id>".into(),
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
        "dm" => {
            if parts.len() < 2 {
                let dm = commands.get("dm").unwrap();
                let _ = LogMessage::log(LogLevel::ERROR, format!("Usage: {}", dm.usage), 0).await;
                return;
            }
            let user_id = parts[1].to_string();
            match dm::new_dm(user_id).await {
                Some(session) => {
                    let key = session.id.clone();
                    {
                        data::SESSIONS.entry(key.clone()).or_insert(session.clone());
                    }
                    {
                        app.update_session(session.clone());
                    }
                    let _ = LogMessage::log(
                        LogLevel::INFO,
                        format!("New session created successfully: {}", &session.id[..8]),
                        5,
                    )
                    .await;
                }
                None => (),
            };
        }
        "my-id" => {
            let config = data::CLIENT.get().unwrap();
            let _ = LogMessage::log(
                LogLevel::INFO,
                format!("Your user_id: {}", &config.user_id),
                0,
            )
            .await;
        }
        cmd => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Unknown command: {}", cmd), 5).await;
        }
    }
}
