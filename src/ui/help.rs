use crate::app::App;
use crate::config::parse_hex_color;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help_modal(f: &mut Frame, app: &App) {
    let bg = parse_hex_color(&app.config.theme.bg);
    let primary = parse_hex_color(&app.config.theme.primary);
    let secondary = parse_hex_color(&app.config.theme.secondary);
    let accent = parse_hex_color(&app.config.theme.accent);
    let text = parse_hex_color(&app.config.theme.text);
    let border = parse_hex_color(&app.config.theme.border);

    let term = f.area();

    // Адаптивные размеры: не сжимается сильнее 66 колонок (если позволяет экран) и не растягивается лишнего
    let width = term.width.saturating_sub(4).clamp(36, 74);
    let height = term.height.saturating_sub(4).clamp(16, 23);

    let area = Rect {
        x: (term.width.saturating_sub(width)) / 2,
        y: (term.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Keybindings ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(bg))
        .border_style(Style::default().fg(primary));

    let inner = outer_block.inner(area);

    f.render_widget(Clear, area);
    f.render_widget(outer_block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Верхний отступ
            Constraint::Min(1),    // Основная сетка
            Constraint::Length(1), // Нижняя подсказка
        ])
        .split(inner);

    // Если экран совсем узкий (< 64 колонок), рисуем в 1 колонку, чтобы ничего не обрезалось
    if width < 64 {
        let single_col = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(layout[1]);

        let lines = build_left_column(primary, secondary, accent, text, border);
        f.render_widget(Paragraph::new(lines), single_col[1]);
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),      // Левый отступ
                Constraint::Percentage(48), // Левая колонка
                Constraint::Length(3),      // Центральный разделитель
                Constraint::Percentage(48), // Правая колонка
                Constraint::Length(1),      // Правый отступ
            ])
            .split(layout[1]);

        let left_lines = build_left_column(primary, secondary, accent, text, border);
        let right_lines = build_right_column(primary, secondary, accent, text, border);

        f.render_widget(Paragraph::new(left_lines), columns[1]);

        let separator: Vec<Line> = (0..columns[2].height)
            .map(|_| Line::from(Span::styled(" │", Style::default().fg(border))))
            .collect();
        f.render_widget(Paragraph::new(separator), columns[2]);

        f.render_widget(Paragraph::new(right_lines), columns[3]);
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(border)),
        Span::styled("Esc", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
        Span::styled(", ", Style::default().fg(border)),
        Span::styled("q", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(border)),
        Span::styled("?", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
        Span::styled(" to close", Style::default().fg(border)),
    ]))
        .alignment(Alignment::Center);

    f.render_widget(footer, layout[2]);
}

fn make_header(title: &'static str, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

fn make_row(
    key: &'static str,
    desc: &'static str,
    key_color: Color,
    text_color: Color,
    dim: Color,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:<10}", key),
            Style::default().fg(key_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(dim)),
        Span::styled(desc, Style::default().fg(text_color)),
    ])
}

fn build_left_column(
    primary: Color,
    secondary: Color,
    accent: Color,
    text: Color,
    border: Color,
) -> Vec<Line<'static>> {
    vec![
        make_header("NAVIGATION", accent),
        Line::from(""),
        make_row("j / ↓", "Move down", primary, text, border),
        make_row("k / ↑", "Move up", primary, text, border),
        make_row("Ctrl+d", "Half page down", primary, text, border),
        make_row("Ctrl+u", "Half page up", primary, text, border),
        make_row("gg", "Jump to top", primary, text, border),
        make_row("G", "Jump to bottom", primary, text, border),
        Line::from(""),
        make_header("PANES & LAYOUT", secondary),
        Line::from(""),
        make_row("o / Enter", "Toggle Inspector", primary, text, border),
        make_row("Tab", "Switch pane focus", primary, text, border),
        make_row("Ctrl+w w", "Cycle windows", primary, text, border),
        make_row("b", "Toggle Dashboard", primary, text, border),
        make_row("t", "Tree / Flat view", primary, text, border),
        make_row("T", "Select color theme", primary, text, border),
    ]
}

fn build_right_column(
    primary: Color,
    secondary: Color,
    accent: Color,
    text: Color,
    border: Color,
) -> Vec<Line<'static>> {
    vec![
        make_header("PROCESS & SEARCH", accent),
        Line::from(""),
        make_row("/", "Search name / :port", primary, text, border),
        make_row("x / d", "Terminate process", primary, text, border),
        make_row("y / Enter", "Confirm action", primary, text, border),
        make_row("Esc", "Cancel / Clear", primary, text, border),
        Line::from(""),
        make_header("SORTING", secondary),
        Line::from(""),
        make_row("c", "Sort by CPU %", primary, text, border),
        make_row("m", "Sort by Memory", primary, text, border),
        make_row("s", "Sort by Disk I/O", primary, text, border),
        make_row("p", "Sort by PID", primary, text, border),
        Line::from(""),
        make_header("GENERAL", accent),
        Line::from(""),
        make_row("? / F1", "Toggle help", primary, text, border),
        make_row("q", "Quit miru", primary, text, border),
    ]
}