use crate::{
    data,
    types::{LogMessage, Session},
    ui::{self, events::handle_events},
};
use common::types::Message;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Layout},
};
use std::{collections::HashMap, time::Duration};

/// ### Runs the terminal UI.
/// 
/// - This function will start the terminal UI and handle user input and events.
/// - It also starts a background task to listen for log messages.
pub async fn run_terminal(mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
    // Start Error updating task
    let error_task = tokio::spawn(async move {
        let log_rx = {
            let channels = data::CHANNELS.lock().await;
            channels.log_rx.clone()
        };

        loop {
            match log_rx.lock().await.recv().await {
                Some(message) => {
                    let hide_after = message.hide_after.clone();
                    set_log(message);

                    if hide_after > Duration::from_secs(0) {
                        tokio::spawn(async move {
                            tokio::time::sleep(hide_after).await;
                            hide_log();
                        });
                    }
                }
                None => break,
            }
        }
    });

    loop {
        update_app_data().await;
        terminal.draw(draw_frame)?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if let Some(cmd) = handle_events(key).await {
                        if cmd == "quit" {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    error_task.abort();
    Ok(())
}

/// ### Updates the application data.
async fn update_app_data() {
    let sessions = {
        let s = data::SESSIONS.lock().await;
        s.clone()
    };
    let messages = {
        let messages = data::MESSAGES.lock().await;

        match get_active_session() {
            Some(ref session) => match messages.get(&session.clone()) {
                Some(msg) => {
                    let m = msg.lock().await;
                    m.to_owned()
                }
                None => Vec::new(),
            },
            None => Vec::new(),
        }
    };
    let user_id = {
        let config = data::CLIENT_CONFIG.lock().await;
        match config.as_ref() {
            Some(cfg) => cfg.user_id.clone(),
            None => String::new(),
        }
    };

    set_sessions(sessions, messages, user_id);
}

/// ### Sets the log message.
fn set_log(message: LogMessage) {
    let mut app = data::APP_STATE.lock().unwrap();
    app.log = Some(message);
}

/// ### Hides the log message.
/// 
/// This function will hide the log message from the UI.
fn hide_log() {
    let mut app = data::APP_STATE.lock().unwrap();
    app.log = None;
}

/// ### Gets the active session.
fn get_active_session() -> Option<String> {
    let app = data::APP_STATE.lock().unwrap();
    app.active_session.clone()
}

/// ### Sets the session data.
fn set_sessions(sessions: HashMap<String, Session>, messages: Vec<Message>, user_id: String) {
    let mut app = data::APP_STATE.lock().unwrap();
    app.sessions = sessions;
    app.messages = messages;
    app.user_id = user_id;
}

/// ### Draws the terminal UI frame.
fn draw_frame(frame: &mut Frame) {
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let [side_panel, main_panel] = layout.areas(frame.area());

    ui::side_pan::render_side_panel(frame, side_panel);
    ui::main_pan::render_main_panel(frame, main_panel);
}
