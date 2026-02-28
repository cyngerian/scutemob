mod app;
mod data;
mod parser;
mod render;
mod tabs;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, time::Duration};

use crate::event::{poll_event, should_quit, AppEvent};
use app::App;

pub fn run(_watch: bool) -> Result<()> {
    // Detect workspace root: walk up from cwd looking for Cargo.toml with [workspace].
    let root = find_workspace_root()?;

    let mut app = App::new(root);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Main loop
    let result = main_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn main_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| render::render(f, app))?;

        match poll_event(Duration::from_millis(200))? {
            Some(AppEvent::Key(key)) => {
                if should_quit(&key) {
                    break;
                }
                handle_key(app, key);
            }
            Some(AppEvent::Resize(_, _)) => {
                // ratatui redraws on next loop iteration
            }
            _ => {
                if let Some(rx) = &app.test_count_rx {
                    if let Ok(count) = rx.try_recv() {
                        app.live_test_count = app::LiveTestCount::Done(count);
                        app.test_count_rx = None;
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.prev_tab(),
        KeyCode::Char('1') => app.jump_to_tab(0),
        KeyCode::Char('2') => app.jump_to_tab(1),
        KeyCode::Char('3') => app.jump_to_tab(2),
        KeyCode::Char('4') => app.jump_to_tab(3),
        KeyCode::Char('5') => app.jump_to_tab(4),
        KeyCode::Char('6') => app.jump_to_tab(5),
        KeyCode::Char('7') => app.jump_to_tab(6),
        KeyCode::Char('r') => {
            if app.current_tab == 6 {
                app.set_cards_filter("ready");
            } else {
                app.reload();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => match app.current_tab {
            1 => app.milestone_scroll_down(),
            2 => app.ability_scroll_down(),
            3 => app.corner_case_scroll_down(),
            5 => app.scripts_scroll_down(),
            6 => app.cards_scroll_down(),
            _ => {}
        },
        KeyCode::Up | KeyCode::Char('k') => match app.current_tab {
            1 => app.milestone_scroll_up(),
            2 => app.ability_scroll_up(),
            3 => app.corner_case_scroll_up(),
            5 => app.scripts_scroll_up(),
            6 => app.cards_scroll_up(),
            _ => {}
        },
        KeyCode::Char('J') => {
            if app.current_tab == 6 {
                app.cards_detail_scroll_down();
            }
        }
        KeyCode::Char('K') => {
            if app.current_tab == 6 {
                app.cards_detail_scroll_up();
            }
        }
        KeyCode::Char('g') => {
            if app.current_tab == 3 {
                app.toggle_gaps_only();
            }
        }
        KeyCode::Char('p') => {
            if app.current_tab == 5 && !app.scripts_show_pending_only {
                app.toggle_scripts_pending_only();
            }
        }
        KeyCode::Char('a') => match app.current_tab {
            5 if app.scripts_show_pending_only => app.toggle_scripts_pending_only(),
            6 => app.set_cards_filter("all"),
            _ => {}
        },
        KeyCode::Char('b') => {
            if app.current_tab == 6 {
                app.set_cards_filter("blocked");
            }
        }
        KeyCode::Char('c') => {
            if app.current_tab == 6 {
                app.set_cards_filter("authored");
            }
        }
        KeyCode::Char('d') => {
            if app.current_tab == 6 {
                app.set_cards_filter("deferred");
            }
        }
        _ => {}
    }
}

/// Walk parent directories to find the Cargo workspace root
/// (the directory containing `Cargo.toml` with `[workspace]`).
fn find_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let toml = dir.join("Cargo.toml");
        if toml.exists() {
            let content = std::fs::read_to_string(&toml).unwrap_or_default();
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        let parent = dir.parent().map(|p| p.to_path_buf());
        match parent {
            Some(p) => dir = p,
            None => anyhow::bail!("Could not find workspace root (no Cargo.toml with [workspace])"),
        }
    }
}
