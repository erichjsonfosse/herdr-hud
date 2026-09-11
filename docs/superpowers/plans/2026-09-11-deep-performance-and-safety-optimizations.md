# Deep Performance, I/O, and Safety Optimizations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement second-round deep optimizations across the codebase to minimize heap allocations on hot paths, halve filesystem syscalls, eliminate zero-copy overheads in IPC, add XDG runtime directory support with lazy candidate evaluation, and install terminal panic safety hooks.

**Architecture:** Stream ANSI string generation into pre-allocated buffers, switch JSON deserialization from child process stdout to zero-copy byte slice parsing, remove redundant `exists()` syscalls before file reading, evaluate socket discovery paths lazily with priority to `$XDG_RUNTIME_DIR`, adopt `Cow<'static, str>` for key token formatting, and guard TUI raw mode with a custom panic hook.

**Tech Stack:** Rust 2024 edition, Cargo, Clap v4, Ratatui v0.29, Crossterm v0.28, Serde / Serde_json, TOML.

## Global Constraints
- Rust 2024 edition.
- Keep unit tests in separate `*_unit.rs` files; do NOT move tests into source files. Link via `#[cfg(test)] #[path = "..._unit.rs"] mod tests;`.
- Global git config has `commit.gpgsign=true`. Always pass `--no-gpg-sign` when committing (`git commit --no-gpg-sign -m "..."`).
- Maintain 100% test passing rate and zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).

---

### Task 1: Single-Allocation Streamed ANSI Status Line

**Files:**
- Modify: `src/ui/ansi.rs`
- Modify: `src/ui/ansi_unit.rs`

**Interfaces:**
- Consumes: `StatusBarConfig`
- Produces: `render_ansi_line(config: &StatusBarConfig) -> String`

- [x] **Step 1: Write test for preallocated streaming behavior**

In `src/ui/ansi_unit.rs`, ensure tests verify:
- Empty hints returns `"🐾 "`.
- Single hint returns `"🐾 \x1b[36m\x1b[1m<Prefix>\x1b[0m Ctrl+B"`.
- Multiple hints separated by ` │ `.

- [x] **Step 2: Run tests to verify current baseline passes**

Run: `cargo test ui::ansi::`
Expected: PASS

- [x] **Step 3: Refactor `render_ansi_line` to stream into single `String`**

In `src/ui/ansi.rs`:
```rust
pub fn render_ansi_line(config: &StatusBarConfig) -> String {
    if config.hints.is_empty() {
        return "🐾 ".to_string();
    }
    let mut output = String::with_capacity(128);
    output.push_str("🐾 ");
    for (i, hint) in config.hints.iter().enumerate() {
        if i > 0 {
            output.push_str("\x1b[90m │ \x1b[0m");
        }
        output.push_str("\x1b[36m\x1b[1m<");
        output.push_str(&hint.key);
        output.push_str(">\x1b[0m ");
        output.push_str(&hint.description);
    }
    output
}
```

- [x] **Step 4: Run tests and clippy**

Run: `cargo test ui::ansi::`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/ui/ansi.rs src/ui/ansi_unit.rs
git commit --no-gpg-sign -m "perf: stream render_ansi_line directly into preallocated buffer"
```

---

### Task 2: Zero-Copy Snapshot Parsing & Borrowed Identifiers in IPC Client

**Files:**
- Modify: `src/client.rs`
- Modify: `src/client_unit.rs`

**Interfaces:**
- Consumes: Raw child process `output.stdout: Vec<u8>`
- Produces: `HerdrClient::fetch_snapshot_sync(&self) -> Option<Value>` via `serde_json::from_slice`

- [x] **Step 1: Write test for invalid UTF-8 bytes handling in `client_unit.rs`**

Add test verifying `fetch_snapshot_sync` handles malformed or non-UTF8 bytes gracefully without panics.

- [x] **Step 2: Update `fetch_snapshot_sync` and `update_state_from_snapshot`**

In `src/client.rs`:
- Replace `String::from_utf8_lossy(&output.stdout)` with `serde_json::from_slice(&output.stdout).ok()`.
- In `update_state_from_snapshot`, keep `focused_ws_id` and `focused_tab_id` as `Option<&str>` without calling `.map(|s| s.to_string())` upfront:
```rust
let focused_ws_id = session.get("focused_workspace_id").and_then(|v| v.as_str());
let focused_tab_id = session.get("focused_tab_id").and_then(|v| v.as_str());
let focused_pane_id = session.get("focused_pane_id").and_then(|v| v.as_str());

