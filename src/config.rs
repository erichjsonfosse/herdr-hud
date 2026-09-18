use crate::helpers::keys::{format_action_key_in_navigate, format_key_chord};
use crate::helpers::paths::{herdr_config_path, plugin_config_path};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyHint {
    pub key: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HudConfig {
    pub prefix_key: String,
    pub hints: Vec<KeyHint>,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self::from_herdr_config(None)
    }
}

impl HudConfig {
    pub fn load() -> Self {
        // 1. Try to read from HUD-specific override JSON if present
        let plugin_override_path = plugin_config_path();
        if let Ok(content) = std::fs::read_to_string(&plugin_override_path)
            && let Ok(cfg) = serde_json::from_str::<HudConfig>(&content)
        {
            return cfg;
        }

        // 2. Read live Herdr TOML configuration
        let herdr_cfg_path = herdr_config_path();
        if let Ok(content) = std::fs::read_to_string(&herdr_cfg_path)
            && let Ok(toml_val) = toml::from_str::<TomlValue>(&content)
        {
            return Self::from_herdr_config(Some(&toml_val));
        }

        // 3. Fallback to default Herdr built-in shortcuts
        Self::from_herdr_config(None)
    }

    pub fn from_herdr_config(config: Option<&TomlValue>) -> Self {
        // Herdr default prefix is "ctrl+b"
        let raw_prefix = config
            .and_then(|c| {
                c.get("prefix")
                    .or_else(|| c.get("keys").and_then(|k| k.get("prefix")))
            })
            .and_then(|v| v.as_str())
            .unwrap_or("ctrl+b");

        let formatted_prefix = format_key_chord(raw_prefix);

        // Helper to extract keys as borrowed string slices
        let get_key = |key_name: &str, default_val: &'static str| -> &str {
            config
                .and_then(|c| {
                    c.get(key_name)
                        .or_else(|| c.get("keybindings").and_then(|k| k.get(key_name)))
                        .or_else(|| c.get("keys").and_then(|k| k.get(key_name)))
                })
                .and_then(|v| v.as_str())
                .unwrap_or(default_val)
        };

        let raw_new_tab = get_key("new_tab", "prefix+c");
        let raw_split_v = get_key("split_vertical", "prefix+v");
        let raw_split_h = get_key("split_horizontal", "prefix+-");
        let raw_close = get_key("close_pane", "prefix+x");
        let raw_zoom = get_key("zoom", "prefix+z");
        let raw_help = get_key("help", "prefix+?");
        let raw_detach = get_key("detach", "prefix+q");

        let hints = vec![
            KeyHint {
                key: format_action_key_in_navigate(raw_new_tab),
                description: "Tab".to_string(),
            },
            KeyHint {
                key: format_action_key_in_navigate(raw_split_v),
                description: "Vert".to_string(),
            },
            KeyHint {
                key: format_action_key_in_navigate(raw_split_h),
                description: "Horiz".to_string(),
            },
            KeyHint {
                key: format_action_key_in_navigate(raw_close),
                description: "Close".to_string(),
            },
            KeyHint {
                key: format_action_key_in_navigate(raw_zoom),
                description: "Zoom".to_string(),
            },
            KeyHint {
                key: format_action_key_in_navigate(raw_help),
                description: "Help".to_string(),
            },
            KeyHint {
                key: format_action_key_in_navigate(raw_detach),
                description: "Detach".to_string(),
            },
        ];

        Self {
            prefix_key: formatted_prefix,
            hints,
        }
    }
}

#[cfg(test)]
#[path = "config_unit.rs"]
mod tests;
