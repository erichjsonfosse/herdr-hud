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
