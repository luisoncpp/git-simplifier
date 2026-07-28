use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RepoPath};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExcludeSubmoduleRequest {
    pub path: RepoPath,
    pub install_hook: bool,
    pub disable_recurse: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExcludeSubmodulePlan {
    pub path: RepoPath,
    pub install_hook: bool,
    pub disable_recurse: bool,
    pub config_lines: Vec<String>,
    pub staging_command: String,
    pub hook_path: PathBuf,
    pub hook_preview: String,
    pub hook_exists: bool,
    pub hook_will_change: bool,
    pub current_ignore: Option<String>,
    pub current_recurse: Option<String>,
    pub(crate) current_hook: Vec<u8>,
    pub(crate) source_head: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExcludeSubmoduleResult {
    pub path: RepoPath,
    pub config_changed: bool,
    pub hook_changed: bool,
}
