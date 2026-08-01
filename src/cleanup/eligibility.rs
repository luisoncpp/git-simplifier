use std::collections::{BTreeMap, BTreeSet};

use crate::rewrite::RefName;

use super::model::{
    CleanupChoice, CleanupExclusion, CleanupKind, ExclusionReason, RemoteCounterpart,
};
use super::refs::{LocalRef, RemoteRef};
use super::state::{self, HEADS_PREFIX, REMOTES_PREFIX};

pub(super) struct Inputs {
    pub base: RefName,
    pub identity: Option<String>,
    pub remote_names: Vec<String>,
    pub locals: Vec<LocalRef>,
    pub remotes: Vec<RemoteRef>,
    pub merged_remotes: BTreeSet<String>,
    pub saved_work: BTreeSet<String>,
}

pub(super) struct Classified {
    pub choices: Vec<CleanupChoice>,
    pub excluded: Vec<CleanupExclusion>,
}

/// Pure: every safety rule lives here, applied to the maximal set before any
/// display toggle can narrow it. No toggle disables an exclusion.
pub(super) fn classify(inputs: Inputs) -> Classified {
    let context = Context::new(&inputs);
    let mut choices = Vec::new();
    let mut excluded = Vec::new();
    let mut local_names = BTreeSet::new();
    for local in &inputs.locals {
        let Some(branch) = local.reference.strip_prefix(HEADS_PREFIX) else {
            continue;
        };
        local_names.insert(branch.to_string());
        if let Some(reason) = context.exclusion(local, branch) {
            excluded.push(CleanupExclusion {
                branch: branch.to_string(),
                reason,
            });
            continue;
        }
        choices.push(context.local_choice(local, branch));
    }
    choices.extend(context.remote_only(&inputs.remotes, &local_names));
    choices.sort_by(|left, right| left.reference.cmp(&right.reference));
    excluded.sort_by(|left, right| left.branch.cmp(&right.branch));
    Classified { choices, excluded }
}

struct Context<'a> {
    base: &'a RefName,
    base_branch: Option<String>,
    identity: Option<&'a str>,
    tracking: BTreeMap<&'a str, &'a RemoteRef>,
    merged_remotes: &'a BTreeSet<String>,
    saved_work: &'a BTreeSet<String>,
    remote_names: &'a [String],
}

impl<'a> Context<'a> {
    fn new(inputs: &'a Inputs) -> Self {
        Self {
            base: &inputs.base,
            base_branch: state::base_branch_name(&inputs.base, &inputs.remote_names),
            identity: inputs.identity.as_deref(),
            tracking: inputs
                .remotes
                .iter()
                .map(|entry| (entry.reference.as_str(), entry))
                .collect(),
            merged_remotes: &inputs.merged_remotes,
            saved_work: &inputs.saved_work,
            remote_names: &inputs.remote_names,
        }
    }

    fn exclusion(&self, local: &LocalRef, branch: &str) -> Option<ExclusionReason> {
        if local.is_head {
            return Some(ExclusionReason::CurrentBranch);
        }
        // `update-ref -d` does not refuse a checked-out branch the way
        // `git branch -d` does; it would leave that worktree's HEAD dangling.
        if !local.worktree.is_empty() {
            return Some(ExclusionReason::CheckedOutInWorktree);
        }
        if self.base_branch.as_deref() == Some(branch) || local.upstream == self.base.as_str() {
            return Some(ExclusionReason::BaseBranch);
        }
        if self.saved_work.contains(branch) {
            return Some(ExclusionReason::SavedWork);
        }
        None
    }

    fn local_choice(&self, local: &LocalRef, branch: &str) -> CleanupChoice {
        CleanupChoice {
            branch: branch.to_string(),
            reference: local.reference.clone(),
            head: local.head.clone(),
            kind: CleanupKind::Local,
            author_email: local.author_email.clone(),
            mine: self.mine(&local.author_email),
            protected: state::is_protected(branch),
            remote: self.counterpart(local),
        }
    }

    /// A counterpart is resolved from the configured upstream or not at all. It
    /// is never guessed as `origin/<name>`: choosing what to destroy on a server
    /// is not the place for the fallback `same_named_remote` uses.
    fn counterpart(&self, local: &LocalRef) -> Option<RemoteCounterpart> {
        if local.upstream_remote.is_empty() || local.upstream_remote == "." {
            return None;
        }
        let entry = self.tracking.get(local.upstream.as_str())?;
        Some(RemoteCounterpart {
            remote: local.upstream_remote.clone(),
            tracking_ref: local.upstream.clone(),
            remote_ref: local.upstream_remote_ref.clone(),
            head: entry.head.clone(),
            merged: self.merged_remotes.contains(&local.upstream),
        })
    }

    fn remote_only(&self, remotes: &[RemoteRef], locals: &BTreeSet<String>) -> Vec<CleanupChoice> {
        let mut choices = Vec::new();
        for entry in remotes {
            let Some((remote, branch)) = self.remote_candidate(entry, locals) else {
                continue;
            };
            choices.push(CleanupChoice {
                branch: branch.clone(),
                reference: entry.reference.clone(),
                head: entry.head.clone(),
                kind: CleanupKind::RemoteOnly,
                author_email: entry.author_email.clone(),
                mine: self.mine(&entry.author_email),
                protected: state::is_protected(&branch),
                remote: Some(RemoteCounterpart {
                    remote,
                    tracking_ref: entry.reference.clone(),
                    // No upstream config exists for a branch with no local, so
                    // the server name follows the standard `refs/heads/` mapping.
                    remote_ref: format!("{HEADS_PREFIX}{branch}"),
                    head: entry.head.clone(),
                    merged: true,
                }),
            });
        }
        choices
    }

    fn remote_candidate(
        &self,
        entry: &RemoteRef,
        locals: &BTreeSet<String>,
    ) -> Option<(String, String)> {
        if !entry.symref.is_empty() || entry.reference == self.base.as_str() {
            return None;
        }
        if entry.reference.ends_with("/HEAD") || !self.merged_remotes.contains(&entry.reference) {
            return None;
        }
        let short = entry.reference.strip_prefix(REMOTES_PREFIX)?;
        let (remote, branch) = state::split_remote(short, self.remote_names)?;
        if locals.contains(&branch) || self.base_branch.as_deref() == Some(branch.as_str()) {
            return None;
        }
        Some((remote, branch))
    }

    fn mine(&self, author_email: &str) -> bool {
        self.identity
            .map(|identity| identity.eq_ignore_ascii_case(author_email))
            .unwrap_or(false)
    }
}
