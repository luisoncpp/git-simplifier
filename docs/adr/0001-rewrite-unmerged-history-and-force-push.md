# Rewrite unmerged history and offer the force-push

Both features (Edit message, Uncommit) work by rebuilding the commits in the **Editable range** — commits reachable from HEAD by first parents that are not yet on **Base**. Feature branches are routinely pushed before they are merged, so a rewrite normally leaves the branch diverged from its own remote-tracking ref. We decided the app both warns before the rewrite and offers to complete it with a force-push, rather than leaving the user with a branch their other git client refuses to push.

## Considered options

- **Refuse to rewrite pushed commits.** Rejected: the boundary that matters is "not on Base", not "not pushed" — a pushed feature branch is still the author's to rewrite. This option would make the default mode unavailable most of the time and silently degrade every operation to a Removal commit.
- **Warn and stop, leave pushing to the user's own client.** Rejected: smaller surface, but the app creates the divergence and then walks away from it. The user meets a non-fast-forward rejection they did not cause and force-pushes blindly.

## Consequences

- The app pushes, so it depends on the platform credential helper working (Git Credential Manager on Git for Windows). Credential entry is never implemented in-app.
- The push must pass the expected SHA explicitly: `--force-with-lease=refs/heads/<branch>:<sha the app observed>`. Bare `--force-with-lease` compares against the local remote-tracking ref, which any background fetch — IDEs and other git clients run them unprompted — silently advances, degrading the lease to a plain `--force`. The explicit form is the only version that preserves the safety argument for offering the button.
