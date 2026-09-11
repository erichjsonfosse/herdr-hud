# Codebase Optimizations & Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement high-impact performance optimizations, configuration cascading (`config.local.toml`), state consolidation, palette action enhancements, and event-driven zero-CPU modal rendering.

**Architecture:**
- **Zero-overhead HUD (`line`):** Decouple `render_ansi_line` from runtime state and eliminate the `herdr api snapshot` subprocess invocation on status bar redraws.
- **Config Cascading:** Check `~/.config/herdr/config.local.toml` before `config.toml` in `herdr_config_path()`.
- **State Consolidation:** Remove duplicate label fields from `StatusBarState`.
- **Palette Actions Polish:** Add `--focus` to workspace creation and explicit `--toggle` to pane zoom.
- **Event-Driven Modal:** Redraw only on state changes/events, calculate accurate mouse hitboxes for category tabs, and dynamically truncate long descriptions for narrow popup windows.
- **Release Profile:** Add LTO, codegen-units, panic-abort, and stripping to `Cargo.toml`.

**Tech Stack:** Rust 2024 edition, Ratatui 0.29, Crossterm 0.28, Clap 4, Serde / Serde JSON, TOML 0.8.

## Global Constraints
- Strictly maintain test file separation: all unit tests live in `*_unit.rs` files linked via `#[path = "..."]`, NEVER inlined into source files.
- All git commits must be made with `--no-gpg-sign`.
- All tests must pass (`cargo test`), `cargo clippy --all-targets -- -D warnings` must have zero warnings, and `cargo fmt --check` must be clean.

---

### Task 1: Add Release Optimization Profile in `Cargo.toml`

**Files:**
- Modify: `Cargo.toml`

**Interfaces:**
- Consumes: Standard Cargo profile configuration.
- Produces: Optimized release binary configuration with LTO and symbol stripping.

- [ ] **Step 1: Update `Cargo.toml` with `[profile.release]`**

Add the release profile to the bottom of `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

- [x] **Step 2: Verify release compilation and check binary size**

Run: `cargo build --release`
Verify that `target/release/herdr-status-bar` builds successfully and is significantly smaller.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml
git commit --no-gpg-sign -m "perf: add release optimization profile with LTO and stripping"
```

---

### Task 2: Decouple `render_ansi_line` and Eliminate Subprocess in `Commands::Line`

**Files:**
- Modify: `src/ui/ansi.rs`
- Modify: `src/ui/ansi_unit.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `StatusBarConfig`
- Produces: `render_ansi_line(config: &StatusBarConfig) -> String`

- [x] **Step 1: Write test for new `render_ansi_line` signature**

In `src/ui/ansi_unit.rs`, update tests to call `render_ansi_line(&config)` directly without passing `&state`.

- [x] **Step 2: Update `src/ui/ansi.rs`**

Change signature:
```rust
pub fn render_ansi_line(config: &StatusBarConfig) -> String {
    let mut hint_parts = Vec::new();
    for hint in &config.hints {
        hint_parts.push(format!(
            "\x1b[36m\x1b[1m<{}>\x1b[0m {}",
            hint.key, hint.description
        ));
    }

    let hints_str = hint_parts.join("\x1b[90m │ \x1b[0m");
    if hints_str.is_empty() {
        "🐾 ".to_string()
    } else {
        format!("🐾 {}", hints_str)
    }
}
```

- [x] **Step 3: Update `src/main.rs` `Commands::Line` branch**

Eliminate `let mut state = StatusBarState::new();` and `client.fetch_snapshot_sync()` from `Commands::Line`:
```rust
        Commands::Line => {
            println!("{}", render_ansi_line(&config));
        }
