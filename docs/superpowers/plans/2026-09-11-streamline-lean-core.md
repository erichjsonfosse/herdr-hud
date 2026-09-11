# Streamline `herdr-status-bar` to Lean Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Streamline `herdr-status-bar` by stripping out unused features (`HerdrMode`, agent statuses, clock, standalone TUI `run` command) and heavy dependencies (`tokio`, `chrono`), keeping only the fast one-shot ANSI HUD (`line`) and interactive popup modal (`menu`).

**Architecture:** 
- Convert `StatusBarState` to store only focused workspace/tab/pane state required by the command palette modal.
- Simplify `StatusBarConfig` to maintain a single list of `hints` (or `shortcuts`), removing mode-specific hint sets, `show_agents`, and `show_clock`.
- Eliminate `src/ui/bar.rs` (interactive terminal loop & widget).
- Remove async runtime (`tokio`) and `chrono`, converting the application and `run_modal_menu` to pure synchronous execution.

**Tech Stack:** Rust 2024 edition, Cargo, Clap v4, Ratatui v0.29, Crossterm v0.28, Serde / Serde JSON, TOML.

## Global Constraints
- Keep unit tests in separate `*_unit.rs` files; do NOT move tests into source files.
- Always commit with `git commit --no-gpg-sign -m "..."`.
- All tests must pass with `cargo test`.
- Linter must pass with `cargo clippy --all-targets -- -D warnings`.
- Code must be formatted with `cargo fmt --check`.

---

### Task 1: Prune `src/state.rs` and Update `src/state_unit.rs`

**Files:**
- Modify: `src/state.rs`
- Modify: `src/state_unit.rs`

**Interfaces:**
- Consumes: Nothing
- Produces:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
  pub struct StatusBarState {
      pub active_workspace_id: Option<String>,
      pub active_workspace_label: Option<String>,
      pub active_workspace: Option<String>,
      pub active_tab_id: Option<String>,
      pub active_tab_label: Option<String>,
      pub active_tab: Option<String>,
      pub active_pane: Option<String>,
  }
  ```
  `HerdrMode`, `AgentStatus`, `AgentEntry`, and `agents` map are removed.

- [ ] **Step 1: Write the updated unit tests in `src/state_unit.rs`**

Replace `src/state_unit.rs` with tests verifying `StatusBarState::new()`, `StatusBarState::default()`, and serde JSON serialization/deserialization without `HerdrMode` or `AgentStatus`:

```rust
use super::*;
use serde_json::json;

#[test]
fn test_status_bar_state_default() {
    let state = StatusBarState::default();
    assert!(state.active_workspace_id.is_none());
    assert!(state.active_workspace_label.is_none());
    assert!(state.active_workspace.is_none());
    assert!(state.active_tab_id.is_none());
    assert!(state.active_tab_label.is_none());
    assert!(state.active_tab.is_none());
    assert!(state.active_pane.is_none());
}

#[test]
fn test_status_bar_state_new() {
    let state = StatusBarState::new();
    assert_eq!(state, StatusBarState::default());
}

#[test]
fn test_status_bar_state_serde_roundtrip_default() {
    let state = StatusBarState::default();
    let serialized = serde_json::to_string(&state).expect("serialize default");
    let deserialized: StatusBarState = serde_json::from_str(&serialized).expect("deserialize default");
    assert_eq!(state, deserialized);
}

#[test]
fn test_status_bar_state_serde_roundtrip_populated() {
    let state = StatusBarState {
        active_workspace_id: Some("ws-1".to_string()),
        active_workspace_label: Some("Primary".to_string()),
        active_workspace: Some("Primary".to_string()),
        active_tab_id: Some("tab-4".to_string()),
        active_tab_label: Some("Editor".to_string()),
        active_tab: Some("Editor".to_string()),
        active_pane: Some("pane-12".to_string()),
    };

    let serialized = serde_json::to_string(&state).expect("serialize populated");
    let deserialized: StatusBarState = serde_json::from_str(&serialized).expect("deserialize populated");
    assert_eq!(state, deserialized);
}
```

- [ ] **Step 2: Update `src/state.rs`**

Remove `HerdrMode`, `AgentStatus`, `AgentEntry`, and `agents` from `src/state.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StatusBarState {
    pub active_workspace_id: Option<String>,
    pub active_workspace_label: Option<String>,
    pub active_workspace: Option<String>,
    pub active_tab_id: Option<String>,
    pub active_tab_label: Option<String>,
    pub active_tab: Option<String>,
    pub active_pane: Option<String>,
}

