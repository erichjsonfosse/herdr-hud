use crate::helpers::paths::discover_socket;
use crate::state::StatusBarState;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

pub struct HerdrClient;

impl Default for HerdrClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HerdrClient {
    pub fn new() -> Self {
        Self
    }

    pub fn discover_socket() -> Option<PathBuf> {
        discover_socket()
    }

    pub fn fetch_snapshot_sync(&self) -> Option<Value> {
        let output = Command::new("herdr")
            .args(["api", "snapshot"])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str(&stdout).ok()
        } else {
            None
        }
    }

    pub fn update_state_from_snapshot(state: &mut StatusBarState, snapshot: &Value) {
        let session = snapshot
            .pointer("/result/snapshot")
            .or_else(|| snapshot.pointer("/data/session_snapshot"))
            .or_else(|| snapshot.get("result").and_then(|r| r.get("snapshot")))
            .or_else(|| snapshot.get("data").and_then(|d| d.get("session_snapshot")))
            .unwrap_or(snapshot);

        // Extract focused IDs directly from session snapshot
        let focused_ws_id = session
            .get("focused_workspace_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let focused_tab_id = session
            .get("focused_tab_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let focused_pane_id = session
            .get("focused_pane_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(ref pane_id) = focused_pane_id {
            state.active_pane = Some(pane_id.clone());
        }

        // Match focused workspace in workspaces array
        if let Some(workspaces) = session.get("workspaces").and_then(|v| v.as_array()) {
            for ws in workspaces {
                let id = ws
                    .get("workspace_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let is_focused = ws.get("focused").and_then(|v| v.as_bool()).unwrap_or(false)
                    || focused_ws_id.as_deref() == Some(id);

                if is_focused {
                    let label = ws.get("label").and_then(|v| v.as_str()).unwrap_or(id);
                    state.active_workspace_id = Some(id.to_string());
                    state.active_workspace_label = Some(label.to_string());
                    state.active_workspace = Some(label.to_string());
                    break;
                }
            }
        }

        // Fallback for workspace if not found in array
        if state.active_workspace_id.is_none()
            && let Some(ref ws_id) = focused_ws_id
        {
            state.active_workspace_id = Some(ws_id.clone());
            state.active_workspace_label = Some(ws_id.clone());
            state.active_workspace = Some(ws_id.clone());
        }

        // Match focused tab in tabs array
        if let Some(tabs) = session.get("tabs").and_then(|v| v.as_array()) {
            for tab in tabs {
                let id = tab.get("tab_id").and_then(|v| v.as_str()).unwrap_or("");
                let is_focused = tab
                    .get("focused")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || focused_tab_id.as_deref() == Some(id);

                if is_focused {
                    let label = tab.get("label").and_then(|v| v.as_str()).unwrap_or(id);
                    state.active_tab_id = Some(id.to_string());
                    state.active_tab_label = Some(label.to_string());
                    state.active_tab = Some(label.to_string());
                    break;
                }
            }
        }

        // Fallback for tab if not found in array
        if state.active_tab_id.is_none()
            && let Some(ref tab_id) = focused_tab_id
        {
            state.active_tab_id = Some(tab_id.clone());
            state.active_tab_label = Some(tab_id.clone());
            state.active_tab = Some(tab_id.clone());
        }
    }
}

#[cfg(test)]
#[path = "client_unit.rs"]
mod tests;
