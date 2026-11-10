use std::path::PathBuf;
use sysinfo::Pid;

#[derive(Clone, Debug)]
pub struct ProcessData {
    pub pid: Pid,
    pub parent_pid: Option<Pid>,
    pub name: String,
    pub cpu: f32,
    pub memory_mb: f64,
    pub disk_read_sec: f64,
    pub disk_write_sec: f64,
    pub exe_path: Option<PathBuf>,
    pub cmd: Vec<String>,
    pub run_time_secs: u64,
    pub listening_ports: Vec<u16>,
    pub depth: usize,
    pub is_last_child: bool,
    pub children_count: usize,
}

#[derive(Clone, Debug)]
pub struct SystemSnapshot {
    pub total_cpu: f64,
    pub core_usages: Vec<f64>,
    pub total_mem: u64,
    pub used_mem: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub net_rx_rate: f64,
    pub net_tx_rate: f64,
    pub processes: Vec<ProcessData>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum InputMode {
    Normal,
    Search,
    ConfirmKill,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum SortCriterion {
    Cpu,
    Memory,
    Disk,
    Pid,
}

pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}