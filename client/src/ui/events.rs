use std::sync::MutexGuard;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    data,
    types::{AppConfig, EditorMode, Panels},
};

pub async fn handle_events(code: KeyCode, modifier: KeyModifiers) -> Option<String> {
    let app = data::APP_STATE.lock().unwrap();

    match app.mode {
        EditorMode::NORMAL => handle_normal_mode(code, modifier, app).await,
        EditorMode::INSERT | EditorMode::COMMAND => {
            handle_insert_cmd_mode(code, modifier, app).await
        }
    }
}

async fn handle_normal_mode(
    code: KeyCode,
    modifier: KeyModifiers,
    mut app: MutexGuard<'_, AppConfig>,
) -> Option<String> {
    match modifier {
        KeyModifiers::NONE => match code {
            KeyCode::Char('i') => {
                app.switch_mode(EditorMode::INSERT);
                return None;
            }
            KeyCode::Char('/') => {
                app.switch_mode(EditorMode::COMMAND);
                return None;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.switch_panel(Panels::SideBar);
                return None;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.switch_panel(Panels::Main);
                return None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match app.active_panel {
                    Panels::Main => {
                        app.msg_auto_scroll = false;
                        app.message_state.select_next();
                    }
                    Panels::SideBar => app.session_state.select_next(),
                }
                return None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match app.active_panel {
                    Panels::Main => {
                        app.msg_auto_scroll = false;
                        app.message_state.select_previous();
                    }
                    Panels::SideBar => app.session_state.select_previous(),
                }
                return None;
            }
            KeyCode::End | KeyCode::Char('g') => {
                match app.active_panel {
                    Panels::Main => {
                        app.msg_auto_scroll = true;
                        app.message_state.select_last();
                    }
                    Panels::SideBar => app.session_state.select_last(),
                }
                return None;
            }
            KeyCode::Enter => {
                if app.active_panel == Panels::SideBar {
                    if let Some(selected) = app.session_state.selected() {
                        let sessions = app.sessions.keys().collect::<Vec<&String>>();
                        app.active_session = Some(sessions[selected].clone());
                    }
                    app.switch_panel(Panels::Main);
                }
                return None;
            }
            _ => None,
        },
        KeyModifiers::CONTROL => match code {
            KeyCode::Char(c) => {
                app.input.push(c);
                app.cursor_pos += 1;
                return Some("quit".into());
            }
            _ => None,
        },
        _ => None,
    }
}

async fn handle_insert_cmd_mode(
    code: KeyCode,
    modifier: KeyModifiers,
    mut app: MutexGuard<'_, AppConfig>,
) -> Option<String> {
    match modifier {
        KeyModifiers::NONE => match code {
            KeyCode::Esc => {
                app.switch_mode(EditorMode::NORMAL);
                app.input.clear();
                app.cursor_pos = 0;
                return None;
            }
            KeyCode::Backspace => {
                if !app.input.is_empty() {
                    app.input.pop().unwrap();
                    app.cursor_pos -= 1;
                }
                return None;
            }
            KeyCode::Enter => {
                let input = app.input.trim().to_string();

                let tx = match app.mode {
                    EditorMode::COMMAND => {
                        if input == "q" {
                            return Some("quit".into());
                        }
                        let channels = data::CHANNELS.lock().await;
                        channels.cmd_tx.clone()
                    }
                    EditorMode::INSERT => {
                        let channels = data::CHANNELS.lock().await;
                        channels.msg_tx.clone()
                    }
                    _ => unreachable!(),
                };

                let _ = tx.lock().await.send(input).await;
                app.input.clear();
                app.cursor_pos = 0;
                return None;
            }
            KeyCode::Char(c) => {
                app.input.push(c);
                app.cursor_pos += 1;
                return None;
            }
            _ => None,
        },
        KeyModifiers::SUPER => match code {
            KeyCode::Backspace | KeyCode::Delete => {
                app.input.clear();
                app.cursor_pos = 0;
                return None;
            }
            _ => None,
        },
        KeyModifiers::CONTROL => match code {
            KeyCode::Backspace | KeyCode::Delete => {
                app.input.clear();
                app.cursor_pos = 0;
                return None;
            }
            KeyCode::Char(c) => {
                app.input.push(c);
                app.cursor_pos += 1;
                return Some("quit".into());
            }
            _ => None,
        },
        KeyModifiers::ALT => match code {
            KeyCode::Backspace | KeyCode::Delete => {
                if app.cursor_pos > 0 {
                    // Trim up to last space of punctuation
                    let before = &app.input.clone()[..app.cursor_pos as usize];
                    let mut new_before = before
                        .trim_end_matches(|c: char| !c.is_whitespace() && !c.is_ascii_punctuation())
                        .to_string();

                    if new_before.len() != 0 {
                        new_before.pop();
                    }
                    app.input = new_before.to_string();
                    app.cursor_pos = new_before.len() as u16;
                }

                return None;
            }
            _ => None,
        },
        _ => None,
    }
}
