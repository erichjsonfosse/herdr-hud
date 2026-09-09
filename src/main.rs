use clap::{Parser, Subcommand};
use herdr_status_bar::client::HerdrClient;
use herdr_status_bar::config::StatusBarConfig;
use herdr_status_bar::state::StatusBarState;
use herdr_status_bar::ui::{render_ansi_line, run_modal_menu, run_tui_loop};


#[derive(Parser)]
#[command(name = "herdr-status-bar")]
#[command(about = "Context-based Zellij-inspired status bar for Herdr")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the interactive full-width bottom status bar
    Run,
    /// Print a single ANSI-formatted status line (useful for tab bar or external scripts)
    Line,
    /// Open the interactive Command Palette & Shortcut Reference modal popup
    Menu,
    /// Test Herdr socket connection and show active snapshot
    Check,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = StatusBarConfig::load();
    let client = HerdrClient::new();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Line => {
            let mut state = StatusBarState::new();
            if let Some(snapshot) = client.fetch_snapshot_sync() {
                HerdrClient::update_state_from_snapshot(&mut state, &snapshot);
            }
            println!("{}", render_ansi_line(&state, &config));
        }
        Commands::Menu => {
            run_modal_menu(client, config).await?;
        }
        Commands::Check => {
            println!("🔍 Inspecting Herdr IPC Connection...");
            match HerdrClient::discover_socket() {
                Some(path) => println!("  [✔] Found Herdr socket at: {}", path.display()),
                None => println!("  [!] No active Herdr socket found on system"),
            }
            if let Some(snapshot) = client.fetch_snapshot_sync() {
                println!("  [✔] Snapshot received:\n{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("  [!] Could not fetch live snapshot from Herdr");
            }
        }
        Commands::Run => {
            run_tui_loop(client, config).await?;
        }
    }

    Ok(())
}

