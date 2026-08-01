pub mod actions;
mod apply;
mod data;
pub mod diffs;
mod prepare;
mod prefs;
mod recents;
pub(crate) mod repository;
mod review_commands;
mod state;

pub use state::AppState;