if let Some(pane_id) = focused_pane_id {
    state.active_pane = Some(pane_id.to_string());
}
```

- [x] **Step 3: Run tests and clippy**

Run: `cargo test client::`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS

- [x] **Step 4: Commit**

```bash
git add src/client.rs src/client_unit.rs
git commit --no-gpg-sign -m "perf: parse snapshot directly from byte slice without string reallocation"
```

---

### Task 3: Direct Config Reading & Syscall Reduction

**Files:**
- Modify: `src/config.rs`
- Modify: `src/config_unit.rs`

**Interfaces:**
- Consumes: Config candidate paths
- Produces: `StatusBarConfig::load() -> StatusBarConfig`

- [x] **Step 1: Update `StatusBarConfig::load` and `get_key` closure**

In `src/config.rs`:
- Replace `if path.exists() && let Ok(content) = std::fs::read_to_string(&path)` with `if let Ok(content) = std::fs::read_to_string(&path)`.
- In `from_herdr_config`, change `get_key` closure to return `&str`:
```rust
let get_key = |key_name: &str, default_val: &'a str| -> &'a str {
    config
        .and_then(|c| {
            c.get(key_name)
                .or_else(|| c.get("keybindings").and_then(|k| k.get(key_name)))
                .or_else(|| c.get("keys").and_then(|k| k.get(key_name)))
        })
        .and_then(|v| v.as_str())
        .unwrap_or(default_val)
};
```

- [x] **Step 2: Run tests and clippy**

Run: `cargo test config::`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS

- [x] **Step 3: Commit**

```bash
git add src/config.rs src/config_unit.rs
git commit --no-gpg-sign -m "perf: reduce filesystem syscalls and intermediate strings in config loading"
```

---

### Task 4: Lazy Evaluation & `$XDG_RUNTIME_DIR` in Socket Discovery

**Files:**
- Modify: `src/helpers/paths.rs`
- Modify: `src/helpers/paths_unit.rs`

**Interfaces:**
- Consumes: `$HERDR_SOCKET`, `$XDG_RUNTIME_DIR`, `$HOME`
- Produces: `discover_socket() -> Option<PathBuf>`

- [x] **Step 1: Write test for `$XDG_RUNTIME_DIR` in `paths_unit.rs`**

Add unit test `test_discover_socket_with_xdg_runtime_dir`:
When `XDG_RUNTIME_DIR` is set and contains `herdr/herdr.sock`, `discover_socket()` discovers it.

- [x] **Step 2: Implement lazy socket evaluation in `paths.rs`**

In `src/helpers/paths.rs`:
1. Check `$HERDR_SOCKET`.
2. Check `home.join(".herdr/herdr.sock")`.
3. Check `home.join(".config/herdr/herdr.sock")`.
4. If `$XDG_RUNTIME_DIR` is set: check `$XDG_RUNTIME_DIR/herdr/herdr.sock`.
5. Fallback: check `/run/user/<uid>/herdr/herdr.sock`.
6. Fallback: check `/tmp/herdr-<uid>.sock`.
Evaluate lazily with `.find_map()` or sequential checks so unnecessary paths and `getuid()` calls are avoided.

- [x] **Step 3: Run tests and clippy**

Run: `cargo test helpers::paths::`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS

- [x] **Step 4: Commit**

```bash
git add src/helpers/paths.rs src/helpers/paths_unit.rs
git commit --no-gpg-sign -m "perf: lazily evaluate socket discovery candidates with XDG_RUNTIME_DIR support"
```

---

### Task 5: Zero-Allocation Key Token Formatting & Streamed Chord Joins

**Files:**
- Modify: `src/helpers/keys.rs`
- Modify: `src/helpers/keys_unit.rs`

**Interfaces:**
- Consumes: Key token/chord strings
- Produces: `format_key_token(token: &str) -> Cow<'static, str>`, `format_key_chord(chord: &str) -> String`

- [x] **Step 1: Update `format_key_token` to return `Cow<'static, str>`**

