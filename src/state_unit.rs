use super::*;
use std::collections::HashMap;

#[test]
fn test_herdr_mode_badge_name() {
    assert_eq!(HerdrMode::Normal.badge_name(), "NORMAL");
    assert_eq!(HerdrMode::Navigate.badge_name(), "NAVIGATE");
    assert_eq!(HerdrMode::Scroll.badge_name(), "SCROLL");
    assert_eq!(HerdrMode::Agent.badge_name(), "AGENT");
}

#[test]
fn test_herdr_mode_default() {
    assert_eq!(HerdrMode::default(), HerdrMode::Normal);
}

#[test]
fn test_agent_status_symbol() {
    assert_eq!(AgentStatus::Idle.symbol(), "○");
    assert_eq!(AgentStatus::Working.symbol(), "●");
    assert_eq!(AgentStatus::Blocked.symbol(), "▲");
    assert_eq!(AgentStatus::Done.symbol(), "✔");
    assert_eq!(AgentStatus::Unknown.symbol(), "?");
}

#[test]
fn test_agent_status_text() {
    assert_eq!(AgentStatus::Idle.text(), "idle");
    assert_eq!(AgentStatus::Working.text(), "working");
    assert_eq!(AgentStatus::Blocked.text(), "blocked");
    assert_eq!(AgentStatus::Done.text(), "done");
    assert_eq!(AgentStatus::Unknown.text(), "unknown");
}

#[test]
fn test_herdr_mode_serde_roundtrip() {
    let variants = [
        (HerdrMode::Normal, "\"normal\""),
        (HerdrMode::Navigate, "\"navigate\""),
        (HerdrMode::Scroll, "\"scroll\""),
        (HerdrMode::Agent, "\"agent\""),
    ];

    for (variant, expected_json) in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        assert_eq!(serialized, expected_json);

        let deserialized: HerdrMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, variant);
    }

    assert!(serde_json::from_str::<HerdrMode>("\"invalid_mode\"").is_err());
}

#[test]
fn test_agent_status_serde_roundtrip() {
    let variants = [
        (AgentStatus::Idle, "\"idle\""),
        (AgentStatus::Working, "\"working\""),
        (AgentStatus::Blocked, "\"blocked\""),
        (AgentStatus::Done, "\"done\""),
        (AgentStatus::Unknown, "\"unknown\""),
    ];

    for (variant, expected_json) in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        assert_eq!(serialized, expected_json);

        let deserialized: AgentStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, variant);
    }

    assert!(serde_json::from_str::<AgentStatus>("\"invalid_status\"").is_err());
}

#[test]
fn test_agent_entry_serde_roundtrip() {
    let entry = AgentEntry {
        pane_id: "pane-101".to_string(),
        agent_name: "code-assistant".to_string(),
        status: AgentStatus::Working,
    };

    let serialized = serde_json::to_string(&entry).unwrap();
    let deserialized: AgentEntry = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized, entry);
    assert_eq!(deserialized.pane_id, "pane-101");
    assert_eq!(deserialized.agent_name, "code-assistant");
    assert_eq!(deserialized.status, AgentStatus::Working);

    // Verify deserialization from raw JSON string
    let raw_json = r#"{"pane_id":"pane-202","agent_name":"reviewer","status":"done"}"#;
    let from_raw: AgentEntry = serde_json::from_str(raw_json).unwrap();
    assert_eq!(from_raw.pane_id, "pane-202");
    assert_eq!(from_raw.agent_name, "reviewer");
    assert_eq!(from_raw.status, AgentStatus::Done);
}

#[test]
fn test_status_bar_state_default() {
    let state = StatusBarState::default();

    assert_eq!(state.mode, HerdrMode::Normal);
    assert_eq!(state.active_workspace, None);
    assert_eq!(state.active_workspace_id, None);
    assert_eq!(state.active_workspace_label, None);
    assert_eq!(state.active_tab, None);
    assert_eq!(state.active_tab_id, None);
    assert_eq!(state.active_tab_label, None);
    assert_eq!(state.active_pane, None);
    assert!(state.agents.is_empty());
    assert_eq!(state.prefix_key, "");
}

#[test]
fn test_status_bar_state_new() {
    let state = StatusBarState::new();

    assert_eq!(state.mode, HerdrMode::Normal);
    assert_eq!(state.active_workspace, None);
    assert_eq!(state.active_workspace_id, None);
    assert_eq!(state.active_workspace_label, None);
    assert_eq!(state.active_tab, None);
    assert_eq!(state.active_tab_id, None);
    assert_eq!(state.active_tab_label, None);
    assert_eq!(state.active_pane, None);
    assert!(state.agents.is_empty());
    assert_eq!(state.prefix_key, "Ctrl+S");
}

#[test]
fn test_status_bar_state_serde_roundtrip_default() {
    let state = StatusBarState::default();
    let serialized = serde_json::to_string(&state).unwrap();
    let deserialized: StatusBarState = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized, state);
}

#[test]
fn test_status_bar_state_serde_roundtrip_populated() {
    let mut agents = HashMap::new();
    agents.insert(
        "pane-1".to_string(),
        AgentEntry {
            pane_id: "pane-1".to_string(),
            agent_name: "planner".to_string(),
            status: AgentStatus::Working,
        },
    );
    agents.insert(
        "pane-2".to_string(),
        AgentEntry {
            pane_id: "pane-2".to_string(),
            agent_name: "tester".to_string(),
            status: AgentStatus::Blocked,
        },
    );

    let state = StatusBarState {
        mode: HerdrMode::Navigate,
        active_workspace: Some("workspace-main".to_string()),
        active_workspace_id: Some("ws-uuid-1".to_string()),
        active_workspace_label: Some("Main Workspace".to_string()),
        active_tab: Some("tab-code".to_string()),
        active_tab_id: Some("tab-uuid-2".to_string()),
        active_tab_label: Some("Code Tab".to_string()),
        active_pane: Some("pane-1".to_string()),
        agents,
        prefix_key: "Ctrl+A".to_string(),
    };

    let serialized = serde_json::to_string(&state).unwrap();
    let deserialized: StatusBarState = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized, state);
    assert_eq!(deserialized.mode, HerdrMode::Navigate);
    assert_eq!(
        deserialized.active_workspace.as_deref(),
        Some("workspace-main")
    );
    assert_eq!(deserialized.active_workspace_id.as_deref(), Some("ws-uuid-1"));
    assert_eq!(
        deserialized.active_workspace_label.as_deref(),
        Some("Main Workspace")
    );
    assert_eq!(deserialized.active_tab.as_deref(), Some("tab-code"));
    assert_eq!(deserialized.active_tab_id.as_deref(), Some("tab-uuid-2"));
    assert_eq!(deserialized.active_tab_label.as_deref(), Some("Code Tab"));
    assert_eq!(deserialized.active_pane.as_deref(), Some("pane-1"));
    assert_eq!(deserialized.prefix_key, "Ctrl+A");
    assert_eq!(deserialized.agents.len(), 2);
    assert_eq!(deserialized.agents["pane-1"].agent_name, "planner");
    assert_eq!(deserialized.agents["pane-1"].status, AgentStatus::Working);
    assert_eq!(deserialized.agents["pane-2"].agent_name, "tester");
    assert_eq!(deserialized.agents["pane-2"].status, AgentStatus::Blocked);
}
