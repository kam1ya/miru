use crate::model::{ProcessData, SortCriterion};
use std::collections::{HashMap, HashSet};
use sysinfo::Pid;

pub fn sort_processes(sort_by: SortCriterion, list: &mut [ProcessData]) {
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
                let io_a = a.disk_read_sec + a.disk_write_sec;
                let io_b = b.disk_read_sec + b.disk_write_sec;
                io_b.partial_cmp(&io_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        SortCriterion::Pid => {
            list.sort_by_key(|p| p.pid.as_u32());
        }
    }
}

pub fn build_sorted_tree(processes: Vec<ProcessData>, sort_by: SortCriterion) -> Vec<ProcessData> {
    let mut by_pid: HashMap<Pid, ProcessData> = HashMap::with_capacity(processes.len());
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

    let mut root_procs: Vec<ProcessData> = by_pid
        .values()
        .filter(|p| p.parent_pid.is_none_or(|parent| !all_pids.contains(&parent)))
        .cloned()
        .collect();

    sort_processes(sort_by, &mut root_procs);

    let mut result = Vec::with_capacity(by_pid.len());

    fn dfs(
        mut node: ProcessData,
        depth: usize,
        is_last: bool,
        sort_by: SortCriterion,
        by_pid: &HashMap<Pid, ProcessData>,
        children_map: &HashMap<Pid, Vec<Pid>>,
        result: &mut Vec<ProcessData>,
    ) {
        node.depth = depth;
        node.is_last_child = is_last;

        let empty = Vec::new();
        let child_pids = children_map.get(&node.pid).unwrap_or(&empty);
        node.children_count = child_pids.len();

        result.push(node);

        let mut child_procs: Vec<ProcessData> = child_pids
            .iter()
            .filter_map(|pid| by_pid.get(pid).cloned())
            .collect();

        sort_processes(sort_by, &mut child_procs);

        let count = child_procs.len();
        for (idx, child) in child_procs.into_iter().enumerate() {
            let last = idx == count - 1;
            dfs(child, depth + 1, last, sort_by, by_pid, children_map, result);
        }
    }

    let root_count = root_procs.len();
    for (idx, root) in root_procs.into_iter().enumerate() {
        let last = idx == root_count - 1;
        dfs(root, 0, last, sort_by, &by_pid, &children_map, &mut result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::Pid;

    fn mock_process(pid: usize, parent: Option<usize>, cpu: f32, mem: f64) -> ProcessData {
        ProcessData {
            pid: Pid::from(pid),
            parent_pid: parent.map(Pid::from),
            name: format!("proc_{}", pid),
            cpu,
            memory_mb: mem,
            disk_read_sec: 0.0,
            disk_write_sec: 0.0,
            exe_path: None,
            cmd: Vec::new(),
            run_time_secs: 10,
            listening_ports: Vec::new(),
            depth: 0,
            is_last_child: false,
            children_count: 0,
        }
    }

    #[test]
    fn test_sort_processes_cpu() {
        let mut list = vec![
            mock_process(1, None, 1.5, 100.0),
            mock_process(2, None, 12.0, 50.0),
            mock_process(3, None, 0.2, 500.0),
        ];

        sort_processes(SortCriterion::Cpu, &mut list);
        assert_eq!(list[0].pid, Pid::from(2));
        assert_eq!(list[1].pid, Pid::from(1));
        assert_eq!(list[2].pid, Pid::from(3));
    }

    #[test]
    fn test_build_sorted_tree_hierarchy() {
        let list = vec![
            mock_process(1, None, 2.0, 100.0),      // Root 1
            mock_process(2, Some(1), 10.0, 50.0),   // Child of 1 (high CPU)
            mock_process(3, Some(1), 1.0, 20.0),    // Child of 1 (low CPU)
            mock_process(4, None, 5.0, 80.0),       // Root 2 (higher CPU than Root 1)
        ];

        let tree = build_sorted_tree(list, SortCriterion::Cpu);

        // Root 2 должен быть первым из-за 5.0% CPU против 2.0% у Root 1
        assert_eq!(tree[0].pid, Pid::from(4));
        assert_eq!(tree[0].depth, 0);

        // Root 1 идёт вторым
        assert_eq!(tree[1].pid, Pid::from(1));
        assert_eq!(tree[1].depth, 0);

        // Дети Root 1 отсортированы по CPU: сначала 2, потом 3
        assert_eq!(tree[2].pid, Pid::from(2));
        assert_eq!(tree[2].depth, 1);
        assert_eq!(tree[3].pid, Pid::from(3));
        assert_eq!(tree[3].depth, 1);
    }
}