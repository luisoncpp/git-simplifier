use std::collections::BTreeMap;

use crate::git::GitRunner;

use super::discover;
use super::errors::CleanupError;
use super::model::{
    CleanupBranchPlan, CleanupChoice, CleanupKind, CleanupPlan, CleanupRequest, KeptReason,
    KeptRemote, LocalDeletion, RemoteDeletion,
};
use super::review;
use super::state;

pub(super) fn create(
    runner: &GitRunner,
    request: CleanupRequest,
) -> Result<CleanupPlan, CleanupError> {
    state::ensure_no_operation(runner)?;
    if request.chosen.is_empty() {
        return Err(CleanupError::EmptySelection);
    }
    // Eligibility is recomputed here rather than trusted from the caller, so the
    // safety exclusions cannot be bypassed through the plan API.
    let discovery = discover::eligible(runner, &request.base)?;
    let index = by_reference(&discovery.choices);
    let mut branches = Vec::new();
    let mut kept_remotes = Vec::new();
    for reference in &request.chosen {
        let Some(choice) = index.get(reference.as_str()) else {
            return Err(CleanupError::NotEligible(reference.clone()));
        };
        let resolved = resolve(choice, request.include_remote_counterparts);
        branches.push(resolved.entry);
        kept_remotes.extend(resolved.kept);
    }
    let local_count = branches.iter().filter(|entry| entry.local.is_some()).count();
    let remote_count = branches
        .iter()
        .filter(|entry| entry.remote.is_some())
        .count();
    if local_count == 0 && remote_count == 0 {
        return Err(CleanupError::EmptySelection);
    }
    let draft = CleanupPlan {
        base: request.base,
        base_head: discovery.base_head,
        branches,
        kept_remotes,
        local_count,
        remote_count,
        commands: Vec::new(),
    };
    Ok(CleanupPlan {
        commands: review::commands(&draft),
        ..draft
    })
}

pub(super) fn verify_current(runner: &GitRunner, plan: &CleanupPlan) -> Result<(), CleanupError> {
    state::ensure_no_operation(runner)?;
    let discovery = discover::eligible(runner, &plan.base)?;
    ensure_unchanged(&discovery.choices, plan)
}

/// Pure. A Base that moved *forward* is deliberately not staleness: advancing
/// Base can only add merged branches, never unmerge one. A Base rewound by a
/// force push *can* unmerge one, and that is caught precisely, because the
/// eligible set was recomputed against the Base that is current now.
fn ensure_unchanged(choices: &[CleanupChoice], plan: &CleanupPlan) -> Result<(), CleanupError> {
    let index = by_reference(choices);
    for entry in &plan.branches {
        let Some(reference) = identity_of(entry) else {
            continue;
        };
        let Some(choice) = index.get(reference) else {
            return Err(CleanupError::StalePlan);
        };
        check_unchanged(choice, entry)?;
    }
    Ok(())
}

fn check_unchanged(choice: &CleanupChoice, entry: &CleanupBranchPlan) -> Result<(), CleanupError> {
    if let Some(local) = &entry.local {
        if choice.head != local.head {
            return Err(CleanupError::StalePlan);
        }
    }
    let Some(remote) = &entry.remote else {
        return Ok(());
    };
    let Some(counterpart) = &choice.remote else {
        return Err(CleanupError::StalePlan);
    };
    if counterpart.head != remote.head || !counterpart.merged {
        return Err(CleanupError::StalePlan);
    }
    Ok(())
}

/// The local ref identifies a local row; a remote-only row is identified by its
/// tracking ref, which is the reference its choice was listed under.
fn identity_of(entry: &CleanupBranchPlan) -> Option<&str> {
    entry
        .local
        .as_ref()
        .map(|local| local.reference.as_str())
        .or_else(|| entry.remote.as_ref().map(|remote| remote.tracking_ref.as_str()))
}

struct Resolved {
    entry: CleanupBranchPlan,
    kept: Option<KeptRemote>,
}

fn resolve(choice: &CleanupChoice, include_remotes: bool) -> Resolved {
    let local = match choice.kind {
        CleanupKind::Local => Some(LocalDeletion {
            reference: choice.reference.clone(),
            head: choice.head.clone(),
        }),
        CleanupKind::RemoteOnly => None,
    };
    let (remote, kept) = remote_for(choice, include_remotes);
    Resolved {
        entry: CleanupBranchPlan {
            branch: choice.branch.clone(),
            local,
            remote,
        },
        kept,
    }
}

fn remote_for(
    choice: &CleanupChoice,
    include_remotes: bool,
) -> (Option<RemoteDeletion>, Option<KeptRemote>) {
    let Some(counterpart) = &choice.remote else {
        // Only worth reporting when the user asked for remote cleanup; a branch
        // that simply has no upstream is otherwise not news.
        let kept = include_remotes.then(|| kept(choice, "", KeptReason::NoUpstream));
        return (None, kept);
    };
    if !include_remotes {
        let kept = kept(choice, &counterpart.tracking_ref, KeptReason::Disabled);
        return (None, Some(kept));
    }
    if !counterpart.merged {
        let kept = kept(choice, &counterpart.tracking_ref, KeptReason::NotMerged);
        return (None, Some(kept));
    }
    let deletion = RemoteDeletion {
        remote: counterpart.remote.clone(),
        remote_ref: counterpart.remote_ref.clone(),
        tracking_ref: counterpart.tracking_ref.clone(),
        head: counterpart.head.clone(),
    };
    (Some(deletion), None)
}

fn kept(choice: &CleanupChoice, tracking_ref: &str, reason: KeptReason) -> KeptRemote {
    KeptRemote {
        branch: choice.branch.clone(),
        tracking_ref: tracking_ref.to_string(),
        reason,
    }
}

fn by_reference(choices: &[CleanupChoice]) -> BTreeMap<&str, &CleanupChoice> {
    choices
        .iter()
        .map(|choice| (choice.reference.as_str(), choice))
        .collect()
}