impl StatusBarState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[path = "state_unit.rs"]
mod tests;
```

- [ ] **Step 3: Temporarily fix downstream references in other modules to compile tests**

Temporarily update `src/lib.rs` (remove `AgentStatus`, `HerdrMode` exports) and check that `src/state.rs` compiles.

- [ ] **Step 4: Commit**

```bash
git add src/state.rs src/state_unit.rs
git commit --no-gpg-sign -m "refactor: simplify StatusBarState and remove HerdrMode and AgentStatus"
```

---

### Task 2: Simplify `src/config.rs` and Update `src/config_unit.rs`

**Files:**
- Modify: `src/config.rs`
- Modify: `src/config_unit.rs`

**Interfaces:**
- Consumes: `src/helpers/paths.rs`
- Produces:
  ```rust
  #[derive(Debug, Clone, Deserialize, Serialize)]
  pub struct StatusBarConfig {
      pub prefix: String,
      pub hints: Vec<KeyHint>,
  }
  ```
  `navigate_hints`, `scroll_hints`, `show_agents`, and `show_clock` are removed.

- [ ] **Step 1: Update `src/config.rs`**

Simplify `StatusBarConfig` to hold `prefix` and `hints`:
- `from_herdr_config(table: &Table) -> Self`:
  Extracts `prefix` and builds `hints` from `keys.command` or `keys` section.
- `default()`:
  Default `prefix: "Ctrl+b"`.
  Default `hints`:
  `<prefix> c` New Tab, `<prefix> v` Split V, `<prefix> -` Split H, `<prefix> Space` Menu, `<prefix> z` Zoom, `<prefix> x` Close.

- [ ] **Step 2: Update `src/config_unit.rs`**

Update configuration tests in `src/config_unit.rs` to verify:
- Default `hints` and prefix.
- Custom hints from TOML and JSON.
- Resolution order (JSON override -> TOML fallback -> defaults).

- [ ] **Step 3: Run config tests**

Run: `cargo test config::`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/config_unit.rs
git commit --no-gpg-sign -m "refactor: streamline StatusBarConfig to single hints list"
```

---

### Task 3: Simplify `src/client.rs` and Update `src/client_unit.rs`

**Files:**
- Modify: `src/client.rs`
- Modify: `src/client_unit.rs`

**Interfaces:**
- Consumes: `StatusBarState`
- Produces:
  `HerdrClient::new()`, `HerdrClient::discover_socket()`, `fetch_snapshot_sync()`, `update_state_from_snapshot(&mut state, &snapshot)`.
  Removes `connect_stream`, `tokio::net::UnixStream`, `socket_path` field, and pane agent status parsing.

- [ ] **Step 1: Update `src/client.rs`**

Remove `socket_path`, `connect_stream`, `tokio::net::UnixStream`, and agent loop from `update_state_from_snapshot`. Only extract `focused_workspace_id`, `focused_tab_id`, `focused_pane_id`, and match `workspaces` and `tabs` arrays to set `active_workspace...`, `active_tab...`, and `active_pane`.

- [ ] **Step 2: Update `src/client_unit.rs`**

Update tests in `src/client_unit.rs` to verify snapshot parsing for workspaces, tabs, and pane IDs without asserting agent statuses.

- [ ] **Step 3: Run client tests**

