mod app;
mod collector;
mod model;
mod ui;

use app::App;
use collector::start_collector;
use model::{InputMode, SortCriterion};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, panic, time::Duration};
use tokio::sync::watch;

fn normalize_char(c: char) -> char {
    match c {
        'q' | 'й' | 'Q' | 'Й' => 'q',
        'j' | 'о' | 'J' | 'О' => 'j',
        'k' | 'л' | 'K' | 'Л' => 'k',
        'x' | 'ч' | 'X' | 'Ч' => 'x',
        'y' | 'н' | 'Y' | 'Н' => 'y',
        'n' | 'т' | 'N' | 'Т' => 'n',
        't' | 'е' | 'T' | 'Е' => 't',
        'g' | 'п' => 'g',
        'G' | 'П' => 'G',
        '/' | '.' => '/',
        'c' | 'с' | 'C' | 'С' => 'c',
        'm' | 'ь' | 'M' | 'Ь' => 'm',
        'd' | 'в' | 'D' | 'В' => 'd',
        'p' | 'з' | 'P' | 'З' => 'p',
        other => other,
    }
}

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
                    match app.input_mode {
                        InputMode::Normal => {
                            let action_code = match key.code {
                                KeyCode::Char(c) => KeyCode::Char(normalize_char(c)),
                                other => other,
                            };

                            match action_code {
                                KeyCode::Char('q') => app.should_quit = true,
                                KeyCode::Char('/') => app.input_mode = InputMode::Search,
                                KeyCode::Char('t') => app.toggle_tree_view(),
                                KeyCode::Tab | KeyCode::Enter => app.toggle_details(),
                                KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
                                KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
                                KeyCode::Char('x') => app.prompt_kill_selected(),
                                KeyCode::Char('c') => app.set_sort(SortCriterion::Cpu),
                                KeyCode::Char('m') => app.set_sort(SortCriterion::Memory),
                                KeyCode::Char('d') => app.set_sort(SortCriterion::Disk),
                                KeyCode::Char('p') => app.set_sort(SortCriterion::Pid),
                                KeyCode::Char('g') | KeyCode::Home => app.move_cursor_to_top(),
                                KeyCode::Char('G') | KeyCode::End => app.move_cursor_to_bottom(),
                                _ => {}
                            }
                        }
                        InputMode::Search => match key.code {
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
                        },
                        InputMode::ConfirmKill => {
                            let action_code = match key.code {
                                KeyCode::Char(c) => KeyCode::Char(normalize_char(c)),
                                other => other,
                            };

                            match action_code {
                                KeyCode::Char('y') | KeyCode::Enter => app.confirm_kill(),
                                KeyCode::Char('n') | KeyCode::Esc => app.cancel_kill(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}