In `src/helpers/keys.rs`:
```rust
use std::borrow::Cow;

pub fn format_key_token(token: &str) -> Cow<'static, str> {
    match token.to_lowercase().as_str() {
        "ctrl" => Cow::Borrowed("Ctrl"),
        "alt" => Cow::Borrowed("Alt"),
        "shift" => Cow::Borrowed("Shift"),
        "cmd" | "super" | "meta" => Cow::Borrowed("Meta"),
        "esc" | "escape" => Cow::Borrowed("Esc"),
        "enter" | "return" => Cow::Borrowed("Enter"),
        "tab" => Cow::Borrowed("Tab"),
        "space" => Cow::Borrowed("Space"),
        "minus" => Cow::Borrowed("-"),
        "plus" => Cow::Borrowed("+"),
        "comma" => Cow::Borrowed(","),
        "period" | "dot" => Cow::Borrowed("."),
        "slash" => Cow::Borrowed("/"),
        "backslash" => Cow::Borrowed("\\"),
        "backtick" => Cow::Borrowed("`"),
        "up" => Cow::Borrowed("▲"),
        "down" => Cow::Borrowed("▼"),
        "left" => Cow::Borrowed("◄"),
        "right" => Cow::Borrowed("►"),
        other => {
            if other.len() == 1 {
                Cow::Owned(other.to_lowercase())
            } else {
                let mut c = other.chars();
                match c.next() {
                    None => Cow::Borrowed(""),
                    Some(f) => Cow::Owned(f.to_uppercase().collect::<String>() + c.as_str()),
                }
            }
        }
    }
}
```

- [x] **Step 2: Stream `format_key_chord` without intermediate vectors**

Stream the tokens directly into a pre-sized `String` buffer.

- [x] **Step 3: Run tests and clippy**

Run: `cargo test helpers::keys::`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS

- [x] **Step 4: Commit**

```bash
git add src/helpers/keys.rs src/helpers/keys_unit.rs
git commit --no-gpg-sign -m "perf: return Cow in format_key_token and stream chord joins"
```

---

### Task 6: Terminal Panic Hook for Raw Mode Crash Safety

**Files:**
- Modify: `src/ui/terminal.rs`
- Modify: `src/ui/terminal_unit.rs`

**Interfaces:**
- Consumes: Crossterm terminal state
- Produces: `TerminalGuard` with panic hook protection

- [x] **Step 1: Write test for panic hook installation and restoration in `terminal_unit.rs`**

- [x] **Step 2: Implement panic hook in `src/ui/terminal.rs`**

Store previous panic hook via `std::panic::take_hook()` and set a custom hook that calls `disable_raw_mode()`, `DisableMouseCapture`, `LeaveAlternateScreen`, `Show` before forwarding to the previous hook. Restore previous hook in `Drop`.

- [x] **Step 3: Run tests and clippy**

Run: `cargo test ui::terminal::`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS

- [x] **Step 4: Commit**

```bash
git add src/ui/terminal.rs src/ui/terminal_unit.rs
git commit --no-gpg-sign -m "fix: add panic hook in TerminalGuard to guarantee terminal restoration on crash"
```

---

### Task 7: Subcommand Dispatch Hygiene in `main.rs`

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: CLI command
- Produces: Lazy resource initialization per command

- [x] **Step 1: Move `StatusBarConfig::load()` and `HerdrClient::new()` into match arms**

In `src/main.rs`:
```rust
match cli.command.unwrap_or(Commands::Line) {
    Commands::Line => {
        let config = StatusBarConfig::load();
        println!("{}", render_ansi_line(&config));
    }
    Commands::Menu => {
        let config = StatusBarConfig::load();
        let client = HerdrClient::new();
        run_modal_menu(client, config)?;
    }
    Commands::Check => {
        let client = HerdrClient::new();
        ...
    }
}
```

- [x] **Step 2: Run tests and clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Run: `cargo test`
Expected: PASS

- [x] **Step 3: Commit**

```bash
git add src/main.rs
git commit --no-gpg-sign -m "refactor: initialize config and client lazily per subcommand"
```

---

### Task 8: End-to-End Verification & Documentation Polish

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-09-11-deep-performance-and-safety-optimizations.md`

- [x] **Step 1: Run comprehensive verification**

Run: `cargo test`
Run: `cargo clippy --all-targets -- -D warnings`
Run: `cargo fmt --check`
Run: `cargo build --release`

- [x] **Step 2: Test release CLI subcommands**

Run: `target/release/herdr-status-bar line`
Run: `target/release/herdr-status-bar check`

- [x] **Step 3: Commit documentation and plan updates**

```bash
git add README.md docs/superpowers/plans/2026-09-11-deep-performance-and-safety-optimizations.md
git commit --no-gpg-sign -m "docs: finalize round 2 performance and safety optimizations"
```
