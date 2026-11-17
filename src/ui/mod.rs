pub mod dashboard;
pub mod help;
pub mod inspector;
pub mod modal;
pub mod table;
mod theme_modal;

use crate::app::App;
use crate::config::parse_hex_color;
use crate::model::{InputMode, SortCriterion};
use crate::util::dim_background;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let bg_color = parse_hex_color(&app.config.theme.bg);
    let text_color = parse_hex_color(&app.config.theme.text);

    // Заливаем общий фон окна цветом активной темы
    f.render_widget(
        Block::default().style(Style::default().bg(bg_color).fg(text_color)),
        f.area(),
    );

    let (search_area, content_area) = if app.show_dashboard {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Dashboard
                Constraint::Length(3), // Search bar / Controls
                Constraint::Min(8),    // Content
            ])
            .split(f.area());

        dashboard::render_top_dashboard(f, app, chunks[0]);
        (chunks[1], chunks[2])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar / Controls
                Constraint::Min(8),    // Content
            ])
            .split(f.area());

        (chunks[0], chunks[1])
    };

    render_search_bar(f, app, search_area);

    if app.show_details {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(content_area);

        table::render_process_table(f, app, main_chunks[0]);
        inspector::render_details_panel(f, app, main_chunks[1]);
    } else {
        table::render_process_table(f, app, content_area);
    }

    let area = f.area();

    if app.input_mode == InputMode::ConfirmKill {
        dim_background(f.buffer_mut(), area);
        modal::render_kill_modal(f, app);
    } else if app.input_mode == InputMode::Help {
        dim_background(f.buffer_mut(), area);
        help::render_help_modal(f, app);
    } else if app.input_mode == InputMode::ThemePicker {
        dim_background(f.buffer_mut(), area);
        theme_modal::render_theme_modal(f, app);
    }
}

fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let sort_label = match app.sort_by {
        SortCriterion::Cpu => "CPU",
        SortCriterion::Memory => "MEM",
        SortCriterion::Disk => "DISK",
        SortCriterion::Pid => "PID",
    };

    let view_mode = if app.is_tree_view { "TREE" } else { "LIST" };
    let border_color = parse_hex_color(&app.config.theme.border);
    let crit_color = parse_hex_color(&app.config.theme.text_crit);
    let warn_color = parse_hex_color(&app.config.theme.text_warn);
    let primary_color = parse_hex_color(&app.config.theme.primary);
    let accent_color = parse_hex_color(&app.config.theme.accent);
    let text_color = parse_hex_color(&app.config.theme.text);

    let (color, title, text) = match app.input_mode {
        InputMode::Normal => {
            if let Some(ref status) = app.status_msg {
                let color = if status.is_error {
                    crit_color
                } else {
                    primary_color
                };
                (color, " Notification ", status.text.clone())
            } else {
                (
                    border_color,
                    " Status & Controls ",
                    format!(
                        "View: [{}]  •  Sort: [{}]   │   [/] Search   [o] Details   [x] Kill   [?] Help   [q] Quit",
                        view_mode, sort_label
                    ),
                )
            }
        }
        InputMode::Search => (
            warn_color,
            " Search / Port Filter (':8080' for ports, Enter to apply, Esc to clear) ",
            format!("> {}", app.search_query),
        ),
        InputMode::ConfirmKill => (
            crit_color,
            " Action Required ",
            "Confirm or cancel process termination".to_string(),
        ),
        InputMode::Help => (
            accent_color,
            " Help Active ",
            "Press Esc, q, or ? to dismiss".to_string(),
        ),
        InputMode::ThemePicker => (
            accent_color,
            " Theme Preview ",
            "Use j/k to preview themes, Enter to save, Esc to revert".to_string(),
        ),
    };

    let p = Paragraph::new(text)
        .style(Style::default().fg(text_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(title),
        );
    f.render_widget(p, area);
}