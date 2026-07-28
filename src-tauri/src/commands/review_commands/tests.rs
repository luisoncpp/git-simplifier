use std::collections::BTreeMap;

use git_helper_core::{
    CommitMetadata, CommitRewrite, ObjectId, RefName, RepoPath, RewriteAction, RewriteOperation,
    RewritePlan, SavedWork, Signature, TreeEntry,
};

use super::{delete_saved_work, restore_saved_work, rewrite};

#[test]
fn uncommit_lists_the_temporary_index_sequence_instead_of_a_rebase() {
    let commands = rewrite(&uncommit_plan());

    assert!(
        !commands.iter().any(|command| command.contains("rebase")),
        "the rewrite engine never runs git rebase: {commands:?}"
    );
    assert!(commands.iter().any(|command| command
        == "GIT_INDEX_FILE=<temp-index> git update-index --add --cacheinfo 100644 2222222222222222222222222222222222222222 secret.txt"));
    assert_eq!(
        commands.last().map(String::as_str),
        Some("git reset --mixed <new-head> -- secret.txt")
    );
}

#[test]
fn the_rewrite_review_names_the_branch_and_the_expected_old_head() {
    let commands = rewrite(&uncommit_plan());

    assert!(commands.iter().any(|command| command
        == "git update-ref -m \"git-helper uncommit\" refs/heads/feature <new-head> 1111111111111111111111111111111111111111"));
}

#[test]
fn an_edit_message_review_omits_the_index_reset() {
    let mut plan = uncommit_plan();
    plan.operation = RewriteOperation::EditMessage;
    plan.selected_paths.clear();
    plan.base_entries.clear();

    let commands = rewrite(&plan);

    assert!(!commands.iter().any(|command| command.contains("reset")));
    assert!(!commands.iter().any(|command| command.contains("<base>")));
}

#[test]
fn saved_work_reviews_use_the_recorded_reference_and_snapshot() {
    let saved = SavedWork {
        branch: "feature".to_string(),
        reference: "refs/githelper/wip/feature".to_string(),
        snapshot: object_id("3333333333333333333333333333333333333333"),
    };

    assert_eq!(
        delete_saved_work(&saved),
        vec![
            "git update-ref -d -m \"git-helper delete-saved-work\" refs/githelper/wip/feature 3333333333333333333333333333333333333333",
        ]
    );
    let restore = restore_saved_work(&saved);
    assert_eq!(
        restore.first().map(String::as_str),
        Some("git -c submodule.recurse=false stash apply --index refs/githelper/wip/feature")
    );
    assert!(restore
        .last()
        .is_some_and(|command| command.starts_with("git update-ref -d")));
}

fn uncommit_plan() -> RewritePlan {
    let mut base_entries = BTreeMap::new();
    base_entries.insert(
        RepoPath::new("secret.txt".to_string()).unwrap(),
        Some(TreeEntry {
            mode: "100644".to_string(),
            kind: "blob".to_string(),
            object: object_id("2222222222222222222222222222222222222222"),
        }),
    );
    RewritePlan {
        operation: RewriteOperation::Uncommit,
        branch: RefName::new("refs/heads/feature".to_string()).unwrap(),
        base_ref: RefName::new("refs/remotes/origin/main".to_string()).unwrap(),
        source_head: object_id("1111111111111111111111111111111111111111"),
        base: object_id("4444444444444444444444444444444444444444"),
        selected_paths: vec![RepoPath::new("secret.txt".to_string()).unwrap()],
        base_entries,
        commits: vec![commit()],
        dropped_commits: Vec::new(),
    }
}

fn commit() -> CommitRewrite {
    let signature = Signature {
        name: "Dev".to_string(),
        email: "dev@example.com".to_string(),
        date: "1700000000 +0000".to_string(),
    };
    CommitRewrite {
        source: object_id("1111111111111111111111111111111111111111"),
        source_tree: object_id("5555555555555555555555555555555555555555"),
        first_parent: Some(object_id("6666666666666666666666666666666666666666")),
        additional_parents: Vec::new(),
        metadata: CommitMetadata {
            author: signature.clone(),
            committer: signature,
            message: b"subject".to_vec(),
        },
        action: RewriteAction::Rebuild,
    }
}

fn object_id(value: &str) -> ObjectId {
    ObjectId::new(value.to_string()).unwrap()
}
