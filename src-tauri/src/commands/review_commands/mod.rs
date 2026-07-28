mod composite;
mod rewrite;
mod saved_work;
#[cfg(test)]
mod tests;

pub(super) use composite::{quick_switch, sync};
pub(super) use rewrite::{exclude_submodule, rewrite};
pub(super) use saved_work::{delete_saved_work, restore_saved_work};
