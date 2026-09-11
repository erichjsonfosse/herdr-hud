use super::*;
use crate::config::StatusBarConfig;
use crate::state::StatusBarState;

fn make_test_action(kind: ActionKind, command: Option<Vec<String>>) -> PaletteAction {
    PaletteAction {
        name: "Test Action",
        key: "test_key".to_string(),
        description: "Test description",
        kind,
        command,
    }
}

#[test]
fn test_palette_categories_contain_expected_items() {
    let config = StatusBarConfig::default();
    let categories = get_palette_categories(&config);

    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0].name, "Workspaces");
    assert_eq!(categories[1].name, "Tabs");
    assert_eq!(categories[2].name, "Panes");

    let tab_actions = &categories[1].actions;
    assert!(tab_actions.iter().any(|a| a.name == "Create New Tab"));
    assert!(tab_actions.iter().any(|a| a.name == "Rename Tab"));
}

#[test]
fn test_trigger_action_direct_command() {
    // When command is present
    let action_with_cmd = make_test_action(
        ActionKind::DirectCommand,
        Some(vec![
            "herdr".to_string(),
            "workspace".to_string(),
            "create".to_string(),
        ]),
    );
    let state = StatusBarState::new();
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    let executed = trigger_action(
        &action_with_cmd,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "workspace".to_string(),
            "create".to_string()
        ])
    );
    assert_eq!(input_target, None);
    assert_eq!(input_buffer, "");

    // When command is None
    let action_no_cmd = make_test_action(ActionKind::DirectCommand, None);
    let mut pending_command = None;
    let executed = trigger_action(
        &action_no_cmd,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_prompt_create_tab() {
    let action = make_test_action(ActionKind::PromptCreateTab, None);
    let state = StatusBarState::new();
    let mut input_target = None;
    let mut input_buffer = "previous input text".to_string();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(input_target, Some(ModalInputTarget::CreateTab));
    assert!(input_buffer.is_empty());
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_prompt_rename_workspace() {
    let action = make_test_action(ActionKind::PromptRenameWorkspace, None);
    let mut state = StatusBarState::new();
    state.active_workspace = Some("dev-workspace".to_string());
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(input_target, Some(ModalInputTarget::RenameWorkspace));
    assert_eq!(input_buffer, "dev-workspace");
    assert_eq!(pending_command, None);

    // Fallback when active workspace label is None
    state.active_workspace = None;
    input_target = None;
    input_buffer = "stale".to_string();
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(input_target, Some(ModalInputTarget::RenameWorkspace));
    assert_eq!(input_buffer, "");
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_prompt_rename_tab() {
    let action = make_test_action(ActionKind::PromptRenameTab, None);
    let mut state = StatusBarState::new();
    state.active_tab = Some("editor-tab".to_string());
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(input_target, Some(ModalInputTarget::RenameTab));
    assert_eq!(input_buffer, "editor-tab");
    assert_eq!(pending_command, None);

    // Fallback when active tab label is None
    state.active_tab = None;
    input_target = None;
    input_buffer = "stale".to_string();
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(input_target, Some(ModalInputTarget::RenameTab));
    assert_eq!(input_buffer, "");
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_close_active_workspace() {
    let action = make_test_action(ActionKind::CloseActiveWorkspace, None);
    let mut state = StatusBarState::new();
    state.active_workspace_id = Some("ws-abc-123".to_string());
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "workspace".to_string(),
            "close".to_string(),
            "ws-abc-123".to_string()
        ])
    );

    // When active_workspace_id is None
    state.active_workspace_id = None;
    pending_command = None;
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_close_active_tab() {
    let action = make_test_action(ActionKind::CloseActiveTab, None);
    let mut state = StatusBarState::new();
    state.active_tab_id = Some("tab-xyz-456".to_string());
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "close".to_string(),
            "tab-xyz-456".to_string()
        ])
    );

    // When active_tab_id is None
    state.active_tab_id = None;
    pending_command = None;
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_close_active_pane() {
    let action = make_test_action(ActionKind::CloseActivePane, None);
    let mut state = StatusBarState::new();
    state.active_pane = Some("pane-def-789".to_string());
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "close".to_string(),
            "pane-def-789".to_string()
        ])
    );

    // When active_pane is None
    state.active_pane = None;
    pending_command = None;
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(pending_command, None);
}

