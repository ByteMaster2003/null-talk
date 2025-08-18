use std::sync::MutexGuard;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::Style,
};
use tui_textarea::{CursorMove, TextArea};

use crate::{
    data,
    types::{AppConfig, EditorMode, Panels},
};

/// ### Handles user input events for the application.
/// 
/// This function will process key events and update the application state accordingly.
pub async fn handle_events(key: KeyEvent) -> Option<String> {
    let code = key.code;
    let modifier = key.modifiers;
    let mut app = data::APP_STATE.lock().unwrap();

    // kill the underline on the current line
    app.input.set_cursor_line_style(Style::default());

    // (optional) make sure your base text style isn't adding underline elsewhere
    app.input.set_style(Style::default());
    app.input.set_tab_length(2);

    match app.mode {
        EditorMode::NORMAL => handle_normal_mode(code, modifier, app).await,
        EditorMode::INSERT | EditorMode::COMMAND => {
            handle_insert_cmd_mode(code, modifier, app).await
        }
    }
}

/// ### Handles key events in normal mode.
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

                        match Some(sessions[selected].clone()) {
                            Some(session_id) => {
                                let s_list = data::SESSIONS.lock().await;
                                match s_list.get(&session_id) {
                                    Some(session) => {
                                        let mut session_lock = data::ACTIVE_SESSION.lock().await;
                                        *session_lock = Some(session.clone());

                                        app.active_session = Some(session_id);
                                    }
                                    None => (),
                                }
                            }
                            None => (),
                        };
                    }

                    app.switch_panel(Panels::Main);
                }
                return None;
            }
            _ => None,
        },
        KeyModifiers::CONTROL => match code {
            KeyCode::Char('c') => return Some("quit".into()),
            _ => None,
        },
        _ => None,
    }
}


/// ### Handles key events in insert and cmd mode.
async fn handle_insert_cmd_mode(
    code: KeyCode,
    modifier: KeyModifiers,
    mut app: MutexGuard<'_, AppConfig>,
) -> Option<String> {
    match modifier {
        KeyModifiers::NONE => match code {
            KeyCode::Esc => {
                app.switch_mode(EditorMode::NORMAL);
                app.input = TextArea::default();
                return None;
            }
            KeyCode::Backspace => {
                app.input.delete_char();
                return None;
            }
            KeyCode::Delete => {
                app.input.delete_next_char();
                return None;
            }
            KeyCode::Left => {
                app.input.move_cursor(CursorMove::Back);
                return None;
            }
            KeyCode::Right => {
                app.input.move_cursor(CursorMove::Forward);
                return None;
            }
            KeyCode::Down => {
                app.input.move_cursor(CursorMove::Down);
                return None;
            }
            KeyCode::Up => {
                app.input.move_cursor(CursorMove::Up);
                return None;
            }
            KeyCode::Tab => {
                app.input.insert_tab();
                return None;
            }
            KeyCode::Char(c) => {
                app.input.insert_char(c);
                return None;
            }
            KeyCode::Enter => {
                let input = app.input.lines().join("\n").trim().to_string();

                if input.is_empty() {
                    return None;
                }

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
                app.input = TextArea::default();
                return None;
            }
            _ => None,
        },
        KeyModifiers::ALT => match code {
            KeyCode::Backspace => {
                app.input.delete_word();
                return None;
            }
            KeyCode::Left => {
                app.input.move_cursor(CursorMove::WordEnd);
                return None;
            }
            KeyCode::Right => {
                app.input.move_cursor(CursorMove::WordForward);
                return None;
            }
            KeyCode::Down => {
                app.input.move_cursor(CursorMove::ParagraphBack);
                return None;
            }
            KeyCode::Up => {
                app.input.move_cursor(CursorMove::ParagraphForward);
                return None;
            }
            KeyCode::Enter => {
                app.input.insert_newline();
                return None;
            }
            _ => None,
        },
        KeyModifiers::CONTROL => match code {
            KeyCode::Char('c') => return Some("quit".into()),
            _ => None,
        },
        KeyModifiers::SHIFT => match code {
            KeyCode::Char(c) => {
                app.input.insert_char(c.to_ascii_uppercase());
                return None;
            }
            _ => None,
        },
        _ => None,
    }
}
