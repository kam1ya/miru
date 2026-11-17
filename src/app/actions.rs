use crate::app::state::AppState;
use crate::model::{InputMode, StatusMessage};
use sysinfo::{Signal, System};

impl AppState {
    pub fn prompt_kill_selected(&mut self) {
        if let Some(target) = self.get_selected_process() {
            self.target_to_kill = Some(target.clone());
            self.input_mode = InputMode::ConfirmKill;
        }
    }

    pub fn confirm_kill(&mut self) {
        if let Some(target) = self.target_to_kill.take() {
            let mut s = System::new().expect("Failed to initialize System");
            s.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[target.pid]), true);
            if let Some(p) = s.process(target.pid) {
                if p.kill_with(Signal::Kill).unwrap_or(false) {
                    self.status_msg = Some(StatusMessage {
                        text: format!("Terminated PID {} ({})", target.pid, target.name),
                        is_error: false,
                    });
                } else {
                    self.status_msg = Some(StatusMessage {
                        text: format!("Failed to kill PID {}: Access denied", target.pid),
                        is_error: true,
                    });
                }
            } else {
                self.status_msg = Some(StatusMessage {
                    text: format!("PID {} already terminated", target.pid),
                    is_error: false,
                });
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn cancel_kill(&mut self) {
        self.target_to_kill = None;
        self.input_mode = InputMode::Normal;
    }
}