#[test]
fn test_trigger_action_split_vertical() {
    let action = make_test_action(ActionKind::SplitVertical, None);
    let mut state = StatusBarState::new();
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    // With active pane
    state.active_pane = Some("pane-split-1".to_string());
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "split".to_string(),
            "--direction".to_string(),
            "right".to_string(),
            "--focus".to_string(),
            "--pane".to_string(),
            "pane-split-1".to_string(),
        ])
    );

    // Without active pane
    state.active_pane = None;
    pending_command = None;
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "split".to_string(),
            "--direction".to_string(),
            "right".to_string(),
            "--focus".to_string(),
        ])
    );
}

#[test]
fn test_trigger_action_split_horizontal() {
    let action = make_test_action(ActionKind::SplitHorizontal, None);
    let mut state = StatusBarState::new();
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    // With active pane
    state.active_pane = Some("pane-split-2".to_string());
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "split".to_string(),
            "--direction".to_string(),
            "down".to_string(),
            "--focus".to_string(),
            "--pane".to_string(),
            "pane-split-2".to_string(),
        ])
    );

    // Without active pane
    state.active_pane = None;
    pending_command = None;
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "split".to_string(),
            "--direction".to_string(),
            "down".to_string(),
            "--focus".to_string(),
        ])
    );
}

#[test]
fn test_trigger_action_toggle_zoom() {
    let action = make_test_action(ActionKind::ToggleZoom, None);
    let mut state = StatusBarState::new();
    let mut input_target = None;
    let mut input_buffer = String::new();
    let mut pending_command = None;

    // With active pane
    state.active_pane = Some("pane-zoom-1".to_string());
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "zoom".to_string(),
            "--pane".to_string(),
            "pane-zoom-1".to_string(),
        ])
    );

    // Without active pane: targets --current
    state.active_pane = None;
    pending_command = None;
    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(executed);
    assert_eq!(
        pending_command,
        Some(vec![
            "herdr".to_string(),
            "pane".to_string(),
            "zoom".to_string(),
            "--current".to_string(),
        ])
    );
}

#[test]
fn test_trigger_action_shortcut_only() {
    let action = make_test_action(ActionKind::ShortcutOnly, None);
    let state = StatusBarState::new();
    let mut input_target = None;
    let mut input_buffer = "unmodified_buffer".to_string();
    let mut pending_command = None;

    let executed = trigger_action(
        &action,
        &state,
        &mut input_target,
        &mut input_buffer,
        &mut pending_command,
    );

    assert!(!executed);
    assert_eq!(pending_command, None);
    assert_eq!(input_target, None);
    assert_eq!(input_buffer, "unmodified_buffer");
}

