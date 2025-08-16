use crate::{
    data,
    types::{LogMessage, Session},
    ui::{self, events::handle_events},
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Layout},
};
use std::{collections::HashMap, time::Duration};

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
        let sessions = {
            let s = data::SESSIONS.lock().await;
            s.clone()
        };
        set_sessions(sessions);

        terminal.draw(draw_frame)?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let res = handle_events(key.code, key.modifiers).await;
                if let Some(cmd) = res {
                    if cmd == "quit" {
                        break;
                    }
                }
            }
        }
    }

    error_task.abort();
    Ok(())
}

fn set_log(message: LogMessage) {
    let mut app = data::APP_STATE.lock().unwrap();
    app.log = Some(message);
}

fn hide_log() {
    let mut app = data::APP_STATE.lock().unwrap();
    app.log = None;
}

fn set_sessions(sessions: HashMap<String, Session>) {
    let mut app = data::APP_STATE.lock().unwrap();
    app.sessions = sessions;
}

fn draw_frame(frame: &mut Frame) {
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let [side_panel, main_panel] = layout.areas(frame.area());

    ui::side_pan::render_side_panel(frame, side_panel);
    ui::main_pan::render_main_panel(frame, main_panel);
}
