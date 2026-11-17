use crate::app::{App, FocusedPane};
use crate::config::parse_hex_color;
use crate::util::format_rate_kbs;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, HighlightSpacing, Row, Table},
    Frame,
};

pub fn render_process_table(f: &mut Frame, app: &mut App, area: Rect) {
    let sec_color = parse_hex_color(&app.config.theme.secondary);
    let border_color = parse_hex_color(&app.config.theme.border);
    let hl_bg = parse_hex_color(&app.config.theme.highlight_bg);
    let hl_fg = parse_hex_color(&app.config.theme.highlight_fg);

    let header = Row::new(vec!["PID", "Process Name", "CPU %", "Memory", "Disk I/O", "Ports"])
        .style(Style::default().fg(sec_color).add_modifier(Modifier::BOLD))
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

            let disk_str = format_rate_kbs(p.disk_read_sec + p.disk_write_sec);

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
            } else if app.search_query.trim().starts_with(':') {
                let searched_port = app.search_query.trim().trim_start_matches(':');
                if let Ok(target) = searched_port.parse::<u16>() {
                    if p.listening_ports.contains(&target) {
                        format!(":{} (match)", target)
                    } else {
                        "-".to_string()
                    }
                } else {
                    "-".to_string()
                }
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

    let is_focused = app.focused_pane == FocusedPane::Table;
    let table_border = if is_focused {
        parse_hex_color(&app.config.theme.primary)
    } else {
        border_color
    };

    let title = if is_focused {
        format!(" Processes ({}) [ACTIVE] ", app.display_processes.len())
    } else {
        format!(" Processes ({}) ", app.display_processes.len())
    };

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(table_border))
                .title(title),
        )
        .row_highlight_style(
            Style::default()
                .bg(hl_bg)
                .fg(hl_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(table, area, &mut app.table_state);
}