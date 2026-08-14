use super::super::data::{DeleteSavedWorkInput, OperationReview, PendingOperation};
use super::super::review_commands;
use super::{current_branch, head_of, saved_work, Prepared};
use crate::commands::state::AppState;

pub(super) fn restore(state: &AppState, id: String) -> Result<Prepared, String> {
    let branch = current_branch(state)?;
    let saved = saved_work(state, &branch)?;
    let head = head_of(state)?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "restore_saved_work".to_string(),
        title: format!("Restore Saved work for {branch}"),
        impact: vec![
            format!("Reapply snapshot {} to the working tree", saved.snapshot),
            format!("Delete {} once the apply succeeds", saved.reference),
        ],
        preserves: vec!["The snapshot ref if the apply fails".to_string()],
        warnings: vec![
            "Restoring consumes the snapshot; it is no longer listed afterwards.".to_string(),
        ],
        commands: review_commands::restore_saved_work(&saved),
        apply_label: "Restore Saved work".to_string(),
    };
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::Restore { id, head }),
        block: None,
    })
}

pub(super) fn delete(
    state: &AppState,
    id: String,
    input: DeleteSavedWorkInput,
) -> Result<Prepared, String> {
    let saved = saved_work(state, &input.branch)?;
    let head = head_of(state)?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "delete_saved_work".to_string(),
        title: format!("Delete Saved work for {}", saved.branch),
        impact: vec![format!("Remove {}", saved.reference)],
        preserves: vec![format!(
            "Snapshot {} stays in the object database until Git prunes it",
            saved.snapshot
        )],
        warnings: vec![
            "The app cannot list or restore this snapshot again after deletion.".to_string(),
        ],
        commands: review_commands::delete_saved_work(&saved),
        apply_label: "Delete Saved work".to_string(),
    };
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::Delete {
            id,
            branch: saved.branch,
            head,
        }),
        block: None,
    })
}
