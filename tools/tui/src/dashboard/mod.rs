mod app;
mod data;
mod parser;
mod render;
mod tabs;

use std::{io, path::PathBuf, time::Duration};
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

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

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
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
            _ => {}
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
        KeyCode::Char('r') => app.reload(),
        KeyCode::Down | KeyCode::Char('j') => match app.current_tab {
            1 => app.milestone_scroll_down(),
            2 => app.ability_scroll_down(),
            3 => app.corner_case_scroll_down(),
            _ => {}
        },
        KeyCode::Up | KeyCode::Char('k') => match app.current_tab {
            1 => app.milestone_scroll_up(),
            2 => app.ability_scroll_up(),
            3 => app.corner_case_scroll_up(),
            _ => {}
        },
        KeyCode::Char('g') => {
            if app.current_tab == 3 {
                app.toggle_gaps_only();
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
