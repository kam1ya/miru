use crate::model::{ProcessData, SystemSnapshot};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::{
    CpuRefreshKind, MemoryRefreshKind, Networks, ProcessRefreshKind, ProcessesToUpdate, System,
};
use tokio::sync::watch;

pub fn start_collector(tx: watch::Sender<Option<SystemSnapshot>>) {
    thread::Builder::new()
        .name("miru-collector".into())
        .spawn(move || {
            let mut system = System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything())
                    .with_processes(ProcessRefreshKind::everything()),
            )
                .expect("Failed to init sysinfo");

            let mut networks = Networks::new_with_refreshed_list()
                .expect("Failed to init networks");

            system.refresh_all();
            thread::sleep(Duration::from_millis(200));

            loop {
                system.refresh_cpu_all();
                system.refresh_memory();
                system.refresh_processes(ProcessesToUpdate::All, true);
                networks.refresh(true);

                // 1. Опрос открытых сетевых сокетов (TCP/UDP)
                let mut ports_map: HashMap<u32, Vec<u16>> = HashMap::new();
                let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
                let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

                if let Ok(sockets) = get_sockets_info(af_flags, proto_flags) {
                    for s in sockets {
                        let port = match s.protocol_socket_info {
                            ProtocolSocketInfo::Tcp(tcp) => tcp.local_port,
                            ProtocolSocketInfo::Udp(udp) => udp.local_port,
                        };
                        for pid in s.associated_pids {
                            let entry = ports_map.entry(pid).or_default();
                            if !entry.contains(&port) {
                                entry.push(port);
                            }
                        }
                    }
                }

                // 2. Сеть
                let mut total_rx = 0;
                let mut total_tx = 0;
                for (_, net) in &networks {
                    total_rx += net.received();
                    total_tx += net.transmitted();
                }

                // 3. Ядра
                let cpus = system.cpus();
                let core_usages: Vec<f64> = cpus.iter().map(|c| c.usage() as f64).collect();
                let cpu_cores = cpus.len().max(1) as f32;

                // 4. Процессы
                let raw_procs: Vec<ProcessData> = system
                    .processes()
                    .iter()
                    .map(|(pid, proc_info)| {
                        let disk_usage = proc_info.disk_usage();
                        let pid_u32 = pid.as_u32();
                        let mut ports = ports_map.remove(&pid_u32).unwrap_or_default();
                        ports.sort_unstable();

                        ProcessData {
                            pid: *pid,
                            parent_pid: proc_info.parent(),
                            name: proc_info.name().to_string_lossy().to_string(),
                            cpu: proc_info.cpu_usage() / cpu_cores,
                            memory_mb: proc_info.memory() as f64 / (1024.0 * 1024.0),
                            disk_read_sec: disk_usage.read_bytes as f64 / 1024.0,
                            disk_write_sec: disk_usage.written_bytes as f64 / 1024.0,
                            exe_path: proc_info.exe().map(|p| p.to_path_buf()),
                            cmd: proc_info
                                .cmd()
                                .iter()
                                .map(|s| s.to_string_lossy().to_string())
                                .collect(),
                            run_time_secs: proc_info.run_time(),
                            listening_ports: ports,
                            depth: 0,
                            is_last_child: false,
                            children_count: 0,
                        }
                    })
                    .collect();

                let snapshot = SystemSnapshot {
                    total_cpu: system.global_cpu_usage() as f64,
                    core_usages,
                    total_mem: system.total_memory(),
                    used_mem: system.used_memory(),
                    total_swap: system.total_swap(),
                    used_swap: system.used_swap(),
                    net_rx_rate: total_rx as f64 / 1024.0,
                    net_tx_rate: total_tx as f64 / 1024.0,
                    processes: raw_procs,
                };

                let _ = tx.send(Some(snapshot));
                thread::sleep(Duration::from_millis(600));
            }
        })
        .expect("Failed to start collector thread");
}