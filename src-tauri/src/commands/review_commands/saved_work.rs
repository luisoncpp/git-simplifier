use git_helper_core::SavedWork;

/// Restore consumes the Saved work ref: the snapshot is deleted once the apply
/// succeeds. The review has to show that deletion, not just the apply.
pub(crate) fn restore_saved_work(saved: &SavedWork) -> Vec<String> {
    vec![
        format!(
            "git -c submodule.recurse=false stash apply --index {}",
            saved.reference
        ),
        format!(
            "git -c submodule.recurse=false stash apply {}  # fallback when the index cannot be restored",
            saved.reference
        ),
        delete_ref(saved),
    ]
}

pub(crate) fn delete_saved_work(saved: &SavedWork) -> Vec<String> {
    vec![delete_ref(saved)]
}

fn delete_ref(saved: &SavedWork) -> String {
    format!(
        "git update-ref -d -m \"git-helper delete-saved-work\" {} {}",
        saved.reference, saved.snapshot
    )
}
