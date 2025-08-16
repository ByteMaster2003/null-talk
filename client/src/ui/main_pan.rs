use crate::{
    data,
    types::{LogLevel, LogMessage, Panels},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_main_panel(frame: &mut Frame, area: Rect) {
    let active_panel = {
        let app = data::APP_STATE.lock().unwrap();
        app.active_panel
    };

    let border_color = match active_panel {
        Panels::SideBar => Color::White,
        Panels::Main => Color::Cyan,
    };

    let main_block = Block::new()
        .borders(Borders::ALL)
        .border_style(border_color);

    // Split Main Panel
    let (header_area, body_area, footer_area, log_area) = split_main_panel(area);

    // Render header, body, footer inside main panel
    render_main_header(frame, header_area, border_color);
    frame.render_widget(Block::new(), body_area);
    render_main_footer(frame, footer_area);

    {
        let app = data::APP_STATE.lock().unwrap();
        if let Some(log) = &app.log {
            render_log_message(frame, log_area, log);
        }
    };

    frame.render_widget(&main_block, area);
}

fn split_main_panel(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let inner_main_area = area.inner(Margin {
        vertical: 0,
        horizontal: 1,
    });
    let main_split = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .split(inner_main_area);

    let header_area = main_split[0];
    let body_area = main_split[1];
    let footer_area = main_split[2];
    let message_area = main_split[3];

    (header_area, body_area, footer_area, message_area)
}

fn render_main_header(frame: &mut Frame, header_area: Rect, border_color: Color) {
    let inner_header_area = header_area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });

    let active_session = {
        let mut app = data::APP_STATE.lock().unwrap();
        app.current_session().cloned()
    };
    let header_text: String = match active_session {
        Some(session) => format!("{:?}: {}", session.mode, session.id),
        None => format!("No active session"),
    };

    frame.render_widget(
        Paragraph::new(header_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::BOTTOM)
                    .border_style(border_color),
            ),
        inner_header_area,
    );
}

fn render_main_footer(frame: &mut Frame, footer_area: Rect) {
    let inner_footer_area = footer_area.inner(Margin {
        vertical: 0,
        horizontal: 1,
    });

    let footer_split = Layout::horizontal([
        Constraint::Length(9), // left side for input
        Constraint::Min(0),    // right side for editor mode
    ])
    .split(inner_footer_area);

    {
        let app = data::APP_STATE.lock().unwrap();
        frame.render_widget(
            Paragraph::new(format!("{:?}: ", &app.mode))
                .style(Style::default().fg(Color::LightMagenta)),
            footer_split[0],
        );
        frame.render_widget(Paragraph::new(app.input.clone()), footer_split[1]);
        frame.set_cursor_position(Position {
            x: footer_split[1].x + app.cursor_pos as u16,
            y: footer_split[1].y,
        });
    }
}

fn render_log_message(frame: &mut Frame, log_area: Rect, log: &LogMessage) {
    let inner_log_area = log_area.inner(Margin {
        vertical: 0,
        horizontal: 1,
    });

    let log_split = Layout::horizontal([
        Constraint::Max(9), // left side for input
        Constraint::Min(0), // right side for editor mode
    ])
    .split(inner_log_area);

    let color = match log.level {
        LogLevel::INFO => Color::Green,
        LogLevel::ERROR => Color::Red,
    };
    frame.render_widget(
        Paragraph::new(format!("{:?}: ", &log.level)).style(Style::default().fg(color)),
        log_split[0],
    );
    frame.render_widget(Paragraph::new(log.msg.clone()), log_split[1]);
}
