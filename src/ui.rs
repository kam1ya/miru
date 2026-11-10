use crate::app::App;
use crate::model::{InputMode, SortCriterion};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Clear, Dataset, Gauge, GraphType, HighlightSpacing, Paragraph,
        Row, Table, Wrap,
    },
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Min(8),
        ])
        .split(f.area());

    render_top_dashboard(f, app, chunks[0]);
    render_search_bar(f, app, chunks[1]);

    if app.show_details {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(chunks[2]);

        render_process_table(f, app, main_chunks[0]);
        render_details_panel(f, app, main_chunks[1]);
    } else {
        render_process_table(f, app, chunks[2]);
    }

    if app.input_mode == InputMode::ConfirmKill {
        render_kill_modal(f, app);
    }
}

fn render_top_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Percentage(34),
            Constraint::Percentage(24),
        ])
        .split(area);

    let cur_cpu = app.cpu_total_history.last().map(|(_, y)| *y).unwrap_or(0.0);
    let dataset = vec![Dataset::default()
        .name(format!("Total: {:.1}%", cur_cpu))
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&app.cpu_total_history)];

    let chart = Chart::new(dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CPU History ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_axis(Axis::default().bounds([0.0, 59.0]).labels(["-60s", "now"]))
        .y_axis(Axis::default().bounds([0.0, 100.0]).labels(["0%", "100%"]));
    f.render_widget(chart, sub_chunks[0]);

    render_cores_heatmap(f, app, sub_chunks[1]);
    render_sys_stats(f, app, sub_chunks[2]);
}

