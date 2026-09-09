mod client;
mod config;
mod helpers;
mod state;
mod ui;

use clap::{Parser, Subcommand};
use client::HerdrClient;
use config::StatusBarConfig;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use state::{HerdrMode, StatusBarState};
use std::io::stdout;
use std::process::Command as SysCommand;
use std::time::Duration;
use ui::{get_palette_categories, render_ansi_line, ActionKind, MenuModalWidget, PaletteAction, StatusBarWidget};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalInputTarget {
    CreateTab,
    RenameWorkspace,
    RenameTab,
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

fn trigger_action(
    action: &PaletteAction,
    state: &StatusBarState,
    input_target: &mut Option<ModalInputTarget>,
    input_buffer: &mut String,
    pending_command: &mut Option<Vec<String>>,
) -> bool {
    match action.kind {
        ActionKind::DirectCommand => {
            if let Some(cmd) = &action.command {
                *pending_command = Some(cmd.clone());
                return true;
            }
        }
        ActionKind::PromptCreateTab => {
            *input_target = Some(ModalInputTarget::CreateTab);
            input_buffer.clear();
        }
        ActionKind::PromptRenameWorkspace => {
            *input_target = Some(ModalInputTarget::RenameWorkspace);
            *input_buffer = state.active_workspace_label.clone().unwrap_or_default();
        }
        ActionKind::PromptRenameTab => {
            *input_target = Some(ModalInputTarget::RenameTab);
            *input_buffer = state.active_tab_label.clone().unwrap_or_default();
        }
        ActionKind::CloseActiveWorkspace => {
            if let Some(ws_id) = &state.active_workspace_id {
                *pending_command = Some(vec![
                    "herdr".to_string(),
                    "workspace".to_string(),
                    "close".to_string(),
                    ws_id.clone(),
                ]);
                return true;
            }
        }
        ActionKind::CloseActiveTab => {
            if let Some(tab_id) = &state.active_tab_id {
                *pending_command = Some(vec![
                    "herdr".to_string(),
                    "tab".to_string(),
                    "close".to_string(),
                    tab_id.clone(),
                ]);
                return true;
            }
        }
        ActionKind::CloseActivePane => {
            if let Some(pane_id) = &state.active_pane {
                *pending_command = Some(vec![
                    "herdr".to_string(),
                    "pane".to_string(),
                    "close".to_string(),
                    pane_id.clone(),
                ]);
                return true;
            }
        }
        ActionKind::SplitVertical => {
            let mut cmd = vec![
                "herdr".to_string(),
                "pane".to_string(),
                "split".to_string(),
                "--direction".to_string(),
                "right".to_string(),
                "--focus".to_string(),
            ];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            }
            *pending_command = Some(cmd);
            return true;
        }
        ActionKind::SplitHorizontal => {
            let mut cmd = vec![
                "herdr".to_string(),
                "pane".to_string(),
                "split".to_string(),
                "--direction".to_string(),
                "down".to_string(),
                "--focus".to_string(),
            ];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            }
            *pending_command = Some(cmd);
            return true;
        }
        ActionKind::ToggleZoom => {
            let mut cmd = vec!["herdr".to_string(), "pane".to_string(), "zoom".to_string()];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            } else {
                cmd.push("--current".to_string());
            }
            *pending_command = Some(cmd);
            return true;
        }
        ActionKind::ShortcutOnly => {}
    }
    false
}

