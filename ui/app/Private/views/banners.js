import { esc } from "../dom.js";
import { syncPause } from "../snapshot.js";

export function banners(state) {
  return `${syncBanner(state)}${errorBanner(state)}${outcomeBanner(state)}`;
}

function syncBanner(state) {
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

function resumeHint(pause) {
  if (pause.retry) return "Nothing was written yet. Reconnect to the remote and retry the fetch.";
  return "Resolve the conflicted files in your editor, then resume. Your Saved work stays anchored until then.";
}

/// With no repository open the empty pane already carries the reason, so the
/// banner would only repeat it.
function errorBanner(state) {
  if (!state.error || !state.snapshot) return "";
  return `<div class="banner bad" role="alert">
    <div><strong>That did not run</strong><p>${esc(state.error)}</p></div>
    <button class="ghost small" data-event="dismiss-error" aria-label="Dismiss error">Dismiss</button>
  </div>`;
}

function outcomeBanner(state) {
  const outcome = state.outcome;
  if (!outcome) return "";
  const details = outcome.details.map((detail) => `<li>${esc(detail)}</li>`).join("");
  return `<div class="banner good">
    <div><strong>${esc(outcome.headline)}</strong><ul>${details}</ul></div>
    ${outcome.offer_force_push
      ? `<button class="primary small" data-event="force-push" ${state.busy ? "disabled" : ""}>Review force push</button>`
      : ""}
    <button class="ghost small" data-event="dismiss-outcome" aria-label="Dismiss result">Dismiss</button>
  </div>`;
}
