use super::terminal::TerminalGuard;
use crate::client::HerdrClient;
use crate::config::StatusBarConfig;
use crate::palette::{
    ActionKind, ModalInputTarget, PaletteCategory, build_prompt_command, get_palette_categories,
    trigger_action,
};
use crate::state::StatusBarState;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use std::io::stdout;
use std::process::Command as SysCommand;
use std::time::Duration;

pub(crate) fn format_action_description(desc: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let char_count = desc.chars().count();
    if char_count > max_width {
        if max_width > 3 {
            let mut s: String = desc.chars().take(max_width - 3).collect();
            s.push_str("...");
            format!("{:<width$} ", s, width = max_width)
        } else {
            let s: String = desc.chars().take(max_width).collect();
            format!("{:<width$} ", s, width = max_width)
        }
    } else {
        format!("{:<width$} ", desc, width = max_width)
    }
}

pub(crate) fn hit_test_category_tab_at(
    column: u16,
    row: u16,
    categories: &[PaletteCategory],
    origin_x: u16,
    origin_y: u16,
) -> Option<usize> {
    let cat_tabs_y = origin_y + 5;
    if row >= cat_tabs_y && row < cat_tabs_y + 3 {
        let mut current_col = origin_x;
        for (i, cat) in categories.iter().enumerate() {
            let tab_len = (cat.name.chars().count() + 9) as u16;
            if column >= current_col && column < current_col + tab_len {
                return Some(i);
            }
            current_col += tab_len + 2;
        }
    }
    None
}

pub(crate) fn hit_test_category_tab(
    column: u16,
    row: u16,
    categories: &[PaletteCategory],
) -> Option<usize> {
    hit_test_category_tab_at(column, row, categories, 1, 1)
}

pub(crate) fn hit_test_action_item_at(
    row: u16,
    total_actions: usize,
    origin_y: u16,
) -> Option<usize> {
    let cat_tabs_y = origin_y + 5;
    let action_list_y_start = cat_tabs_y + 3;
    if row >= action_list_y_start {
        let clicked_idx = (row - action_list_y_start) as usize;
        if clicked_idx < total_actions {
            return Some(clicked_idx);
        }
    }
    None
}

pub(crate) fn hit_test_action_item(row: u16, total_actions: usize) -> Option<usize> {
    hit_test_action_item_at(row, total_actions, 1)
}

pub struct MenuModalWidget<'a> {
    pub state: &'a StatusBarState,
    pub categories: &'a [PaletteCategory],
    pub active_category_idx: usize,
    pub active_action_idx: usize,
    pub prompt_data: Option<(&'a str, &'a str, &'a str)>,
}

impl<'a> Widget for MenuModalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ws_title = if let Some(ws) = &self.state.active_workspace {
            format!(" 🐾 Herdr Command Palette & Reference [{}] ", ws)
        } else {
            " 🐾 Herdr Command Palette & Reference ".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                ws_title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 10 || inner.width < 40 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(5), // Tuxedo Cat ASCII Mascot
                Constraint::Length(3), // Category Navigation Tabs
                Constraint::Min(6),    // Action List OR Prompt
                Constraint::Length(2), // Footer Keybinding Guidance
            ])
            .split(inner);

        // 1. Compact Tuxedo Cat Mascot Header
        let cat_lines = vec![
            Line::from(vec![
                Span::styled("   /\\___/\\   ", Style::default().fg(Color::White)),
                Span::styled(
                    "  🐾 Herdr Action Palette",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  ( ", Style::default().fg(Color::White)),
                Span::styled(
                    "o",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" . ", Style::default().fg(Color::White)),
                Span::styled(
                    "o",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" )  ", Style::default().fg(Color::White)),
                Span::styled(
                    "  Select an action with keyboard or mouse to execute",
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::styled("  /| ", Style::default().fg(Color::White)),
                Span::styled("░█░", Style::default().fg(Color::Cyan)),
                Span::styled(" |\\  ", Style::default().fg(Color::White)),
                Span::styled(
                    "  Navigate categories with [1], [2], [3] or [◄ / ►]",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(Span::styled(
                "  (___m_m)  ",
                Style::default().fg(Color::White),
            )),
        ];
        Paragraph::new(cat_lines).render(chunks[0], buf);

        // 2. Category Navigation Tabs (Workspaces, Tabs, Panes)
        let mut tab_spans = Vec::new();
        for (i, cat) in self.categories.iter().enumerate() {
            let is_active = i == self.active_category_idx;
            let tab_style = if is_active {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            };

            let label = format!("  [{}. {}]  ", cat.key_number, cat.name);
            tab_spans.push(Span::styled(label, tab_style));
            tab_spans.push(Span::raw("  "));
        }

        let cat_bar = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));
        let cat_inner = cat_bar.inner(chunks[1]);
        cat_bar.render(chunks[1], buf);
        Paragraph::new(Line::from(tab_spans)).render(cat_inner, buf);

        // 3. Action List OR Text Input Prompt
        if let Some((target_type, current_name, input_buffer)) = self.prompt_data {
            let prompt_lines = if current_name.is_empty() {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            "  ✨  ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            target_type,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            " (Optional name, press Enter for default)",
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            "    Tab Name: [ ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            input_buffer,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            " ]",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(""),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            "  ✏️  Rename ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            target_type,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(" (Current: \"{}\")", current_name),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            "    New Name: [ ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            input_buffer,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            " ]",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(""),
                ]
            };
            Paragraph::new(prompt_lines).render(chunks[2], buf);
        } else {
            let active_cat = &self.categories[self.active_category_idx];
            let mut action_lines = Vec::new();

            for (idx, action) in active_cat.actions.iter().enumerate() {
                let is_selected = idx == self.active_action_idx;

                let row_style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(30, 45, 60))
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let pointer = if is_selected { " ▶ " } else { "   " };
                let exec_badge = match action.kind {
                    ActionKind::DirectCommand
                    | ActionKind::SplitVertical
                    | ActionKind::SplitHorizontal
                    | ActionKind::ToggleZoom => Span::styled(
                        " [Execute ↵] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    ActionKind::PromptCreateTab => Span::styled(
                        " [Create ↵]  ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    ActionKind::CloseActiveWorkspace
                    | ActionKind::CloseActiveTab
                    | ActionKind::CloseActivePane => Span::styled(
                        " [Close ↵]   ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    ActionKind::PromptRenameWorkspace | ActionKind::PromptRenameTab => {
                        Span::styled(
                            " [Rename ↵]  ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    }
                    ActionKind::ShortcutOnly => {
                        Span::styled(" [Shortcut]  ", Style::default().fg(Color::DarkGray))
                    }
                };

                let max_desc_width = (chunks[2].width.saturating_sub(55) as usize).min(40);
                let line = Line::from(vec![
                    Span::styled(
                        pointer,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<16} ", format!("<{}>", action.key)),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("{:<20} ", action.name), row_style),
                    Span::styled(
                        format_action_description(action.description, max_desc_width),
                        Style::default().fg(Color::Gray),
                    ),
                    exec_badge,
                ]);

                action_lines.push(line);
            }

            Paragraph::new(action_lines).render(chunks[2], buf);
        }

        // 4. Footer Guidance
        let footer_spans = if self.prompt_data.is_some() {
            vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Confirm  │", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    " [Esc] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Cancel", Style::default().fg(Color::DarkGray)),
            ]
        } else {
            vec![
                Span::styled(
                    " [▲/▼ or j/k] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Select Action  │", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    " [1/2/3 or ◄/►] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Category  │", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    " [Enter / Click] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Execute  │", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    " [Esc / q] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Close", Style::default().fg(Color::DarkGray)),
            ]
        };

        let footer_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));
        let footer_inner = footer_block.inner(chunks[3]);
        footer_block.render(chunks[3], buf);
        Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .render(footer_inner, buf);
    }
}

