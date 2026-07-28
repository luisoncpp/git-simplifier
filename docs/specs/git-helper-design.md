# Git Helper — Design Document

**Status:** Draft — revised 2026-07-27 after a design grilling session
**Stack:** Tauri (Rust core + web UI), shelling out to the real `git` CLI
**Platform:** Windows first, Git for Windows as the binary

**Terminology** is defined in [`/CONTEXT.md`](../../CONTEXT.md) — *Base*, *Editable range*, *Uncommit*, *Rewrite*, *Removal commit*, *Edit message*, *Saved work*, *Excluded submodule*. Decisions with lasting consequences are in [`docs/adr/`](../adr/).

**Main features**, in the user's priority order: Uncommit files · Edit message · Excluded submodules · Sync with base · Quick branch switch. Split branch (§5), the repo health panel (§6), merge drivers (§8) and overlay branches (§9) remain in scope as later work.

---

## 1. Purpose

A desktop tool that turns a handful of routine-but-awkward git operations into single, safe actions. It is not a general git client and does not aim to replace one — it is designed to run *alongside* existing clients (Fork, GitKraken, the terminal, IDE integrations) on the same repository.

### Target user

Everyone on the team, not just people who don't know git. The operations covered are ones that experienced users also get wrong or find tedious. This has a direct design consequence: **the app shows the equivalent git commands for every operation it performs.** Teaching is a side effect, not a mode.

### Non-goals

- Replacing a full git client (no commit graph browsing, blame, log search, etc. in v1)
- Authoring merge logic (see §8)
- Owning repository state or requiring exclusive access

---

## 2. Core principles

### 2.1 Stateless operator on a normal repo

The app never becomes the source of truth. Concretely:

- No cached branch/ref state across invocations; read from the repo every time
- Watch `.git` for external changes and refresh
- Always detect and gracefully report "the repo is mid-rebase/merge/cherry-pick" — the user may have started it in another tool
- Any state the app *does* need lives in the repo under its own ref namespace (§2.3), not in an external database

### 2.2 Recording is load-bearing

Every write operation must:

1. Record the prior position of every ref it will touch
2. Create backup refs for any content that would otherwise be unreachable
3. Append an entry to an operation log

This is what makes destructive git safe to hand to a GUI. It is not a v2 feature — retrofitting it means auditing every operation a second time.

There is **no one-click Undo button** (see `docs/adr/0002-no-undo-button-recording-and-recovery-panel.md`). `update-ref` restores refs without touching the working tree, so a button labelled "Undo" would routinely leave a worktree that doesn't match what the user had. Instead the recording feeds a **recovery panel**: past operations, the refs each moved, and a copy-pasteable command to get back. Recording carries the safety value; the button carried the ambiguity.

### 2.3 Ref namespace

All app-owned refs live under a private namespace so they never appear in `git branch` or clutter other clients' UIs:

```
refs/githelper/backup/<timestamp>-<operation>
refs/githelper/wip/<branch>
refs/githelper/overlay/<name>        # future, see §9
```

The operation log lives at `.git/githelper/oplog.json` (inside `.git`, so it is never committed and never appears in `status`).

**Which operations need a backup ref.** Only the ones that create content nothing else references. `git stash create` (§4.4, §4.5) produces a snapshot commit referenced by nothing — without a backup ref it is unreachable the instant it exists and `gc` may delete it. Edit message and Uncommit (§4.1, §4.2) only move a branch pointer, and `update-ref` writes the prior position to the branch reflog, so for those the recovery panel simply shows `git reset --hard <prior sha>` and no app-owned ref is created.

### 2.4 Minimize worktree churn

This is a Unity codebase. Every file written to the working tree risks triggering an editor reimport, which can cost minutes. Wherever an operation can be done via the index or a hidden worktree instead of writing to the user's checkout, it must be.

---

## 3. Architecture

### 3.1 Layers

