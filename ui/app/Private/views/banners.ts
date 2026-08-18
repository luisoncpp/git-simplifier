import { esc } from "../dom.ts";
import { actionVerb } from "../review-mode.ts";
import { currentBranch, overviewOf, presentBranch, quickSwitchPause, savedWorkFor, syncPause } from "../snapshot.ts";
import type { SyncPause } from "../snapshot.ts";
import type { AppState, OperationOutcome } from "../types.ts";

export function banners(state: AppState): string {
  return `${syncBanner(state)}${quickSwitchBanner(state)}${untrackedBlockBanner(state)}${presentBanner(state)}${savedWorkBanner(state)}${warningBanner(state)}${errorBanner(state)}${outcomeBanner(state)}`;
}

function syncBanner(state: AppState): string {
  const pause = syncPause(state);
  if (!pause) return "";
  return `<div class="banner warn" role="alert">
    <div><strong>${esc(pause.label)}</strong>
      <p>${pause.resumable ? resumeHint(state, pause) : "This phase is not resumable from here."}</p></div>
    ${syncPrimaryAction(state, pause)}
  </div>`;
}

function syncPrimaryAction(state: AppState, pause: SyncPause): string {
  const disabled = state.busy ? "disabled" : "";
  const mergeOpen = overviewOf(state)?.merge_in_progress;
  if (pause.phase === "base-merge-conflict" && mergeOpen) {
    return `<button class="primary small" data-event="commit-merge" ${disabled}>${actionVerb(state.skipReview)} merge commit</button>`;
  }
  if (pause.resumable) {
    return `<button class="primary small" data-event="resume-sync" ${disabled}>${actionVerb(state.skipReview)} ${pause.retry ? "retry" : "resume"}</button>`;
  }
  return `<button class="ghost small" data-event="set-view" data-value="recovery">Inspect recovery</button>`;
}

function untrackedBlockBanner(state: AppState): string {
  const block = state.block;
  if (!block || block.kind !== "untracked_overwrite") return "";
  const disabled = state.busy ? "disabled" : "";
  const paths = block.paths.map((path) => `<li>${esc(path)}</li>`).join("");
  return `<div class="banner warn" role="alert">
    <div><strong>Untracked files would be overwritten</strong>
      <p>${esc(block.message)}</p><ul>${paths}</ul></div>
    <div class="banner-actions">
      <button class="primary small" data-event="switch-with-merge" ${disabled}>Switch with merge</button>
      <button class="ghost small" data-event="dismiss-block" ${disabled}>Dismiss</button>
    </div>
  </div>`;
}

function quickSwitchBanner(state: AppState): string {
  if (!quickSwitchPause(state)) return "";
  if (state.outcome?.offer_resolve_pull) return "";
  const disabled = state.busy ? "disabled" : "";
  return `<div class="banner warn" role="alert">
    <div><strong>Pull could not fast-forward</strong>
      <p>Choose how to update the branch you just switched onto.</p></div>
    <div class="banner-actions">
      <button class="primary small" data-event="resolve-pull-replace" ${disabled}>Replace with remote</button>
      <button class="ghost small" data-event="resolve-pull-merge" ${disabled}>Pull with merge</button>
      <button class="ghost small" data-event="resolve-pull-cancel" ${disabled}>Cancel pull</button>
    </div>
  </div>`;
}

function presentBanner(state: AppState): string {
  if (hidesPresentBanner(state)) return "";
  const present = presentBranch(state);
  return present ? presentMarkup(present, state.busy) : "";
}

function hidesPresentBanner(state: AppState): boolean {
  return Boolean(currentBranch(state) || state.outcome?.offer_switch_to_present);
}

function presentMarkup(present: string, busy: boolean): string {
  const disabled = busy ? "disabled" : "";
  return `<div class="banner warn" role="status">
    <div><strong>You are in History</strong>
      <p>Switch to ${esc(present)} to return to present.</p></div>
    <button class="primary small" data-event="switch-to" data-value="${esc(present)}" ${disabled}>Switch to ${esc(present)}</button>
  </div>`;
}

/// Notice and offer — never auto-restore. Hidden while a result already offers
/// restore, or while a pull decision must finish first.
function savedWorkBanner(state: AppState): string {
  const branch = currentBranch(state);
  if (!savedWorkFor(state, branch)) return "";
  if (state.dismissedSavedWorkBranch === branch) return "";
  if (state.outcome?.offer_restore_saved_work) return "";
  if (state.outcome?.offer_resolve_pull || quickSwitchPause(state)) return "";
  const disabled = state.busy ? "disabled" : "";
  return `<div class="banner warn" role="status">
    <div><strong>Saved work is waiting</strong>
      <p>This branch has a snapshot from a previous visit. Restore it when you are ready.</p></div>
    <div class="banner-actions">
      <button class="primary small" data-event="restore-saved" ${disabled}>${actionVerb(state.skipReview)} restore</button>
      <button class="ghost small" data-event="dismiss-saved-work-notice" ${disabled}>Dismiss</button>
    </div>
  </div>`;
}

