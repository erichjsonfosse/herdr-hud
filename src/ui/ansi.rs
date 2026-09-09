use crate::config::StatusBarConfig;
use crate::state::{AgentStatus, HerdrMode, StatusBarState};
use chrono::Local;

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
#[path = "ansi_unit.rs"]
mod tests;
