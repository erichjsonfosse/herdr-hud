# Codebase Restructuring & Test Expansion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the `herdr-status-bar` codebase into a clean library + binary architecture, decomposing `main.rs` and `ui.rs` into focused modules while maintaining strict separation of test files (`*_unit.rs`), followed by adding tests for previously untested areas.

**Architecture:** 
- Expose domain logic, config, state, palette, and UI widgets through `src/lib.rs` under the library target `herdr_status_bar`.
- Reduce `src/main.rs` to a lean entry point (~40 lines) solely responsible for CLI parsing and dispatching to runners.
- Separate command palette actions (`src/palette/`) and TUI rendering (`src/ui/`).
- Preserve the user's design choice: keep unit test suites in separate `*_unit.rs` files linked via `#[cfg(test)] #[path = "..._unit.rs"] mod tests;`.

**Tech Stack:** Rust 2024 edition, Cargo, Clap v4, Ratatui v0.29, Crossterm v0.28, Tokio, Serde/Serde_json, TOML.

---

### Task 1: Create `src/lib.rs` and Convert to Library + Binary Target

**Files:**
- Create: `src/lib.rs`
- Modify: `src/main.rs:1-30`
- Existing test: `cargo test`

**Interfaces:**
- Produces: `herdr_status_bar` library crate exporting `client`, `config`, `helpers`, `state`, and `ui` modules.
- Consumes: Existing module definitions in `src/`.

- [ ] **Step 1: Create `src/lib.rs`**
Expose public modules from the crate root:
```rust
pub mod client;
pub mod config;
pub mod helpers;
pub mod state;
pub mod ui;
```

- [ ] **Step 2: Update `src/main.rs` to import from library crate**
Replace the local module declarations at the top of `src/main.rs`:
```rust
use herdr_status_bar::client::HerdrClient;
use herdr_status_bar::config::StatusBarConfig;
use herdr_status_bar::state::{HerdrMode, StatusBarState};
use herdr_status_bar::ui::{
    get_palette_categories, render_ansi_line, ActionKind, MenuModalWidget, PaletteAction,
    StatusBarWidget,
};
```

- [ ] **Step 3: Verify library and binary build & existing tests pass**
Run: `cargo test`
Expected: 12 passed; 0 failed.

- [ ] **Step 4: Commit**
```bash
git add src/lib.rs src/main.rs
git commit -m "refactor: introduce src/lib.rs for library and binary separation"
```

---

### Task 2: Extract `palette` Domain Module from `ui.rs` & `main.rs`

**Files:**
- Create: `src/palette/mod.rs`
- Create: `src/palette/actions.rs`
- Create: `src/palette/actions_unit.rs`
- Modify: `src/lib.rs`
- Modify: `src/ui.rs`
- Modify: `src/ui_unit.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces:
  - `herdr_status_bar::palette::ActionKind`
  - `herdr_status_bar::palette::PaletteAction`
  - `herdr_status_bar::palette::PaletteCategory`
  - `herdr_status_bar::palette::ModalInputTarget`
  - `herdr_status_bar::palette::get_palette_categories(config: &StatusBarConfig) -> Vec<PaletteCategory>`
  - `herdr_status_bar::palette::trigger_action(action: &PaletteAction, state: &StatusBarState, input_target: &mut Option<ModalInputTarget>, input_buffer: &mut String, pending_command: &mut Option<Vec<String>>) -> bool`

- [ ] **Step 1: Create `src/palette/actions.rs`**
Move `ActionKind`, `PaletteAction`, `PaletteCategory`, and `get_palette_categories` from `src/ui.rs`, and move `ModalInputTarget` and `trigger_action` from `src/main.rs` into `src/palette/actions.rs`.
Link its test file at the bottom:
```rust
#[cfg(test)]
#[path = "actions_unit.rs"]
mod tests;
```

- [ ] **Step 2: Create `src/palette/mod.rs`**
Re-export items:
```rust
pub mod actions;
pub use actions::{
    get_palette_categories, trigger_action, ActionKind, ModalInputTarget, PaletteAction,
    PaletteCategory,
};
```

- [ ] **Step 3: Create `src/palette/actions_unit.rs`**
Move `test_palette_categories_contain_expected_items` from `src/ui_unit.rs` to `src/palette/actions_unit.rs`.

- [ ] **Step 4: Update `src/ui.rs`, `src/ui_unit.rs`, `src/lib.rs`, and `src/main.rs`**
Update imports to use `crate::palette::*` and `herdr_status_bar::palette::*`.

- [ ] **Step 5: Run tests to verify zero regressions**
Run: `cargo test`
Expected: 12 passed; 0 failed.

- [ ] **Step 6: Commit**
```bash
git add src/palette/ src/lib.rs src/ui.rs src/ui_unit.rs src/main.rs
git commit -m "refactor: extract palette domain logic into src/palette"
```

---

### Task 3: Decompose `src/ui.rs` into Modular Submodules

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/ansi.rs`
- Create: `src/ui/ansi_unit.rs`
- Create: `src/ui/modal.rs`
- Create: `src/ui/bar.rs`
- Delete: `src/ui.rs`
- Delete: `src/ui_unit.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces:
  - `herdr_status_bar::ui::ansi::render_ansi_line`
  - `herdr_status_bar::ui::modal::MenuModalWidget`
  - `herdr_status_bar::ui::bar::StatusBarWidget`

- [ ] **Step 1: Create `src/ui/ansi.rs` and `src/ui/ansi_unit.rs`**
Move `render_ansi_line` from `src/ui.rs` into `src/ui/ansi.rs`.
Move `test_render_ansi_line_basic` and `test_render_ansi_line_with_agents` into `src/ui/ansi_unit.rs`.

- [ ] **Step 2: Create `src/ui/modal.rs`**
Move `MenuModalWidget` and its `Widget` implementation from `src/ui.rs` into `src/ui/modal.rs`.

- [ ] **Step 3: Create `src/ui/bar.rs`**
Move `StatusBarWidget` and its `Widget` implementation from `src/ui.rs` into `src/ui/bar.rs`.

- [ ] **Step 4: Create `src/ui/mod.rs` and clean up old `ui.rs` & `ui_unit.rs`**
In `src/ui/mod.rs`, re-export:
```rust
pub mod ansi;
pub mod bar;
pub mod modal;

