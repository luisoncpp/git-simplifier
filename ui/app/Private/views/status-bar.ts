import { esc } from "../dom.ts";
import type { AppState } from "../types.ts";

/// The fetch bar outranks the generic busy spinner: the fetch runs inside the
/// busy window, and the user needs its progress and stop control instead.
export function statusBar(state: AppState): string {
  if (state.fetch.active) return fetchStatus(state);
  if (state.busy) return `<span class="spinner" aria-hidden="true"></span>Working…`;
  if (state.review) return "Review pending — nothing has been written yet.";
  return "Ready";
}

function fetchStatus(state: AppState): string {
  const { phase, done, total } = state.fetch;
  if (!phase || !total) {
    return `<span class="spinner" aria-hidden="true"></span><span>Fetching remotes…</span>${stopButton()}`;
  }
  const percent = Math.min(100, Math.round((done / total) * 100));
  return `<span class="fetch-progress" role="progressbar" aria-label="Fetch progress"
      aria-valuemin="0" aria-valuemax="100" aria-valuenow="${percent}"><span class="fetch-fill" style="width:${percent}%"></span></span>
    <span class="fetch-label">${esc(phase)} ${percent}%</span>${stopButton()}`;
}

/// Never disabled: the whole point is interrupting the busy window.
function stopButton(): string {
  return `<button class="fetch-stop" data-event="cancel-fetch" title="Stop the fetch" aria-label="Stop the fetch">&times;</button>`;
}
