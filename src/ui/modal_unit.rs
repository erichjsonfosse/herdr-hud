use super::*;
use crate::config::HudConfig;
use crate::palette::get_palette_categories;

#[test]
fn test_format_action_description_zero_width() {
    assert_eq!(format_action_description("Some description", 0), "");
}

#[test]
fn test_format_action_description_fits_within_width() {
    let result = format_action_description("Short", 10);
    assert_eq!(result, "Short      "); // 10 chars padded + 1 trailing space = 11 chars
    assert_eq!(result.len(), 11);
}

#[test]
fn test_format_action_description_exact_width() {
    let result = format_action_description("Exactly10!", 10);
    assert_eq!(result, "Exactly10! ");
}

#[test]
fn test_format_action_description_truncation() {
    let result =
        format_action_description("This is a very long description that exceeds the limit", 15);
    assert_eq!(result, "This is a ve... ");
    assert_eq!(result.chars().count(), 16); // 15 chars + 1 space
}

#[test]
fn test_format_action_description_tiny_width() {
    let result = format_action_description("Hello", 3);
    assert_eq!(result, "Hel ");
}

#[test]
fn test_hit_test_category_tab_hits_correct_tabs() {
    let config = HudConfig::default();
    let categories = get_palette_categories(&config);
    // Categories are:
    // 0: "Workspaces" -> "  [1. Workspaces]  " (len 19, cols 1..20)
    // separator: 2 spaces (cols 20..22)
    // 1: "Tabs"       -> "  [2. Tabs]  "       (len 13, cols 22..35)
    // separator: 2 spaces (cols 35..37)
    // 2: "Panes"      -> "  [3. Panes]  "      (len 14, cols 37..51)

    let row = 6; // cat_tabs_y is 1 + 5 = 6

    // Tab 0
    assert_eq!(hit_test_category_tab(1, row, &categories), Some(0));
    assert_eq!(hit_test_category_tab(19, row, &categories), Some(0));

    // Separator between 0 and 1
    assert_eq!(hit_test_category_tab(20, row, &categories), None);
    assert_eq!(hit_test_category_tab(21, row, &categories), None);

    // Tab 1
    assert_eq!(hit_test_category_tab(22, row, &categories), Some(1));
    assert_eq!(hit_test_category_tab(34, row, &categories), Some(1));

    // Separator between 1 and 2
    assert_eq!(hit_test_category_tab(35, row, &categories), None);
    assert_eq!(hit_test_category_tab(36, row, &categories), None);

    // Tab 2
    assert_eq!(hit_test_category_tab(37, row, &categories), Some(2));
    assert_eq!(hit_test_category_tab(50, row, &categories), Some(2));

    // Outside tabs
    assert_eq!(hit_test_category_tab(51, row, &categories), None);
    assert_eq!(hit_test_category_tab(100, row, &categories), None);

    // Wrong row
    assert_eq!(hit_test_category_tab(1, 5, &categories), None);
    assert_eq!(hit_test_category_tab(1, 9, &categories), None);
}

#[test]
fn test_hit_test_action_item() {
    let total_actions = 4;

    assert_eq!(hit_test_action_item(8, total_actions), None);
    assert_eq!(hit_test_action_item(9, total_actions), Some(0));
    assert_eq!(hit_test_action_item(10, total_actions), Some(1));
    assert_eq!(hit_test_action_item(11, total_actions), Some(2));
    assert_eq!(hit_test_action_item(12, total_actions), Some(3));
    assert_eq!(hit_test_action_item(13, total_actions), None);
}

#[test]
fn test_hit_test_with_custom_origin() {
    let config = HudConfig::default();
    let categories = get_palette_categories(&config);
    // Custom origin: x=10, y=5
    // cat_tabs_y is 5 + 5 = 10
    let row = 10;
    // Tab 0 starts at x=10, len=19 -> cols 10..29
    assert_eq!(
        hit_test_category_tab_at(10, row, &categories, 10, 5),
        Some(0)
    );
    assert_eq!(
        hit_test_category_tab_at(28, row, &categories, 10, 5),
        Some(0)
    );

    // Action list starts at cat_tabs_y + 3 = 13
    assert_eq!(hit_test_action_item_at(12, 4, 5), None);
    assert_eq!(hit_test_action_item_at(13, 4, 5), Some(0));
    assert_eq!(hit_test_action_item_at(16, 4, 5), Some(3));
    assert_eq!(hit_test_action_item_at(17, 4, 5), None);
}
