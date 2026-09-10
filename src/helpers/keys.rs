pub fn format_key_token(token: &str) -> String {
    match token.to_lowercase().as_str() {
        "ctrl" => "Ctrl".to_string(),
        "alt" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "cmd" | "super" | "meta" => "Meta".to_string(),
        "esc" | "escape" => "Esc".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "tab" => "Tab".to_string(),
        "space" => "Space".to_string(),
        "minus" => "-".to_string(),
        "plus" => "+".to_string(),
        "comma" => ",".to_string(),
        "period" | "dot" => ".".to_string(),
        "slash" => "/".to_string(),
        "backslash" => "\\".to_string(),
        "backtick" => "`".to_string(),
        "up" => "▲".to_string(),
        "down" => "▼".to_string(),
        "left" => "◄".to_string(),
        "right" => "►".to_string(),
        other => {
            if other.len() == 1 {
                other.to_lowercase()
            } else {
                let mut c = other.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
        }
    }
}

pub fn format_key_chord(chord: &str) -> String {
    let parts: Vec<&str> = chord
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return chord.to_string();
    }
    let formatted_parts: Vec<String> = parts
        .into_iter()
        .map(|p| {
            let token = format_key_token(p);
            if token.len() == 1 && token.chars().next().unwrap().is_ascii_alphabetic() {
                token.to_uppercase()
            } else {
                token
            }
        })
        .collect();
    formatted_parts.join("+")
}

pub fn format_action_key_in_navigate(raw_action: &str) -> String {
    if raw_action.starts_with("prefix+") {
        let suffix = &raw_action["prefix+".len()..];
        format_key_token(suffix)
    } else {
        format_key_chord(raw_action)
    }
}

#[cfg(test)]
#[path = "keys_unit.rs"]
mod tests;
