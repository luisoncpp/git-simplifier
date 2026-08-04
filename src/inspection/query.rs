use serde::{Deserialize, Serialize};

use super::model::DiffCompare;

/// Local Files-diff discovery constraints. Applied during `ls-files` / before
/// body reads — not as a post-pass over a maximal ignored tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UntrackedFilters {
    pub exclude_older_than_head: bool,
    pub exclude_root_dot: bool,
    pub exclude_node_modules: bool,
    pub respect_gitignore: bool,
    pub exclude_unknown_types: bool,
}

impl Default for UntrackedFilters {
    fn default() -> Self {
        Self {
            exclude_older_than_head: true,
            exclude_root_dot: true,
            exclude_node_modules: true,
            respect_gitignore: true,
            exclude_unknown_types: true,
        }
    }
}

/// Compare mode plus Local untracked discovery filters.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FilesDiffQuery {
    pub compare: DiffCompare,
    #[serde(default)]
    pub untracked: UntrackedFilters,
}

impl FilesDiffQuery {
    pub fn new(compare: DiffCompare) -> Self {
        Self {
            compare,
            untracked: UntrackedFilters::default(),
        }
    }
}
