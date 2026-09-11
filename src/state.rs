use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HerdrMode {
    #[default]
    Normal,
    Navigate,
    Scroll,
    Agent,
}

impl HerdrMode {
    #[allow(dead_code)]
    pub fn badge_name(&self) -> &'static str {
        match self {
            HerdrMode::Normal => "NORMAL",
            HerdrMode::Navigate => "NAVIGATE",
            HerdrMode::Scroll => "SCROLL",
            HerdrMode::Agent => "AGENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

impl AgentStatus {
    #[allow(dead_code)]
    pub fn symbol(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "○",
            AgentStatus::Working => "●",
            AgentStatus::Blocked => "▲",
            AgentStatus::Done => "✔",
            AgentStatus::Unknown => "?",
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "idle",
            AgentStatus::Working => "working",
            AgentStatus::Blocked => "blocked",
            AgentStatus::Done => "done",
            AgentStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEntry {
    pub pane_id: String,
    pub agent_name: String,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StatusBarState {
    pub mode: HerdrMode,
    pub active_workspace: Option<String>,
    pub active_workspace_id: Option<String>,
    pub active_workspace_label: Option<String>,
    pub active_tab: Option<String>,
    pub active_tab_id: Option<String>,
    pub active_tab_label: Option<String>,
    pub active_pane: Option<String>,
    pub agents: HashMap<String, AgentEntry>,
    pub prefix_key: String,
}

impl StatusBarState {
    pub fn new() -> Self {
        Self {
            mode: HerdrMode::Normal,
            active_workspace: None,
            active_workspace_id: None,
            active_workspace_label: None,
            active_tab: None,
            active_tab_id: None,
            active_tab_label: None,
            active_pane: None,
            agents: HashMap::new(),
            prefix_key: "Ctrl+S".to_string(),
        }
    }
}

#[cfg(test)]
#[path = "state_unit.rs"]
mod tests;
