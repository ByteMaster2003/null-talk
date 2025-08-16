use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph},
};

use crate::{data, types::Panels};

pub fn render_side_panel(frame: &mut Frame, area: Rect) {
    let active_panel = {
        let app = data::APP_STATE.lock().unwrap();
        app.active_panel
    };

    let border_color = match active_panel {
        Panels::SideBar => Color::Cyan,
        Panels::Main => Color::White,
    };

    let main_block = Block::new()
        .borders(Borders::ALL)
        .border_style(border_color);

    frame.render_widget(main_block, area);

    // Split side panel
    let (header_area, session_area, _) = split_side_panel(area);

    // Render header, body, footer inside main panel
    render_header(frame, header_area, border_color);
    render_sessions(frame, session_area);
}

fn split_side_panel(area: Rect) -> (Rect, Rect, Rect) {
    let inner_main_area = area.inner(Margin {
        vertical: 0,
        horizontal: 1,
    });
    let main_split = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(4),
    ])
    .split(inner_main_area);

    let header_area = main_split[0];
    let body_area = main_split[1];
    let footer_area = main_split[2];

    (header_area, body_area, footer_area)
}

fn render_header(frame: &mut Frame, header_area: Rect, border_color: Color) {
    let header_inner_area = header_area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });

    let title = Paragraph::new("SESSIONS")
        .style(Style::default().fg(Color::Magenta))
        .alignment(Alignment::Center)
        .block(
            Block::new()
                .borders(Borders::BOTTOM)
                .border_style(border_color),
        );

    frame.render_widget(title, header_inner_area);
}

fn render_sessions(frame: &mut Frame, area: Rect) {
    let sidebar_area = area.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    let (sessions, active_session) = {
        let app = data::APP_STATE.lock().unwrap();
        (app.sessions.clone(), app.active_session.clone())
    };

    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(_, s)| {
            let chat_mode = format!("{:?}", s.1.mode);
            let chat_id = format!("{}", &s.0[..8]);

            // calculate available width
            let total_width = sidebar_area.width as usize;
            let spacing = total_width.saturating_sub(chat_mode.len() + chat_id.len() + 2); // 2 for safety

            let line = Line::from(vec![
                Span::raw(format!("{:?}", s.1.mode)),
                Span::raw(" ".repeat(spacing)), // dynamic padding
                Span::raw(format!("{}", &s.0[..8])),
            ]);

            match &active_session {
                Some(active) => {
                    if s.0.clone() == *active {
                        ListItem::new(line)
                            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
                    } else {
                        ListItem::new(line)
                    }
                }
                _ => ListItem::new(line),
            }
        })
        .collect();

    let list_widget = List::new(items)
        .block(Block::new())
        .highlight_style(Color::Magenta)
        .add_modifier(Modifier::BOLD)
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    {
        let mut app: std::sync::MutexGuard<'_, crate::types::AppConfig> =
            data::APP_STATE.lock().unwrap();
        app.sidebar_scroll_state = app.sidebar_scroll_state.content_length(app.sessions.len());
        app.sidebar_max_scroll = app
            .sessions
            .len()
            .saturating_sub(sidebar_area.height.saturating_sub(2) as usize);

        // Render stateful widget
        frame.render_stateful_widget(list_widget, sidebar_area, &mut app.session_state);
    }
}
