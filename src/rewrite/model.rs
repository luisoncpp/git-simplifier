use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new(value: String) -> Result<Self, String> {
        if value.len() < 7
            || value
                .chars()
                .any(|character| !character.is_ascii_hexdigit())
        {
            return Err(format!("invalid object id: {value}"));
        }
        Ok(Self(value))
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let value = String::from_utf8(bytes.to_vec()).map_err(|_| "object id is not UTF-8")?;
        Self::new(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RefName(String);

impl RefName {
    pub fn new(value: String) -> Result<Self, String> {
        if value.is_empty() || value.contains('\0') || value.starts_with('-') {
            return Err(format!("invalid ref name: {value}"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RefName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RepoPath(String);

impl RepoPath {
    pub fn new(value: String) -> Result<Self, String> {
        if value.is_empty() || value.contains('\0') || value.starts_with('-') {
            return Err(format!("invalid repository path: {value}"));
        }
        Ok(Self(value))
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let value = String::from_utf8(bytes.to_vec()).map_err(|_| "path is not UTF-8")?;
        Self::new(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreeEntry {
    pub mode: String,
    pub kind: String,
    pub object: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
    pub date: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub author: Signature,
    pub committer: Signature,
    pub message: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RewriteAction {
    Rebuild,
    Drop,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RewriteOperation {
    Uncommit,
    EditMessage,
}

impl RewriteOperation {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Uncommit => "uncommit",
            Self::EditMessage => "edit-message",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommitRewrite {
    pub source: ObjectId,
    pub source_tree: ObjectId,
    pub first_parent: Option<ObjectId>,
    pub additional_parents: Vec<ObjectId>,
    pub metadata: CommitMetadata,
    pub action: RewriteAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UncommitRequest {
    pub base: RefName,
    pub paths: Vec<RepoPath>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub base: RefName,
    pub commit: ObjectId,
    pub message: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RewritePlan {
    pub operation: RewriteOperation,
    pub branch: RefName,
    pub base_ref: RefName,
    pub source_head: ObjectId,
    pub base: ObjectId,
    pub selected_paths: Vec<RepoPath>,
    pub base_entries: BTreeMap<RepoPath, Option<TreeEntry>>,
    pub commits: Vec<CommitRewrite>,
    pub dropped_commits: Vec<ObjectId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplyResult {
    pub old_head: ObjectId,
    pub new_head: ObjectId,
    pub dropped_commits: Vec<ObjectId>,
}
