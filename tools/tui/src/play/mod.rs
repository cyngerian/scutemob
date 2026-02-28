//! Interactive play mode — human vs bots in a Commander game.

pub mod app;
pub mod input;
pub mod panels;
pub mod render;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::PlayApp;
use input::handle_key;
use render::render;

/// Run the interactive play mode.
pub fn run(player_count: u32, bot_type: String, bot_delay_ms: u64) -> anyhow::Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the app
    let mut app = PlayApp::new(player_count, &bot_type)?;
    app.bot_delay_ms = bot_delay_ms;

    // Start the game
    app.start_game()?;

    // Main loop
    let result = main_loop(&mut terminal, &mut app);

    // Flush log before restoring terminal
    app.flush_log();
    let log_path = app.log_path.clone();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("Game log saved to: {}", log_path.display());

    result
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut PlayApp,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        if app.game_over() {
            // Wait for q to quit after game over
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
                        return Ok(());
                    }
                }
            }
            continue;
        }

        // If it's a bot's turn, execute the bot action
        if app.is_bot_turn() {
            app.execute_bot_turn()?;

            // Use poll as delay — allows user to quit during bot turns
            let delay = Duration::from_millis(app.bot_delay_ms.max(10));
            if event::poll(delay)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            event::KeyCode::Char('q') => {
                                app.should_quit = true;
                            }
                            event::KeyCode::Char('c')
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                app.should_quit = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        // Human's turn — auto-pass if enabled, otherwise poll for input
        if app.auto_pass {
            if app.should_stop_auto_pass() {
                app.auto_pass = false;
                app.status_message = Some("Your main phase — auto-pass stopped".into());
            } else {
                let cmd = mtg_engine::Command::PassPriority {
                    player: app.human_player,
                };
                app.execute_command(cmd)?;

                // Still allow q/z during auto-pass
                if event::poll(Duration::from_millis(10))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                event::KeyCode::Char('q') => app.should_quit = true,
                                event::KeyCode::Char('c')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.should_quit = true;
                                }
                                event::KeyCode::Char('z') => {
                                    app.auto_pass = false;
                                    app.status_message = Some("Auto-pass OFF".into());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                continue;
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key)?;
                }
            }
        }
    }
}