#[test]
fn test_trigger_action_all_catalog_actions() {
    let config = StatusBarConfig::default();
    let categories = get_palette_categories(&config);

    let mut state = StatusBarState::new();
    state.active_workspace_id = Some("ws-cat-1".to_string());
    state.active_workspace = Some("cat-workspace".to_string());
    state.active_tab_id = Some("tab-cat-1".to_string());
    state.active_tab = Some("cat-tab".to_string());
    state.active_pane = Some("pane-cat-1".to_string());

    for category in categories {
        for action in category.actions {
            let mut input_target = None;
            let mut input_buffer = String::new();
            let mut pending_command = None;

            let executed = trigger_action(
                &action,
                &state,
                &mut input_target,
                &mut input_buffer,
                &mut pending_command,
            );

            match action.kind {
                ActionKind::DirectCommand => {
                    assert!(executed, "Action '{}' should execute", action.name);
                    assert_eq!(pending_command, action.command);
                }
                ActionKind::PromptCreateTab => {
                    assert!(!executed);
                    assert_eq!(input_target, Some(ModalInputTarget::CreateTab));
                    assert!(input_buffer.is_empty());
                }
                ActionKind::PromptRenameWorkspace => {
                    assert!(!executed);
                    assert_eq!(input_target, Some(ModalInputTarget::RenameWorkspace));
                    assert_eq!(input_buffer, "cat-workspace");
                }
                ActionKind::PromptRenameTab => {
                    assert!(!executed);
                    assert_eq!(input_target, Some(ModalInputTarget::RenameTab));
                    assert_eq!(input_buffer, "cat-tab");
                }
                ActionKind::CloseActiveWorkspace => {
                    assert!(executed);
                    assert_eq!(
                        pending_command,
                        Some(vec![
                            "herdr".to_string(),
                            "workspace".to_string(),
                            "close".to_string(),
                            "ws-cat-1".to_string(),
                        ])
                    );
                }
                ActionKind::CloseActiveTab => {
                    assert!(executed);
                    assert_eq!(
                        pending_command,
                        Some(vec![
                            "herdr".to_string(),
                            "tab".to_string(),
                            "close".to_string(),
                            "tab-cat-1".to_string(),
                        ])
                    );
                }
                ActionKind::CloseActivePane => {
                    assert!(executed);
                    assert_eq!(
                        pending_command,
                        Some(vec![
                            "herdr".to_string(),
                            "pane".to_string(),
                            "close".to_string(),
                            "pane-cat-1".to_string(),
                        ])
                    );
                }
                ActionKind::SplitVertical => {
                    assert!(executed);
                    assert_eq!(
                        pending_command,
                        Some(vec![
                            "herdr".to_string(),
                            "pane".to_string(),
                            "split".to_string(),
                            "--direction".to_string(),
                            "right".to_string(),
                            "--focus".to_string(),
                            "--pane".to_string(),
                            "pane-cat-1".to_string(),
                        ])
                    );
                }
                ActionKind::SplitHorizontal => {
                    assert!(executed);
                    assert_eq!(
                        pending_command,
                        Some(vec![
                            "herdr".to_string(),
                            "pane".to_string(),
                            "split".to_string(),
                            "--direction".to_string(),
                            "down".to_string(),
                            "--focus".to_string(),
                            "--pane".to_string(),
                            "pane-cat-1".to_string(),
                        ])
                    );
                }
                ActionKind::ToggleZoom => {
                    assert!(executed);
                    assert_eq!(
                        pending_command,
                        Some(vec![
                            "herdr".to_string(),
                            "pane".to_string(),
                            "zoom".to_string(),
                            "--pane".to_string(),
                            "pane-cat-1".to_string(),
                        ])
                    );
                }
                ActionKind::ShortcutOnly => {
                    assert!(!executed);
                    assert_eq!(pending_command, None);
                    assert_eq!(input_target, None);
                }
            }
        }
    }
}