pub use ansi::render_ansi_line;
pub use bar::StatusBarWidget;
pub use modal::MenuModalWidget;
```
Delete `src/ui.rs` and `src/ui_unit.rs`.

- [ ] **Step 5: Run tests to verify all 12 pass**
Run: `cargo test`
Expected: 12 passed; 0 failed.

- [ ] **Step 6: Commit**
```bash
git add src/ui/ src/lib.rs
git rm src/ui.rs src/ui_unit.rs
git commit -m "refactor: split ui into modular ansi, modal, and bar submodules"
```

---

### Task 4: Move Interactive Loops from `main.rs` into `ui/modal.rs` & `ui/bar.rs`

**Files:**
- Modify: `src/ui/modal.rs`
- Modify: `src/ui/bar.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces:
  - `herdr_status_bar::ui::run_modal_menu(client: HerdrClient, config: StatusBarConfig) -> Result<(), Box<dyn std::error::Error>>`
  - `herdr_status_bar::ui::run_tui_loop(client: HerdrClient, config: StatusBarConfig) -> Result<(), Box<dyn std::error::Error>>`
- Consumes:
  - `main.rs` becomes a 45-line entry point handling only CLI parsing.

- [ ] **Step 1: Move `run_modal_menu` into `src/ui/modal.rs`**
Transfer terminal setup, event loop, and teardown for the modal palette into `src/ui/modal.rs`.

- [ ] **Step 2: Move `run_tui_loop` into `src/ui/bar.rs`**
Transfer terminal setup, poll loop, and teardown for the bottom bar into `src/ui/bar.rs`.

- [ ] **Step 3: Re-export runners from `src/ui/mod.rs`**
Add:
```rust
pub use bar::run_tui_loop;
pub use modal::run_modal_menu;
```

- [ ] **Step 4: Simplify `src/main.rs`**
Keep only `Cli`, `Commands`, and `main()`, delegating directly to `run_modal_menu`, `run_tui_loop`, and `render_ansi_line`.

- [ ] **Step 5: Run tests & verify binary builds**
Run: `cargo test && cargo build`
Expected: Exit code 0.

- [ ] **Step 6: Commit**
```bash
git add src/ui/modal.rs src/ui/bar.rs src/ui/mod.rs src/main.rs
git commit -m "refactor: extract interactive tui loops from main.rs into ui submodules"
```

---

### Task 5: Add Test Suite for Palette Actions (`src/palette/actions_unit.rs`)

**Files:**
- Modify: `src/palette/actions_unit.rs`

