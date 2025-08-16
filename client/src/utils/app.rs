use crate::{
    data,
    types::{AppConfig, EditorMode, Panels, Session},
};
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::{HashMap, hash_map::Entry};

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            mode: EditorMode::NORMAL,
            sessions: HashMap::new(),
            active_session: None,
            session_state: ListState::default(),

            active_panel: Panels::Main,

            sidebar_scroll: 0,
            sidebar_max_scroll: 0,
            sidebar_scroll_state: ScrollbarState::default(),

            scroll: 0,
            max_scroll: 0,
            scroll_state: ScrollbarState::default(),

            input: String::new(),
            cursor_pos: 0,
            log: None,
        }
    }

    pub fn switch_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    pub fn switch_panel(&mut self, panel: Panels) {
        if self.active_panel != panel {
            self.active_panel = panel;
        }
    }

    pub fn reset_session(&mut self) {
        self.active_session = None;
    }

    pub fn current_session(&mut self) -> Option<&Session> {
        self.active_session
            .as_ref()
            .and_then(|id| self.sessions.get(id.as_str()))
    }
}

pub fn update_session(session: Session) {
    let mut app = data::APP_STATE.lock().unwrap();
    let key = session.id.clone();

    match app.sessions.entry(key.clone()) {
        Entry::Occupied(_) => (),
        Entry::Vacant(entry) => {
            entry.insert(session.clone());
        }
    };
    app.active_session = Some(key.clone());
}
