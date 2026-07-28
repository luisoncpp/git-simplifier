use crate::git::GitRunner;

use super::errors::SplitError;
use super::model::{SplitBranchPlan, SplitBranchRequest};
use super::state;
use super::{paths, review};

pub(crate) fn create(
    runner: &GitRunner,
    request: SplitBranchRequest,
) -> Result<SplitBranchPlan, SplitError> {
    let source_branch = state::read_branch(runner)?;
    let source_head = state::read_id(runner, "HEAD")?;
    let base = state::read_id(runner, request.base.as_str())?;
    let merge_base = state::merge_base(runner, &base, &source_head)?;
    let new_branch_ref = reserve_branch(runner, &request.new_branch)?;
    let selection = paths::select(
        runner,
        request.paths,
        paths::Range {
            from: &merge_base,
            to: &source_head,
        },
    )?;
    let message_is_derived = request.message.is_none();
    let message = request
        .message
        .unwrap_or_else(|| derive_message(&source_branch, selection.carried.len()));
    let draft = SplitBranchPlan {
        source_branch,
        source_head,
        base_ref: request.base,
        base,
        merge_base,
        new_branch: request.new_branch,
        new_branch_ref,
        selected_paths: selection.selected,
        changed_paths: selection.carried,
        companion_paths: selection.companions,
        message,
        message_is_derived,
        commands: Vec::new(),
    };
    Ok(SplitBranchPlan {
        commands: review::commands(&draft),
        ..draft
    })
}

pub(crate) fn verify_current(runner: &GitRunner, plan: &SplitBranchPlan) -> Result<(), SplitError> {
    if state::read_branch(runner)? != plan.source_branch {
        return Err(SplitError::StalePlan);
    }
    if state::read_id(runner, "HEAD")? != plan.source_head {
        return Err(SplitError::StalePlan);
    }
    if state::read_id(runner, plan.base_ref.as_str())? != plan.base {
        return Err(SplitError::StalePlan);
    }
    if state::branch_exists(runner, &plan.new_branch_ref)? {
        return Err(SplitError::ExistingBranch(plan.new_branch.clone()));
    }
    Ok(())
}

fn reserve_branch(runner: &GitRunner, branch: &str) -> Result<String, SplitError> {
    state::validate_branch_name(runner, branch)?;
    let reference = state::branch_ref(branch);
    if state::branch_exists(runner, &reference)? {
        return Err(SplitError::ExistingBranch(branch.to_string()));
    }
    Ok(reference)
}

fn derive_message(branch: &str, count: usize) -> Vec<u8> {
    let unit = if count == 1 { "file" } else { "files" };
    format!("Split {count} {unit} from {branch}\n").into_bytes()
}
