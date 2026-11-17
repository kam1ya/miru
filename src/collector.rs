use crate::model::{ProcessData, SystemSnapshot};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, RefreshKind, System,
};
use tokio::sync::watch;

pub fn start_collector(tx: watch::Sender<Option<SystemSnapshot>>) {
    thread::Builder::new()
        .name("miru-collector".into())
        .spawn(move || {
            let refresh_kind = RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything());

            let mut sys = System::new_with_specifics(refresh_kind)
                .expect("Failed to initialize sysinfo System");

            let mut networks =
                Networks::new_with_refreshed_list().expect("Failed to initialize Networks");

            let mut disks = Disks::new_with_refreshed_list().expect("Failed to initialize Disks");

            // Первичный прогрев счётчиков CPU
            sys.refresh_cpu_usage();
            thread::sleep(Duration::from_millis(200));

            loop {
                sys.refresh_specifics(refresh_kind);
                networks.refresh(true);
                disks.refresh(true);

                // 1. Сетевые метрики (суммарно по всем интерфейсам)
                let mut total_rx_rate = 0.0;
                let mut total_tx_rate = 0.0;
                for (_name, net) in &networks {
                    total_rx_rate += net.received() as f64 / 1024.0;
                    total_tx_rate += net.transmitted() as f64 / 1024.0;
                }

                // 2. Сбор открытых слушающих сокетов и привязка к PID
                let ports_map = collect_listening_ports();

                // 3. Сбор списка процессов с нормализацией нагрузки CPU (0..100%)
                let cpu_count = sys.cpus().len().max(1) as f32;
                let mut processes = Vec::with_capacity(sys.processes().len());

                for (&pid, p) in sys.processes() {
                    let disk_usage = p.disk_usage();
                    let pid_u32 = pid.as_u32();
                    let listening_ports = ports_map.get(&pid_u32).cloned().unwrap_or_default();

                    processes.push(ProcessData {
                        pid,
                        parent_pid: p.parent(),
                        name: p.name().to_string_lossy().to_string(),
                        cpu: p.cpu_usage() / cpu_count,
                        memory_mb: p.memory() as f64 / 1024.0 / 1024.0,
                        disk_read_sec: disk_usage.read_bytes as f64 / 1024.0,
                        disk_write_sec: disk_usage.written_bytes as f64 / 1024.0,
                        exe_path: p.exe().map(|path| path.to_path_buf()),
                        cmd: p
                            .cmd()
                            .iter()
                            .map(|s| s.to_string_lossy().to_string())
                            .collect(),
                        run_time_secs: p.run_time(),
                        listening_ports,
                        depth: 0,
                        is_last_child: false,
                        children_count: 0,
                    });
                }

                // 4. Метрики процессора и памяти
                let total_cpu = sys.global_cpu_usage() as f64;
                let core_usages: Vec<f64> = sys.cpus().iter().map(|c| c.usage() as f64).collect();

                let snapshot = SystemSnapshot {
                    total_cpu,
                    core_usages,
                    total_mem: sys.total_memory(),
                    used_mem: sys.used_memory(),
                    total_swap: sys.total_swap(),
                    used_swap: sys.used_swap(),
                    net_rx_rate: total_rx_rate,
                    net_tx_rate: total_tx_rate,
                    processes,
                };

                if tx.send(Some(snapshot)).is_err() {
                    break;
                }

                thread::sleep(Duration::from_millis(600));
            }
        })
        .expect("Failed to spawn miru-collector thread");
}

fn collect_listening_ports() -> HashMap<u32, Vec<u16>> {
    let mut map: HashMap<u32, HashSet<u16>> = HashMap::new();
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

    if let Ok(sockets) = get_sockets_info(af_flags, proto_flags) {
        for socket in sockets {
            let port = match socket.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp) => tcp.local_port,
                ProtocolSocketInfo::Udp(udp) => udp.local_port,
            };

            for pid in socket.associated_pids {
                map.entry(pid).or_default().insert(port);
            }
        }
    }

    map.into_iter()
        .map(|(pid, ports)| {
            let mut sorted: Vec<u16> = ports.into_iter().collect();
            sorted.sort_unstable();
            (pid, sorted)
        })
        .collect()
}