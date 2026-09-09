# Herdr Status Bar & Command Palette (`herdr-status-bar`)

A context-based, Zellij-inspired status bar and interactive command palette plugin for the [Herdr](https://herdr.dev) AI coding agent workspace manager.

---

## ✨ Features

* **Interactive Command Palette & Quick Reference (`menu`):**
  * **3 Structured Categories:** `[1. Workspaces]`, `[2. Tabs]`, and `[3. Panes]`.
  * **Interactive Inline Renaming:** Rename tabs and workspaces with live text input directly within the modal.
  * **Dynamic Resource IDs:** Automatically targets active workspace, tab, and pane IDs for creation, closing, and splitting actions.
  * **Full Keyboard Navigation:** 
    * `1`, `2`, `3` or `◄` / `►` (and `Tab` / `Shift+Tab`) to switch categories.
    * `▲` / `▼` (and `j` / `k`) to navigate actions.
    * `Enter` to execute tasks or confirm rename input.
    * `Esc` / `q` to dismiss.
  * **Mouse Pointer Support:** Click category tabs to switch views, or click any action row to execute it immediately.
  * **Tuxedo Cat ASCII Mascot:** Styled header banner with visual guidance.
* **Streamlined Tab Bar HUD (`line`):** High-contrast ANSI one-liner prefixed with the Tuxedo Cat paw mascot (`🐾 <Ctrl+B> Prefix │ ...`) for embedding into Herdr's top `tab_bar_right`.
* **Zero Boilerplate:** Automatically discovers active Herdr configuration (`prefix`, keybindings) from `$HERDR_CONFIG_PATH` or `~/.config/herdr/config.toml`.

---

## 🚀 Installation & Build

### 1. Build from Source
```bash
cargo build --release
```

The optimized binary is located at `target/release/herdr-status-bar`.

### 2. Install as a Herdr Plugin
```bash
herdr plugin install .
```

---

## 📖 Usage & Configuration

### 1. Top Tab Bar HUD Setup
In your `~/.config/herdr/config.local.toml` (or `config.toml`):

```toml
[ui]
tab_bar_position = "top"
tab_bar_right = [
  { type = "command", command = "/home/erichjsonfosse/projects/herdr-status-bar/main/target/release/herdr-status-bar line" }
]
```

### 2. Floating Command Palette Modal (`Ctrl+B Space`)
In your `~/.config/herdr/config.local.toml` (or `config.toml`), add under `[[keys.command]]`:

```toml
[[keys.command]]
key = "prefix+space"
type = "popup"
command = "/home/erichjsonfosse/projects/herdr-status-bar/main/target/release/herdr-status-bar menu"
width = "75%"
height = 22
description = "Herdr Command Palette & Shortcuts"
```

Then reload Herdr:
```bash
herdr server reload-config
```
*(or press `Ctrl+B Shift+R`)*

### 3. CLI Subcommands
* **Open Interactive Command Palette:**
  ```bash
  herdr-status-bar menu
  ```
* **Print ANSI Status Line:**
  ```bash
  herdr-status-bar line
  ```
* **IPC Socket Diagnostics:**
  ```bash
  herdr-status-bar check
  ```

---

## 📄 License

MIT License © 2026 Erich Json Fosse
