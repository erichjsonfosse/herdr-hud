use super::*;

#[test]
fn test_status_bar_state_default() {
    let state = StatusBarState::default();
    assert!(state.active_workspace_id.is_none());
    assert!(state.active_workspace_label.is_none());
    assert!(state.active_workspace.is_none());
    assert!(state.active_tab_id.is_none());
    assert!(state.active_tab_label.is_none());
    assert!(state.active_tab.is_none());
    assert!(state.active_pane.is_none());
}

#[test]
fn test_status_bar_state_new() {
    let state = StatusBarState::new();
    assert_eq!(state, StatusBarState::default());
}

#[test]
fn test_status_bar_state_serde_roundtrip_default() {
    let state = StatusBarState::default();
    let serialized = serde_json::to_string(&state).expect("serialize default");
    let deserialized: StatusBarState =
        serde_json::from_str(&serialized).expect("deserialize default");
    assert_eq!(state, deserialized);
}

#[test]
fn test_status_bar_state_serde_roundtrip_populated() {
    let state = StatusBarState {
        active_workspace_id: Some("ws-1".to_string()),
        active_workspace_label: Some("Primary".to_string()),
        active_workspace: Some("Primary".to_string()),
        active_tab_id: Some("tab-4".to_string()),
        active_tab_label: Some("Editor".to_string()),
        active_tab: Some("Editor".to_string()),
        active_pane: Some("pane-12".to_string()),
    };

    let serialized = serde_json::to_string(&state).expect("serialize populated");
    let deserialized: StatusBarState =
        serde_json::from_str(&serialized).expect("deserialize populated");
    assert_eq!(state, deserialized);
}
