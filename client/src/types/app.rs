use crate::types::{LogMessage, Message, Session};
use dashmap::DashMap;
use ratatui::widgets::{ListState, ScrollbarState};
use tui_textarea::TextArea;

/// ### Represents the different modes of the text editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    NORMAL,
    INSERT,
    COMMAND,
}

/// ### Represents the different panels in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panels {
    Main,
    SideBar,
}

/// ### Represents the application configuration.
pub struct AppState {
    /// The current mode of the editor.
    pub mode: EditorMode,
    /// The user ID of the current user.
    pub user_id: String,

    /// The sessions of the current user.
    pub sessions: DashMap<String, Session>,
    /// The active session of the current user.
    pub active_session: Option<String>,
    /// The state of the session list. will store the currently selected session.
    pub session_state: ListState,

    /// The messages of the current user.
    pub messages: Vec<Message>,
    /// The state of the message list. will store the currently selected message.
    pub message_state: ListState,
    /// Whether the message list should auto-scroll.
    pub msg_auto_scroll: bool,

    /// The currently active panel.
    pub active_panel: Panels,

    /// Scroll position for the sidebar.
    pub sidebar_scroll: usize,
    /// Maximum scroll state for the sidebar.
    pub sidebar_max_scroll: usize,
    /// Scroll state for the sidebar.
    pub sidebar_scroll_state: ScrollbarState,

    /// Scroll position for the main content area.
    pub scroll: usize,
    /// Maximum scroll position for the main content area.
    pub max_scroll: usize,
    /// Scroll state for the main content area.
    pub scroll_state: ScrollbarState,

    /// Input text area for the user.
    pub input: TextArea<'static>,
    pub log: Option<LogMessage>,
}

impl AppState {
    /// Initializes a new AppState instance with default values.
    pub fn new() -> Self {
        AppState {
            mode: EditorMode::NORMAL,
            user_id: String::new(),

            sessions: DashMap::new(),
            messages: Vec::new(),
            active_session: None,
            session_state: ListState::default(),
            message_state: ListState::default(),
            msg_auto_scroll: true,

            active_panel: Panels::Main,

            sidebar_scroll: 0,
            sidebar_max_scroll: 0,
            sidebar_scroll_state: ScrollbarState::default(),

            scroll: 0,
            max_scroll: 0,
            scroll_state: ScrollbarState::default(),

            input: TextArea::default(),
            log: None,
        }
    }

    /// Switches the editor mode.
    pub fn switch_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// Switches the active panel.
    pub fn switch_panel(&mut self, panel: Panels) {
        if self.active_panel != panel {
            self.active_panel = panel;
        }
    }

    /// Resets the current session.
    pub fn reset_session(&mut self) {
        self.active_session = None;
    }

    pub fn update_session(&mut self, session: Session) {
        let key = session.id.clone();

        self.sessions.entry(key.clone()).or_insert(session.clone());
        self.active_session = Some(key.clone());
    }

    pub fn set_log(&mut self, message: Option<LogMessage>) {
        self.log = message;
    }

    pub fn set_sessions(
        &mut self,
        sessions: DashMap<String, Session>,
        messages: Vec<Message>,
        user_id: String,
    ) {
        self.sessions = sessions;
        self.messages = messages;
        self.user_id = user_id;
    }

    pub fn current_session(&self) -> Option<Session> {
        match &self.active_session {
            Some(key) => {
                match self.sessions.get(key) {
                    Some(s) => return Some(s.clone()),
                    None => return None,
                };
            }
            None => return None,
        }
    }
}
