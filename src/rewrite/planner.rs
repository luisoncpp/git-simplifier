use std::collections::BTreeMap;

use crate::git::GitRunner;

use super::errors::RewriteError;
use super::model::{
    CommitRewrite, EditMessageRequest, ObjectId, RepoPath, RewriteAction, RewriteOperation,
    RewritePlan, TreeEntry, UncommitRequest,
};
use super::objects::{self, TreeSnapshot};
use super::preflight;

struct MessageEdit {
    target: ObjectId,
    message: Vec<u8>,
}

pub(crate) fn create(
    runner: &GitRunner,
    request: UncommitRequest,
) -> Result<RewritePlan, RewriteError> {
    validate_request(&request)?;
    let state = preflight::inspect(runner, &request.base)?;
    let base_tree = objects::read_tree(runner, &state.base)?;
    let base_entries = select_base_entries(&base_tree, &request.paths);
    let commits = build_commits(runner, &state.commits, &base_entries)?;
    let dropped_commits = commits
        .iter()
        .filter(|commit| commit.action == RewriteAction::Drop)
        .map(|commit| commit.source.clone())
        .collect();
    Ok(RewritePlan {
        operation: RewriteOperation::Uncommit,
        branch: state.branch,
        base_ref: request.base,
        source_head: state.head,
        base: state.base,
        selected_paths: request.paths,
        base_entries,
        commits,
        dropped_commits,
    })
}

pub(crate) fn create_edit_message(
    runner: &GitRunner,
    request: EditMessageRequest,
) -> Result<RewritePlan, RewriteError> {
    let state = preflight::inspect(runner, &request.base)?;
    let target = preflight::resolve_commit(runner, &request.commit)?;
    ensure_editable_target(&state.commits, &target)?;
    let edit = MessageEdit {
        target,
        message: request.message,
    };
    let commits = build_message_commits(runner, &state, &edit)?;
    Ok(RewritePlan {
        operation: RewriteOperation::EditMessage,
        branch: state.branch,
        base_ref: request.base,
        source_head: state.head,
        base: state.base,
        selected_paths: Vec::new(),
        base_entries: BTreeMap::new(),
        commits,
        dropped_commits: Vec::new(),
    })
}

fn validate_request(request: &UncommitRequest) -> Result<(), RewriteError> {
    if request.paths.is_empty() {
        return Err(RewriteError::InvalidState(
            "at least one path is required".to_string(),
        ));
    }
    Ok(())
}

fn ensure_editable_target(commits: &[ObjectId], target: &ObjectId) -> Result<(), RewriteError> {
    if commits.iter().any(|commit| commit == target) {
        return Ok(());
    }
    Err(RewriteError::InvalidState(
        "Commit is outside the Editable range".to_string(),
    ))
}

fn select_base_entries(
    tree: &TreeSnapshot,
    paths: &[RepoPath],
) -> BTreeMap<RepoPath, Option<TreeEntry>> {
    paths
        .iter()
        .map(|path| (path.clone(), tree.get(path).cloned()))
        .collect()
}

fn build_commits(
    runner: &GitRunner,
    sources: &[ObjectId],
    base_entries: &BTreeMap<RepoPath, Option<TreeEntry>>,
) -> Result<Vec<CommitRewrite>, RewriteError> {
    let mut effective = BTreeMap::new();
    let mut result = Vec::new();
    for source in sources {
        let commit = objects::read_commit(runner, source)?;
        let source_tree = objects::read_tree(runner, &commit.tree)?;
        let edited_tree = apply_edits(source_tree, base_entries);
        let parent_tree = parent_snapshot(runner, &commit.parents, &effective, base_entries)?;
        let action = if edited_tree == parent_tree {
            RewriteAction::Drop
        } else {
            RewriteAction::Rebuild
        };
        effective.insert(source.clone(), edited_tree);
        result.push(CommitRewrite {
            source: source.clone(),
            source_tree: commit.tree,
            first_parent: commit.parents.first().cloned(),
            additional_parents: commit.parents.iter().skip(1).cloned().collect(),
            metadata: commit.metadata,
            action,
        });
    }
    Ok(result)
}

fn build_message_commits(
    runner: &GitRunner,
    state: &preflight::RepoState,
    edit: &MessageEdit,
) -> Result<Vec<CommitRewrite>, RewriteError> {
    state
        .commits
        .iter()
        .map(|source| message_commit(runner, source, edit))
        .collect()
}

fn message_commit(
    runner: &GitRunner,
    source: &ObjectId,
    edit: &MessageEdit,
) -> Result<CommitRewrite, RewriteError> {
    let mut commit = objects::read_commit(runner, source)?;
    if source == &edit.target {
        commit.metadata.message = edit.message.clone();
    }
    Ok(CommitRewrite {
        source: source.clone(),
        source_tree: commit.tree,
        first_parent: commit.parents.first().cloned(),
        additional_parents: commit.parents.iter().skip(1).cloned().collect(),
        metadata: commit.metadata,
        action: RewriteAction::Rebuild,
    })
}

fn parent_snapshot(
    runner: &GitRunner,
    parents: &[ObjectId],
    effective: &BTreeMap<ObjectId, TreeSnapshot>,
    base_entries: &BTreeMap<RepoPath, Option<TreeEntry>>,
) -> Result<TreeSnapshot, RewriteError> {
    let Some(parent) = parents.first() else {
        return Ok(BTreeMap::new());
    };
    if let Some(snapshot) = effective.get(parent) {
        return Ok(snapshot.clone());
    }
    let commit = objects::read_commit(runner, parent)?;
    let tree = objects::read_tree(runner, &commit.tree)?;
    Ok(apply_edits(tree, base_entries))
}

fn apply_edits(
    mut tree: TreeSnapshot,
    base_entries: &BTreeMap<RepoPath, Option<TreeEntry>>,
) -> TreeSnapshot {
    for (path, entry) in base_entries {
        match entry {
            Some(entry) => {
                tree.insert(path.clone(), entry.clone());
            }
            None => {
                tree.remove(path);
            }
        }
    }
    tree
}
