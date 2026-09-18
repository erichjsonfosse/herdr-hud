use clap::{Parser, Subcommand};
use herdr_hud::client::HerdrClient;
use herdr_hud::config::{HudConfig};
use herdr_hud::ui::{render_ansi_line, run_modal_menu};

#[derive(Parser)]
#[command(name = "herdr-hud")]
#[command(about = "Context-based HUD, and command palette for Herdr")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print a single ANSI-formatted line (useful for tab bar or external scripts)
    Line,
    /// Open the interactive Command Palette & Shortcut Reference modal popup
    Menu,
    /// Test Herdr socket connection and show active snapshot
    Check,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Line) {
        Commands::Line => {
            let config = HudConfig::load();
            println!("{}", render_ansi_line(&config));
        }
        Commands::Menu => {
            let config = HudConfig::load();
            let client = HerdrClient::new();
            run_modal_menu(client, config)?;
        }
        Commands::Check => {
            let client = HerdrClient::new();
            println!("🔍 Inspecting Herdr IPC Connection...");
            match HerdrClient::discover_socket() {
                Some(path) => println!("  [✔] Found Herdr socket at: {}", path.display()),
                None => println!("  [!] No active Herdr socket found on system"),
            }
            if let Some(snapshot) = client.fetch_snapshot_sync() {
                println!(
                    "  [✔] Snapshot received:\n{}",
                    serde_json::to_string_pretty(&snapshot)?
                );
            } else {
                println!("  [!] Could not fetch live snapshot from Herdr");
            }
        }
    }

    Ok(())
}
