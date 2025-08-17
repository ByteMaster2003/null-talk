use std::collections::HashMap;

use common::types::Message;
use ratatui::widgets::{ListState, ScrollbarState};
use tui_textarea::TextArea;

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
    pub user_id: String,

    pub sessions: HashMap<String, Session>,
    pub active_session: Option<String>,
    pub session_state: ListState,
    pub messages: Vec<Message>,
    pub message_state: ListState,
    pub msg_auto_scroll: bool,

    pub active_panel: Panels,

    pub sidebar_scroll: usize,
    pub sidebar_max_scroll: usize,
    pub sidebar_scroll_state: ScrollbarState,

    pub scroll: usize,
    pub max_scroll: usize,
    pub scroll_state: ScrollbarState,

    pub input: TextArea<'static>,
    pub log: Option<LogMessage>,
}
