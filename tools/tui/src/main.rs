mod dashboard;
mod docs;
mod event;
mod play;
mod theme;
mod widgets;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mtg-tui",
    about = "TUI tools for the MTG Commander Rules Engine",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Project progress dashboard — live view of all tracking docs
    Dashboard {
        /// Watch source docs for changes and auto-refresh
        #[arg(long)]
        watch: bool,
    },
    /// Browse and read project markdown docs (CLAUDE.md, docs/, memory/)
    Docs {
        /// Jump directly to a file matching this name (partial match)
        file: Option<String>,
    },
    /// Interactive play — human vs bots in a Commander game
    Play {
        /// Number of players (2-6)
        #[arg(long, default_value = "4")]
        players: u32,

        /// Bot type: random or heuristic
        #[arg(long, default_value = "random")]
        bot: String,

        /// Delay between bot actions in ms (0 for instant)
        #[arg(long, default_value = "200")]
        delay: u64,
    },
    /// Game state stepper — step through a game script interactively
    Stepper {
        /// Path to game script JSON file
        script: Option<String>,
        /// Directory to browse for scripts
        #[arg(long)]
        dir: Option<String>,
    },
    /// Interactive card browser
    Cards {
        /// Initial search query
        #[arg(long)]
        search: Option<String>,
    },
    /// Comprehensive Rules explorer
    Rules {
        /// Initial search query
        #[arg(long)]
        search: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Dashboard { watch } => dashboard::run(watch),
        Commands::Docs { file } => docs::run(file),
        Commands::Play {
            players,
            bot,
            delay,
        } => play::run(players, bot, delay),
        Commands::Stepper { .. } => {
            eprintln!("Stepper not yet implemented (planned for Phase 2)");
            Ok(())
        }
        Commands::Cards { .. } => {
            eprintln!("Card browser not yet implemented (planned for Phase 3)");
            Ok(())
        }
        Commands::Rules { .. } => {
            eprintln!("Rules explorer not yet implemented (planned for Phase 4)");
            Ok(())
        }
    }
}
