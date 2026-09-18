use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HudState {
    pub active_workspace_id: Option<String>,
    pub active_workspace: Option<String>,
    pub active_tab_id: Option<String>,
    pub active_tab: Option<String>,
    pub active_pane: Option<String>,
}

impl HudState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[path = "state_unit.rs"]
mod tests;
