use crate::app::{App, FocusedPane};
use crate::config::parse_hex_color;
use crate::util::{format_rate_kbs, format_uptime};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_details_panel(f: &mut Frame, app: &App, area: Rect) {
    let primary_color = parse_hex_color(&app.config.theme.primary);
    let sec_color = parse_hex_color(&app.config.theme.secondary);
    let accent_color = parse_hex_color(&app.config.theme.accent);
    let text_color = parse_hex_color(&app.config.theme.text);
    let border_dim = parse_hex_color(&app.config.theme.border);

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
                Span::styled("PID: ", Style::default().fg(sec_color).add_modifier(Modifier::BOLD)),
                Span::styled(p.pid.to_string(), Style::default().fg(text_color)),
                Span::styled(" | ", Style::default().fg(border_dim)),
                Span::styled(
                    "Parent PID: ",
                    Style::default().fg(sec_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    p.parent_pid.map(|pid| pid.to_string()).unwrap_or_else(|| "-".into()),
                    Style::default().fg(text_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(sec_color).add_modifier(Modifier::BOLD)),
                Span::styled(&p.name, Style::default().fg(text_color)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Listening Ports: ",
                    Style::default().fg(sec_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(ports, Style::default().fg(accent_color)),
            ]),
            Line::from(vec![
                Span::styled("Uptime: ", Style::default().fg(sec_color).add_modifier(Modifier::BOLD)),
                Span::styled(uptime, Style::default().fg(text_color)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Disk Activity: ",
                    Style::default().fg(sec_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "Read: {} | Write: {}",
                        format_rate_kbs(p.disk_read_sec),
                        format_rate_kbs(p.disk_write_sec)
                    ),
                    Style::default().fg(text_color),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Executable Path:",
                Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(exe, Style::default().fg(text_color))),
            Line::from(""),
            Line::from(Span::styled(
                "Command Line Arguments:",
                Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(cmd, Style::default().fg(text_color))),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No process selected",
            Style::default().fg(border_dim),
        ))]
    };

    let is_focused = app.focused_pane == FocusedPane::Inspector;
    let active_border = if is_focused {
        primary_color
    } else {
        border_dim
    };

    let title = if is_focused {
        format!(" Inspector [ACTIVE] (line {}) ", app.inspector_scroll)
    } else {
        format!(" Inspector (line {}) ", app.inspector_scroll)
    };

    let p = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(active_border)),
        )
        .scroll((app.inspector_scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}