use super::*;
use crate::helpers::paths::{EnvVarGuard, TEST_ENV_MUTEX, TempDir};
use toml::Value as TomlValue;

#[test]
fn test_from_herdr_config_defaults() {
    let config = HudConfig::from_herdr_config(None);
    assert_eq!(config.prefix_key, "Ctrl+B");
    assert_eq!(config.hints[0].key, "c");
    assert_eq!(config.hints[0].description, "Tab");
    assert_eq!(config.hints[1].key, "v");
    assert_eq!(config.hints[1].description, "Vert");
    assert_eq!(config.hints[2].key, "-");
    assert_eq!(config.hints[2].description, "Horiz");
}

#[test]
fn test_from_herdr_config_custom_prefix() {
    let toml_str = r#"
        prefix = "ctrl+a"
    "#;
    let toml_val: TomlValue = toml::from_str(toml_str).unwrap();
    let config = HudConfig::from_herdr_config(Some(&toml_val));

    assert_eq!(config.prefix_key, "Ctrl+A");
}

#[test]
fn test_from_herdr_config_custom_keys() {
    let toml_str = r#"
        prefix = "alt+space"
        [keybindings]
        new_tab = "prefix+t"
        split_vertical = "prefix+s"
    "#;
    let toml_val: TomlValue = toml::from_str(toml_str).unwrap();
    let config = HudConfig::from_herdr_config(Some(&toml_val));

    assert_eq!(config.prefix_key, "Alt+Space");
    assert_eq!(config.hints[0].key, "t");
    assert_eq!(config.hints[0].description, "Tab");
    assert_eq!(config.hints[1].key, "s");
    assert_eq!(config.hints[1].description, "Vert");
    assert_eq!(config.hints[2].key, "-");
    assert_eq!(config.hints[2].description, "Horiz");
}

#[test]
fn test_from_herdr_config_keys_section_fallback() {
    let toml_str = r#"
        [keys]
        prefix = "meta+space"
        new_tab = "prefix+n"
    "#;
    let toml_val: TomlValue = toml::from_str(toml_str).unwrap();
    let config = HudConfig::from_herdr_config(Some(&toml_val));

    assert_eq!(config.prefix_key, "Meta+Space");
    assert_eq!(config.hints[0].key, "n");
}

#[test]
fn test_load_resolution_order_json_override() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("load_json");

    let json_path = temp.path().join("hud.json");
    let json_content = serde_json::json!({
        "prefix_key": "Ctrl+Space",
        "hints": [
            { "key": "Ctrl+Space", "description": "Custom Prefix" },
            { "key": "t", "description": "Custom Tab" }
        ]
    });
    std::fs::write(&json_path, json_content.to_string()).expect("failed to write test json config");

    let toml_path = temp.path().join("config.toml");
    std::fs::write(&toml_path, "prefix = \"alt+x\"\n").expect("failed to write test toml config");

    let _hud_guard = EnvVarGuard::set("HERDR_HUD_CONFIG", &json_path);
    let _toml_guard = EnvVarGuard::set("HERDR_CONFIG_PATH", &toml_path);

    let config = HudConfig::load();

    assert_eq!(config.prefix_key, "Ctrl+Space");
    assert_eq!(config.hints.len(), 2);
    assert_eq!(config.hints[0].key, "Ctrl+Space");
    assert_eq!(config.hints[0].description, "Custom Prefix");
    assert_eq!(config.hints[1].key, "t");
    assert_eq!(config.hints[1].description, "Custom Tab");
}

#[test]
fn test_load_resolution_order_toml_fallback() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("load_toml");

    let non_existent_json = temp.path().join("non_existent.json");
    let toml_path = temp.path().join("config.toml");
    let toml_content = r#"
        prefix = "ctrl+j"
        [keybindings]
        new_tab = "prefix+w"
        split_vertical = "prefix+s"
    "#;
    std::fs::write(&toml_path, toml_content).expect("failed to write test toml config");

    let _hud_guard = EnvVarGuard::set("HERDR_HUD_CONFIG", &non_existent_json);
    let _toml_guard = EnvVarGuard::set("HERDR_CONFIG_PATH", &toml_path);

    let config = HudConfig::load();

    assert_eq!(config.prefix_key, "Ctrl+J");
    assert_eq!(config.hints[0].key, "w");
    assert_eq!(config.hints[0].description, "Tab");
    assert_eq!(config.hints[1].key, "s");
    assert_eq!(config.hints[1].description, "Vert");
    assert_eq!(config.hints[2].key, "-");
    assert_eq!(config.hints[2].description, "Horiz");
}

#[test]
fn test_load_resolution_order_defaults_fallback() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("load_defaults");

    let non_existent_json = temp.path().join("non_existent.json");
    let non_existent_toml = temp.path().join("non_existent.toml");

    let _hud_guard = EnvVarGuard::set("HERDR_HUD_CONFIG", &non_existent_json);
    let _toml_guard = EnvVarGuard::set("HERDR_CONFIG_PATH", &non_existent_toml);

    let config = HudConfig::load();

    assert_eq!(config.prefix_key, "Ctrl+B");
    assert_eq!(config.hints[0].key, "c");
    assert_eq!(config.hints[0].description, "Tab");
    assert_eq!(config.hints[1].key, "v");
    assert_eq!(config.hints[1].description, "Vert");
    assert_eq!(config.hints[2].key, "-");
    assert_eq!(config.hints[2].description, "Horiz");
}
