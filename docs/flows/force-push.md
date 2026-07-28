# Force-push Flow

## Trigger

The caller offers to publish a rewritten current branch after Uncommit or Edit message.

## Sequence

1. `plan_force_push` reads the symbolic current branch and its local upstream configuration.
2. The plan resolves the upstream remote-tracking ref and records its current SHA as the lease expectation.
3. Applying the plan rechecks the branch, HEAD, upstream configuration, and expected SHA. Any difference rejects the stale plan before a push.
4. The operation records the local refs and exact explicit-lease command in the oplog.
5. Git runs `push --force-with-lease=<remote-branch>:<observed-sha> <remote> HEAD:<remote-branch>`.
6. A successful push finishes the oplog entry; a failed push leaves the in-flight entry available to the recovery surface.

## Reads

- Current symbolic branch and HEAD.
- `branch.<name>.remote` and `branch.<name>.merge` from local config.
- The corresponding `refs/remotes/<remote>/<branch>` SHA.

## Writes and side effects

- Pushes the current branch to its configured remote branch.
- Records the observed local refs and command under `.git/githelper/oplog.json`.
- Uses no shell interpolation; remote and refspec values are separate Git argv entries.

## Common failure modes

- Detached HEAD, no upstream, or a local-only upstream cannot be planned.
- A changed HEAD, upstream configuration, or remote-tracking SHA rejects application as a stale plan.
- A remote-side lease mismatch is reported by Git and does not overwrite the remote branch.