pub fn run_modal_menu(
    client: HerdrClient,
    config: StatusBarConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pending_command: Option<Vec<String>> = None;

    {
        let _guard = TerminalGuard::new(true)?;
        let stdout = stdout();
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

        let mut dirty = true;
        loop {
            if dirty {
                let prompt_data = input_target.map(|target| {
                    let (target_label, current_label) = match target {
                        ModalInputTarget::CreateTab => ("Create New Tab", ""),
                        ModalInputTarget::RenameWorkspace => {
                            ("Workspace", state.active_workspace.as_deref().unwrap_or(""))
                        }
                        ModalInputTarget::RenameTab => {
                            ("Tab", state.active_tab.as_deref().unwrap_or(""))
                        }
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
                dirty = false;
            }

            match event::read()? {
                Event::Key(key) => {
                    // Ignore key release events
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }
                    dirty = true;

                    if let Some(target) = input_target {
                        // In interactive text prompt mode
                        match key.code {
                            KeyCode::Esc => {
                                input_target = None;
                                input_buffer.clear();
                            }
                            KeyCode::Enter => {
                                if let Some(cmd) =
                                    build_prompt_command(&target, &input_buffer, &state)
                                {
                                    pending_command = Some(cmd);
                                    break;
                                } else {
                                    input_target = None;
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
                                    active_action_idx =
                                        categories[active_cat_idx].actions.len().saturating_sub(1);
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
                Event::Mouse(MouseEvent {
                    kind, column, row, ..
                }) if input_target.is_none() => {
                    dirty = true;
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
                                active_action_idx =
                                    categories[active_cat_idx].actions.len().saturating_sub(1);
                            }
                        }
                        MouseEventKind::Down(MouseButton::Left) => {
                            if let Some(cat_idx) = hit_test_category_tab(column, row, &categories) {
                                active_cat_idx = cat_idx;
                                active_action_idx = 0;
                            } else {
                                let total_actions = categories[active_cat_idx].actions.len();
                                if let Some(clicked_idx) = hit_test_action_item(row, total_actions)
                                {
                                    active_action_idx = clicked_idx;
                                    let action =
                                        &categories[active_cat_idx].actions[active_action_idx];
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
                Event::Resize(_, _) => {
                    dirty = true;
                }
                _ => {}
            }
        }
    }

    // Execute selected command if triggered and wait for RPC completion
    if let Some(cmd) = pending_command
        && !cmd.is_empty()
    {
        let mut proc = SysCommand::new(&cmd[0]);
        if cmd.len() > 1 {
            proc.args(&cmd[1..]);
        }
        let _ = proc.status();
    }

    Ok(())
}

#[cfg(test)]
#[path = "modal_unit.rs"]
mod tests;
