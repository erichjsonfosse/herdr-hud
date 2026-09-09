use crate::config::StatusBarConfig;
use crate::palette::{ActionKind, PaletteCategory};
use crate::state::{AgentStatus, HerdrMode, StatusBarState};
use chrono::Local;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};



pub struct StatusBarWidget<'a> {
    pub state: &'a StatusBarState,
    pub config: &'a StatusBarConfig,
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mut spans = Vec::new();

        // 1. Mascot Prefix Icon
        spans.push(Span::raw("🐾 "));

        // 2. Contextual Shortcuts
        let hints = match self.state.mode {
            HerdrMode::Normal => &self.config.normal_hints,
            HerdrMode::Navigate => &self.config.navigate_hints,
            HerdrMode::Scroll => &self.config.scroll_hints,
            HerdrMode::Agent => &self.config.normal_hints,
        };

        for (i, hint) in hints.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
            }

            spans.push(Span::styled(
                format!("<{}> ", hint.key),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                &hint.description,
                Style::default().fg(Color::White),
            ));
        }

        // 3. Right-Aligned Info (Agents & Clock)
        let mut right_spans = Vec::new();

        if self.config.show_agents && !self.state.agents.is_empty() {
            for (_, agent) in &self.state.agents {
                let (sym_color, sym) = match agent.status {
                    AgentStatus::Working => (Color::LightGreen, "●"),
                    AgentStatus::Blocked => (Color::LightRed, "▲"),
                    AgentStatus::Done => (Color::LightCyan, "✔"),
                    AgentStatus::Idle => (Color::DarkGray, "○"),
                    AgentStatus::Unknown => (Color::Gray, "?"),
                };

                right_spans.push(Span::styled(
                    format!(" {} ", sym),
                    Style::default().fg(sym_color),
                ));
                right_spans.push(Span::styled(
                    format!("{}: {} ", agent.agent_name, agent.status.text()),
                    Style::default().fg(Color::Gray),
                ));
            }
        }

        if self.config.show_clock {
            let time_str = Local::now().format("%H:%M").to_string();
            right_spans.push(Span::styled(
                format!("[ {} ]", time_str),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let left_line = Line::from(spans);
        let left_len = left_line.width();

        let right_line = Line::from(right_spans);
        let right_len = right_line.width();

        if (left_len + right_len + 1) <= area.width as usize {
            let padding = area.width as usize - (left_len + right_len);
            let mut combined = left_line.spans;
            combined.push(Span::raw(" ".repeat(padding)));
            combined.extend(right_line.spans);
            Paragraph::new(Line::from(combined)).render(area, buf);
        } else {
            Paragraph::new(left_line).render(area, buf);
        }
    }
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
                Span::styled("  🐾 Herdr Action Palette", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ( ", Style::default().fg(Color::White)),
                Span::styled("o", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" . ", Style::default().fg(Color::White)),
                Span::styled("o", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" )  ", Style::default().fg(Color::White)),
                Span::styled("  Select an action with keyboard or mouse to execute", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("  /| ", Style::default().fg(Color::White)),
                Span::styled("░█░", Style::default().fg(Color::Cyan)),
                Span::styled(" |\\  ", Style::default().fg(Color::White)),
                Span::styled("  Navigate categories with [1], [2], [3] or [◄ / ►]", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(Span::styled("  (___m_m)  ", Style::default().fg(Color::White))),
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
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
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
                        Span::styled("  ✨  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled(target_type, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" (Optional name, press Enter for default)", Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("    Tab Name: [ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(input_buffer, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                        Span::styled(" ]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(""),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  ✏️  Rename ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(target_type, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" (Current: \"{}\")", current_name), Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("    New Name: [ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(input_buffer, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                        Span::styled(" ]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
                    ActionKind::DirectCommand | ActionKind::SplitVertical | ActionKind::SplitHorizontal | ActionKind::ToggleZoom => {
                        Span::styled(" [Execute ↵] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    }
                    ActionKind::PromptCreateTab => {
                        Span::styled(" [Create ↵]  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    }
                    ActionKind::CloseActiveWorkspace | ActionKind::CloseActiveTab | ActionKind::CloseActivePane => {
                        Span::styled(" [Close ↵]   ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    }
                    ActionKind::PromptRenameWorkspace | ActionKind::PromptRenameTab => {
                        Span::styled(" [Rename ↵]  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    }
                    ActionKind::ShortcutOnly => Span::styled(" [Shortcut]  ", Style::default().fg(Color::DarkGray)),
                };

                let line = Line::from(vec![
                    Span::styled(pointer, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<16} ", format!("<{}>", action.key)), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<20} ", action.name), row_style),
                    Span::styled(format!("{:<38} ", action.description), Style::default().fg(Color::Gray)),
                    exec_badge,
                ]);

                action_lines.push(line);
            }

            Paragraph::new(action_lines).render(chunks[2], buf);
        }

        // 4. Footer Guidance
        let footer_spans = if self.prompt_data.is_some() {
            vec![
                Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Confirm  │", Style::default().fg(Color::DarkGray)),
                Span::styled(" [Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Cancel", Style::default().fg(Color::DarkGray)),
            ]
        } else {
            vec![
                Span::styled(" [▲/▼ or j/k] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("Select Action  │", Style::default().fg(Color::DarkGray)),
                Span::styled(" [1/2/3 or ◄/►] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("Category  │", Style::default().fg(Color::DarkGray)),
                Span::styled(" [Enter / Click] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Execute  │", Style::default().fg(Color::DarkGray)),
                Span::styled(" [Esc / q] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
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

pub fn render_ansi_line(state: &StatusBarState, config: &StatusBarConfig) -> String {
    let mode_color = match state.mode {
        HerdrMode::Normal => "\x1b[36m",
        HerdrMode::Navigate => "\x1b[33m",
        HerdrMode::Scroll => "\x1b[35m",
        HerdrMode::Agent => "\x1b[32m",
    };

    let hints = match state.mode {
        HerdrMode::Normal => &config.normal_hints,
        HerdrMode::Navigate => &config.navigate_hints,
        HerdrMode::Scroll => &config.scroll_hints,
        HerdrMode::Agent => &config.normal_hints,
    };

    let mut hint_parts = Vec::new();
    for hint in hints {
        hint_parts.push(format!(
            "{}\x1b[1m<{}>\x1b[0m {}",
            mode_color, hint.key, hint.description
        ));
    }

    let hints_str = hint_parts.join("\x1b[90m │ \x1b[0m");

    let mut agent_str = String::new();
    if config.show_agents && !state.agents.is_empty() {
        for (_, agent) in &state.agents {
            let sym = match agent.status {
                AgentStatus::Working => "\x1b[32m●\x1b[0m",
                AgentStatus::Blocked => "\x1b[31m▲\x1b[0m",
                AgentStatus::Done => "\x1b[36m✔\x1b[0m",
                AgentStatus::Idle => "\x1b[90m○\x1b[0m",
                AgentStatus::Unknown => "\x1b[90m?\x1b[0m",
            };
            agent_str.push_str(&format!(" {} {}: {}", sym, agent.agent_name, agent.status.text()));
        }
    }

    let time_str = if config.show_clock {
        format!(" \x1b[90m[ {} ]\x1b[0m", Local::now().format("%H:%M"))
    } else {
        String::new()
    };

    format!("🐾 {}{}{}", hints_str, agent_str, time_str)
}

#[cfg(test)]
#[path = "ui_unit.rs"]
mod tests;

