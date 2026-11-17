use crate::config::AppConfig;
use crate::model::{InputMode, ProcessData, SortCriterion, StatusMessage, SystemSnapshot};
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::TableState;
use sysinfo::Pid;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum FocusedPane {
    Table,
    Inspector,
}

pub struct AppState {
    pub config: AppConfig,
    pub cpu_total_history: Vec<(f64, f64)>,
    pub latest_snapshot: Option<SystemSnapshot>,
    pub display_processes: Vec<ProcessData>,
    pub table_state: TableState,
    pub selected_pid: Option<Pid>,
    pub focused_pane: FocusedPane,
    pub search_query: String,
    pub input_mode: InputMode,
    pub sort_by: SortCriterion,
    pub is_tree_view: bool,
    pub show_details: bool,
    pub show_dashboard: bool,
    pub inspector_scroll: u16,
    pub target_to_kill: Option<ProcessData>,
    pub status_msg: Option<StatusMessage>,
    pub should_quit: bool,
    pub pending_g: bool,
    pub pending_ctrl_w: bool,
    pub matcher: SkimMatcherV2,
    pub theme_picker_idx: usize,
    pub previous_preset: String,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::load_or_default();
        let initial_history: Vec<(f64, f64)> = (0..60).map(|i| (i as f64, 0.0)).collect();
        let previous_preset = config.section.preset.clone();

        let mut state = Self {
            config,
            cpu_total_history: initial_history,
            latest_snapshot: None,
            display_processes: Vec::new(),
            table_state: TableState::default(),
            selected_pid: None,
            focused_pane: FocusedPane::Table,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            sort_by: SortCriterion::Cpu,
            is_tree_view: false,
            show_details: false,
            show_dashboard: true,
            inspector_scroll: 0,
            target_to_kill: None,
            status_msg: None,
            should_quit: false,
            pending_g: false,
            pending_ctrl_w: false,
            matcher: SkimMatcherV2::default(),
            theme_picker_idx: 0,
            previous_preset,
        };

        state.table_state.select(Some(0));
        state
    }
}