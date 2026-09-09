use super::*;
use crate::config::StatusBarConfig;

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
