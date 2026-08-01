use std::ffi::OsString;

use super::model::{CleanupPlan, RemoteDeletion};

pub(super) struct RemotePush {
    pub remote: String,
    pub deletions: Vec<RemoteDeletion>,
}

/// One push per remote, so a cleanup is a single network round trip per server.
pub(super) fn group(plan: &CleanupPlan) -> Vec<RemotePush> {
    let mut pushes: Vec<RemotePush> = Vec::new();
    for deletion in plan.branches.iter().filter_map(|entry| entry.remote.as_ref()) {
        if let Some(push) = pushes.iter_mut().find(|push| push.remote == deletion.remote) {
            push.deletions.push(deletion.clone());
            continue;
        }
        pushes.push(RemotePush {
            remote: deletion.remote.clone(),
            deletions: vec![deletion.clone()],
        });
    }
    pushes
}

/// `--atomic` makes a remote lose every chosen branch or none, so a lost lease
/// leaves the oplog record wholly true instead of partly true.
pub(super) fn push_args(push: &RemotePush) -> Vec<OsString> {
    let mut values = vec![OsString::from("push"), OsString::from("--atomic")];
    values.extend(push.deletions.iter().map(|entry| OsString::from(lease(entry))));
    values.push(OsString::from(&push.remote));
    values.extend(
        push.deletions
            .iter()
            .map(|entry| OsString::from(format!(":{}", entry.remote_ref))),
    );
    values
}

pub(super) fn push_command(push: &RemotePush) -> String {
    let leases = join(push.deletions.iter().map(lease));
    let refspecs = join(push.deletions.iter().map(|entry| format!(":{}", entry.remote_ref)));
    format!("git push --atomic {leases} {} {refspecs}", push.remote)
}

/// The explicit `<ref>:<sha>` form is compared against the server's real value
/// at push time, not against a tracking ref that may predate someone else's
/// push. It is the only thing standing between a cleanup and unfetched work.
fn lease(deletion: &RemoteDeletion) -> String {
    format!(
        "--force-with-lease={}:{}",
        deletion.remote_ref, deletion.head
    )
}

fn join(values: impl Iterator<Item = String>) -> String {
    values.collect::<Vec<_>>().join(" ")
}
