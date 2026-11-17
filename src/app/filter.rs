use crate::app::state::AppState;
use crate::model::ProcessData;
use crate::tree::{build_sorted_tree, sort_processes};
use fuzzy_matcher::FuzzyMatcher;

impl AppState {
    pub fn rebuild_display_list(&mut self) {
        let snapshot = match &self.latest_snapshot {
            Some(s) => s,
            None => return,
        };

        let mut list = snapshot.processes.clone();
        let query = self.search_query.trim();

        if !query.is_empty() {
            if query.starts_with(':') {
                let port_str = query.trim_start_matches(':');
                if let Ok(target_port) = port_str.parse::<u16>() {
                    list.retain(|p| p.listening_ports.contains(&target_port));
                } else {
                    list.clear();
                }
                sort_processes(self.sort_by, &mut list);
            } else {
                let mut matched: Vec<(i64, ProcessData)> = list
                    .into_iter()
                    .filter_map(|p| {
                        self.matcher
                            .fuzzy_match(&p.name, query)
                            .map(|score| (score, p))
                    })
                    .collect();
                matched.sort_by_key(|a| std::cmp::Reverse(a.0));
                list = matched.into_iter().map(|(_, p)| p).collect();
            }

            self.display_processes = list;
        } else if self.is_tree_view {
            self.display_processes = build_sorted_tree(list, self.sort_by);
        } else {
            sort_processes(self.sort_by, &mut list);
            self.display_processes = list;
        }

        self.clamp_cursor();
    }
}

#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::model::{ProcessData, SystemSnapshot};
    use sysinfo::Pid;

    fn make_proc(pid: usize, name: &str, ports: Vec<u16>) -> ProcessData {
        ProcessData {
            pid: Pid::from(pid),
            parent_pid: None,
            name: name.to_string(),
            cpu: 1.0,
            memory_mb: 50.0,
            disk_read_sec: 0.0,
            disk_write_sec: 0.0,
            exe_path: None,
            cmd: Vec::new(),
            run_time_secs: 60,
            listening_ports: ports,
            depth: 0,
            is_last_child: false,
            children_count: 0,
        }
    }

    #[test]
    fn test_filter_by_name_and_port() {
        let mut app = App::new();
        app.latest_snapshot = Some(SystemSnapshot {
            total_cpu: 10.0,
            core_usages: vec![10.0],
            total_mem: 1024,
            used_mem: 512,
            total_swap: 0,
            used_swap: 0,
            net_rx_rate: 0.0,
            net_tx_rate: 0.0,
            processes: vec![
                make_proc(100, "postgres.exe", vec![5432]),
                make_proc(200, "rustrover64.exe", vec![52289, 63342]),
                make_proc(300, "chrome.exe", vec![]),
            ],
        });

        // 1. Поиск по имени (fuzzy)
        app.search_query = "post".to_string();
        app.rebuild_display_list();
        assert_eq!(app.display_processes.len(), 1);
        assert_eq!(app.display_processes[0].name, "postgres.exe");

        // 2. Поиск по точному номеру порта через двоеточие
        app.search_query = ":63342".to_string();
        app.rebuild_display_list();
        assert_eq!(app.display_processes.len(), 1);
        assert_eq!(app.display_processes[0].pid, Pid::from(200));

        // 3. Пустой запрос возвращает все процессы
        app.search_query.clear();
        app.rebuild_display_list();
        assert_eq!(app.display_processes.len(), 3);
    }
}