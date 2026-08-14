pub mod actions;
mod apply;
pub mod bash;
pub mod codechart;
mod data;
pub mod diffs;
mod ide_spawn;
pub mod ide;
pub mod prefs;
mod prepare;
mod project_settings;
mod recents;
pub(crate) mod repository;
mod review_commands;
mod state;
pub mod terminal;

pub use state::AppState;
