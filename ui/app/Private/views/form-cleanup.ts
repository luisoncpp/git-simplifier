import { esc } from "../dom.ts";
import { cleanupChoices, cleanupSelection, cleanupTicked } from "../draft/index.ts";
import { baseRef } from "../snapshot.ts";
import type {
  AppState,
  CleanupBranch,
  CleanupExclusion,
  CleanupExclusionReason,
  Draft,
} from "../types.ts";
import { emptyState, fieldNote } from "./parts.ts";

export function cleanupForm(state: AppState): string {
  const base = baseRef(state);
  if (!base) {
    return emptyState("Set a Base ref", "Cleanup deletes branches that are fully merged into Base, so Base has to be chosen first.");
  }
  const discovery = state.cleanupBranches;
  if (discovery && !discovery.choices.length) {
    return emptyState("Nothing to clean up", `No branch is fully merged into ${base}.${notOfferedText(discovery.excluded)}`);
  }
  return `<fieldset><legend>Branches already merged into Base</legend>
    ${fieldNote(`Every branch listed is contained in ${base}, so deleting it loses no commits. A squash-merged branch is not listed: Git cannot prove it was integrated.`)}
    ${toggles(state)}
    ${checklist(state)}
    ${notOffered(state)}
  </fieldset>`;
}

function toggles(state: AppState): string {
  const draft = state.draft;
  return `${toggle("toggle-cleanup-only-mine", draft.cleanupOnlyMine, "Only branches created by me")}
    ${toggle("toggle-cleanup-remotes", draft.cleanupRemotes, "Also delete the branch on its remote")}
    ${toggle("toggle-cleanup-all-remote", draft.cleanupAllRemote, "Check all remote branches")}
    ${toggleHints(state)}`;
}

/// Authored markup, not user data, so these carry their own tags unescaped.
const toggleHints = (state: AppState): string => identityHint(state) + hiddenRemoteHint(state.draft);

const identityHint = (state: AppState): string =>
  state.draft.cleanupOnlyMine && !state.cleanupBranches?.identity
    ? `<p class="hint">Git has no <code>user.email</code> set, so no branch can be matched to you.</p>`
    : "";

const hiddenRemoteHint = (draft: Draft): string =>
  draft.cleanupAllRemote && !draft.cleanupRemotes
    ? `<p class="hint">Remote-only branches stay hidden while remote deletion is off — deleting one is a remote deletion.</p>`
    : "";

const toggle = (event: string, checked: boolean, label: string): string =>
  `<label class="check-row inline"><input type="checkbox" data-event="${event}"${checked ? " checked" : ""} /> ${label}</label>`;

function checklist(state: AppState): string {
  const shown = cleanupChoices(state);
  const total = state.cleanupBranches?.choices.length ?? 0;
  const selected = cleanupSelection(state).length;
  return `<div class="list-tools">
      <input type="search" placeholder="Filter ${total} branch(es)" data-event="cleanup-filter"
        data-focus="cleanup-filter" value="${esc(state.draft.cleanupFilter)}" aria-label="Filter merged branches" />
      <span class="count" aria-live="polite">${selected} of ${shown.length} ticked</span>
    </div>
    <div class="check-list" data-scroll="cleanup">${shown.map((choice) => row(state, choice)).join("") || noMatches(state)}</div>`;
}

function row(state: AppState, choice: CleanupBranch): string {
  const checked = cleanupTicked(state, choice) ? " checked" : "";
  return `<label class="check-row">
    <input type="checkbox" data-event="toggle-cleanup-branch" data-focus="cleanup:${esc(choice.reference)}"
      value="${esc(choice.reference)}"${checked} />
    <code>${esc(choice.branch)}</code>${badges(choice)}
    <span class="was">${esc(choice.author_email || "unknown author")}</span>
  </label>`;
}

function badges(choice: CleanupBranch): string {
  return [
    choice.protected ? badge("shared", "A well-known shared name, so it is never ticked for you") : "",
    choice.kind === "remote_only" ? badge("remote only", "No local branch; only the remote copy is deleted") : "",
    remoteAhead(choice),
  ].join("");
}

const remoteAhead = (choice: CleanupBranch): string =>
  choice.remote && !choice.remote.merged
    ? badge("remote ahead", "The remote copy has commits Base does not contain, so it is left alone")
    : "";

const badge = (label: string, title: string): string =>
  `<span class="status-tag" title="${esc(title)}">${esc(label)}</span>`;

const noMatches = (state: AppState): string =>
  `<p class="hint pad">No branch matches these filters.</p>${state.draft.cleanupOnlyMine ? `<p class="hint pad">Untick “Only branches created by me” to see branches written by someone else.</p>` : ""}`;

/// An absence is explained rather than hidden: a safety rule removing a branch
/// is exactly the thing a user would otherwise go hunting for.
function notOffered(state: AppState): string {
  const text = notOfferedText(state.cleanupBranches?.excluded ?? []);
  return text ? `<p class="hint">${esc(text.trim())}</p>` : "";
}

function notOfferedText(excluded: CleanupExclusion[]): string {
  if (!excluded.length) return "";
  const described = excluded.map((entry) => `${entry.branch} (${REASONS[entry.reason] ?? entry.reason})`);
  return ` Not offered: ${described.join(", ")}.`;
}

const REASONS: Record<CleanupExclusionReason, string> = {
  current_branch: "checked out here",
  checked_out_in_worktree: "checked out in another worktree",
  base_branch: "the branch Base tracks",
  saved_work: "has Saved work",
};
