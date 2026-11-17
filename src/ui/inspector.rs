use crate::app::{App, FocusedPane};
use crate::config::parse_hex_color;
use crate::util::{format_rate_kbs, format_uptime};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_details_panel(f: &mut Frame, app: &App, area: Rect) {
    let border_color = parse_hex_color(&app.config.theme.primary);
    let sec_color = parse_hex_color(&app.config.theme.secondary);

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
                Span::styled("PID: ", Style::default().fg(sec_color)),
                Span::raw(p.pid.to_string()),
                Span::raw(" | "),
                Span::styled("Parent PID: ", Style::default().fg(sec_color)),
                Span::raw(p.parent_pid.map(|pid| pid.to_string()).unwrap_or_else(|| "-".into())),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(sec_color)),
                Span::raw(&p.name),
            ]),
            Line::from(vec![
                Span::styled("Listening Ports: ", Style::default().fg(sec_color)),
                Span::raw(ports),
            ]),
            Line::from(vec![
                Span::styled("Uptime: ", Style::default().fg(sec_color)),
                Span::raw(uptime),
            ]),
            Line::from(vec![
                Span::styled("Disk Activity: ", Style::default().fg(sec_color)),
                Span::raw(format!(
                    "Read: {} | Write: {}",
                    format_rate_kbs(p.disk_read_sec),
                    format_rate_kbs(p.disk_write_sec)
                )),
            ]),
            Line::from(""),
            Line::from(Span::styled("Executable Path:", Style::default().fg(border_color))),
            Line::from(Span::raw(exe)),
            Line::from(""),
            Line::from(Span::styled("Command Line Arguments:", Style::default().fg(border_color))),
            Line::from(Span::raw(cmd)),
        ]
    } else {
        vec![Line::from("No process selected")]
    };

    let is_focused = app.focused_pane == FocusedPane::Inspector;
    let active_border = if is_focused {
        border_color
    } else {
        parse_hex_color(&app.config.theme.border)
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