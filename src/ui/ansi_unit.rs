use super::*;
use crate::config::KeyHint;

#[test]
fn test_render_ansi_line_basic() {
    let config = StatusBarConfig::default();
    let line = render_ansi_line(&config);

    assert!(line.starts_with("🐾 "));
    assert!(line.contains("<Ctrl+B>"));
    assert!(line.contains("Prefix"));
    assert!(line.contains("<c>"));
    assert!(line.contains("Tab"));
    assert!(line.contains("<v>"));
    assert!(line.contains("Split"));
    assert!(line.contains("\x1b[90m │ \x1b[0m"));
}

#[test]
fn test_render_ansi_line_custom_hints() {
    let config = StatusBarConfig {
        prefix_key: "Alt+A".to_string(),
        hints: vec![
            KeyHint {
                key: "Alt+A".to_string(),
                description: "Prefix".to_string(),
            },
            KeyHint {
                key: "t".to_string(),
                description: "New".to_string(),
            },
        ],
    };

    let line = render_ansi_line(&config);
    assert_eq!(
        line,
        "🐾 \x1b[36m\x1b[1m<Alt+A>\x1b[0m Prefix\x1b[90m │ \x1b[0m\x1b[36m\x1b[1m<t>\x1b[0m New"
    );
}

#[test]
fn test_render_ansi_line_empty_hints() {
    let config = StatusBarConfig {
        prefix_key: "Ctrl+B".to_string(),
        hints: vec![],
    };

    let line = render_ansi_line(&config);
    assert_eq!(line, "🐾 ");
}

#[test]
fn test_render_ansi_line_single_hint_has_no_separator() {
    let config = StatusBarConfig {
        prefix_key: "Ctrl+B".to_string(),
        hints: vec![KeyHint {
            key: "q".to_string(),
            description: "Quit".to_string(),
        }],
    };

    let line = render_ansi_line(&config);
    assert_eq!(line, "🐾 \x1b[36m\x1b[1m<q>\x1b[0m Quit");
    assert!(!line.contains("│"));
}
