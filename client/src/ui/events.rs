use crate::{
    data,
    net::{cmd::process_command, dm::send_dm},
    types::{AppState, EditorMode, Panels},
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::Style,
};
use std::sync::MutexGuard;
use tui_textarea::{CursorMove, TextArea};

/// ### Handles user input events for the application.
///
/// This function will process key events and update the application state accordingly.
pub async fn handle_events(key: KeyEvent) {
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
    mut app: MutexGuard<'_, AppState>,
) {
    let shutdown_tx = {
        let channels = data::CHANNELS.get().unwrap();
        channels.shutdown_tx.clone()
    };

    match modifier {
        KeyModifiers::NONE => match code {
            KeyCode::Char('i') => {
                app.switch_panel(Panels::Main);
                app.switch_mode(EditorMode::INSERT);
            }
            KeyCode::Char('/') => {
                app.switch_panel(Panels::Main);
                app.switch_mode(EditorMode::COMMAND);
            }
            KeyCode::Left | KeyCode::Char('h') => app.switch_panel(Panels::SideBar),
            KeyCode::Right | KeyCode::Char('l') => app.switch_panel(Panels::Main),
            KeyCode::Down | KeyCode::Char('j') => match app.active_panel {
                Panels::Main => {
                    app.msg_auto_scroll = false;
                    app.message_state.select_next();
                }
                Panels::SideBar => app.session_state.select_next(),
            },
            KeyCode::Up | KeyCode::Char('k') => match app.active_panel {
                Panels::Main => {
                    app.msg_auto_scroll = false;
                    app.message_state.select_previous();
                }
                Panels::SideBar => app.session_state.select_previous(),
            },
            KeyCode::End | KeyCode::Char('g') => match app.active_panel {
                Panels::Main => {
                    app.msg_auto_scroll = true;
                    app.message_state.select_last();
                }
                Panels::SideBar => app.session_state.select_last(),
            },
            KeyCode::Enter => {
                if app.active_panel == Panels::SideBar {
                    if let Some(selected) = app.session_state.selected() {
                        let sessions = app
                            .sessions
                            .iter()
                            .map(|entry| entry.key().clone())
                            .collect::<Vec<String>>();

                        match Some(sessions[selected].clone()) {
                            Some(session_id) => {
                                app.active_session = Some(session_id);
                            }
                            None => (),
                        };
                    }

                    app.switch_panel(Panels::Main);
                }
            }
            _ => (),
        },
        KeyModifiers::CONTROL => match code {
            KeyCode::Char('c') => {
                let _ = shutdown_tx.send(true);
            }
            _ => (),
        },
        _ => (),
    }
}

/// ### Handles key events in insert and cmd mode.
async fn handle_insert_cmd_mode(
    code: KeyCode,
    modifier: KeyModifiers,
    mut app: MutexGuard<'_, AppState>,
) {
    let shutdown_tx = {
        let channels = data::CHANNELS.get().unwrap();
        channels.shutdown_tx.clone()
    };

    match modifier {
        KeyModifiers::NONE => match code {
            KeyCode::Esc => {
                app.switch_mode(EditorMode::NORMAL);
                app.input = TextArea::default();
            }
            KeyCode::Backspace => {
                app.input.delete_char();
            }
            KeyCode::Delete => {
                app.input.delete_next_char();
            }
            KeyCode::Left => app.input.move_cursor(CursorMove::Back),
            KeyCode::Right => app.input.move_cursor(CursorMove::Forward),
            KeyCode::Down => app.input.move_cursor(CursorMove::Down),
            KeyCode::Up => app.input.move_cursor(CursorMove::Up),
            KeyCode::Tab => {
                app.input.insert_tab();
            }
            KeyCode::Char(c) => app.input.insert_char(c),
            KeyCode::Enter => {
                let input = app.input.lines().join("\n").trim().to_string();
                if input.is_empty() {
                    return;
                }

                match app.mode {
                    EditorMode::COMMAND => {
                        if input == "q" {
                            let _ = shutdown_tx.send(true);
                        } else {
                            process_command(&input, &mut app).await;
                        }
                    }
                    EditorMode::INSERT => {
                        if let Some(session) = &app.active_session {
                            send_dm(input, session.clone()).await;
                        }
                    }
                    _ => (),
                };
                app.input = TextArea::default();
                app.switch_mode(EditorMode::NORMAL);
            }
            _ => (),
        },
        KeyModifiers::ALT => match code {
            KeyCode::Backspace => {
                app.input.delete_word();
            }
            KeyCode::Left => app.input.move_cursor(CursorMove::WordEnd),
            KeyCode::Right => app.input.move_cursor(CursorMove::WordForward),
            KeyCode::Down => app.input.move_cursor(CursorMove::ParagraphBack),
            KeyCode::Up => app.input.move_cursor(CursorMove::ParagraphForward),
            KeyCode::Enter => {
                app.input.insert_newline();
            }
            _ => (),
        },
        KeyModifiers::CONTROL => match code {
            KeyCode::Char('c') => {
                let _ = shutdown_tx.send(true);
            }
            _ => (),
        },
        KeyModifiers::SHIFT => match code {
            KeyCode::Char(c) => {
                app.input.insert_char(c.to_ascii_uppercase());
            }
            _ => (),
        },
        _ => (),
    }
}
