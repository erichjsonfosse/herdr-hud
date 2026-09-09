use super::*;
use crate::state::AgentEntry;

#[test]
fn test_render_ansi_line_basic() {
    let state = StatusBarState::new();
    let config = StatusBarConfig::default();
    let line = render_ansi_line(&state, &config);

    assert!(line.starts_with("🐾 "));
    assert!(line.contains("<Ctrl+B>"));
    assert!(line.contains("Prefix"));
    assert!(line.contains("<c>"));
    assert!(line.contains("Tab"));
    assert!(line.contains("<v>"));
    assert!(line.contains("Split"));
}

#[test]
fn test_render_ansi_line_with_agents() {
    let mut state = StatusBarState::new();
    let mut config = StatusBarConfig::default();
    config.show_agents = true;

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "agy".to_string(),
            status: AgentStatus::Working,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(line.contains("●"));
    assert!(line.contains("agy: working"));
}