```

- [x] **Step 4: Verify test suite and benchmark runtime**

Run: `cargo test ui::ansi::`
Run: `time ./target/release/herdr-status-bar line` (verify < 1ms execution)

- [x] **Step 5: Commit**

```bash
git add src/ui/ansi.rs src/ui/ansi_unit.rs src/main.rs
git commit --no-gpg-sign -m "perf: eliminate IPC subprocess from line command and decouple render_ansi_line"
```

---

### Task 3: Support `config.local.toml` in Config Resolution

**Files:**
- Modify: `src/helpers/paths.rs`
- Modify: `src/helpers/paths_unit.rs`

**Interfaces:**
- Consumes: Environment variable `HERDR_CONFIG_PATH` and filesystem paths `~/.config/herdr/config.local.toml`, `~/.config/herdr/config.toml`.
- Produces: `herdr_config_path() -> PathBuf`

- [x] **Step 1: Write failing test in `src/helpers/paths_unit.rs`**

Add tests:
- `test_herdr_config_path_prefers_config_local_toml`: when `config.local.toml` exists in `~/.config/herdr/`, `herdr_config_path()` returns it.
- `test_herdr_config_path_falls_back_to_config_toml`: when only `config.toml` exists, it returns `config.toml`.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test helpers::paths::tests::test_herdr_config_path_prefers_config_local_toml`
Expected: FAIL

- [x] **Step 3: Implement in `src/helpers/paths.rs`**

```rust
pub fn herdr_config_path() -> PathBuf {
    if let Ok(custom) = std::env::var("HERDR_CONFIG_PATH") {
        return PathBuf::from(custom);
    }
    let local_path = home_dir().join(".config/herdr/config.local.toml");
    if local_path.exists() {
        return local_path;
    }
    home_dir().join(".config/herdr/config.toml")
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test helpers::paths::`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/helpers/paths.rs src/helpers/paths_unit.rs
git commit --no-gpg-sign -m "feat: check config.local.toml before config.toml in herdr_config_path"
```

---

### Task 4: Consolidate Redundant Fields in `StatusBarState` & `HerdrClient`

**Files:**
- Modify: `src/state.rs`
- Modify: `src/state_unit.rs`
- Modify: `src/client.rs`
- Modify: `src/client_unit.rs`
- Modify: `src/palette/actions.rs`
- Modify: `src/palette/actions_unit.rs`
- Modify: `src/ui/modal.rs`

**Interfaces:**
- Consumes: Herdr snapshot JSON
- Produces: `StatusBarState { active_workspace_id, active_workspace, active_tab_id, active_tab, active_pane }`

- [x] **Step 1: Update `StatusBarState` in `src/state.rs`**

Remove `active_workspace_label` and `active_tab_label`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StatusBarState {
    pub active_workspace_id: Option<String>,
    pub active_workspace: Option<String>,
    pub active_tab_id: Option<String>,
    pub active_tab: Option<String>,
    pub active_pane: Option<String>,
}
```

- [x] **Step 2: Update `src/client.rs` and `src/client_unit.rs`**

In `src/client.rs`:
Set `state.active_workspace = Some(label.to_string())` and `state.active_tab = Some(label.to_string())`.
In `src/client_unit.rs`:
Remove assertions on `.active_workspace_label` and `.active_tab_label`.

- [x] **Step 3: Update `src/palette/actions.rs` and `src/ui/modal.rs`**

In `src/palette/actions.rs`:
Replace `state.active_workspace_label` with `state.active_workspace`.
Replace `state.active_tab_label` with `state.active_tab`.
In `src/ui/modal.rs`:
Replace `state.active_workspace_label` with `state.active_workspace`.
Replace `state.active_tab_label` with `state.active_tab`.

- [x] **Step 4: Update `src/state_unit.rs` and `src/palette/actions_unit.rs`**

Update test assertions in `state_unit.rs` and `actions_unit.rs`.

- [x] **Step 5: Run tests to verify all pass**

Run: `cargo test`
Expected: PASS (all 71 tests passing)

- [x] **Step 6: Commit**

```bash
git add src/state.rs src/state_unit.rs src/client.rs src/client_unit.rs src/palette/actions.rs src/palette/actions_unit.rs src/ui/modal.rs
git commit --no-gpg-sign -m "refactor: consolidate duplicate label fields in StatusBarState"
```

---

