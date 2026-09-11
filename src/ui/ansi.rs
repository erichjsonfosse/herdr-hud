use crate::config::StatusBarConfig;

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

#[cfg(test)]
#[path = "ansi_unit.rs"]
mod tests;
