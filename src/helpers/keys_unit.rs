use super::*;

#[test]
fn test_format_key_token() {
    assert_eq!(format_key_token("ctrl"), "Ctrl");
    assert_eq!(format_key_token("minus"), "-");
    assert_eq!(format_key_token("escape"), "Esc");
}

#[test]
fn test_format_key_token_borrowed_cow() {
    match format_key_token("ctrl") {
        std::borrow::Cow::Borrowed(b) => assert_eq!(b, "Ctrl"),
        std::borrow::Cow::Owned(_) => panic!("expected borrowed Cow for static key token"),
    }
}

#[test]
fn test_format_key_token_arrows() {
    assert_eq!(format_key_token("up"), "▲");
    assert_eq!(format_key_token("down"), "▼");
    assert_eq!(format_key_token("left"), "◄");
    assert_eq!(format_key_token("right"), "►");
    assert_eq!(format_key_token("UP"), "▲");
    assert_eq!(format_key_token("Down"), "▼");
    assert_eq!(format_key_token("LEFT"), "◄");
    assert_eq!(format_key_token("Right"), "►");
}

#[test]
fn test_format_key_token_modifiers() {
    assert_eq!(format_key_token("shift"), "Shift");
    assert_eq!(format_key_token("alt"), "Alt");
    assert_eq!(format_key_token("cmd"), "Meta");
    assert_eq!(format_key_token("super"), "Meta");
    assert_eq!(format_key_token("meta"), "Meta");
    assert_eq!(format_key_token("SHIFT"), "Shift");
    assert_eq!(format_key_token("ALT"), "Alt");
    assert_eq!(format_key_token("CMD"), "Meta");
    assert_eq!(format_key_token("SUPER"), "Meta");
    assert_eq!(format_key_token("META"), "Meta");
}

#[test]
fn test_format_key_token_keys() {
    assert_eq!(format_key_token("space"), "Space");
    assert_eq!(format_key_token("enter"), "Enter");
    assert_eq!(format_key_token("return"), "Enter");
    assert_eq!(format_key_token("tab"), "Tab");
    assert_eq!(format_key_token("esc"), "Esc");
    assert_eq!(format_key_token("escape"), "Esc");
    assert_eq!(format_key_token("SPACE"), "Space");
    assert_eq!(format_key_token("ENTER"), "Enter");
    assert_eq!(format_key_token("RETURN"), "Enter");
    assert_eq!(format_key_token("TAB"), "Tab");
}

#[test]
fn test_format_key_token_symbols() {
    assert_eq!(format_key_token("slash"), "/");
    assert_eq!(format_key_token("backslash"), "\\");
    assert_eq!(format_key_token("backtick"), "`");
    assert_eq!(format_key_token("dot"), ".");
    assert_eq!(format_key_token("period"), ".");
    assert_eq!(format_key_token("comma"), ",");
    assert_eq!(format_key_token("plus"), "+");
    assert_eq!(format_key_token("minus"), "-");
}

#[test]
fn test_format_key_token_fallback_words_and_chars() {
    assert_eq!(format_key_token("a"), "a");
    assert_eq!(format_key_token("Z"), "z");
    assert_eq!(format_key_token("home"), "Home");
    assert_eq!(format_key_token("pageup"), "Pageup");
    assert_eq!(format_key_token(""), "");
}

#[test]
fn test_format_key_chord() {
    assert_eq!(format_key_chord("ctrl+b"), "Ctrl+B");
    assert_eq!(format_key_chord("prefix+minus"), "Prefix+-");
}

#[test]
fn test_format_key_chord_capitalization_and_words() {
    assert_eq!(format_key_chord("ctrl+shift+a"), "Ctrl+Shift+A");
    assert_eq!(format_key_chord("meta+z"), "Meta+Z");
    assert_eq!(format_key_chord("alt+space"), "Alt+Space");
    assert_eq!(format_key_chord("ctrl+enter"), "Ctrl+Enter");
    assert_eq!(format_key_chord("prefix+return"), "Prefix+Enter");
    assert_eq!(format_key_chord("shift+tab"), "Shift+Tab");
    assert_eq!(format_key_chord("super+up"), "Meta+▲");
    assert_eq!(format_key_chord("cmd+down"), "Meta+▼");
    assert_eq!(format_key_chord("meta+left"), "Meta+◄");
    assert_eq!(format_key_chord("alt+right"), "Alt+►");
    assert_eq!(format_key_chord("prefix+slash"), "Prefix+/");
    assert_eq!(format_key_chord("ctrl+backslash"), "Ctrl+\\");
    assert_eq!(format_key_chord("alt+backtick"), "Alt+`");
    assert_eq!(format_key_chord("ctrl+dot"), "Ctrl+.");
    assert_eq!(format_key_chord("ctrl+period"), "Ctrl+.");
    assert_eq!(format_key_chord("ctrl+comma"), "Ctrl+,");
    assert_eq!(format_key_chord("ctrl+plus"), "Ctrl++");
    assert_eq!(format_key_chord("customword"), "Customword");
    assert_eq!(format_key_chord(""), "");
    assert_eq!(format_key_chord("   "), "   ");
    assert_eq!(format_key_chord("ctrl + b"), "Ctrl+B");
}

#[test]
fn test_format_action_key_in_navigate() {
    assert_eq!(format_action_key_in_navigate("prefix+c"), "c");
    assert_eq!(format_action_key_in_navigate("prefix+minus"), "-");
    assert_eq!(format_action_key_in_navigate("prefix+up"), "▲");
    assert_eq!(format_action_key_in_navigate("ctrl+b"), "Ctrl+B");
}
