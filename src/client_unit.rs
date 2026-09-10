use super::*;
use serde_json::json;

#[test]
fn test_update_state_from_herdr_v09_snapshot() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "id": "cli:api:snapshot",
        "result": {
            "type": "session_snapshot",
            "snapshot": {
                "focused_workspace_id": "w2",
                "focused_tab_id": "w2:t2",
                "focused_pane_id": "w2:p2",
                "workspaces": [
                    {
                        "workspace_id": "w1",
                        "label": "general",
                        "focused": false
                    },
                    {
                        "workspace_id": "w2",
                        "label": "my-project",
                        "focused": true
                    }
                ],
                "tabs": [
                    {
                        "tab_id": "w2:t1",
                        "label": "editor",
                        "focused": false
                    },
                    {
                        "tab_id": "w2:t2",
                        "label": "terminal",
                        "focused": true
                    }
                ],
                "panes": [
                    {
                        "pane_id": "w2:p2",
                        "agent": "agy",
                        "agent_status": "working"
                    }
                ]
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_workspace_id.as_deref(), Some("w2"));
    assert_eq!(state.active_workspace_label.as_deref(), Some("my-project"));
    assert_eq!(state.active_tab_id.as_deref(), Some("w2:t2"));
    assert_eq!(state.active_tab_label.as_deref(), Some("terminal"));
    assert_eq!(state.active_pane.as_deref(), Some("w2:p2"));

    let agent = state.agents.get("w2:p2").expect("agent should exist");
    assert_eq!(agent.agent_name, "agy");
    assert_eq!(agent.status, AgentStatus::Working);
}

#[test]
fn test_update_state_from_legacy_data_snapshot() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "data": {
            "session_snapshot": {
                "focused_workspace_id": "w1",
                "focused_tab_id": "w1:t1",
                "focused_pane_id": "w1:p1",
                "workspaces": [
                    {
                        "workspace_id": "w1",
                        "label": "default",
                        "focused": true
                    }
                ],
                "tabs": [
                    {
                        "tab_id": "w1:t1",
                        "label": "main",
                        "focused": true
                    }
                ],
                "panes": [
                    {
                        "pane_id": "w1:p1",
                        "agent": "coder",
                        "agent_status": "blocked"
                    }
                ]
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_workspace_id.as_deref(), Some("w1"));
    assert_eq!(state.active_workspace_label.as_deref(), Some("default"));
    assert_eq!(state.active_tab_id.as_deref(), Some("w1:t1"));
    assert_eq!(state.active_tab_label.as_deref(), Some("main"));
    assert_eq!(state.active_pane.as_deref(), Some("w1:p1"));

    let agent = state.agents.get("w1:p1").expect("agent should exist");
    assert_eq!(agent.status, AgentStatus::Blocked);
}

#[test]
fn test_update_state_empty_fallback() {
    let mut state = StatusBarState::new();
    let snapshot = json!({});

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_workspace_id, None);
    assert_eq!(state.active_workspace_label, None);
    assert_eq!(state.active_tab_id, None);
    assert_eq!(state.active_tab_label, None);
    assert_eq!(state.active_pane, None);
    assert!(state.agents.is_empty());
}

#[test]
fn test_snapshot_workspace_fallback_when_workspaces_missing() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "result": {
            "snapshot": {
                "focused_workspace_id": "ws-orphan"
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_workspace_id.as_deref(), Some("ws-orphan"));
    assert_eq!(state.active_workspace_label.as_deref(), Some("ws-orphan"));
    assert_eq!(state.active_workspace.as_deref(), Some("ws-orphan"));
}

#[test]
fn test_snapshot_workspace_fallback_when_workspace_not_found_in_array() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "result": {
            "snapshot": {
                "focused_workspace_id": "ws-orphan",
                "workspaces": [
                    {
                        "workspace_id": "ws-other",
                        "label": "Other Workspace",
                        "focused": false
                    }
                ]
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_workspace_id.as_deref(), Some("ws-orphan"));
    assert_eq!(state.active_workspace_label.as_deref(), Some("ws-orphan"));
    assert_eq!(state.active_workspace.as_deref(), Some("ws-orphan"));
}

