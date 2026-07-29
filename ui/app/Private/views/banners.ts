import { esc } from "../dom.ts";
import { quickSwitchPause, syncPause } from "../snapshot.ts";
import type { SyncPause } from "../snapshot.ts";
import type { AppState, OperationOutcome } from "../types.ts";

export function banners(state: AppState): string {
  return `${syncBanner(state)}${quickSwitchBanner(state)}${errorBanner(state)}${outcomeBanner(state)}`;
}

function syncBanner(state: AppState): string {
  const pause = syncPause(state);
  if (!pause) return "";
  return `<div class="banner warn" role="alert">
    <div><strong>${esc(pause.label)}</strong>
      <p>${pause.resumable ? resumeHint(pause) : "This phase is not resumable from here."}</p></div>
    ${pause.resumable
      ? `<button class="primary small" data-event="resume-sync" ${state.busy ? "disabled" : ""}>Review ${pause.retry ? "retry" : "resume"}</button>`
      : `<button class="ghost small" data-event="set-view" data-value="recovery">Inspect recovery</button>`}
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

function resumeHint(pause: SyncPause): string {
  if (pause.retry) return "Nothing was written yet. Reconnect to the remote and retry the fetch.";
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
    return `<button class="primary small" data-event="force-push" ${disabled}>Review force push</button>`;
  }
  if (outcome.offer_publish_branch) {
    return `<button class="primary small" data-event="publish-branch"
      data-value="${esc(outcome.offer_publish_branch)}" ${disabled}>Review push of ${esc(outcome.offer_publish_branch)}</button>`;
  }
  return "";
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
  const tone = outcome.offer_resolve_pull ? "warn" : "good";
  return `<div class="banner ${tone}">
    <div><strong>${esc(outcome.headline)}</strong><ul>${details}</ul></div>
    ${followUp(state, outcome)}
    <button class="ghost small" data-event="dismiss-outcome" aria-label="Dismiss result">Dismiss</button>
  </div>`;
}
