use super::*;
use crate::config::KeyHint;
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
    let config = StatusBarConfig {
        show_agents: true,
        ..Default::default()
    };

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "agy".to_string(),
            status: AgentStatus::Working,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[32m●\x1b[0m"));
    assert!(line.contains("●"));
    assert!(line.contains("agy: working"));
}

#[test]
fn test_render_ansi_line_with_clock() {
    let state = StatusBarState::new();
    let config = StatusBarConfig {
        show_clock: true,
        ..Default::default()
    };

    let before = chrono::Local::now();
    let line = render_ansi_line(&state, &config);
    let after = chrono::Local::now();

    let before_str = format!("[ {} ]", before.format("%H:%M"));
    let after_str = format!("[ {} ]", after.format("%H:%M"));
    assert!(
        line.contains(&before_str) || line.contains(&after_str),
        "Expected line to contain clock '{}' or '{}', got: {}",
        before_str,
        after_str,
        line
    );
    assert!(line.contains("\x1b[90m[ "));

    let clock_start = line.find("[ ").expect("Clock open bracket not found");
    let clock_slice = &line[clock_start..clock_start + 9];
    assert!(clock_slice.ends_with(" ]"));
    let time_parts: Vec<&str> = clock_slice[2..7].split(':').collect();
    assert_eq!(time_parts.len(), 2);
    let hour: u32 = time_parts[0].parse().expect("Valid hour digits");
    let min: u32 = time_parts[1].parse().expect("Valid minute digits");
    assert!(hour < 24);
    assert!(min < 60);
}

#[test]
fn test_render_ansi_line_clock_disabled() {
    let state = StatusBarState::new();
    let config = StatusBarConfig {
        show_clock: false,
        ..Default::default()
    };

    let line = render_ansi_line(&state, &config);
    assert!(!line.contains("[ "));
}

#[test]
fn test_render_ansi_line_agent_status_blocked() {
    let mut state = StatusBarState::new();
    let config = StatusBarConfig {
        show_agents: true,
        ..Default::default()
    };

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "coder".to_string(),
            status: AgentStatus::Blocked,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[31m▲\x1b[0m"));
    assert!(line.contains("coder: blocked"));
}

#[test]
fn test_render_ansi_line_agent_status_done() {
    let mut state = StatusBarState::new();
    let config = StatusBarConfig {
        show_agents: true,
        ..Default::default()
    };

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "reviewer".to_string(),
            status: AgentStatus::Done,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[36m✔\x1b[0m"));
    assert!(line.contains("reviewer: done"));
}

#[test]
fn test_render_ansi_line_agent_status_idle() {
    let mut state = StatusBarState::new();
    let config = StatusBarConfig {
        show_agents: true,
        ..Default::default()
    };

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "watcher".to_string(),
            status: AgentStatus::Idle,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[90m○\x1b[0m"));
    assert!(line.contains("watcher: idle"));
}

#[test]
fn test_render_ansi_line_agent_status_unknown() {
    let mut state = StatusBarState::new();
    let config = StatusBarConfig {
        show_agents: true,
        ..Default::default()
    };

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "mystery".to_string(),
            status: AgentStatus::Unknown,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[90m?\x1b[0m"));
    assert!(line.contains("mystery: unknown"));
}

#[test]
fn test_render_ansi_line_show_agents_disabled() {
    let mut state = StatusBarState::new();
    let config = StatusBarConfig {
        show_agents: false,
        ..Default::default()
    };

    state.agents.insert(
        "p1".to_string(),
        AgentEntry {
            pane_id: "p1".to_string(),
            agent_name: "coder".to_string(),
            status: AgentStatus::Blocked,
        },
    );

    let line = render_ansi_line(&state, &config);
    assert!(!line.contains("coder: blocked"));
    assert!(!line.contains("▲"));
}

#[test]
fn test_render_ansi_line_modes() {
    let config = StatusBarConfig {
        normal_hints: vec![KeyHint {
            key: "Ctrl+B".to_string(),
            description: "Prefix".to_string(),
        }],
        navigate_hints: vec![KeyHint {
            key: "hjkl".to_string(),
            description: "Move".to_string(),
        }],
        scroll_hints: vec![KeyHint {
            key: "j/k".to_string(),
            description: "Line".to_string(),
        }],
        ..Default::default()
    };

    let mut state = StatusBarState::new();
    state.mode = HerdrMode::Normal;
    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[36m"));
    assert!(line.contains("<Ctrl+B>"));

    state.mode = HerdrMode::Navigate;
    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[33m"));
    assert!(line.contains("<hjkl>"));

    state.mode = HerdrMode::Scroll;
    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[35m"));
    assert!(line.contains("<j/k>"));

    state.mode = HerdrMode::Agent;
    let line = render_ansi_line(&state, &config);
    assert!(line.contains("\x1b[32m"));
    assert!(line.contains("<Ctrl+B>"));
}

#[test]
fn test_render_ansi_line_deterministic_agent_ordering() {
    let mut state = StatusBarState::new();
    let config = StatusBarConfig {
        show_agents: true,
        ..Default::default()
    };

    // Insert agents out of order
    state.agents.insert(
        "pane-c".to_string(),
        AgentEntry {
            pane_id: "pane-c".to_string(),
            agent_name: "charlie".to_string(),
            status: AgentStatus::Done,
        },
    );
    state.agents.insert(
        "pane-a".to_string(),
        AgentEntry {
            pane_id: "pane-a".to_string(),
            agent_name: "alice".to_string(),
            status: AgentStatus::Working,
        },
    );
    state.agents.insert(
        "pane-b".to_string(),
        AgentEntry {
            pane_id: "pane-b".to_string(),
            agent_name: "bob".to_string(),
            status: AgentStatus::Blocked,
        },
    );

    let line = render_ansi_line(&state, &config);
    let pos_a = line.find("alice").expect("alice should appear");
    let pos_b = line.find("bob").expect("bob should appear");
    let pos_c = line.find("charlie").expect("charlie should appear");

    assert!(pos_a < pos_b);
    assert!(pos_b < pos_c);
}