```
┌─────────────────────────────────────────┐
│  UI (web, in Tauri webview)             │
│  - operation panels, repo health, undo  │
├─────────────────────────────────────────┤
│  Tauri command boundary (typed)         │
├─────────────────────────────────────────┤
│  Operations layer (Rust)                │
│  - one module per feature               │
│  - each: preflight → backup → execute   │
├─────────────────────────────────────────┤
│  Git layer (Rust)                       │
│  - run_git() primitive                  │
│  - porcelain parsers                    │
│  - repo lock                            │
├─────────────────────────────────────────┤
│  git CLI subprocess                     │
└─────────────────────────────────────────┘
```

### 3.2 The `run_git` primitive

Everything is built on one function:

```rust
fn run_git(repo: &Path, args: &[&str], opts: GitOpts) -> Result<GitOutput, GitError>
```

`args` is an **argv array and is never passed through a shell.** Every bash snippet in this document is pseudocode: `$(...)`, pipes and `sed` do not exist at the call site and must be expressed as separate `run_git` calls in Rust. The one exception is hook *files* — git runs hooks through the `sh.exe` bundled with Git for Windows, so hooks may be POSIX `sh`.

`GitError` carries the exit code and **raw stderr**. When an operation fails, the UI shows git's actual message — a friendly paraphrase loses information the user needs.

### 3.3 Environment — never let git go interactive

A subprocess with no TTY that decides to prompt will hang forever. Every invocation sets:

| Variable | Value | Reason |
|---|---|---|
| `GIT_TERMINAL_PROMPT` | `0` | Fail instead of prompting for credentials |
| `GIT_EDITOR` | `true` | Never open an editor |
| `GIT_PAGER` | `cat` | Never page |
| `LC_ALL` | `C` | Stable, parseable output |
| `GIT_OPTIONAL_LOCKS` | `0` | *Read-only calls only* — prevents background `status` from taking `index.lock` and colliding with the user's terminal |

### 3.4 Locating the git binary

**v1 targets Windows, with Git for Windows as the binary.** macOS is a later target; the notes below are kept for when it arrives.

Do not assume `PATH`. GUI apps launched from Finder on macOS get a minimal environment, and `git` there is frequently the `xcode-select` shim that pops a "install command line tools" dialog.

- Resolve the binary once at startup (login-shell PATH probe, then known locations)
- Verify with `git --version`; enforce a minimum version (§3.7)
- Expose an override in settings
- Hard-fail at startup with a clear message rather than failing mysteriously later

### 3.5 Credentials

`sync` fetches, so this must work on day one.

- Rely on the OS credential helper where configured
- For SSH keys with a passphrase and no running agent, detect the failure and instruct the user rather than trying to prompt
- Never implement credential entry in-app; delegate to the platform

### 3.6 Concurrency

A single mutex per repository serializes all write operations. Read operations may run concurrently but use `GIT_OPTIONAL_LOCKS=0`. Racing on `index.lock` produces failures nobody can diagnose.

### 3.7 Parsing

- `git status --porcelain=v2 -z`
- `-z` (NUL-separated) on every command that lists paths
- `git log --format=` with explicit, unambiguous separators
- Never parse human-readable output
- Minimum git version: **2.38** (for `rebase --update-refs`, used later by §9; also gives `zdiff3`, `restore`, and modern `worktree`)

---

## 4. Features — MVP

Ordered by implementation sequence. The first three are deliberately low-stakes: they exist to build out `run_git`, the backup/undo plumbing, and the UI shell against operations that cannot lose work.

### 4.1 Uncommit files

**Intent:** a file was committed by accident. Make `HEAD` match **Base** for that file, while leaving the local file exactly as it is.

Terms — **Base**, **Editable range**, **Rewrite**, **Removal commit** — are defined in `/CONTEXT.md`.

