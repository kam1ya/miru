use crate::app::App;
use crate::config::parse_hex_color;
use crate::util::{format_bytes, format_rate_kbs};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph},
    Frame,
};

pub fn render_top_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Percentage(34),
            Constraint::Percentage(24),
        ])
        .split(area);

    let primary_color = parse_hex_color(&app.config.theme.primary);
    let border_color = parse_hex_color(&app.config.theme.border);
    let text_color = parse_hex_color(&app.config.theme.text);

    let cur_cpu = app.cpu_total_history.last().map(|(_, y)| *y).unwrap_or(0.0);
    let dataset = vec![Dataset::default()
        .name(format!("Total: {:.1}%", cur_cpu))
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(primary_color))
        .data(&app.cpu_total_history)];

    let chart = Chart::new(dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CPU History ")
                .border_style(Style::default().fg(border_color)),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(border_color))
                .bounds([0.0, 59.0])
                .labels(["-60s", "now"]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(border_color))
                .bounds([0.0, 100.0])
                .labels(["0%", "100%"]),
        );
    f.render_widget(chart, sub_chunks[0]);

    render_cores_heatmap(f, app, sub_chunks[1]);
    render_sys_stats(f, app, sub_chunks[2], text_color);
}

fn render_cores_heatmap(f: &mut Frame, app: &App, area: Rect) {
    let border_color = parse_hex_color(&app.config.theme.border);
    let primary = parse_hex_color(&app.config.theme.primary);
    let accent = parse_hex_color(&app.config.theme.accent);
    let warn = parse_hex_color(&app.config.theme.text_warn);
    let crit = parse_hex_color(&app.config.theme.text_crit);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Core Activity ")
        .border_style(Style::default().fg(border_color));

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
    let rows = num_cores.div_ceil(cols);

    let mut lines: Vec<Line> = Vec::with_capacity(rows);

    for r in 0..rows {
        let mut spans = Vec::with_capacity(cols * 3);
        for c in 0..cols {
            let idx = r * cols + c;
            if idx < num_cores {
                let usage = snapshot.core_usages[idx];
                let (color, bar_symbol) =
                    heatmap_style(usage, primary, accent, warn, crit, border_color);

                spans.push(Span::styled(
                    format!(" {:>2}:", idx + 1),
                    Style::default().fg(border_color),
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

fn heatmap_style(
    usage: f64,
    primary: Color,
    accent: Color,
    warn: Color,
    crit: Color,
    idle: Color,
) -> (Color, &'static str) {
    if usage > 85.0 {
        (crit, "■■■■")
    } else if usage > 55.0 {
        (warn, "■■■ ")
    } else if usage > 25.0 {
        (accent, "■■  ")
    } else if usage > 8.0 {
        (primary, "■   ")
    } else {
        (idle, "·   ")
    }
}

fn render_sys_stats(f: &mut Frame, app: &App, area: Rect, text_color: Color) {
    let border_color = parse_hex_color(&app.config.theme.border);
    let accent_color = parse_hex_color(&app.config.theme.accent);
    let primary_color = parse_hex_color(&app.config.theme.primary);
    let secondary_color = parse_hex_color(&app.config.theme.secondary);

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
                .border_style(Style::default().fg(border_color)),
        )
        .gauge_style(Style::default().fg(accent_color))
        .percent(mem_percent)
        .label(Span::styled(
            format!("{} / {}", format_bytes(used_mem), format_bytes(total_mem)),
            Style::default().fg(text_color).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(ram_gauge, side_chunks[0]);

    let net_text = vec![Line::from(vec![
        Span::styled(
            "RX: ",
            Style::default()
                .fg(secondary_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format_rate_kbs(net_rx), Style::default().fg(text_color)),
        Span::styled(" | ", Style::default().fg(border_color)),
        Span::styled(
            "TX: ",
            Style::default()
                .fg(primary_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format_rate_kbs(net_tx), Style::default().fg(text_color)),
    ])];
    let net_widget = Paragraph::new(net_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network ")
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(net_widget, side_chunks[1]);
}