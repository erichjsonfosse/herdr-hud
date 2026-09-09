use super::*;

#[test]
fn test_format_key_token() {
    assert_eq!(format_key_token("ctrl"), "Ctrl");
    assert_eq!(format_key_token("minus"), "-");
    assert_eq!(format_key_token("escape"), "Esc");
}

#[test]
fn test_format_key_chord() {
    assert_eq!(format_key_chord("ctrl+b"), "Ctrl+B");
    assert_eq!(format_key_chord("prefix+minus"), "Prefix+-");
}

#[test]
fn test_format_action_key_in_navigate() {
    assert_eq!(format_action_key_in_navigate("prefix+c"), "c");
    assert_eq!(format_action_key_in_navigate("prefix+minus"), "-");
}
