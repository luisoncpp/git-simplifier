# Listed paths and pathspecs disagree unless both are pinned to the repository root

Two Git defaults decide what a path *string* means, and they can disagree with each other inside one operation:

- `git diff --name-only` / `--name-status` print **root-relative** names by default, but print **prefix-relative** names when `diff.relative` is set.
- The magic pathspec `:(literal)foo` is resolved against the **current prefix**, never the root.

So a repository opened below its Git root produces root-relative names that a `:(literal)` pathspec cannot match. Nothing errors: the diff is simply empty, and an operation that treats "no diff" as a user mistake reports something false — in our case *"the selected paths carry no changes over the Base"* for paths the picker had just listed as changed.

The failure is worse than a plain bug because it **splits across the plan/apply boundary**. Planning listed changed names with no pathspec, so it matched fine and produced a review. Only apply used a pathspec, so the operation failed *after* the user had read and approved a review that was correct. A subdirectory is not exotic — the repository dialog accepts any folder.

Rules for any path-based operation:

- Read names with `--no-relative`, so `diff.relative` cannot reshape identifiers that other code matches on.
- Write pathspecs as `:(top,literal)`, so `top` anchors at the root and `literal` still disables globbing.
- **Pin both ends or neither.** Fixing only the pathspec moves the breakage to repositories with `diff.relative` set; the two settings have to agree.
- Treat an empty diff as an internal inconsistency, not user error, when planning already proved those paths changed. A wrong-but-plausible message sends the user looking at their repository instead of at the tool.

Test it by opening the fixture repository at a subdirectory (`GitRepository::open` on `root/sub`) — running every test from the root hides this entire class of bug. `git rev-parse --show-prefix` is what Git is applying, and it is empty in every root-based test.

Related: [review-commands-must-be-derived-from-the-plan](./review-commands-must-be-derived-from-the-plan.md).