**The user picks files, never commits.** The picker lists everything in `git diff --name-status <base>...HEAD`. Asking which commit introduced a file is asking a question the user can't answer in order to reach an outcome they've already described.

#### Two modes

**Rewrite (default).** Every commit in the **Editable range** is rebuilt so that it records base's version of the chosen paths — as if the file had never been staged once. Commits left empty are dropped, and the UI says which.

**Removal commit (fallback).** A new commit on top takes the paths back to base's content. Used when the paths were introduced by commits outside the editable range (already on base, or arrived through a merged side branch).

#### Mechanism — a first-parent chain rewrite, no rebase

`git rebase` is the obvious tool and the wrong one: it checks files out, which in a Unity repo means a reimport. Because the tree edit is *identical at every commit* (set path → base's blob, or remove it), the whole chain can be rebuilt with plumbing in a temporary index:

1. Fetch base (§ below) and compute the editable range
2. Walk it oldest → newest, following **first parents only**
3. For each commit: read its tree, apply the path edit, `commit-tree` with the previously rebuilt commit as first parent — preserving message, author and any additional parents verbatim
4. Skip commits whose rebuilt tree equals their parent's
5. One `update-ref` on the branch makes the whole thing visible

Zero worktree writes, no conflicts possible by construction, and **atomic**: a crash before step 5 leaves the branch untouched and the operation invisible to other clients.

**First-parent only** is a safety rule, not an optimisation. Commits that arrived through a merged side branch are a teammate's, exist under those SHAs in their repo, and are never rewritten — even though they are on the branch and not on base. If the file came in through such a merge, the tree edit is applied from the merge commit forward and the result is still correct.

#### After the rewrite

- The working-tree file is **never touched**
- The **index is reset to HEAD for the affected paths** — otherwise the file stays staged and the next commit silently repeats the accident
- Files that exist in base end up as ordinary unstaged modifications; files that don't end up **untracked**

For the newly-untracked files, offer an ignore rule with the exact lines shown first. Default to `.git/info/exclude` (personal, not committed); `.gitignore` is available as an explicit choice, because adding personal noise to a committed file is a strange thing for a cleanup button to do silently.

#### Base freshness

Base is a remote ref, so **fetch it immediately before computing the editable range, and again before the force-push.** A stale base means commits already merged to base look editable — the app would rewrite commits the whole team has and then offer to force-push them. One refspec, no tags. Offline: proceed, but show base's age prominently.

#### Afterwards

Rewriting diverges the branch from its own remote. The app warns before and offers the force-push after — see `docs/adr/0001-rewrite-unmerged-history-and-force-push.md`, including why the lease must carry an explicit SHA.

**Preflight:** no merge/rebase/cherry-pick/bisect in progress; not detached HEAD; base configured; HEAD and base share a merge base; editable range non-empty. **Uncommitted and staged changes do NOT block** — the operation writes nothing to the worktree, so requiring a clean tree would impose exactly the churn §2.4 exists to avoid. Warn (don't block) if another worktree has this branch checked out.

---

### 4.2 Edit message

**Intent:** fix the wording of a commit on this branch.

**Reach: any commit in the Editable range**, not just HEAD — same boundary as Uncommit. Typos are usually spotted while reviewing your own branch, by which point the bad commit is no longer the last one. The UI is a commit list with an editable message per row.

**This is the same engine as §4.1.** Both are the first-parent chain rewrite described there; Edit message changes the *message* and leaves the tree alone, Uncommit changes the *tree* and leaves the message alone. Range computation, fetch-before, atomicity, preflight, reflog recovery and the force-push offer are all shared. Build it once.

The name matters: **never call this "amend".** `git commit --amend` also sweeps staged changes into the commit, which is exactly the bug this operation must not have, and reusing git's word for a button that must not behave like git's command invites it back.

**Do not use `git commit --amend -m`.** The plain form sweeps any staged changes into the amended commit — a nasty surprise for a button labelled "change the description." `--only` is supposed to prevent this but the flag semantics are subtle enough not to rely on.

Build the commit explicitly instead:

```bash
GIT_AUTHOR_NAME=$(git log -1 --format=%an) \
GIT_AUTHOR_EMAIL=$(git log -1 --format=%ae) \
GIT_AUTHOR_DATE=$(git log -1 --format=%aD) \
NEW=$(git commit-tree "HEAD^{tree}" \
        $(git rev-parse HEAD^@ | sed 's/^/-p /') \
        -m "new description")
git update-ref HEAD "$NEW" "reword"
```

This reuses the commit's tree verbatim, preserves all parents (so it works on merge commits), preserves original authorship, and touches neither index nor worktree. It also produces a clean reflog entry for recovery.

The snippet above is **pseudocode** (§3.2): there is no shell, so `$(git log -1 …)` and the `sed` building `-p` flags become separate `run_git` calls — `rev-parse <commit>^@`, split lines, interleave `-p` — and the author fields are passed as environment variables on the `commit-tree` invocation.

**Guardrails**
- Commits already on **Base** are outside the editable range and cannot be edited at all.
- Editing a commit that has been pushed diverges the branch from its remote; warn, then offer the force-push (ADR 0001).
- Preflight is §4.1's, unchanged — including that a dirty worktree does not block.

---

### 4.3 Excluded submodules

**Intent, in the user's words:** *"don't touch this submodule — don't include it in the branch, nor in local changes."*

This is **exclusion**, a standing rule, not a cleanup operation. The observed problem is that a submodule shows as modified permanently (its pointer drifts on its own, because submodules don't follow a branch switch) and then gets swept into a commit by `git add -A`.

**Exclusion is personal**, stored repo-locally. Everything that implements it — `.git/config`, `.git/hooks/` — is uncommittable by design, and a team-wide freeze would need a sanctioned way to advance the pointer that nobody has designed. Without one, the submodule reference silently rots.

Three independent parts, all required:

| Part | Mechanism |
|---|---|
| Stop the noise | `submodule.<path>.ignore = all` in repo-local config |
| Stop the accident | `pre-commit` hook checking `git diff --cached --name-only --ignore-submodules=none -- <paths>` |
| Clean what's already committed | **§4.1 Uncommit**, with the submodule path selected |

**No pin manifest.** The doc previously proposed one holding `pinned` and `local` SHAs. Both are cheaply derivable from the repo — `pinned` is whatever base records, `local` is `git submodule status` — so caching them creates app-owned state that can go stale, which §2.1 forbids. The one case worth persisting, a submodule's local SHA across a branch switch, belongs next to that branch's **Saved work** ref, not in a global manifest.

**A gitlink is just a tree entry**, so cleanup needs no new mechanism: §4.1's chain rewrite sets the submodule path back to base's gitlink SHA across the editable range, resets the index for it, and never touches the submodule's working directory. The local checkout stays exactly where it is and git reports it as an unstaged change the user simply isn't committing — which is the stated intent.

Say plainly in the UI that this resets to **base's current pointer**, which teammates keep moving — "clean" does not mean "back to what I had yesterday".

**`ignore = all` is a display setting, not a guard** — see `docs/lessons-learned/submodule-ignore-all-hides-changes-from-diff-cached.md`. Verified: `git add -A` stages the pointer regardless, and the naive `pre-commit` check returns empty under `ignore = all` and silently allows the commit. The `--ignore-submodules=none` flag is mandatory, in the hook and anywhere else the app inspects submodule state.

**Never overwrite an existing hook.** Read `.git/hooks/pre-commit` first; if something is there, show what would be appended and let the user decide.

Also offer, as opt-in with the exact config lines shown before writing: `submodule.recurse = false`, and a staging pathspec that excludes submodule paths (`git add -u -- ':!Submodules/'`, confirmed to leave the pointer unstaged).

---

### 4.4 Quick branch switch

**Intent:** move between branches without losing in-progress work.

**Do not use the shared stash stack.** `git stash push`/`pop` is a global LIFO that the user's terminal and other clients also write to; if they stash manually between our push and our pop, indices shift and we restore the wrong snapshot.

Use per-branch WIP refs instead:

```bash
# leaving a branch
S=$(git stash create)                              # snapshot commit; worktree untouched
git update-ref refs/githelper/wip/<branch> "$S"
git reset --hard                                   # only after the ref is written
git checkout <target>

# returning to it
git stash apply refs/githelper/wip/<branch>
git update-ref -d refs/githelper/wip/<branch>
```

Per-branch refs are unambiguous, survive a crash, are invisible to other clients, and let the branch list show **"this branch has saved work"** — an affordance no other client offers.

The snapshot is called **Saved work** (see `/CONTEXT.md`) and covers **tracked changes only**.

**Untracked files stay where they are.** `git stash create` doesn't include them, and that is now a deliberate choice rather than a gap. `git stash push -u` would copy them into the object store and then *delete them from disk* — in a Unity repo that's a batch of assets and `.meta` files vanishing and reappearing, i.e. a reimport, and it leaves those files existing in exactly one place: a commit referenced only by our ref. `git reset --hard` does not remove untracked files, so they survive the whole switch untouched. Offer "also stash untracked files" as an explicit opt-in, never as the default.

The one case that needs handling is a file that is untracked here and **tracked on the target branch** — checkout would clobber it and git refuses. Detect that in preflight and refuse first, naming the files.

**Restoring: notice and offer, never automatic.** The app is not the only client, so a branch left through the app is frequently returned to via a terminal or IDE — which would strand the saved work in a ref the user doesn't know exists. On repo open and on any `.git`-watcher refresh, if the current branch has saved work, show a banner offering to restore it. Automatic restoration is wrong twice over: it writes to the worktree without the user asking (a Unity reimport merely from opening the app), and it's actively harmful when the user has since made new changes on that branch.

Saved-work refs are **listed, never auto-expired** — branch, age, file count, explicit delete. Auto-expiring the only copy of someone's uncommitted work to keep a ref list tidy is not a trade worth making.

**Other known gaps to handle explicitly**
- Staged/unstaged split: `apply --index` restores it but fails more often. Attempt it, fall back to a plain apply, and say which happened.
- **Submodules do not follow a checkout.** After switching, submodule worktrees are silently still at the old branch's commits. Record submodule state alongside the WIP ref and restore it on return — this is why §4.3 comes first.

**Worktrees as an alternative.** For "let me look at another branch for two minutes," `git worktree add` is strictly better: nothing moves in the current checkout, so Unity never reimports. Offer both:

- *Switch* — same checkout, WIP ref preserved
- *Open in new worktree* — separate directory, shared object store, no churn

A worktree manager with sane defaults (shared LFS, auto-cleanup, open-in-editor) may end up more valuable than the switch itself.

---

### 4.5 Sync with base

**Intent:** bring the current branch up to date with the latest base, without losing uncommitted work.

The real UX win is not automating `fetch → stash → merge → pop`. It is **separating the two conflict classes**, which vanilla git conflates: a conflict from the merge is *branch vs. branch*; a conflict from reapplying WIP is *your uncommitted work vs. the new base*. Different problems, different guidance.

**Sequence**

1. `git fetch origin develop` — into the remote-tracking ref only.
   **The app does not write to a local branch named after base.** Base is a remote ref (`/CONTEXT.md`), the merge in step 3 uses `origin/develop`, and a local `develop` is irrelevant to the operation. Silently moving a branch the user may have checked out in another worktree is exactly the state-ownership §2.1 forbids, and dropping the write removes the "local develop has diverged" failure path from sync entirely. A stale local `develop` is cosmetic. If keeping it current is wanted, it belongs in a separate explicit action, not as a hidden side effect of pressing Sync.
2. `S=$(git stash create)` — snapshot without touching the worktree or the stash list. Save as a backup ref.
3. Clean the tree, then `git merge --no-edit origin/develop`.
   **Any conflict here is labelled a base-merge conflict.**
4. Reapply `S` (`git stash apply`, or `cherry-pick -n`).
   **Any conflict here is labelled a WIP-reapply conflict.**

**Notes**
- `git merge --autostash` (2.27+) does steps 2–4 in one shot, but loses the conflict-class distinction and the durable backup ref. Do it manually.
- Untracked files aren't in `stash create`. The real risk is the merge wanting to write a path that exists untracked — detect this in preflight and refuse with a clear message before starting.
- Merge is the correct default (predictable, non-rewriting, low anxiety). Rebase can be an alternative mode later, not in MVP.

**Sync is resumable, and must recognise its own unfinished work.**

A conflict at step 3 is the normal case, not the exception. Resolution is delegated to the user's existing `mergetool`, which means they leave the app, resolve in Rider or VS Code, and commit the merge there — while their uncommitted work sits unapplied in a ref nothing knows about. That is the stranding failure from §4.4, except here the app caused it by stopping halfway through its own operation.

- Before starting, the oplog records that a sync is in flight, which saved-work ref belongs to it, and the phase reached
- A fetch failure remains recorded and can be retried safely while the recorded branch and HEAD are unchanged
- On app open or `.git`-watcher refresh, an in-flight sync is detected and surfaced: *"A sync was interrupted. The base merge is resolved — your uncommitted work is saved and hasn't been put back yet. Reapply it?"*
- The offer stands regardless of where the conflict was resolved, per §2.1
- Conflict-class labelling survives the round trip: work reapplied in phase two is still a **WIP-reapply conflict**

**Future:** sync can update *several* branches at once. With `fetch origin develop:develop` plus a hidden worktree, non-checked-out branches can be merged without touching the user's working copy. Not MVP — but don't design it out.

---

## 5. Split branch (post-MVP)

**Intent:** create a new branch containing only some of the current branch's changes.

Decide the semantics explicitly, because "split" hides two different operations:

| Mode | Effect on original branch |
|---|---|
| **Copy** | untouched |
| **Move** | must be rewritten, or receive a revert commit |

**MVP for this feature: copy, by file path.**

Implement in a **hidden worktree** so the user's checkout never churns:

```bash
git worktree add --detach <tmp> <base>
git diff <base>...<current> -- <paths> | git -C <tmp> apply
git -C <tmp> add -A && git -C <tmp> commit -m "…"
git branch <new> <tmp-HEAD>
git worktree remove <tmp>
```

Hunk-level splitting requires a staging UI (lazygit / GitHub-style). Defer.

**Unity note:** `.meta` files must always move with their asset. Enforce this in the file picker rather than trusting the user to notice.

---

## 6. Repo health panel

Runs on repo open. Cheap to build, prevents a whole category of confusing failures, and is where the merge-driver work later plugs in.

Checks:

- git version meets minimum
- **merge drivers referenced in `.gitattributes` but not defined in config** (see §8 — this is a correctness prerequisite, not a nicety)
- configured driver executables actually resolve on disk
- git-LFS installed, if `.gitattributes` references it
- submodules initialized
- merge / rebase / cherry-pick / bisect in progress
- Unity-specific: `UnityYAMLMerge` configured; `core.fsmonitor` and `core.untrackedCache` enabled (large win on big projects)

Each check that fails offers a one-click fix, always showing the exact config lines before writing, and always writing to `.git/config` (repo-local) rather than `--global`.

---

## 7. Operation log and undo

### Log entry shape

```jsonc
{
  "id": "2026-07-26T14:03:11Z-sync",
  "operation": "sync",
  "started": "…", "finished": "…",
  "refs_before": { "refs/heads/feature-x": "a1b2…", "HEAD": "a1b2…" },
  "refs_after":  { "refs/heads/feature-x": "f9e8…" },
  "snapshots":   { "wip": "refs/githelper/backup/…-wip" },
  "commands": [ "git fetch origin develop:develop", "…" ],
  "reversible": true
}
```

- `commands` is also what the UI displays as "here's what this did" — the teaching surface from §1.
- Undo restores `refs_before` via `update-ref` and reapplies snapshots.
- Some operations are not cleanly reversible once the user has edited files afterwards; mark those and warn rather than pretending.
- Backup refs are garbage-collected by the app after N days, never by `gc` (they're refs, so they keep objects alive — this is intentional).

---

## 8. Merge drivers — scope boundary

Git's merge is line-based and content-agnostic. Custom merge drivers are the escape hatch, and they are configured per path pattern via `.gitattributes` + `merge.<name>.driver` in config.

**The critical asymmetry:** `.gitattributes` is committed and shared; **driver definitions live in local config and cannot be committed** (deliberately — otherwise cloning a repo would let it run arbitrary commands during a merge). In practice this means most teammates never configure them, the attribute rules silently fall back to plain text merge, and nobody notices until a scene file is mangled.

### In scope for this app

- **Detection** (MVP, required): `sync` performs merges. If the repo declares `*.unity merge=unityyaml` and the user's config doesn't define it, this app will silently text-merge scene files. Detection is not optional.
- **One-click installation** of a curated set: UnityYAMLMerge, `ours` (`driver = true`), `union`, LFS.
- **Validation** that driver executables resolve (the UnityYAMLMerge path moves with every Unity version — this breaks constantly).
- **A team-shareable `githelper.json`** committed to the repo listing recommended driver definitions, which the app offers to apply with a human confirming. This solves the "definitions can't be committed" problem without weakening git's security model, and is probably the single highest-value item in this section.
- **Config defaults**: `merge.conflictStyle = zdiff3` (strictly better than the default), situational `-X ignore-space-change` / `-X find-renames`.

### Out of scope — separate project

Writing the actual merge logic. A driver is a pure `(base, ours, theirs) → merged` function with no UI and a completely different failure mode: the app failing means an annoyed user retries; a driver failing means **silently corrupted commits discovered days later**. It needs adversarial testing against thousands of real conflicts, and it should ship as a standalone binary usable from plain CLI git.

**The seam:** *this app configures and orchestrates merges; it does not decide how two versions of a file combine.*

### If structure-aware merging is pursued later

Register one dispatcher driver that routes by path to per-format handlers:

```ini
[merge "smart"]
    name = structure-aware merge
    driver = githelper merge-driver --path=%P --base=%O --ours=%A --theirs=%B --marker-size=%L
    recursive = binary
```

One config entry for the whole team; handlers added without touching anyone's `.gitattributes`.

Non-negotiable handler properties:

1. **Fall back, never fail** — unparseable input, unknown extension, or a handler panic all fall through to plain three-way merge
2. **Conservative** — auto-resolve only genuinely disjoint changes; otherwise emit markers *scoped to the changed node*, which is already better than a whole-hunk conflict
3. **Verify output** — re-parse the merged result before writing; if it doesn't parse, fall back
4. **Deterministic and fast** — runs per-file during merge, rebase, cherry-pick, and stash apply

Check **Mergiraf** (tree-sitter based, conservative, multi-language) before writing anything; detecting and configuring it may deliver most of the benefit for a few days of work. Reserve custom handlers for formats nobody else covers.

Also relevant: `merge=union` already solves append-heavy files (changelogs, index files) with zero code. Reach for it first.

---

## 9. Overlay branches (v2 — the differentiating feature)

The idea: `develop → A → B`, where `A` is what gets merged upstream and `B` additionally carries local-only changes (debug logging, test scenes, local server URLs).

This is a **stacked-diff / patch-queue** model with substantial prior art: Jujutsu (`jj`), git-branchless, Graphite, Sapling, StGit. Jujutsu is especially relevant — its working copy *is* a commit (stash disappears as a concept) and descendants auto-rebase when an ancestor changes, which is exactly the invariant this needs. **Seriously evaluate using `jj` on a git-backed repo as the engine** rather than reimplementing stack maintenance.

If built on plain git:

- Mark local-only commits with a **commit trailer** (`X-Local-Only: debug`) rather than external state — the marking survives rebases and any tool can read it
- Hidden refs under `refs/githelper/overlay/…`
- Every operation touching `A` re-stacks `B` via `git rebase --onto … --update-refs`
- **A `pre-push` hook that refuses to push any commit carrying the trailer** — this is the safety net that makes the scheme non-scary
- Generalize to *N* named, individually toggleable overlays ("debug logging," "test scene," "local server"). For game development this is the genuinely killer feature.

---

## 10. Testing

- **Fixture-repo harness before the *first* feature**, not the second. §4 originally justified its ordering by the first features being low-stakes "operations that cannot lose work" — no longer true: Uncommit is now a history rewrite ending in a force-push offer. The engine is shared by Uncommit, Edit message and submodule cleanup, so testing it once pays for three features. Fixtures required before Uncommit ships:
  - a straight chain in `base..HEAD`
  - a chain with base merged in (two-parent rebuild)
  - a chain with a teammate's branch merged in — the first-parent rule; a bug here silently rewrites someone else's commits and then force-pushes them
  - a commit containing only the uncommitted file (drop-empty)
  - a file touched in several commits in the range
  - a submodule gitlink as the uncommitted path
- **Chaos tests:** kill the process mid-operation at each step and verify the repo is recoverable and the undo log is consistent.
- **Cross-client tests:** perform an operation, then verify another client (plain CLI) sees a sane repo — no leaked locks, no unexpected refs in `git branch`.
- **Historical replay** (if merge drivers are ever built): replay every merge commit in the repo's history and compare driver output against the recorded resolution. This is the eval harness that makes a driver trustworthy; build it before the driver.

---

## 11. Roadmap

| Phase | Contents |
|---|---|
| **0 — Foundations** | `run_git`, repo lock, porcelain parsers, backup refs, oplog + recovery panel, **fixture harness**, repo health panel |
| **1 — The rewrite engine** | Uncommit files · Edit message · Excluded submodules — one chain-rewrite engine, three entry points |
| **2 — Movement** | Quick branch switch (WIP refs) · Worktree manager |
| **3 — The big one** | Sync with base, with conflict-class separation |
| **4** | Split branch (copy, by path, via hidden worktree) |
| **5** | Merge driver detection → curated install → shareable `githelper.json` |
| **6** | Overlay branches / stacked local-only commits |

Phase 5's *detection* half is a hard dependency of Phase 3 and must ship with it; only the installation UI is deferrable.

---

## 12. Open questions

Still open:

1. **Multi-branch sync** — worth the hidden-worktree complexity, or is current-branch-only enough?
2. **Jujutsu as the engine** for overlays — decide before writing any stack-maintenance code, not after.
3. **Windows specifics** — Windows is now the v1 target (§3.4), but path length limits and line endings (`core.autocrlf`) still need explicit scoping. Credential helpers are settled: the platform helper, never in-app entry (ADR 0001).

Resolved:

- **Base branch discovery** — the app asks on first repo open and stores the answer repo-locally. Never inferred silently: a wrong base makes every operation quietly wrong. Exactly one base per repository; no per-branch override.
- **Conflict resolution UI** — out of scope; delegate to the user's `mergetool`. This is what makes §4.5's resumability mandatory rather than nice-to-have.
- **Undo** — dropped as a button, kept as recording plus a recovery panel (ADR 0002).