### Task 5: Enhance Palette Actions with `--focus` and Explicit Zoom `--toggle`

**Files:**
- Modify: `src/palette/actions.rs`
- Modify: `src/palette/actions_unit.rs`

**Interfaces:**
- Consumes: Palette actions catalog
- Produces: CLI commands for "Create Workspace" with `--focus` and "Toggle Zoom" with `--toggle`

- [x] **Step 1: Write tests for action command generation**

In `src/palette/actions_unit.rs`, test that:
- "Create Workspace" command contains `"--focus"`.
- "Toggle Zoom" command contains `"--toggle"`.

- [x] **Step 2: Implement in `src/palette/actions.rs`**

In `get_palette_categories`:
```rust
                PaletteAction {
                    name: "Create Workspace",
                    key: format!("{} Shift+N", p),
                    description: "Create and switch to a new workspace",
                    kind: ActionKind::DirectCommand,
                    command: Some(vec![
                        "herdr".to_string(),
                        "workspace".to_string(),
                        "create".to_string(),
                        "--focus".to_string(),
                    ]),
                },
```
In `trigger_action` for `ActionKind::ToggleZoom`:
```rust
        ActionKind::ToggleZoom => {
            let mut cmd = vec![
                "herdr".to_string(),
                "pane".to_string(),
                "zoom".to_string(),
                "--toggle".to_string(),
            ];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            } else {
                cmd.push("--current".to_string());
            }
            *pending_command = Some(cmd);
            return true;
        }
```

- [x] **Step 3: Run tests to verify they pass**

Run: `cargo test palette::actions::`
Expected: PASS

- [x] **Step 4: Commit**

```bash
git add src/palette/actions.rs src/palette/actions_unit.rs
git commit --no-gpg-sign -m "feat: add --focus to workspace creation and --toggle to pane zoom"
```

---

### Task 6: Event-Driven Modal Loop, Accurate Mouse Hitboxes, and Responsive Truncation

**Files:**
- Modify: `src/ui/modal.rs`

**Interfaces:**
- Consumes: Crossterm events, Ratatui layout
- Produces: Event-driven modal loop (0% idle CPU), accurate mouse hitbox detection, and responsive description rendering

- [x] **Step 1: Implement responsive description truncation in `MenuModalWidget`**

In `MenuModalWidget::render`, compute available space for the description:
- Line width minus fixed column widths (pointer, key, action name, execution badge).
- Truncate description gracefully if terminal/popup width is narrow.

- [x] **Step 2: Implement accurate mouse hitbox detection for category tabs**

In `run_modal_menu` mouse event handling:
Instead of `column < terminal_size.width / 3`:
Compute actual tab character ranges:
- Tab 1: column 1..18
- Tab 2: column 20..31
- Tab 3: column 33..45
Map click column within these ranges to select the category.

- [x] **Step 3: Convert modal loop to event-driven dirty-flag rendering**

Structure the loop with `dirty` boolean flag and render only on state mutations or window resize events.

- [x] **Step 4: Verify test suite, clippy, and formatting**

Run: `cargo test`
Run: `cargo clippy --all-targets -- -D warnings`
Run: `cargo fmt --check`

- [x] **Step 5: Commit**

```bash
git add src/ui/modal.rs
git commit --no-gpg-sign -m "perf: event-driven modal rendering, accurate mouse hitboxes, and responsive truncation"
```

---

### Task 7: Full Verification and Documentation Update

**Files:**
- Modify: `README.md` (if needed)

- [x] **Step 1: Run comprehensive verification**

Run: `cargo test`
Run: `cargo clippy --all-targets -- -D warnings`
Run: `cargo fmt --check`
Run: `cargo build --release`

- [x] **Step 2: Test CLI subcommands**

Run: `./target/release/herdr-status-bar line`
Run: `./target/release/herdr-status-bar check`

- [x] **Step 3: Commit documentation or final adjustments if any**

```bash
git add README.md
git commit --no-gpg-sign -m "docs: document performance optimizations and config.local.toml support"
```