Run: `cargo test client::`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/client.rs src/client_unit.rs
git commit --no-gpg-sign -m "refactor: remove socket streaming and agent parsing from client"
```

---

### Task 4: Streamline `src/ui/ansi.rs` and Update `src/ui/ansi_unit.rs`

**Files:**
- Modify: `src/ui/ansi.rs`
- Modify: `src/ui/ansi_unit.rs`

**Interfaces:**
- Consumes: `StatusBarState`, `StatusBarConfig`
- Produces: `pub fn render_ansi_line(state: &StatusBarState, config: &StatusBarConfig) -> String`

- [ ] **Step 1: Update `src/ui/ansi.rs`**

Simplify `render_ansi_line`:
- Format: `🐾 ` + joined hints (e.g. `\x1b[36m\x1b[1m<Ctrl+B c>\x1b[0m New Tab`).
- No clock formatting.
- No agent status formatting.
- No mode match branching.

- [ ] **Step 2: Update `src/ui/ansi_unit.rs`**

Update unit tests to test:
- Basic rendering of `render_ansi_line` with default hints.
- Rendering with custom hints.
- Empty hints rendering (`🐾 `).

- [ ] **Step 3: Run ANSI tests**

Run: `cargo test ui::ansi::`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/ui/ansi.rs src/ui/ansi_unit.rs
git commit --no-gpg-sign -m "refactor: simplify render_ansi_line removing clock, agents, and modes"
```

---

### Task 5: Remove `src/ui/bar.rs` and Clean Up `src/ui/mod.rs` & `src/lib.rs`

**Files:**
- Delete: `src/ui/bar.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `src/ui/ansi.rs`, `src/ui/modal.rs`, `src/ui/terminal.rs`
- Produces: `herdr_status_bar::ui::{render_ansi_line, run_modal_menu}`

- [ ] **Step 1: Delete `src/ui/bar.rs`**

Remove `src/ui/bar.rs`.

- [ ] **Step 2: Update `src/ui/mod.rs`**

Remove `pub mod bar;` and `pub use bar::{StatusBarWidget, run_tui_loop};`.
Keep `pub mod ansi;`, `pub mod modal;`, `pub mod terminal;`.
Re-export `render_ansi_line` and `run_modal_menu`.

- [ ] **Step 3: Update `src/lib.rs`**

Ensure re-exports in `src/lib.rs` cleanly expose `client`, `config`, `helpers`, `palette`, `state`, `ui`.

- [ ] **Step 4: Commit**

```bash
git rm src/ui/bar.rs
git add src/ui/mod.rs src/lib.rs
git commit --no-gpg-sign -m "refactor: remove standalone terminal bar module"
```

---

### Task 6: Remove Tokio & Chrono, Make Modal Menu and Main Synchronous

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/ui/modal.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: Synchronous crossterm and ratatui
- Produces:
  - `pub fn run_modal_menu(client: HerdrClient, config: StatusBarConfig) -> Result<(), Box<dyn std::error::Error>>`
  - `fn main() -> Result<(), Box<dyn std::error::Error>>`
  - CLI default subcommand: `Line`. Subcommands: `Line`, `Menu`, `Check`.

- [ ] **Step 1: Convert `run_modal_menu` in `src/ui/modal.rs` to synchronous `pub fn`**

Change `pub async fn run_modal_menu` to `pub fn run_modal_menu`.

- [ ] **Step 2: Update `src/main.rs`**

- Remove `#[tokio::main]`.
- Remove `Commands::Run`.
- Default to `Commands::Line` (or CLI without args prints the status line).
- Call `run_modal_menu(client, config)?;` synchronously.

- [ ] **Step 3: Remove `tokio` and `chrono` from `Cargo.toml`**

Remove:
```toml
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
```
from `Cargo.toml`.

- [ ] **Step 4: Verify build and tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/ui/modal.rs src/main.rs
git commit --no-gpg-sign -m "refactor: remove tokio and chrono dependencies; convert to synchronous execution"
```

---

### Task 7: Update README and Final Verification

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update `README.md`**

Update features, usage instructions, and CLI subcommands to reflect the lean core (`line`, `menu`, `check`), removing references to `run`, clock, and agent status.

- [ ] **Step 2: Run complete verification suite**

Run:
1. `cargo test`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo fmt --check`

Expected: ALL PASS with zero warnings and zero errors.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit --no-gpg-sign -m "docs: update README for lean status bar and command palette"
```
