use crate::{
    data,
    types::LogMessage,
    ui::{self, events::handle_events},
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Layout},
};
use std::time::Duration;
use tokio::sync::mpsc;

/// ### Runs the terminal UI.
///
/// - This function will start the terminal UI and handle user input and events.
/// - It also starts a background task to listen for log messages.
pub async fn run(
    mut terminal: DefaultTerminal,
    mut log_rx: mpsc::Receiver<LogMessage>,
) -> color_eyre::Result<()> {
    let _ = tokio::spawn(async move {
        let mut shutdown_rx = {
            let channels = data::CHANNELS.get().unwrap();
            channels.shutdown_tx.subscribe()
        };

        loop {
            tokio::select! {
            _ = shutdown_rx.recv() => {
                break;
            }

            Some(message) = log_rx.recv() => {
                    let hide_after = message.hide_after.clone();
                    {
                        data::APP_STATE.lock().unwrap().set_log(Some(message));
                    }

                    if hide_after > Duration::from_secs(0) {
                        tokio::spawn(async move {
                            tokio::time::sleep(hide_after).await;
                            {
                                data::APP_STATE.lock().unwrap().set_log(None);
                            }

                        });
                    }
                }
            }
        }
    });

    let mut shutdown_rx = {
        let channels = data::CHANNELS.get().unwrap();
        channels.shutdown_tx.subscribe()
    };

    loop {
        // 1. Update and Draw (Synchronous-ish operations)
        update_app_data().await;
        terminal.draw(draw_frame)?;

        tokio::select! {
            // Branch A: Listen for the Global Shutdown Signal
            _ = shutdown_rx.recv() => {
                break;
            }

            // Branch B: Handle Terminal Events
            // We use a small timeout to ensure the loop stays responsive
            res = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(100))) => {
                if res?? {
                    match event::read()? {
                        Event::Key(key) if key.kind == KeyEventKind::Press => handle_events(key).await,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// ### Updates the application data.
async fn update_app_data() {
    let sessions = data::SESSIONS.clone();
    let messages = {
        let messages = data::MESSAGES.clone();

        match get_active_session().await {
            Some(id) => match messages.get(&id) {
                Some(msg) => msg.to_owned(),
                None => Vec::new(),
            },
            None => Vec::new(),
        }
    };
    let user_id = {
        match data::CLIENT.get() {
            Some(cfg) => cfg.user_id.clone(),
            None => String::new(),
        }
    };

    data::APP_STATE
        .lock()
        .unwrap()
        .set_sessions(sessions, messages, user_id);
}

/// ### Gets the active session.
async fn get_active_session() -> Option<String> {
    let app = data::APP_STATE.lock().unwrap();
    app.active_session.clone()
}

/// ### Draws the terminal UI frame.
fn draw_frame(frame: &mut Frame) {
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let [side_panel, main_panel] = layout.areas(frame.area());

    ui::side_pan::render_side_panel(frame, side_panel);
    ui::main_pan::render_main_panel(frame, main_panel);
}