function resumeHint(state: AppState, pause: SyncPause): string {
  if (pause.retry) return "Nothing was written yet. Reconnect to the remote and retry the fetch.";
  if (pause.phase === "base-merge-conflict" && overviewOf(state)?.merge_in_progress) {
    return "Resolve conflicts in your editor, then use Commit merge here so unrelated files cannot land in the PR.";
  }
  return "Resolve the conflicted files in your editor, then resume. Your Saved work stays anchored until then.";
}

/// A rewrite needs a force push; a branch the remote has never seen needs its
/// first push instead, which is a different operation with different risks.
function followUp(state: AppState, outcome: OperationOutcome): string {
  const disabled = state.busy ? "disabled" : "";
  if (outcome.offer_resolve_pull) {
    return `<div class="banner-actions">
      <button class="primary small" data-event="resolve-pull-replace" ${disabled}>Replace with remote</button>
      <button class="ghost small" data-event="resolve-pull-merge" ${disabled}>Pull with merge</button>
      <button class="ghost small" data-event="resolve-pull-cancel" ${disabled}>Cancel pull</button>
    </div>`;
  }
  if (outcome.offer_force_push) {
    return `<button class="primary small" data-event="force-push" ${disabled}>${actionVerb(state.skipReview)} force push</button>`;
  }
  if (outcome.offer_publish_branch) {
    return `<button class="primary small" data-event="publish-branch"
      data-value="${esc(outcome.offer_publish_branch)}" ${disabled}>${actionVerb(state.skipReview)} push of ${esc(outcome.offer_publish_branch)}</button>`;
  }
  if (outcome.offer_restore_saved_work) {
    return `<div class="banner-actions">
      <button class="primary small" data-event="restore-saved" ${disabled}>${actionVerb(state.skipReview)} restore</button>
      <button class="ghost small" data-event="dismiss-outcome" ${disabled}>Dismiss</button>
    </div>`;
  }
  return presentFollowUp(outcome, disabled) || resumeFollowUp(state, outcome, disabled);
}

function presentFollowUp(outcome: OperationOutcome, disabled: string): string {
  if (!outcome.offer_switch_to_present) return "";
  return `<button class="primary small" data-event="switch-to"
      data-value="${esc(outcome.offer_switch_to_present)}" ${disabled}>Switch to ${esc(outcome.offer_switch_to_present)}</button>`;
}

function resumeFollowUp(state: AppState, outcome: OperationOutcome, disabled: string): string {
  if (!outcome.offer_resume_sync) return "";
  return `<button class="primary small" data-event="resume-sync" ${disabled}>${actionVerb(state.skipReview)} resume sync</button>`;
}

/// With no repository open the empty pane already carries the reason, so the
/// banner would only repeat it.
function warningBanner(state: AppState): string {
  if (!state.warning || !state.snapshot) return "";
  return `<div class="banner warn" role="alert">
    <div><strong>Fetch failed</strong><p>${esc(state.warning)}</p></div>
    <button class="ghost small" data-event="dismiss-warning" aria-label="Dismiss warning">Dismiss</button>
  </div>`;
}

/// With no repository open the empty pane already carries the reason, so the
/// banner would only repeat it.
function errorBanner(state: AppState): string {
  if (!state.error || !state.snapshot) return "";
  return `<div class="banner bad" role="alert">
    <div><strong>That did not run</strong><p>${esc(state.error)}</p></div>
    <button class="ghost small" data-event="dismiss-error" aria-label="Dismiss error">Dismiss</button>
  </div>`;
}

function outcomeBanner(state: AppState): string {
  const outcome = state.outcome;
  if (!outcome) return "";
  const details = outcome.details.map((detail) => `<li>${esc(detail)}</li>`).join("");
  const tone = outcome.offer_resolve_pull || outcome.has_warning ? "warn" : "good";
  return `<div class="banner ${tone}">
    <div><strong>${esc(outcome.headline)}</strong><ul>${details}</ul></div>
    ${followUp(state, outcome)}
    ${outcome.offer_restore_saved_work ? "" : `<button class="ghost small" data-event="dismiss-outcome" aria-label="Dismiss result">Dismiss</button>`}
  </div>`;
}
