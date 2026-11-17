mod actions;
mod filter;
mod navigation;
mod state;

pub use state::{AppState, FocusedPane};
use crate::config::PRESETS;
use crate::model::{InputMode, StatusMessage};

pub type App = AppState;

impl App {
    pub fn apply_snapshot(&mut self, snapshot: crate::model::SystemSnapshot) {
        self.cpu_total_history.remove(0);
        for pt in &mut self.cpu_total_history {
            pt.0 -= 1.0;
        }
        self.cpu_total_history.push((59.0, snapshot.total_cpu));

        self.latest_snapshot = Some(snapshot);
        self.rebuild_display_list();
    }

    pub fn toggle_tree_view(&mut self) {
        self.is_tree_view = !self.is_tree_view;
        self.rebuild_display_list();
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
        if !self.show_details {
            self.focused_pane = FocusedPane::Table;
            self.inspector_scroll = 0;
        }
    }

    pub fn set_sort(&mut self, criterion: crate::model::SortCriterion) {
        self.sort_by = criterion;
        self.rebuild_display_list();
    }

    pub fn toggle_dashboard(&mut self) {
        self.show_dashboard = !self.show_dashboard;
    }

    pub fn open_theme_picker(&mut self) {
        self.previous_preset = self.config.section.preset.clone();
        self.theme_picker_idx = PRESETS
            .iter()
            .position(|&p| p.eq_ignore_ascii_case(&self.previous_preset))
            .unwrap_or(0);
        self.input_mode = InputMode::ThemePicker;
    }

    pub fn theme_picker_down(&mut self) {
        self.theme_picker_idx = (self.theme_picker_idx + 1) % PRESETS.len();
        self.config.preview_preset(PRESETS[self.theme_picker_idx]);
    }

    pub fn theme_picker_up(&mut self) {
        if self.theme_picker_idx == 0 {
            self.theme_picker_idx = PRESETS.len() - 1;
        } else {
            self.theme_picker_idx -= 1;
        }
        self.config.preview_preset(PRESETS[self.theme_picker_idx]);
    }

    pub fn confirm_theme(&mut self) {
        let chosen = PRESETS[self.theme_picker_idx];
        self.config.apply_and_save_preset(chosen);
        self.status_msg = Some(StatusMessage {
            text: format!("Theme saved: {}", chosen),
            is_error: false,
        });
        self.input_mode = InputMode::Normal;
    }

    pub fn cancel_theme_picker(&mut self) {
        let prev = self.previous_preset.clone();
        self.config.preview_preset(&prev);
        self.input_mode = InputMode::Normal;
    }
}