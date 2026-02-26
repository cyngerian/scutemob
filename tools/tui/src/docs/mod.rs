mod app;
pub mod markdown;
mod render;

use std::{io, path::PathBuf, time::Duration};

use anyhow::Result;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::event::{poll_event, should_quit, AppEvent};
use app::App;

pub fn run(file: Option<String>) -> Result<()> {
    let root = find_workspace_root()?;
    let mut app = App::new(&root, file);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = main_loop(&mut terminal, &mut app);

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
        terminal.draw(|f| render::render(f, f.area(), app))?;

        match poll_event(Duration::from_millis(200))? {
            Some(AppEvent::Key(key)) => {
                if !app.search_mode && should_quit(&key) {
                    break;
                }
                handle_key(app, key);
            }
            Some(AppEvent::Resize(_, _)) => {}
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyModifiers;

    if app.search_mode {
        match key.code {
            KeyCode::Esc => app.exit_search(),
            KeyCode::Enter => app.exit_search(),
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                app.clear_search();
            }
            KeyCode::Backspace => app.search_pop(),
            KeyCode::Char(c) => app.search_push(c),
            _ => {}
        }
        return;
    }

    match key.code {
        // File list navigation
        KeyCode::Down | KeyCode::Char('j') => app.list_down(),
        KeyCode::Up | KeyCode::Char('k') => app.list_up(),

        // Content scrolling (capital J/K or PgDn/PgUp)
        KeyCode::Char('J') => app.content_down(3),
        KeyCode::Char('K') => app.content_up(3),
        KeyCode::PageDown => app.content_down(20),
        KeyCode::PageUp => app.content_up(20),
        KeyCode::Home => app.scroll = 0,
        KeyCode::End => app.scroll = app.content_lines,

        // Search
        KeyCode::Char('/') => app.enter_search(),

        _ => {}
    }
}

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
        match dir.parent().map(|p| p.to_path_buf()) {
            Some(p) => dir = p,
            None => anyhow::bail!("Could not find workspace root"),
        }
    }
}
