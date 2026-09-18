# Herdr HUD & Command Palette (`herdr-hud`)

An interactive command palette plugin for the [Herdr](https://herdr.dev) AI coding agent workspace manager.

---

## ✨ Features

* **Streamlined Tab Bar HUD (`line`):** High-contrast ANSI one-liner for embedding into Herdr's top `tab_bar_right`.
* **Interactive Command Palette & Quick Reference:**
  * **3 Structured Categories:** `[1. Workspaces]`, `[2. Tabs]`, and `[3. Panes]`.
  * **Interactive Tab Creation:** Prompt for a custom tab name directly in the modal before creating and focusing the new tab.
  * **Interactive Inline Renaming:** Rename active workspaces and tabs with live text input directly within the modal.
  * **Keyboard Navigation**
  * **Mouse Pointer Support:** Click category tabs to switch views, or click any action row to execute it immediately.

---

## 🚀 Installation

### Install as a Herdr Plugin
Install directly from GitHub using Herdr's plugin manager:
```bash
herdr plugin install erichjsonfosse/herdr-hud
```

## Testing

### Run Test Suite
```bash
cargo test
```

Runs the test suite across all modules (key parsing, path discovery, IPC client, config mapping, actions catalog, ANSI formatting, and modal widgets).

---

## 📖 Usage & Configuration

### 1. Top Tab Bar HUD Setup
In your `~/.config/herdr/config.local.toml`:

```toml
[ui]
tab_bar_position = "top"
tab_bar_right = [
  { type = "command", command = "herdr-hud line" }
]
```
> **Note:** If `herdr-hud` is not in your `$PATH`, provide the absolute binary path (e.g. `/<project-path>/target/release/herdr-hud line`).

### 2. Floating Command Palette Modal (`Ctrl+B Space`)
In your `~/.config/herdr/config.local.toml` (or `config.toml`), add under `[[keys.command]]`:

```toml
[[keys.command]]
key = "prefix+space"
type = "popup"
command = "herdr-hud menu"
width = "75%"
height = 22
description = "Herdr Command Palette & Shortcuts"
```
> **Note:** If `herdr-hud` is not in your `$PATH`, provide the absolute binary path (e.g. `/<project-path>/target/release/herdr-hud menu`).

Then reload Herdr:
```bash
herdr server reload-config
```
*(or press `Ctrl+B Shift+R`)*

### 3. CLI Subcommands
* **Print ANSI HUD Line (Default):**
  ```bash
  herdr-hud line
  # or simply:
  herdr-hud
  ```
* **Open Interactive Command Palette:**
  ```bash
  herdr-hud menu
  ```
* **IPC Socket Diagnostics:**
  ```bash
  herdr-hud check
  ```

---

## 📄 License

MIT License © 2026 Erich Json Fosse
