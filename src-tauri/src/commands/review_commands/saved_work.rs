use git_helper_core::SavedWork;

/// Restore consumes the Saved work ref once the snapshot is in the tree.
/// The review must show the deletion and the dirty-tree park fallback.
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
        "git -c submodule.recurse=false stash create  # only when local dirt would be overwritten"
            .to_string(),
        "git update-ref -m \"git-helper park-restore-dirt\" refs/githelper/restore-park/<op-id> <park> ''"
            .to_string(),
        "git reset --hard --no-recurse-submodules HEAD  # only when parking local dirt".to_string(),
        format!(
            "git -c submodule.recurse=false stash apply --index {}  # after park",
            saved.reference
        ),
        "git add -u --  # stage restored work before reapplying parked dirt".to_string(),
        "git -c submodule.recurse=false stash apply refs/githelper/restore-park/<op-id>".to_string(),
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