fn render_cores_heatmap(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Core Activity ")
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let snapshot = match &app.latest_snapshot {
        Some(s) => s,
        None => return,
    };

    let num_cores = snapshot.core_usages.len();
    if num_cores == 0 || inner.height == 0 || inner.width == 0 {
        return;
    }

    let cols = if num_cores <= 8 { 2 } else if num_cores <= 16 { 3 } else { 4 };
    let rows = (num_cores + cols - 1) / cols;

    let mut lines: Vec<Line> = Vec::new();

    for r in 0..rows {
        let mut spans = Vec::new();
        for c in 0..cols {
            let idx = r * cols + c;
            if idx < num_cores {
                let usage = snapshot.core_usages[idx];
                let (color, bar_symbol) = heatmap_style(usage);

                spans.push(Span::styled(
                    format!(" {:>2}:", idx + 1),
                    Style::default().fg(Color::DarkGray),
                ));
                spans.push(Span::styled(
                    format!("{:<4}", bar_symbol),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    format!("{:>3.0}% ", usage),
                    Style::default().fg(color),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn heatmap_style(usage: f64) -> (Color, &'static str) {
    if usage > 85.0 {
        (Color::Rgb(255, 60, 60), "■■■■")
    } else if usage > 60.0 {
        (Color::Rgb(255, 140, 0), "■■■ ")
    } else if usage > 35.0 {
        (Color::Rgb(255, 215, 0), "■■  ")
    } else if usage > 10.0 {
        (Color::Rgb(0, 200, 150), "■   ")
    } else {
        (Color::Rgb(60, 90, 120), "·   ")
    }
}

fn render_sys_stats(f: &mut Frame, app: &App, area: Rect) {
    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let (used_mem, total_mem, net_rx, net_tx) = match &app.latest_snapshot {
        Some(s) => (s.used_mem, s.total_mem, s.net_rx_rate, s.net_tx_rate),
        None => (0, 0, 0.0, 0.0),
    };

    let mem_percent = if total_mem > 0 {
        ((used_mem as f64 / total_mem as f64) * 100.0) as u16
    } else {
        0
    };

    let ram_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" RAM ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(Color::Rgb(180, 100, 255)))
        .percent(mem_percent)
        .label(format!(
            "{:.1} / {:.1} GiB",
            used_mem as f64 / 1024.0 / 1024.0 / 1024.0,
            total_mem as f64 / 1024.0 / 1024.0 / 1024.0
        ));
    f.render_widget(ram_gauge, side_chunks[0]);

    let net_text = vec![Line::from(vec![
        Span::styled("RX: ", Style::default().fg(Color::Green)),
        Span::raw(format!("{:.1} KB/s", net_rx)),
        Span::raw(" | "),
        Span::styled("TX: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{:.1} KB/s", net_tx)),
    ])];
    let net_widget = Paragraph::new(net_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(net_widget, side_chunks[1]);
}

fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let sort_label = match app.sort_by {
        SortCriterion::Cpu => "CPU",
        SortCriterion::Memory => "MEM",
        SortCriterion::Disk => "DISK",
        SortCriterion::Pid => "PID",
    };

    let view_mode = if app.is_tree_view { "TREE" } else { "LIST" };

    let (border_color, title, text) = match app.input_mode {
        InputMode::Normal => {
            if let Some(ref status) = app.status_msg {
                let color = if status.is_error { Color::Red } else { Color::Green };
                (color, " Notification ", status.text.clone())
            } else {
                (
                    Color::DarkGray,
                    " Controls ",
                    format!(
                        "[/] Search (e.g. ':8080') | [t] Mode: [{}] | [c/m/d/p] Sort: [{}] | [x] Kill | [Tab] Details | [q] Quit",
                        view_mode, sort_label
                    ),
                )
            }
        }
        InputMode::Search => (
            Color::Yellow,
            " Search / Port Filter (':8080' for ports, Enter to confirm, Esc to clear) ",
            format!("> {}", app.search_query),
        ),
        InputMode::ConfirmKill => (
            Color::Red,
            " Action Required ",
            "Confirm or cancel process termination".to_string(),
        ),
    };

    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title),
    );
    f.render_widget(p, area);
}

fn render_process_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["PID", "Process Name", "CPU %", "Memory", "Disk I/O", "Ports"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let is_tree = app.is_tree_view && app.search_query.trim().is_empty();

    let rows: Vec<Row> = app
        .display_processes
        .iter()
        .map(|p| {
            let cpu_color = if p.cpu > 25.0 {
                Color::Red
            } else if p.cpu > 5.0 {
                Color::Yellow
            } else {
                Color::White
            };

            let mem_str = if p.memory_mb >= 1024.0 {
                format!("{:.2} GB", p.memory_mb / 1024.0)
            } else {
                format!("{:.1} MB", p.memory_mb)
            };

            let disk_io = p.disk_read_sec + p.disk_write_sec;
            let disk_str = if disk_io > 1024.0 {
                format!("{:.1} MB/s", disk_io / 1024.0)
            } else if disk_io > 0.0 {
                format!("{:.1} KB/s", disk_io)
            } else {
                "0".to_string()
            };

            // Префикс дерева: ├─ / └─
            let name_display = if is_tree {
                if p.depth == 0 {
                    p.name.clone()
                } else {
                    let indent = "  ".repeat(p.depth - 1);
                    let branch = if p.is_last_child { "└─ " } else { "├─ " };
                    format!("{}{}{}", indent, branch, p.name)
                }
            } else {
                p.name.clone()
            };

            let ports_str = if p.listening_ports.is_empty() {
                "-".to_string()
            } else {
                p.listening_ports
                    .iter()
                    .take(3)
                    .map(|port| format!(":{}", port))
                    .collect::<Vec<_>>()
                    .join(",")
            };

            Row::new(vec![
                p.pid.to_string(),
                name_display,
                format!("{:.1}%", p.cpu),
                mem_str,
                disk_str,
                ports_str,
            ])
                .style(Style::default().fg(cpu_color))
        })
        .collect();

    let title = format!(" Processes ({}) ", app.display_processes.len());
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(40),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
        ],
    )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(30, 45, 75))
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_details_panel(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(p) = app.get_selected_process() {
        let exe = p
            .exe_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let cmd = if p.cmd.is_empty() {
            "N/A".to_string()
        } else {
            p.cmd.join(" ")
        };

        let uptime = format_uptime(p.run_time_secs);

        let ports = if p.listening_ports.is_empty() {
            "None".to_string()
        } else {
            p.listening_ports
                .iter()
                .map(|pt| format!(":{}", pt))
                .collect::<Vec<_>>()
                .join(", ")
        };

        vec![
            Line::from(vec![
                Span::styled("PID: ", Style::default().fg(Color::Yellow)),
                Span::raw(p.pid.to_string()),
                Span::raw(" | "),
                Span::styled("Parent PID: ", Style::default().fg(Color::Yellow)),
                Span::raw(p.parent_pid.map(|pid| pid.to_string()).unwrap_or_else(|| "-".into())),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                Span::raw(&p.name),
            ]),
            Line::from(vec![
                Span::styled("Listening Ports: ", Style::default().fg(Color::Green)),
                Span::raw(ports),
            ]),
            Line::from(vec![
                Span::styled("Uptime: ", Style::default().fg(Color::Yellow)),
                Span::raw(uptime),
            ]),
            Line::from(vec![
                Span::styled("Disk Activity: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!(
                    "Read: {:.1} KB/s | Write: {:.1} KB/s",
                    p.disk_read_sec, p.disk_write_sec
                )),
            ]),
            Line::from(""),
            Line::from(Span::styled("Executable Path:", Style::default().fg(Color::Cyan))),
            Line::from(Span::raw(exe)),
            Line::from(""),
            Line::from(Span::styled("Command Line Arguments:", Style::default().fg(Color::Cyan))),
            Line::from(Span::raw(cmd)),
        ]
    } else {
        vec![Line::from("No process selected")]
    };

    let p = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Process Inspector ")
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}

fn render_kill_modal(f: &mut Frame, app: &App) {
    let target_name = app
        .target_to_kill
        .as_ref()
        .map(|p| format!("{} (PID: {})", p.name, p.pid))
        .unwrap_or_else(|| "Unknown".to_string());

    let area = centered_rect(50, 20, f.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm Termination ")
        .style(Style::default().bg(Color::Rgb(20, 20, 25)))
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Are you sure you want to terminate "),
            Span::styled(target_name, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Y] Yes, kill  ", Style::default().bg(Color::Red).fg(Color::White)),
            Span::raw("    "),
            Span::styled("  [N / Esc] Cancel  ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        ]),
    ];

    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_uptime(secs: u64) -> String {
    if secs == 0 {
        return "0s".to_string();
    }

    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}