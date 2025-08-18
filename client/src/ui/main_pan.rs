use crate::{
    data,
    types::{LogLevel, LogMessage, Panels},
};
use chrono::DateTime;
use common::types::Message;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph},
};

/// ### Renders the main panel.
/// 
/// This function will render the main panel of the application, including the header, body, footer, and any log messages.
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
    render_messages(frame, body_area);
    render_main_footer(frame, footer_area);

    {
        let app = data::APP_STATE.lock().unwrap();
        if let Some(log) = &app.log {
            render_log_message(frame, log_area, log);
        }
    };

    frame.render_widget(&main_block, area);
}

/// ### Splits the main panel into header, body, footer, and log areas.
fn split_main_panel(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let inner_main_area = area.inner(Margin {
        vertical: 0,
        horizontal: 1,
    });

    let required_height_for_input = {
        let app = data::APP_STATE.lock().unwrap();
        app.input.lines().len()
    };

    let main_split = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(required_height_for_input as u16 + 1),
        Constraint::Length(2),
    ])
    .split(inner_main_area);

    let header_area = main_split[0];
    let body_area = main_split[1];
    let footer_area = main_split[2];
    let message_area = main_split[3];

    (header_area, body_area, footer_area, message_area)
}

/// ### Renders the main header
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
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::BOTTOM)
                    .border_style(border_color),
            ),
        inner_header_area,
    );
}

/// ### Renders the main footer
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

    let (prompt_area, input_area) = (footer_split[0], footer_split[1]);

    {
        let app = data::APP_STATE.lock().unwrap();
        frame.render_widget(
            Paragraph::new(format!("{:?}: ", &app.mode))
                .style(Style::default().fg(Color::LightMagenta)),
            prompt_area,
        );

        frame.render_widget(&app.input.clone(), input_area);
    }
}

/// ### Renders the log message if there is any
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

/// ### Renders the messages in the main panel
/// 
/// This function will render the messages in the main panel, including any user messages and system messages.
fn render_messages(frame: &mut Frame, area: Rect) {
    let message_area = area.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    let (messages, user_id) = {
        let app = data::APP_STATE.lock().unwrap();
        (app.messages.clone(), app.user_id.clone())
    };

    let no_msg_line = Line::from("No Messages Yet!").alignment(Alignment::Center);

    let items: Vec<ListItem> = if messages.is_empty() {
        vec![ListItem::new(no_msg_line)]
    } else {
        messages
            .iter()
            .enumerate()
            .map(|(_, message)| format_message(message.clone(), user_id.clone(), message_area))
            .collect()
    };

    let list_widget = List::new(items)
        .highlight_symbol("| ")
        .highlight_spacing(HighlightSpacing::Always);

    {
        let mut app: std::sync::MutexGuard<'_, crate::types::AppConfig> =
            data::APP_STATE.lock().unwrap();
        app.scroll_state = app.scroll_state.content_length(app.messages.len());
        app.max_scroll = app
            .messages
            .len()
            .saturating_sub(message_area.height.saturating_sub(2) as usize);

        // Render stateful widget
        frame.render_stateful_widget(list_widget, message_area, &mut app.message_state);
        // frame.render_stateful_widget(
        //     Scrollbar::new(ScrollbarOrientation::VerticalRight),
        //     message_area,
        //     &mut app.scroll_state,
        // );

        if app.msg_auto_scroll {
            app.message_state.select_last();
        }
    }
}

/// ### Formats a message for display in the main panel.
fn format_message(message: Message, user_id: String, message_area: Rect) -> ListItem<'static> {
    let msg = message.clone();

    let date_time_string = format_date_time(msg.timestamps);
    let prompt_style = {
        if user_id == msg.sender_id {
            Style::default().fg(Color::LightYellow)
        } else {
            Style::default().fg(Color::Cyan)
        }
    };

    // First line with username and timestamp
    let line1 = Line::from(Span::styled(
        format!(
            "{} | {}",
            msg.username.unwrap_or_else(|| msg.sender_id[..8].into()),
            date_time_string,
        ),
        prompt_style,
    ))
    .alignment(Alignment::Left);

    // Second line message content
    let wrapped_lines = wrap_text_to_width(
        &String::from_utf8_lossy(&msg.content).to_string(),
        message_area.width,
    );

    let mut lines = Vec::new();
    lines.push(line1);
    lines.extend(wrapped_lines);

    ListItem::new(lines.clone())
}

/// ### Formats a date and time for display in the main panel.
/// 
/// # Example
/// ```no_run
/// let formatted = format_date_time(1633036800000);
/// assert_eq!(formatted, "01 Oct 00:00");
/// ```
fn format_date_time(ts: u128) -> String {
    let secs = (ts / 1000) as i64;
    let nsecs = ((ts % 1000) * 1_000_000) as u32;

    let datetime = DateTime::from_timestamp(secs, nsecs).unwrap();

    // Format like "17 Aug 3:41"
    datetime.format("%d %b %-H:%M").to_string()
}

/// ### Wraps text to fit within a specified width.
fn wrap_text_to_width(text: &str, max_width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        // if adding this word would exceed max_width
        if current.len() + word.len() + 1 > (max_width - 2) as usize {
            lines.push(
                Line::from(Span::styled(
                    format!(
                        "{}{}",
                        match lines.len() {
                            0 => "> ",
                            _ => "  ",
                        },
                        current.clone()
                    ),
                    Style::default().fg(Color::Gray),
                ))
                .alignment(Alignment::Left),
            );
            current.clear();
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(
            Line::from(Span::styled(
                format!(
                    "{}{}",
                    match lines.len() {
                        0 => "> ",
                        _ => "  ",
                    },
                    current
                ),
                Style::default().fg(Color::Gray),
            ))
            .alignment(Alignment::Left),
        );
    }

    lines.push(Line::from(""));

    lines
}
