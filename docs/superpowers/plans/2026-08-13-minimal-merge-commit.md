# Minimal Merge Commit Implementation Plan

> **Status:** Implemented (not committed). See [commit-merge flow](../flows/commit-merge.md).

**Goal:** One button finishes an in-progress merge so `Base…HEAD` does not gain files that were not in the PR before the conflict.

**Delivered:** `src/merge_commit/` deep module, **Commit merge** rail tab, Sync banner integration, `offer_resume_sync` follow-up, tests in `tests/merge_commit_fixtures.rs`, `test/ui-commit-merge.test.mjs`, and flow/architecture docs.

**Core algorithm:** Temporary index `read-tree --empty` + `read-tree -m <merge-base> HEAD MERGE_HEAD`; overlay stage-0 resolutions from the real index for conflicted paths only; `write-tree`; apply with `read-tree <tree>` + `commit --no-edit`. Staged paths outside the merge tree stay uncommitted.