async fn run_modal_menu(
    client: HerdrClient,
    config: StatusBarConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = StatusBarState::new();
    if let Some(snapshot) = client.fetch_snapshot_sync() {
        HerdrClient::update_state_from_snapshot(&mut state, &snapshot);
    }

    // Drain initial leftover events from opening shortcut chord (e.g. Space)
    while event::poll(Duration::from_millis(20))? {
        let _ = event::read()?;
    }

    let categories = get_palette_categories(&config);
    let mut active_cat_idx: usize = 0;
    let mut active_action_idx: usize = 0;
    let mut input_target: Option<ModalInputTarget> = None;
    let mut input_buffer = String::new();
    let mut pending_command: Option<Vec<String>> = None;

    loop {
        let terminal_size = terminal.size()?;

        let prompt_data = input_target.map(|target| {
            let (target_label, current_label) = match target {
                ModalInputTarget::CreateTab => (
                    "Create New Tab",
                    "",
                ),
                ModalInputTarget::RenameWorkspace => (
                    "Workspace",
                    state.active_workspace_label.as_deref().unwrap_or(""),
                ),
                ModalInputTarget::RenameTab => (
                    "Tab",
                    state.active_tab_label.as_deref().unwrap_or(""),
                ),
            };
            (target_label, current_label, input_buffer.as_str())
        });

        terminal.draw(|f| {
            let size = f.area();
            f.render_widget(
                MenuModalWidget {
                    state: &state,
                    categories: &categories,
                    active_category_idx: active_cat_idx,
                    active_action_idx,
                    prompt_data,
                },
                size,
            );
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Ignore key release events
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    if let Some(target) = input_target {
                        // In interactive text prompt mode
                        match key.code {
                            KeyCode::Esc => {
                                input_target = None;
                                input_buffer.clear();
                            }
                            KeyCode::Enter => {
                                match target {
                                    ModalInputTarget::CreateTab => {
                                        let mut cmd = vec![
                                            "herdr".to_string(),
                                            "tab".to_string(),
                                            "create".to_string(),
                                            "--focus".to_string(),
                                        ];
                                        if !input_buffer.trim().is_empty() {
                                            cmd.push("--label".to_string());
                                            cmd.push(input_buffer.trim().to_string());
                                        }
                                        pending_command = Some(cmd);
                                        break;
                                    }
                                    ModalInputTarget::RenameWorkspace => {
                                        if !input_buffer.trim().is_empty() {
                                            if let Some(ws_id) = &state.active_workspace_id {
                                                pending_command = Some(vec![
                                                    "herdr".to_string(),
                                                    "workspace".to_string(),
                                                    "rename".to_string(),
                                                    ws_id.clone(),
                                                    input_buffer.trim().to_string(),
                                                ]);
                                                break;
                                            }
                                        }
                                        input_target = None;
                                    }
                                    ModalInputTarget::RenameTab => {
                                        if !input_buffer.trim().is_empty() {
                                            if let Some(tab_id) = &state.active_tab_id {
                                                pending_command = Some(vec![
                                                    "herdr".to_string(),
                                                    "tab".to_string(),
                                                    "rename".to_string(),
                                                    tab_id.clone(),
                                                    input_buffer.trim().to_string(),
                                                ]);
                                                break;
                                            }
                                        }
                                        input_target = None;
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                input_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                input_buffer.push(c);
                            }
                            _ => {}
                        }
                    } else {
                        // Normal menu navigation mode
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break;
                            }
                            KeyCode::Char('1') => {
                                active_cat_idx = 0;
                                active_action_idx = 0;
                            }
                            KeyCode::Char('2') => {
                                active_cat_idx = 1;
                                active_action_idx = 0;
                            }
                            KeyCode::Char('3') => {
                                active_cat_idx = 2;
                                active_action_idx = 0;
                            }
                            KeyCode::Left | KeyCode::BackTab => {
                                if active_cat_idx > 0 {
                                    active_cat_idx -= 1;
                                } else {
                                    active_cat_idx = categories.len() - 1;
                                }
                                active_action_idx = 0;
                            }
                            KeyCode::Right | KeyCode::Tab => {
                                active_cat_idx = (active_cat_idx + 1) % categories.len();
                                active_action_idx = 0;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if active_action_idx > 0 {
                                    active_action_idx -= 1;
                                } else {
                                    active_action_idx = categories[active_cat_idx].actions.len().saturating_sub(1);
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let total = categories[active_cat_idx].actions.len();
                                if total > 0 {
                                    active_action_idx = (active_action_idx + 1) % total;
                                }
                            }
                            KeyCode::Enter => {
                                let action = &categories[active_cat_idx].actions[active_action_idx];
                                if trigger_action(
                                    action,
                                    &state,
                                    &mut input_target,
                                    &mut input_buffer,
                                    &mut pending_command,
                                ) {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(MouseEvent { kind, column, row, .. }) => {
                    if input_target.is_none() {
                        match kind {
                            MouseEventKind::ScrollDown => {
                                let total = categories[active_cat_idx].actions.len();
                                if total > 0 {
                                    active_action_idx = (active_action_idx + 1) % total;
                                }
                            }
                            MouseEventKind::ScrollUp => {
                                if active_action_idx > 0 {
                                    active_action_idx -= 1;
                                } else {
                                    active_action_idx = categories[active_cat_idx].actions.len().saturating_sub(1);
                                }
                            }
                            MouseEventKind::Down(MouseButton::Left) => {
                                let area_y = 1;
                                let cat_tabs_y = area_y + 5;
                                let action_list_y_start = cat_tabs_y + 3;

                                if row >= cat_tabs_y && row < cat_tabs_y + 3 {
                                    if column < terminal_size.width / 3 {
                                        active_cat_idx = 0;
                                        active_action_idx = 0;
                                    } else if column < (terminal_size.width * 2) / 3 {
                                        active_cat_idx = 1;
                                        active_action_idx = 0;
                                    } else {
                                        active_cat_idx = 2;
                                        active_action_idx = 0;
                                    }
                                } else if row >= action_list_y_start {
                                    let clicked_idx = (row - action_list_y_start) as usize;
                                    let total_actions = categories[active_cat_idx].actions.len();
                                    if clicked_idx < total_actions {
                                        active_action_idx = clicked_idx;
                                        let action = &categories[active_cat_idx].actions[active_action_idx];
                                        if trigger_action(
                                            action,
                                            &state,
                                            &mut input_target,
                                            &mut input_buffer,
                                            &mut pending_command,
                                        ) {
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    // Execute selected command if triggered and wait for RPC completion
    if let Some(cmd) = pending_command {
        if !cmd.is_empty() {
            let mut proc = SysCommand::new(&cmd[0]);
            if cmd.len() > 1 {
                proc.args(&cmd[1..]);
            }
            let _ = proc.status();
        }
    }

    Ok(())
}

async fn run_tui_loop(
    client: HerdrClient,
    config: StatusBarConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = StatusBarState::new();
    let mut last_snapshot_fetch = std::time::Instant::now();

    // Initial snapshot fetch
    if let Some(snapshot) = client.fetch_snapshot_sync() {
        HerdrClient::update_state_from_snapshot(&mut state, &snapshot);
    }

    loop {
        // Redraw UI
        terminal.draw(|f| {
            let size = f.area();
            f.render_widget(
                StatusBarWidget {
                    state: &state,
                    config: &config,
                },
                size,
            );
        })?;

        // Handle events / keypresses
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if state.mode == HerdrMode::Normal {
                            break;
                        } else {
                            state.mode = HerdrMode::Normal;
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.mode = match state.mode {
                            HerdrMode::Normal => HerdrMode::Navigate,
                            HerdrMode::Navigate => HerdrMode::Normal,
                            _ => HerdrMode::Normal,
                        };
                    }
                    KeyCode::Char('v') if state.mode == HerdrMode::Navigate => {
                        state.mode = HerdrMode::Normal;
                    }
                    KeyCode::Char('-') if state.mode == HerdrMode::Navigate => {
                        state.mode = HerdrMode::Normal;
                    }
                    _ => {}
                }
            }
        }

        // Periodically refresh snapshot from Herdr
        if last_snapshot_fetch.elapsed() > Duration::from_secs(1) {
            if let Some(snapshot) = client.fetch_snapshot_sync() {
                HerdrClient::update_state_from_snapshot(&mut state, &snapshot);
            }
            last_snapshot_fetch = std::time::Instant::now();
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
