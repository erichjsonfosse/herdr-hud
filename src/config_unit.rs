use super::*;
use toml::Value as TomlValue;

#[test]
fn test_from_herdr_config_defaults() {
    let config = StatusBarConfig::from_herdr_config(None);
    assert_eq!(config.prefix_key, "Ctrl+B");
    assert_eq!(config.normal_hints[0].key, "Ctrl+B");
    assert_eq!(config.normal_hints[0].description, "Prefix");
    assert_eq!(config.normal_hints[1].key, "c");
    assert_eq!(config.normal_hints[1].description, "Tab");
    assert_eq!(config.normal_hints[2].key, "v");
    assert_eq!(config.normal_hints[2].description, "Split");
}

#[test]
fn test_from_herdr_config_custom_prefix() {
    let toml_str = r#"
        prefix = "ctrl+a"
    "#;
    let toml_val: TomlValue = toml::from_str(toml_str).unwrap();
    let config = StatusBarConfig::from_herdr_config(Some(&toml_val));

    assert_eq!(config.prefix_key, "Ctrl+A");
    assert_eq!(config.normal_hints[0].key, "Ctrl+A");
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
    let config = StatusBarConfig::from_herdr_config(Some(&toml_val));

    assert_eq!(config.prefix_key, "Alt+Space");
    assert_eq!(config.normal_hints[1].key, "t");
    assert_eq!(config.normal_hints[1].description, "Tab");
    assert_eq!(config.normal_hints[2].key, "s");
    assert_eq!(config.normal_hints[2].description, "Split");
}
