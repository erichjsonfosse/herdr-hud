use crate::client::HerdrClient;
use crate::config::StatusBarConfig;
use crate::state::{AgentStatus, HerdrMode, StatusBarState};
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use std::io::stdout;
use std::time::Duration;

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

pub async fn run_tui_loop(
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
