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