#[test]
fn test_build_prompt_command_create_tab_with_name() {
    let state = StatusBarState::new();
    let cmd = build_prompt_command(&ModalInputTarget::CreateTab, "my-new-tab", &state);
    assert_eq!(
        cmd,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "create".to_string(),
            "--focus".to_string(),
            "--label".to_string(),
            "my-new-tab".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_create_tab_whitespace_trimming() {
    let state = StatusBarState::new();
    let cmd = build_prompt_command(&ModalInputTarget::CreateTab, "   trimmed-tab   ", &state);
    assert_eq!(
        cmd,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "create".to_string(),
            "--focus".to_string(),
            "--label".to_string(),
            "trimmed-tab".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_create_tab_empty_or_spaces() {
    let state = StatusBarState::new();
    let cmd_empty = build_prompt_command(&ModalInputTarget::CreateTab, "", &state);
    assert_eq!(
        cmd_empty,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "create".to_string(),
            "--focus".to_string(),
        ])
    );

    let cmd_spaces = build_prompt_command(&ModalInputTarget::CreateTab, "   ", &state);
    assert_eq!(
        cmd_spaces,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "create".to_string(),
            "--focus".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_rename_workspace_success() {
    let mut state = StatusBarState::new();
    state.active_workspace_id = Some("ws-alpha-1".to_string());

    let cmd = build_prompt_command(&ModalInputTarget::RenameWorkspace, "project-work", &state);
    assert_eq!(
        cmd,
        Some(vec![
            "herdr".to_string(),
            "workspace".to_string(),
            "rename".to_string(),
            "ws-alpha-1".to_string(),
            "project-work".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_rename_workspace_whitespace_trimming() {
    let mut state = StatusBarState::new();
    state.active_workspace_id = Some("ws-alpha-1".to_string());

    let cmd = build_prompt_command(
        &ModalInputTarget::RenameWorkspace,
        "   project-work   ",
        &state,
    );
    assert_eq!(
        cmd,
        Some(vec![
            "herdr".to_string(),
            "workspace".to_string(),
            "rename".to_string(),
            "ws-alpha-1".to_string(),
            "project-work".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_rename_workspace_empty_or_whitespace() {
    let mut state = StatusBarState::new();
    state.active_workspace_id = Some("ws-alpha-1".to_string());

    let cmd_empty = build_prompt_command(&ModalInputTarget::RenameWorkspace, "", &state);
    assert_eq!(cmd_empty, None);

    let cmd_spaces = build_prompt_command(&ModalInputTarget::RenameWorkspace, "   \t\n   ", &state);
    assert_eq!(cmd_spaces, None);
}

#[test]
fn test_build_prompt_command_rename_workspace_missing_id() {
    let state = StatusBarState::new(); // active_workspace_id is None
    let cmd = build_prompt_command(&ModalInputTarget::RenameWorkspace, "new-name", &state);
    assert_eq!(cmd, None);
}

#[test]
fn test_build_prompt_command_rename_tab_success() {
    let mut state = StatusBarState::new();
    state.active_tab_id = Some("tab-beta-2".to_string());

    let cmd = build_prompt_command(&ModalInputTarget::RenameTab, "editor", &state);
    assert_eq!(
        cmd,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "rename".to_string(),
            "tab-beta-2".to_string(),
            "editor".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_rename_tab_whitespace_trimming() {
    let mut state = StatusBarState::new();
    state.active_tab_id = Some("tab-beta-2".to_string());

    let cmd = build_prompt_command(&ModalInputTarget::RenameTab, "   editor   ", &state);
    assert_eq!(
        cmd,
        Some(vec![
            "herdr".to_string(),
            "tab".to_string(),
            "rename".to_string(),
            "tab-beta-2".to_string(),
            "editor".to_string(),
        ])
    );
}

#[test]
fn test_build_prompt_command_rename_tab_empty_or_whitespace() {
    let mut state = StatusBarState::new();
    state.active_tab_id = Some("tab-beta-2".to_string());

    let cmd_empty = build_prompt_command(&ModalInputTarget::RenameTab, "", &state);
    assert_eq!(cmd_empty, None);

    let cmd_spaces = build_prompt_command(&ModalInputTarget::RenameTab, "   \t\n   ", &state);
    assert_eq!(cmd_spaces, None);
}

#[test]
fn test_build_prompt_command_rename_tab_missing_id() {
    let state = StatusBarState::new(); // active_tab_id is None
    let cmd = build_prompt_command(&ModalInputTarget::RenameTab, "editor", &state);
    assert_eq!(cmd, None);
}
