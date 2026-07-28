use std::collections::HashMap;
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};

use super::errors::ApplyError;
use super::model::{CommitRewrite, ObjectId, RewriteAction, RewritePlan, TreeEntry};

pub(crate) fn build_history(
    runner: &GitRunner,
    plan: &RewritePlan,
    index: &std::path::Path,
) -> Result<ObjectId, ApplyError> {
    let mut rewritten = HashMap::new();
    for commit in &plan.commits {
        let tree = rewrite_tree(runner, plan, commit, index)?;
        let mapped = commit_with_tree(runner, commit, tree, &rewritten)?;
        rewritten.insert(commit.source.clone(), mapped);
    }
    let last = plan
        .commits
        .last()
        .ok_or_else(|| ApplyError::InvalidPlan("missing final commit".to_string()))?;
    rewritten
        .get(&last.source)
        .cloned()
        .ok_or_else(|| ApplyError::InvalidPlan("final commit was not materialized".to_string()))
}

fn rewrite_tree(
    runner: &GitRunner,
    plan: &RewritePlan,
    commit: &CommitRewrite,
    index: &std::path::Path,
) -> Result<ObjectId, ApplyError> {
    let environment = OsString::from(index.as_os_str());
    let read_args = vec!["read-tree", commit.source_tree.as_str()];
    let read = GitCommand::write(GitRunner::command_args(&read_args))
        .with_environment(OsString::from("GIT_INDEX_FILE"), environment.clone());
    runner.run_unlocked(read)?;
    for (path, entry) in &plan.base_entries {
        update_index(runner, path.as_str(), entry.as_ref(), &environment)?;
    }
    let write = GitCommand::write(GitRunner::command_args(&["write-tree"]))
        .with_environment(OsString::from("GIT_INDEX_FILE"), environment);
    let output = runner.run_unlocked(write)?;
    parse_id(&output.stdout)
        .map_err(|error| ApplyError::InvalidPlan(format!("write-tree failed: {error}")))
}

fn update_index(
    runner: &GitRunner,
    path: &str,
    entry: Option<&TreeEntry>,
    index: &OsString,
) -> Result<(), ApplyError> {
    let values = match entry {
        Some(entry) => vec![
            "update-index",
            "--add",
            "--cacheinfo",
            entry.mode.as_str(),
            entry.object.as_str(),
            path,
        ],
        None => vec!["update-index", "--force-remove", "--", path],
    };
    let command = GitCommand::write(GitRunner::command_args(&values))
        .with_environment(OsString::from("GIT_INDEX_FILE"), index.clone());
    runner.run_unlocked(command)?;
    Ok(())
}

fn commit_with_tree(
    runner: &GitRunner,
    commit: &CommitRewrite,
    tree: ObjectId,
    rewritten: &HashMap<ObjectId, ObjectId>,
) -> Result<ObjectId, ApplyError> {
    if commit.action == RewriteAction::Drop {
        return dropped_parent(commit, rewritten);
    }
    let command = commit_command(commit, tree, rewritten);
    let output = runner.run_unlocked(command)?;
    parse_id(&output.stdout)
        .map_err(|error| ApplyError::InvalidPlan(format!("commit-tree failed: {error}")))
}

fn dropped_parent(
    commit: &CommitRewrite,
    rewritten: &HashMap<ObjectId, ObjectId>,
) -> Result<ObjectId, ApplyError> {
    commit
        .first_parent
        .as_ref()
        .map(|parent| rewritten.get(parent).unwrap_or(parent).clone())
        .ok_or_else(|| {
            ApplyError::InvalidPlan("a dropped commit has no rewritten parent".to_string())
        })
}

fn commit_command(
    commit: &CommitRewrite,
    tree: ObjectId,
    rewritten: &HashMap<ObjectId, ObjectId>,
) -> GitCommand {
    let mut args = vec![
        "commit-tree".to_string(),
        tree.to_string(),
        "-F".to_string(),
        "-".to_string(),
    ];
    append_parent(&mut args, commit.first_parent.as_ref(), rewritten);
    for parent in &commit.additional_parents {
        args.push("-p".to_string());
        args.push(parent.to_string());
    }
    let command = GitCommand::write(args.into_iter().map(OsString::from).collect())
        .with_stdin(commit.metadata.message.clone());
    let command = set_signature(command, "GIT_AUTHOR", &commit.metadata.author);
    set_signature(command, "GIT_COMMITTER", &commit.metadata.committer)
}

fn append_parent(
    args: &mut Vec<String>,
    parent: Option<&ObjectId>,
    rewritten: &HashMap<ObjectId, ObjectId>,
) {
    let Some(parent) = parent else {
        return;
    };
    args.push("-p".to_string());
    args.push(rewritten.get(parent).unwrap_or(parent).to_string());
}

fn set_signature(
    command: GitCommand,
    prefix: &str,
    signature: &super::model::Signature,
) -> GitCommand {
    command
        .with_environment(
            OsString::from(format!("{prefix}_NAME")),
            OsString::from(signature.name.as_str()),
        )
        .with_environment(
            OsString::from(format!("{prefix}_EMAIL")),
            OsString::from(signature.email.as_str()),
        )
        .with_environment(
            OsString::from(format!("{prefix}_DATE")),
            OsString::from(signature.date.as_str()),
        )
}

pub(super) fn parse_id(bytes: &[u8]) -> Result<ObjectId, ApplyError> {
    let line = bytes
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    let end = line.iter().rposition(|byte| !matches!(byte, b' ' | b'\r'));
    let line = end.map(|index| &line[..=index]).unwrap_or_default();
    ObjectId::from_bytes(line)
        .map_err(|error| ApplyError::InvalidPlan(format!("{error}; Git output was {bytes:?}")))
}
