pub mod actions;
pub use actions::{
    ActionKind, ModalInputTarget, PaletteAction, PaletteCategory, build_prompt_command,
    get_palette_categories, trigger_action,
};
