use crate::config::StatusBarConfig;
use crate::state::StatusBarState;

pub fn render_ansi_line(_state: &StatusBarState, config: &StatusBarConfig) -> String {
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

#[cfg(test)]
#[path = "ansi_unit.rs"]
mod tests;
