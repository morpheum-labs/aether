mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aether", version, about = "Aether local developer tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a deterministic backtest against an in-memory demo dataset
    Backtest(commands::backtest::BacktestArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Backtest(args) => commands::backtest::run(args),
    }
}
