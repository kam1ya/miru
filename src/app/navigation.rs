use crate::app::state::{AppState, FocusedPane};
use crate::model::ProcessData;

impl AppState {
    pub fn move_cursor_down(&mut self) {
        let len = self.display_processes.len();
        if len == 0 {
            return;
        }
        let next = match self.table_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.table_state.select(Some(next));
        self.sync_selected_pid();
        self.inspector_scroll = 0;
    }

    pub fn move_cursor_up(&mut self) {
        let len = self.display_processes.len();
        if len == 0 {
            return;
        }
        let prev = match self.table_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.table_state.select(Some(prev));
        self.sync_selected_pid();
        self.inspector_scroll = 0;
    }

    pub fn half_page_down(&mut self) {
        if self.focused_pane == FocusedPane::Inspector {
            self.inspector_scroll = self.inspector_scroll.saturating_add(8);
        } else {
            let len = self.display_processes.len();
            if len == 0 {
                return;
            }
            let next = match self.table_state.selected() {
                Some(i) => (i + 10).min(len - 1),
                None => 0,
            };
            self.table_state.select(Some(next));
            self.sync_selected_pid();
            self.inspector_scroll = 0;
        }
    }

    pub fn half_page_up(&mut self) {
        if self.focused_pane == FocusedPane::Inspector {
            self.inspector_scroll = self.inspector_scroll.saturating_sub(8);
        } else {
            let len = self.display_processes.len();
            if len == 0 {
                return;
            }
            let prev = match self.table_state.selected() {
                Some(i) => i.saturating_sub(10),
                None => 0,
            };
            self.table_state.select(Some(prev));
            self.sync_selected_pid();
            self.inspector_scroll = 0;
        }
    }

    pub fn move_cursor_to_top(&mut self) {
        if !self.display_processes.is_empty() {
            self.table_state.select(Some(0));
            self.sync_selected_pid();
            self.inspector_scroll = 0;
        }
    }

    pub fn move_cursor_to_bottom(&mut self) {
        if !self.display_processes.is_empty() {
            self.table_state.select(Some(self.display_processes.len() - 1));
            self.sync_selected_pid();
            self.inspector_scroll = 0;
        }
    }

    pub fn scroll_inspector_down(&mut self) {
        self.inspector_scroll = self.inspector_scroll.saturating_add(1);
    }

    pub fn scroll_inspector_up(&mut self) {
        self.inspector_scroll = self.inspector_scroll.saturating_sub(1);
    }

    pub fn set_focus(&mut self, pane: FocusedPane) {
        if self.show_details {
            self.focused_pane = pane;
        }
    }

    pub fn toggle_pane_focus(&mut self) {
        if self.show_details {
            self.focused_pane = match self.focused_pane {
                FocusedPane::Table => FocusedPane::Inspector,
                FocusedPane::Inspector => FocusedPane::Table,
            };
        }
    }

    pub fn get_selected_process(&self) -> Option<&ProcessData> {
        self.table_state
            .selected()
            .and_then(|idx| self.display_processes.get(idx))
    }

    pub fn sync_selected_pid(&mut self) {
        self.selected_pid = self.get_selected_process().map(|p| p.pid);
    }

    pub fn clamp_cursor(&mut self) {
        let len = self.display_processes.len();
        if len == 0 {
            self.table_state.select(None);
            self.selected_pid = None;
            return;
        }

        if let Some(target_pid) = self.selected_pid {
            if let Some(new_idx) = self.display_processes.iter().position(|p| p.pid == target_pid) {
                self.table_state.select(Some(new_idx));
                return;
            }
        }

        let current = self.table_state.selected().unwrap_or(0);
        let valid_idx = current.min(len - 1);
        self.table_state.select(Some(valid_idx));
        self.sync_selected_pid();
    }
}