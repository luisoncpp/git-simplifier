use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName, RepoPath, Signature};

use super::errors::InspectionError;
use super::model::{
    ChangedPath, EditableCommit, LocalBranchChoice, RemoteBaseChoice, RepositoryOverview,
    SubmoduleChoice, WorktreeSummary,
};

pub(crate) fn overview(runner: &GitRunner) -> Result<RepositoryOverview, InspectionError> {
    let branch = optional_text(runner, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    let base = optional_ref(runner, "githelper.base")?;
    let upstream = optional_ref_name(
        runner,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )?;
    let head = object_id(runner, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let git_version = runner.git_version().to_string();
    let worktree = worktree_summary(runner)?;
    let path = runner.repo_path().display().to_string();
    let name = runner
        .repo_path()
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("repository")
        .to_string();
    Ok(RepositoryOverview {
        path,
        name,
        branch,
        base,
        upstream,
        head,
        git_version,
        worktree,
        saved_work_count: 0,
        recovery_count: 0,
        sync_status: None,
        quick_switch_status: None,
    })
}

pub(crate) fn base_choices(runner: &GitRunner) -> Result<Vec<RemoteBaseChoice>, InspectionError> {
    let output = run(
        runner,
        &[
            "for-each-ref",
            "--format=%(refname)%00%(objectname)",
            "refs/remotes",
        ],
    )?;
    output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(parse_base_choice)
        .filter(|choice| {
            choice
                .as_ref()
                .map(|item| !item.display.ends_with("/HEAD"))
                .unwrap_or(true)
        })
        .collect()
}

pub(crate) fn changed_paths(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<ChangedPath>, InspectionError> {
    ensure_remote_base(base)?;
    let base = base.as_str().to_string();
    let output = run(
        runner,
        &[
            "diff",
            "--name-status",
            "-z",
            // Root-relative names are typed identifiers the planners match on,
            // so `diff.relative` or a subdirectory must not reshape them.
            "--no-relative",
            &format!("{base}...HEAD"),
            "--",
        ],
    )?;
    parse_changed_paths(&output)
}

pub(crate) fn editable_commits(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<EditableCommit>, InspectionError> {
    ensure_remote_base(base)?;
    let range = format!("{}..HEAD", base.as_str());
    let output = run(
        runner,
        &[
            "log",
            "--first-parent",
            "--reverse",
            "--format=%H%x00%h%x00%an%x00%ae%x00%aI%x00%s%x00%B%x1e",
            &range,
        ],
    )?;
    parse_commits(&output)
}

pub(crate) fn local_branches(
    runner: &GitRunner,
) -> Result<Vec<LocalBranchChoice>, InspectionError> {
    let current = optional_text(runner, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    let output = run(
        runner,
        &[
            "for-each-ref",
            "--format=%(refname:short)%00%(objectname)",
            "refs/heads",
        ],
    )?;
    let saved = saved_work_branches(runner)?;
    let mut branches = output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| parse_branch(line, current.as_deref(), &saved))
        .collect::<Result<Vec<_>, _>>()?;
    let local_names = branches
        .iter()
        .map(|branch| branch.name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    branches.extend(remote_only_branches(runner, &local_names, &saved)?);
    Ok(branches)
}

const WIP_PREFIX: &str = "refs/githelper/wip/";

fn saved_work_branches(runner: &GitRunner) -> Result<Vec<String>, InspectionError> {
    let saved = run(
        runner,
        &["for-each-ref", "--format=%(refname)", "refs/githelper/wip"],
    )?;
    let mut branches = Vec::new();
    for line in saved
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let reference = String::from_utf8(line.to_vec())
            .map_err(|_| InspectionError::Parse("Saved work branch was not UTF-8".to_string()))?;
        // `%(refname:strip=N)` counts every component including the branch name, so
        // `strip=4` over this four-component prefix returned an empty string for a
        // simple name and dropped the first segment of a slashed one.
        let Some(branch) = reference.strip_prefix(WIP_PREFIX) else {
            continue;
        };
        branches.push(branch.to_string());
    }
    Ok(branches)
}

fn remote_only_branches(
    runner: &GitRunner,
    local_names: &std::collections::BTreeSet<String>,
    saved: &[String],
) -> Result<Vec<LocalBranchChoice>, InspectionError> {
    let output = run(
        runner,
        &[
            "for-each-ref",
            "--format=%(refname)%00%(objectname)",
            "refs/remotes",
        ],
    )?;
    let mut choices = Vec::new();
    for line in output.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let fields = line.split(|byte| *byte == 0).collect::<Vec<_>>();
        if fields.len() != 2 {
            return Err(InspectionError::Parse(
                "remote branch output was malformed".to_string(),
            ));
        }
        let reference = String::from_utf8(fields[0].to_vec())
            .map_err(|_| InspectionError::Parse("remote branch was not UTF-8".to_string()))?;
        let Some(short) = reference.strip_prefix("refs/remotes/") else {
            continue;
        };
        if short.ends_with("/HEAD") {
            continue;
        }
        let Some((_, local_name)) = short.split_once('/') else {
            continue;
        };
        if local_name.is_empty() || local_names.contains(local_name) {
            continue;
        }
        choices.push(LocalBranchChoice {
            name: local_name.to_string(),
            head: object_id_bytes(fields[1])?,
            current: false,
            saved_work: saved.iter().any(|branch| branch == local_name),
            remote: Some(short.to_string()),
        });
    }
    Ok(choices)
}

pub(crate) fn submodules(runner: &GitRunner) -> Result<Vec<SubmoduleChoice>, InspectionError> {
    let output = run(runner, &["ls-tree", "-r", "-z", "--full-tree", "HEAD"])?;
    output
        .split(|byte| *byte == 0)
        .filter(|record| record.starts_with(b"160000 commit "))
        .map(|record| parse_submodule(runner, record))
        .collect()
}

pub(crate) fn set_base(runner: &GitRunner, base: RefName) -> Result<(), InspectionError> {
    ensure_remote_base(&base)?;
    run_write(
        runner,
        &[
            "config",
            "--local",
            "--replace-all",
            "githelper.base",
            base.as_str(),
        ],
    )?;
    Ok(())
}

fn worktree_summary(runner: &GitRunner) -> Result<WorktreeSummary, InspectionError> {
    let output = run(
        runner,
        &["status", "--porcelain=v2", "-z", "--ignore-submodules=none"],
    )?;
    let mut summary = WorktreeSummary {
        staged: 0,
        unstaged: 0,
        untracked: 0,
        conflicts: 0,
    };
    for record in output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        if record.starts_with(b"? ") {
            summary.untracked += 1;
            continue;
        }
        if record.starts_with(b"u ") {
            summary.conflicts += 1;
            continue;
        }
        let Some(x) = record.get(2) else { continue };
        let Some(y) = record.get(3) else { continue };
        if *x != b'.' {
            summary.staged += 1;
        }
        if *y != b'.' {
            summary.unstaged += 1;
        }
    }
    Ok(summary)
}

fn parse_base_choice(line: &[u8]) -> Result<RemoteBaseChoice, InspectionError> {
    let fields = line.split(|byte| *byte == 0).collect::<Vec<_>>();
    if fields.len() != 2 {
        return Err(InspectionError::Parse(
            "remote ref output was malformed".to_string(),
        ));
    }
    let reference = String::from_utf8(fields[0].to_vec())
        .map_err(|_| InspectionError::Parse("remote ref was not UTF-8".to_string()))?;
    let head = object_id_bytes(fields[1])?;
    let display = reference
        .strip_prefix("refs/remotes/")
        .unwrap_or(&reference)
        .to_string();
    Ok(RemoteBaseChoice {
        reference: RefName::new(reference).map_err(InspectionError::InvalidBase)?,
        display,
        head,
    })
}

fn parse_changed_paths(output: &[u8]) -> Result<Vec<ChangedPath>, InspectionError> {
    let fields = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut index = 0;
    while index < fields.len() {
        let status = String::from_utf8(fields[index].to_vec())
            .map_err(|_| InspectionError::Parse("path status was not UTF-8".to_string()))?;
        index += 1;
        let first = RepoPath::new(
            String::from_utf8(
                fields
                    .get(index)
                    .ok_or_else(|| InspectionError::Parse("path status had no path".to_string()))?
                    .to_vec(),
            )
            .map_err(|_| InspectionError::Parse("path was not UTF-8".to_string()))?,
        )
        .map_err(InspectionError::Parse)?;
        index += 1;
        let previous_path = status.starts_with('R').then(|| first.clone());
        let path = if previous_path.is_some() {
            let value = String::from_utf8(
                fields
                    .get(index)
                    .ok_or_else(|| {
                        InspectionError::Parse("rename status had no target".to_string())
                    })?
                    .to_vec(),
            )
            .map_err(|_| InspectionError::Parse("rename target was not UTF-8".to_string()))?;
            index += 1;
            RepoPath::new(value).map_err(InspectionError::Parse)?
        } else {
            first
        };
        result.push(ChangedPath {
            path,
            previous_path,
            status,
        });
    }
    Ok(result)
}

fn parse_commits(output: &[u8]) -> Result<Vec<EditableCommit>, InspectionError> {
    output
        .split(|byte| *byte == 0x1e)
        .filter(|record| !record.trim_ascii().is_empty())
        .map(|record| {
            let fields = record.split(|byte| *byte == 0).collect::<Vec<_>>();
            if fields.len() < 7 {
                return Err(InspectionError::Parse(
                    "commit output was malformed".to_string(),
                ));
            }
            let id = object_id_bytes(fields[0])?;
            let short_id = String::from_utf8(fields[1].to_vec())
                .map_err(|_| InspectionError::Parse("short commit id was not UTF-8".to_string()))?;
            let name = String::from_utf8(fields[2].to_vec())
                .map_err(|_| InspectionError::Parse("author was not UTF-8".to_string()))?;
            let email = String::from_utf8(fields[3].to_vec())
                .map_err(|_| InspectionError::Parse("author email was not UTF-8".to_string()))?;
            let date = String::from_utf8(fields[4].to_vec())
                .map_err(|_| InspectionError::Parse("author date was not UTF-8".to_string()))?;
            let subject = String::from_utf8(fields[5].to_vec())
                .map_err(|_| InspectionError::Parse("subject was not UTF-8".to_string()))?;
            let message = String::from_utf8_lossy(fields[6]).trim_end().to_string();
            Ok(EditableCommit {
                id,
                short_id,
                subject,
                message,
                author: Signature { name, email, date },
            })
        })
        .collect()
}

fn parse_branch(
    line: &[u8],
    current: Option<&str>,
    saved: &[String],
) -> Result<LocalBranchChoice, InspectionError> {
    let fields = line.split(|byte| *byte == 0).collect::<Vec<_>>();
    if fields.len() != 2 {
        return Err(InspectionError::Parse(
            "branch output was malformed".to_string(),
        ));
    }
    let name = String::from_utf8(fields[0].to_vec())
        .map_err(|_| InspectionError::Parse("branch was not UTF-8".to_string()))?;
    Ok(LocalBranchChoice {
        current: current == Some(name.as_str()),
        saved_work: saved.iter().any(|branch| branch == &name),
        name,
        head: object_id_bytes(fields[1])?,
        remote: None,
    })
}

fn parse_submodule(runner: &GitRunner, record: &[u8]) -> Result<SubmoduleChoice, InspectionError> {
    let tab = record
        .iter()
        .position(|byte| *byte == b'\t')
        .ok_or_else(|| InspectionError::Parse("submodule entry had no path".to_string()))?;
    let fields = record[..tab]
        .split(|byte| *byte == b' ')
        .collect::<Vec<_>>();
    if fields.len() != 3 {
        return Err(InspectionError::Parse(
            "submodule entry was malformed".to_string(),
        ));
    }
    let path = RepoPath::new(
        String::from_utf8(record[tab + 1..].to_vec())
            .map_err(|_| InspectionError::Parse("submodule path was not UTF-8".to_string()))?,
    )
    .map_err(InspectionError::Parse)?;
    let object = object_id_bytes(fields[2])?;
    let key = format!("submodule.{}.ignore", path.as_str());
    let excluded =
        optional_text(runner, &["config", "--local", "--get", &key])?.as_deref() == Some("all");
    Ok(SubmoduleChoice {
        path,
        object,
        excluded,
    })
}

pub(super) fn ensure_remote_base(base: &RefName) -> Result<(), InspectionError> {
    if base.as_str().starts_with("refs/remotes/") {
        return Ok(());
    }
    Err(InspectionError::InvalidBase(
        "Base must be a remote-tracking ref".to_string(),
    ))
}

fn optional_ref(runner: &GitRunner, key: &str) -> Result<Option<RefName>, InspectionError> {
    optional_text(runner, &["config", "--local", "--get", key])?
        .map(|value| RefName::new(value).map_err(InspectionError::InvalidBase))
        .transpose()
}

fn optional_ref_name(
    runner: &GitRunner,
    args: &[&str],
) -> Result<Option<RefName>, InspectionError> {
    optional_text(runner, args)?
        .map(|value| RefName::new(value).map_err(InspectionError::InvalidBase))
        .transpose()
}

fn object_id(runner: &GitRunner, args: &[&str]) -> Result<ObjectId, InspectionError> {
    object_id_bytes(run(runner, args)?.trim_ascii())
}
fn object_id_bytes(bytes: &[u8]) -> Result<ObjectId, InspectionError> {
    ObjectId::new(
        String::from_utf8(bytes.to_vec())
            .map_err(|_| InspectionError::Parse("object id was not UTF-8".to_string()))?
            .trim()
            .to_string(),
    )
    .map_err(InspectionError::Parse)
}
fn optional_text(runner: &GitRunner, args: &[&str]) -> Result<Option<String>, InspectionError> {
    match runner.run(GitCommand::read(
        args.iter().map(|value| OsString::from(*value)).collect(),
    )) {
        Ok(output) => Ok(Some(
            String::from_utf8(output.stdout)
                .map_err(|_| InspectionError::Parse("Git output was not UTF-8".to_string()))?
                .trim()
                .to_string(),
        )
        .filter(|value| !value.is_empty())),
        Err(_) => Ok(None),
    }
}
fn run(runner: &GitRunner, args: &[&str]) -> Result<Vec<u8>, InspectionError> {
    Ok(runner
        .run(GitCommand::read(
            args.iter().map(|value| OsString::from(*value)).collect(),
        ))?
        .stdout)
}
fn run_write(runner: &GitRunner, args: &[&str]) -> Result<Vec<u8>, InspectionError> {
    Ok(runner
        .run(GitCommand::write(
            args.iter().map(|value| OsString::from(*value)).collect(),
        ))?
        .stdout)
}
