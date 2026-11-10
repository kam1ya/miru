use crate::model::{InputMode, ProcessData, SortCriterion, StatusMessage, SystemSnapshot};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::TableState;
use std::collections::{HashMap, HashSet};
use sysinfo::{Pid, Signal, System};

pub struct App {
    pub cpu_total_history: Vec<(f64, f64)>,
    pub latest_snapshot: Option<SystemSnapshot>,
    pub display_processes: Vec<ProcessData>,
    pub table_state: TableState,
    pub search_query: String,
    pub input_mode: InputMode,
    pub sort_by: SortCriterion,
    pub is_tree_view: bool,
    pub show_details: bool,
    pub target_to_kill: Option<ProcessData>,
    pub status_msg: Option<StatusMessage>,
    pub should_quit: bool,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        let initial_history: Vec<(f64, f64)> = (0..60).map(|i| (i as f64, 0.0)).collect();

        let mut app = Self {
            cpu_total_history: initial_history,
            latest_snapshot: None,
            display_processes: Vec::new(),
            table_state: TableState::default(),
            search_query: String::new(),
            input_mode: InputMode::Normal,
            sort_by: SortCriterion::Cpu,
            is_tree_view: false,
            show_details: false,
            target_to_kill: None,
            status_msg: None,
            should_quit: false,
            matcher: SkimMatcherV2::default(),
        };

        app.table_state.select(Some(0));
        app
    }

    pub fn apply_snapshot(&mut self, snapshot: SystemSnapshot) {
        // Обновляем график истории CPU
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

    pub fn rebuild_display_list(&mut self) {
        let snapshot = match &self.latest_snapshot {
            Some(s) => s,
            None => return,
        };

        let mut list = snapshot.processes.clone();

        // Если активен поиск (по имени или порту)
        if !self.search_query.trim().is_empty() {
            let query = self.search_query.trim();
            let is_port_search = query.starts_with(':');

            if is_port_search {
                let target_port: Option<u16> = query.trim_start_matches(':').parse().ok();
                if let Some(port) = target_port {
                    list.retain(|p| p.listening_ports.contains(&port));
                }
            } else {
                let mut matched: Vec<(i64, ProcessData)> = list
                    .into_iter()
                    .filter_map(|p| {
                        self.matcher
                            .fuzzy_match(&p.name, query)
                            .map(|score| (score, p))
                    })
                    .collect();
                matched.sort_by(|a, b| b.0.cmp(&a.0));
                list = matched.into_iter().map(|(_, p)| p).collect();
            }

            self.display_processes = list;
        } else if self.is_tree_view {
            // Построение дерева
            self.display_processes = Self::build_process_tree(list);
        } else {
            // Обычный плоский отсортированный список
            Self::sort_processes(self.sort_by, &mut list);
            self.display_processes = list;
        }

        // Фиксация курсора
        let len = self.display_processes.len();
        if len == 0 {
            self.table_state.select(None);
        } else {
            let current = self.table_state.selected().unwrap_or(0);
            if current >= len {
                self.table_state.select(Some(len - 1));
            }
        }
    }

    fn build_process_tree(processes: Vec<ProcessData>) -> Vec<ProcessData> {
        let mut by_pid: HashMap<Pid, ProcessData> = HashMap::new();
        let mut children_map: HashMap<Pid, Vec<Pid>> = HashMap::new();
        let all_pids: HashSet<Pid> = processes.iter().map(|p| p.pid).collect();

        for p in processes {
            if let Some(ppid) = p.parent_pid {
                if all_pids.contains(&ppid) {
                    children_map.entry(ppid).or_default().push(p.pid);
                }
            }
            by_pid.insert(p.pid, p);
        }

        // Корневые процессы (у кого нет родителя в текущем списке)
        let mut roots: Vec<Pid> = by_pid
            .values()
            .filter(|p| p.parent_pid.map_or(true, |parent| !all_pids.contains(&parent)))
            .map(|p| p.pid)
            .collect();

        // Сортируем корни по имени
        roots.sort_by(|a, b| {
            by_pid[a].name.to_lowercase().cmp(&by_pid[b].name.to_lowercase())
        });

        let mut result = Vec::new();

        fn dfs(
            pid: Pid,
            depth: usize,
            is_last: bool,
            by_pid: &HashMap<Pid, ProcessData>,
            children_map: &HashMap<Pid, Vec<Pid>>,
            result: &mut Vec<ProcessData>,
        ) {
            if let Some(p) = by_pid.get(&pid) {
                let mut node = p.clone();
                node.depth = depth;
                node.is_last_child = is_last;

                let empty = Vec::new();
                let children = children_map.get(&pid).unwrap_or(&empty);
                node.children_count = children.len();
                result.push(node);

                for (idx, &child_pid) in children.iter().enumerate() {
                    let last = idx == children.len() - 1;
                    dfs(child_pid, depth + 1, last, by_pid, children_map, result);
                }
            }
        }

        for (idx, root) in roots.iter().enumerate() {
            let last = idx == roots.len() - 1;
            dfs(*root, 0, last, &by_pid, &children_map, &mut result);
        }

        result
    }

    fn sort_processes(sort_by: SortCriterion, list: &mut [ProcessData]) {
        match sort_by {
            SortCriterion::Cpu => {
                list.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortCriterion::Memory => {
                list.sort_by(|a, b| {
                    b.memory_mb
                        .partial_cmp(&a.memory_mb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortCriterion::Disk => {
                list.sort_by(|a, b| {
                    (b.disk_read_sec + b.disk_write_sec)
                        .partial_cmp(&(a.disk_read_sec + a.disk_write_sec))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortCriterion::Pid => {
                list.sort_by(|a, b| a.pid.as_u32().cmp(&b.pid.as_u32()));
            }
        }
    }

    pub fn set_sort(&mut self, criterion: SortCriterion) {
        self.sort_by = criterion;
        self.rebuild_display_list();
    }

    pub fn get_selected_process(&self) -> Option<&ProcessData> {
        self.table_state
            .selected()
            .and_then(|idx| self.display_processes.get(idx))
    }

    pub fn prompt_kill_selected(&mut self) {
        if let Some(target) = self.get_selected_process() {
            self.target_to_kill = Some(target.clone());
            self.input_mode = InputMode::ConfirmKill;
        }
    }

    pub fn confirm_kill(&mut self) {
        if let Some(target) = self.target_to_kill.take() {
            // Для убийства процесса используем локальный легкий System::new()
            let mut s = System::new().expect("Failed to initialize local System instance");
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

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

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
    }

    pub fn move_cursor_to_top(&mut self) {
        if !self.display_processes.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn move_cursor_to_bottom(&mut self) {
        if !self.display_processes.is_empty() {
            self.table_state.select(Some(self.display_processes.len() - 1));
        }
    }
}