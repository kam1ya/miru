use crate::app::App;
use crate::config::parse_hex_color;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_kill_modal(f: &mut Frame, app: &App) {
    let border_dim = parse_hex_color(&app.config.theme.border);
    let crit_color = parse_hex_color(&app.config.theme.text_crit);

    let target_name = app
        .target_to_kill
        .as_ref()
        .map(|p| format!("{} (PID: {})", p.name, p.pid))
        .unwrap_or_else(|| "Unknown".to_string());

    let term = f.area();

    // Фиксированная высота 7 строк — кнопки гарантированно поместятся
    let width = (target_name.len() as u16 + 44).clamp(48, term.width.saturating_sub(4));
    let height = 7.min(term.height.saturating_sub(2));

    let area = Rect {
        x: (term.width.saturating_sub(width)) / 2,
        y: (term.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm Termination ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Reset))
        .border_style(Style::default().fg(crit_color));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Terminate process "),
            Span::styled(
                target_name,
                Style::default().fg(crit_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[ ", Style::default().fg(border_dim)),
            Span::styled(
                "Y / Enter",
                Style::default().fg(crit_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ] ", Style::default().fg(border_dim)),
            Span::raw("Kill      "),
            Span::styled("[ ", Style::default().fg(border_dim)),
            Span::styled(
                "N / Esc",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ] ", Style::default().fg(border_dim)),
            Span::raw("Cancel"),
        ]),
    ];

    let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}