**Interfaces:**
- Tests `trigger_action` with all variants of `ActionKind`:
  - `DirectCommand` (verify command extraction)
  - `PromptCreateTab` (verify `input_target` and cleared buffer)
  - `PromptRenameWorkspace` (verify active workspace label loaded into buffer)
  - `PromptRenameTab` (verify active tab label loaded into buffer)
  - `CloseActiveWorkspace` (verify command `herdr workspace close <ws_id>`)
  - `CloseActiveTab` (verify command `herdr tab close <tab_id>`)
  - `CloseActivePane` (verify command `herdr pane close <pane_id>`)
  - `SplitVertical` & `SplitHorizontal` (verify direction flags and pane targeting)
  - `ToggleZoom` (verify zoom command formatting)
  - `ShortcutOnly` (verify returns false and leaves command empty)

- [ ] **Step 1: Write unit tests in `src/palette/actions_unit.rs`**
- [ ] **Step 2: Run `cargo test -- test_trigger_action`**
- [ ] **Step 3: Commit**
```bash
git add src/palette/actions_unit.rs
git commit -m "test: add comprehensive unit tests for palette trigger_action"
```

---

### Task 6: Add Test Suite for State & Enums (`src/state_unit.rs`)

**Files:**
- Create: `src/state_unit.rs`
- Modify: `src/state.rs`

**Interfaces:**
- Tests:
  - `HerdrMode::badge_name` for all variants (`Normal`, `Navigate`, `Scroll`, `Agent`).
  - `AgentStatus::symbol` and `AgentStatus::text` for all variants.
  - Serde roundtrip serialization/deserialization for `HerdrMode`, `AgentStatus`, and `StatusBarState`.

- [ ] **Step 1: Link `state_unit.rs` in `src/state.rs`**
Add:
```rust
#[cfg(test)]
#[path = "state_unit.rs"]
mod tests;
```

- [ ] **Step 2: Implement test cases in `src/state_unit.rs`**
- [ ] **Step 3: Run `cargo test -- state`**
- [ ] **Step 4: Commit**
```bash
git add src/state.rs src/state_unit.rs
git commit -m "test: add unit tests for state enums and serialization"
```

---

### Task 7: Add Test Suite for Paths & Socket Discovery (`src/helpers/paths_unit.rs`)

**Files:**
- Create: `src/helpers/paths_unit.rs`
- Modify: `src/helpers/paths.rs`

**Interfaces:**
- Tests:
  - `home_dir` returns valid non-empty path.
  - `herdr_config_path` defaults to `~/.config/herdr/config.toml` and respects `HERDR_CONFIG_PATH` env var.
  - `plugin_config_path` defaults to `~/.config/herdr/status-bar.json` and respects `HERDR_STATUS_BAR_CONFIG` env var.
  - `discover_socket` respects `HERDR_SOCKET` env var when file exists.

- [ ] **Step 1: Link `paths_unit.rs` in `src/helpers/paths.rs`**
Add:
```rust
#[cfg(test)]
#[path = "paths_unit.rs"]
mod tests;
```

- [ ] **Step 2: Implement test cases in `src/helpers/paths_unit.rs`**
- [ ] **Step 3: Run `cargo test -- paths`**
- [ ] **Step 4: Commit**
```bash
git add src/helpers/paths.rs src/helpers/paths_unit.rs
git commit -m "test: add unit tests for path and socket discovery helpers"
```

---

### Task 8: Expand Tests for `keys`, `client`, `config`, and `ui/ansi`

**Files:**
- Modify: `src/helpers/keys_unit.rs`
- Modify: `src/client_unit.rs`
- Modify: `src/config_unit.rs`
- Modify: `src/ui/ansi_unit.rs`

**Interfaces:**
- Keys: test arrow tokens (`up`, `down`, `left`, `right`), `meta`/`cmd`/`super`, `space`, `enter`, punctuation, and multi-key chords.
- Client: test snapshot fallback logic when IDs exist without arrays, and agent statuses (`done`, `idle`, `unknown`).
- Config: test `StatusBarConfig::load` fallback resolution and `navigate_hints`/`scroll_hints`.
- UI/ANSI: test `render_ansi_line` with `show_clock = true` and all agent statuses (`Blocked`, `Done`, `Idle`, `Unknown`).

- [ ] **Step 1: Add new test cases to each unit test file**
- [ ] **Step 2: Run full test suite: `cargo test`**
- [ ] **Step 3: Run `cargo clippy` and `cargo fmt --check`**
- [ ] **Step 4: Commit**
```bash
git add src/
git commit -m "test: expand coverage for keys, client snapshots, config, and ansi rendering"
```