#[test]
fn test_snapshot_tab_fallback_when_tabs_missing() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "result": {
            "snapshot": {
                "focused_tab_id": "tab-orphan"
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_tab_id.as_deref(), Some("tab-orphan"));
    assert_eq!(state.active_tab_label.as_deref(), Some("tab-orphan"));
    assert_eq!(state.active_tab.as_deref(), Some("tab-orphan"));
}

#[test]
fn test_snapshot_tab_fallback_when_tab_not_found_in_array() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "result": {
            "snapshot": {
                "focused_tab_id": "tab-orphan",
                "tabs": [
                    {
                        "tab_id": "tab-other",
                        "label": "Other Tab",
                        "focused": false
                    }
                ]
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.active_tab_id.as_deref(), Some("tab-orphan"));
    assert_eq!(state.active_tab_label.as_deref(), Some("tab-orphan"));
    assert_eq!(state.active_tab.as_deref(), Some("tab-orphan"));
}

#[test]
fn test_snapshot_pane_identification_id_fallback() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "result": {
            "snapshot": {
                "panes": [
                    {
                        "id": "pane-with-id-only",
                        "agent": "worker-1",
                        "agent_status": "done"
                    },
                    {
                        "pane_id": "pane-with-pane-id",
                        "id": "ignored-id",
                        "agent": "worker-2",
                        "agent_status": "idle"
                    },
                    {
                        "agent": "worker-no-id",
                        "agent_status": "working"
                    },
                    {
                        "pane_id": "",
                        "agent": "worker-empty-pane-id"
                    }
                ]
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(state.agents.len(), 2);
    let agent1 = state
        .agents
        .get("pane-with-id-only")
        .expect("should find pane by id");
    assert_eq!(agent1.pane_id, "pane-with-id-only");
    assert_eq!(agent1.agent_name, "worker-1");
    assert_eq!(agent1.status, AgentStatus::Done);

    let agent2 = state
        .agents
        .get("pane-with-pane-id")
        .expect("should prefer pane_id");
    assert_eq!(agent2.pane_id, "pane-with-pane-id");
    assert_eq!(agent2.agent_name, "worker-2");
    assert_eq!(agent2.status, AgentStatus::Idle);
}

#[test]
fn test_snapshot_agent_statuses() {
    let mut state = StatusBarState::new();
    let snapshot = json!({
        "result": {
            "snapshot": {
                "panes": [
                    {
                        "pane_id": "p-working",
                        "agent": "agy",
                        "agent_status": "working"
                    },
                    {
                        "pane_id": "p-blocked",
                        "agent": "coder",
                        "agent_status": "blocked"
                    },
                    {
                        "pane_id": "p-done",
                        "agent": "reviewer",
                        "agent_status": "done"
                    },
                    {
                        "pane_id": "p-idle",
                        "agent": "watcher",
                        "agent_status": "idle"
                    },
                    {
                        "pane_id": "p-default-idle",
                        "agent": "standby"
                    },
                    {
                        "pane_id": "p-unknown",
                        "agent": "custom",
                        "agent_status": "unrecognized_state_str"
                    }
                ]
            }
        }
    });

    HerdrClient::update_state_from_snapshot(&mut state, &snapshot);

    assert_eq!(
        state.agents.get("p-working").unwrap().status,
        AgentStatus::Working
    );
    assert_eq!(
        state.agents.get("p-blocked").unwrap().status,
        AgentStatus::Blocked
    );
    assert_eq!(
        state.agents.get("p-done").unwrap().status,
        AgentStatus::Done
    );
    assert_eq!(
        state.agents.get("p-idle").unwrap().status,
        AgentStatus::Idle
    );
    assert_eq!(
        state.agents.get("p-default-idle").unwrap().status,
        AgentStatus::Idle
    );
    assert_eq!(
        state.agents.get("p-unknown").unwrap().status,
        AgentStatus::Unknown
    );
}
