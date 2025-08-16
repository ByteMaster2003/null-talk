use std::collections::HashMap;

use ratatui::widgets::{ListState, ScrollbarState};

use crate::types::{LogMessage, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    NORMAL,
    INSERT,
    COMMAND,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panels {
    Main,
    SideBar,
}

pub struct AppConfig {
    pub mode: EditorMode,

    pub sessions: HashMap<String, Session>,
    pub active_session: Option<String>,
    pub session_state: ListState,

    pub active_panel: Panels,

    pub sidebar_scroll: usize,
    pub sidebar_max_scroll: usize,
    pub sidebar_scroll_state: ScrollbarState,

    pub scroll: usize,
    pub max_scroll: usize,
    pub scroll_state: ScrollbarState,

    pub input: String,
    pub cursor_pos: u16,
    pub log: Option<LogMessage>,
}
