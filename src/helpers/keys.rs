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

pub fn format_key_chord(chord: &str) -> String {
    let mut parts = chord
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let first = match parts.next() {
        Some(first) => first,
        None => return chord.to_string(),
    };

    let mut output = String::with_capacity(chord.len());

    let append_token = |buf: &mut String, p: &str| {
        let token = format_key_token(p);
        if token.len() == 1 && token.chars().next().unwrap().is_ascii_alphabetic() {
            buf.push(token.chars().next().unwrap().to_ascii_uppercase());
        } else {
            buf.push_str(&token);
        }
    };

    append_token(&mut output, first);

    for part in parts {
        output.push('+');
        append_token(&mut output, part);
    }

    output
}

pub fn format_action_key_in_navigate(raw_action: &str) -> String {
    if let Some(suffix) = raw_action.strip_prefix("prefix+") {
        format_key_token(suffix).into_owned()
    } else {
        format_key_chord(raw_action)
    }
}

#[cfg(test)]
#[path = "keys_unit.rs"]
mod tests;
