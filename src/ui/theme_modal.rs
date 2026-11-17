use crate::app::App;
use crate::config::{parse_hex_color, ResolvedTheme, PRESETS};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_theme_modal(f: &mut Frame, app: &App) {
    let bg = parse_hex_color(&app.config.theme.bg);
    let primary = parse_hex_color(&app.config.theme.primary);
    let border = parse_hex_color(&app.config.theme.border);
    let text = parse_hex_color(&app.config.theme.text);
    let hl_bg = parse_hex_color(&app.config.theme.highlight_bg);
    let hl_fg = parse_hex_color(&app.config.theme.highlight_fg);

    let term = f.area();
    let width = 40.min(term.width.saturating_sub(4));
    let height = (PRESETS.len() as u16 + 6).min(term.height.saturating_sub(2));

    let area = Rect {
        x: (term.width.saturating_sub(width)) / 2,
        y: (term.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select Theme ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(bg))
        .border_style(Style::default().fg(primary));

    let mut lines = Vec::with_capacity(PRESETS.len() + 4);
    lines.push(Line::from(""));

    for (idx, &name) in PRESETS.iter().enumerate() {
        let is_selected = idx == app.theme_picker_idx;
        let palette = ResolvedTheme::from_preset(name);

        let c_prim = parse_hex_color(&palette.primary);
        let c_sec = parse_hex_color(&palette.secondary);
        let c_acc = parse_hex_color(&palette.accent);
        let c_crit = parse_hex_color(&palette.text_crit);

        let pointer = if is_selected { " >> " } else { "    " };
        let name_style = if is_selected {
            Style::default()
                .bg(hl_bg)
                .fg(hl_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(text)
        };

        lines.push(Line::from(vec![
            Span::styled(pointer, Style::default().fg(primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<15}", name), name_style),
            Span::raw(" "),
            Span::styled("■ ", Style::default().fg(c_prim)),
            Span::styled("■ ", Style::default().fg(c_sec)),
            Span::styled("■ ", Style::default().fg(c_acc)),
            Span::styled("■", Style::default().fg(c_crit)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
            Span::styled(" Save   ", Style::default().fg(border)),
            Span::styled("Esc", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(border)),
        ])
            .alignment(Alignment::Center),
    );

    let p = Paragraph::new(lines).block(block);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}