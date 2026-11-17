mod app;
mod collector;
mod config;
mod model;
mod tree;
mod ui;
mod util;

use app::{App, FocusedPane};
use collector::start_collector;
use model::{InputMode, SortCriterion};
use util::normalize_key_char;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, panic, time::Duration};
use tokio::sync::watch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_panic(info);
    }));

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let (tx, mut rx) = watch::channel(None);
    start_collector(tx);

    let frame_interval = Duration::from_millis(16);

    while !app.should_quit {
        if rx.has_changed()? {
            let snap = rx.borrow_and_update().clone();
            if let Some(snapshot) = snap {
                app.apply_snapshot(snapshot);
            }
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        if event::poll(frame_interval)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(&mut app, key);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Search => handle_search_mode(app, key),
        InputMode::ConfirmKill => handle_confirm_kill_mode(app, key),
        InputMode::Help => handle_help_mode(app, key),
        InputMode::ThemePicker => handle_theme_picker_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    if is_ctrl {
        if let KeyCode::Char(ch) = key.code {
            match normalize_key_char(ch) {
                'w' => {
                    app.pending_ctrl_w = true;
                    return;
                }
                'd' => {
                    app.half_page_down();
                    return;
                }
                'u' => {
                    app.half_page_up();
                    return;
                }
                _ => {}
            }
        }
    }

    if app.pending_ctrl_w {
        app.pending_ctrl_w = false;
        if let KeyCode::Char(ch) = key.code {
            match normalize_key_char(ch) {
                'w' => app.toggle_pane_focus(),
                'h' => app.set_focus(FocusedPane::Table),
                'l' => app.set_focus(FocusedPane::Inspector),
                _ => {}
            }
        }
        return;
    }

    let code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(normalize_key_char(c)),
        other => other,
    };

    if app.pending_g {
        app.pending_g = false;
        if code == KeyCode::Char('g') {
            app.move_cursor_to_top();
            return;
        }
    }

    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') | KeyCode::F(1) => app.input_mode = InputMode::Help,
        KeyCode::Char('o') | KeyCode::Enter => app.toggle_details(),
        KeyCode::Tab => app.toggle_pane_focus(),

        KeyCode::Esc => {
            if app.focused_pane == FocusedPane::Inspector {
                app.focused_pane = FocusedPane::Table;
            } else if app.show_details {
                app.toggle_details();
            }
        }

        KeyCode::Char('j') | KeyCode::Down => {
            if app.focused_pane == FocusedPane::Inspector {
                app.scroll_inspector_down();
            } else {
                app.move_cursor_down();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.focused_pane == FocusedPane::Inspector {
                app.scroll_inspector_up();
            } else {
                app.move_cursor_up();
            }
        }

        KeyCode::Char('g') => {
            app.pending_g = true;
        }
        KeyCode::Home => app.move_cursor_to_top(),

        KeyCode::Char('G') | KeyCode::End => app.move_cursor_to_bottom(),

        KeyCode::Char('/') => app.input_mode = InputMode::Search,
        KeyCode::Char('b') => app.toggle_dashboard(),
        KeyCode::Char('t') => app.toggle_tree_view(),

        KeyCode::Char('x') | KeyCode::Char('d') => app.prompt_kill_selected(),

        KeyCode::Char('c') => app.set_sort(SortCriterion::Cpu),
        KeyCode::Char('m') => app.set_sort(SortCriterion::Memory),
        KeyCode::Char('s') => app.set_sort(SortCriterion::Disk),
        KeyCode::Char('p') => app.set_sort(SortCriterion::Pid),

        KeyCode::Char('T') => app.open_theme_picker(),

        _ => {}
    }
}

fn handle_search_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => app.input_mode = InputMode::Normal,
        KeyCode::Esc => {
            app.search_query.clear();
            app.rebuild_display_list();
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.rebuild_display_list();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.rebuild_display_list();
        }
        _ => {}
    }
}

fn handle_confirm_kill_mode(app: &mut App, key: KeyEvent) {
    let action_code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(normalize_key_char(c)),
        other => other,
    };

    match action_code {
        KeyCode::Char('y') | KeyCode::Enter => app.confirm_kill(),
        KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => app.cancel_kill(),
        _ => {}
    }
}

fn handle_help_mode(app: &mut App, key: KeyEvent) {
    let action_code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(normalize_key_char(c)),
        other => other,
    };

    match action_code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::F(1) | KeyCode::Enter => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

fn handle_theme_picker_mode(app: &mut App, key: KeyEvent) {
    let action_code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(normalize_key_char(c)),
        other => other,
    };

    match action_code {
        KeyCode::Char('j') | KeyCode::Down => app.theme_picker_down(),
        KeyCode::Char('k') | KeyCode::Up => app.theme_picker_up(),
        KeyCode::Enter => app.confirm_theme(),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('T') => app.cancel_theme_picker(),
        _ => {}
    }
}