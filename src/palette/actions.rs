use crate::config::StatusBarConfig;
use crate::state::StatusBarState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    DirectCommand,
    PromptCreateTab,
    PromptRenameWorkspace,
    PromptRenameTab,
    CloseActiveWorkspace,
    CloseActiveTab,
    CloseActivePane,
    SplitVertical,
    SplitHorizontal,
    ToggleZoom,
    ShortcutOnly,
}

#[derive(Debug, Clone)]
pub struct PaletteAction {
    pub name: &'static str,
    pub key: String,
    pub description: &'static str,
    pub kind: ActionKind,
    pub command: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct PaletteCategory {
    pub name: &'static str,
    pub key_number: char,
    pub actions: Vec<PaletteAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalInputTarget {
    CreateTab,
    RenameWorkspace,
    RenameTab,
}

pub fn get_palette_categories(config: &StatusBarConfig) -> Vec<PaletteCategory> {
    let p = &config.prefix_key;
    vec![
        PaletteCategory {
            name: "Workspaces",
            key_number: '1',
            actions: vec![
                PaletteAction {
                    name: "Create Workspace",
                    key: format!("{} Shift+N", p),
                    description: "Create and switch to a new workspace",
                    kind: ActionKind::DirectCommand,
                    command: Some(vec![
                        "herdr".to_string(),
                        "workspace".to_string(),
                        "create".to_string(),
                    ]),
                },
                PaletteAction {
                    name: "Rename Workspace",
                    key: format!("{} Shift+W", p),
                    description: "Rename the active workspace",
                    kind: ActionKind::PromptRenameWorkspace,
                    command: None,
                },
                PaletteAction {
                    name: "Navigate Workspaces",
                    key: format!("{} W", p),
                    description: "Open the interactive workspace selector",
                    kind: ActionKind::ShortcutOnly,
                    command: None,
                },
                PaletteAction {
                    name: "Close Workspace",
                    key: format!("{} Shift+D", p),
                    description: "Close active workspace and its tabs",
                    kind: ActionKind::CloseActiveWorkspace,
                    command: None,
                },
            ],
        },
        PaletteCategory {
            name: "Tabs",
            key_number: '2',
            actions: vec![
                PaletteAction {
                    name: "Create New Tab",
                    key: format!("{} C", p),
                    description: "Open a new tab in current workspace",
                    kind: ActionKind::PromptCreateTab,
                    command: None,
                },
                PaletteAction {
                    name: "Rename Tab",
                    key: format!("{} Shift+T", p),
                    description: "Rename the currently active tab",
                    kind: ActionKind::PromptRenameTab,
                    command: None,
                },
                PaletteAction {
                    name: "Next Tab",
                    key: format!("{} N", p),
                    description: "Switch to next tab on the right",
                    kind: ActionKind::ShortcutOnly,
                    command: None,
                },
                PaletteAction {
                    name: "Previous Tab",
                    key: format!("{} P", p),
                    description: "Switch to previous tab on the left",
                    kind: ActionKind::ShortcutOnly,
                    command: None,
                },
                PaletteAction {
                    name: "Close Tab",
                    key: format!("{} Shift+X", p),
                    description: "Close the currently active tab",
                    kind: ActionKind::CloseActiveTab,
                    command: None,
                },
            ],
        },
        PaletteCategory {
            name: "Panes",
            key_number: '3',
            actions: vec![
                PaletteAction {
                    name: "Split Vertical",
                    key: format!("{} V", p),
                    description: "Split current pane side-by-side (right)",
                    kind: ActionKind::SplitVertical,
                    command: None,
                },
                PaletteAction {
                    name: "Split Horizontal",
                    key: format!("{} -", p),
                    description: "Split current pane top-and-bottom (down)",
                    kind: ActionKind::SplitHorizontal,
                    command: None,
                },
                PaletteAction {
                    name: "Toggle Zoom",
                    key: format!("{} Z", p),
                    description: "Maximize or restore focused pane size",
                    kind: ActionKind::ToggleZoom,
                    command: None,
                },
                PaletteAction {
                    name: "Close Pane",
                    key: format!("{} X", p),
                    description: "Close the currently focused terminal pane",
                    kind: ActionKind::CloseActivePane,
                    command: None,
                },
                PaletteAction {
                    name: "Focus Pane",
                    key: format!("{} hjkl", p),
                    description: "Move focus between neighboring panes",
                    kind: ActionKind::ShortcutOnly,
                    command: None,
                },
            ],
        },
    ]
}

pub fn trigger_action(
    action: &PaletteAction,
    state: &StatusBarState,
    input_target: &mut Option<ModalInputTarget>,
    input_buffer: &mut String,
    pending_command: &mut Option<Vec<String>>,
) -> bool {
    match action.kind {
        ActionKind::DirectCommand => {
            if let Some(cmd) = &action.command {
                *pending_command = Some(cmd.clone());
                return true;
            }
        }
        ActionKind::PromptCreateTab => {
            *input_target = Some(ModalInputTarget::CreateTab);
            input_buffer.clear();
        }
        ActionKind::PromptRenameWorkspace => {
            *input_target = Some(ModalInputTarget::RenameWorkspace);
            *input_buffer = state.active_workspace_label.clone().unwrap_or_default();
        }
        ActionKind::PromptRenameTab => {
            *input_target = Some(ModalInputTarget::RenameTab);
            *input_buffer = state.active_tab_label.clone().unwrap_or_default();
        }
        ActionKind::CloseActiveWorkspace => {
            if let Some(ws_id) = &state.active_workspace_id {
                *pending_command = Some(vec![
                    "herdr".to_string(),
                    "workspace".to_string(),
                    "close".to_string(),
                    ws_id.clone(),
                ]);
                return true;
            }
        }
        ActionKind::CloseActiveTab => {
            if let Some(tab_id) = &state.active_tab_id {
                *pending_command = Some(vec![
                    "herdr".to_string(),
                    "tab".to_string(),
                    "close".to_string(),
                    tab_id.clone(),
                ]);
                return true;
            }
        }
        ActionKind::CloseActivePane => {
            if let Some(pane_id) = &state.active_pane {
                *pending_command = Some(vec![
                    "herdr".to_string(),
                    "pane".to_string(),
                    "close".to_string(),
                    pane_id.clone(),
                ]);
                return true;
            }
        }
        ActionKind::SplitVertical => {
            let mut cmd = vec![
                "herdr".to_string(),
                "pane".to_string(),
                "split".to_string(),
                "--direction".to_string(),
                "right".to_string(),
                "--focus".to_string(),
            ];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            }
            *pending_command = Some(cmd);
            return true;
        }
        ActionKind::SplitHorizontal => {
            let mut cmd = vec![
                "herdr".to_string(),
                "pane".to_string(),
                "split".to_string(),
                "--direction".to_string(),
                "down".to_string(),
                "--focus".to_string(),
            ];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            }
            *pending_command = Some(cmd);
            return true;
        }
        ActionKind::ToggleZoom => {
            let mut cmd = vec!["herdr".to_string(), "pane".to_string(), "zoom".to_string()];
            if let Some(pane_id) = &state.active_pane {
                cmd.push("--pane".to_string());
                cmd.push(pane_id.clone());
            } else {
                cmd.push("--current".to_string());
            }
            *pending_command = Some(cmd);
            return true;
        }
        ActionKind::ShortcutOnly => {}
    }
    false
}

#[cfg(test)]
#[path = "actions_unit.rs"]
mod tests;
