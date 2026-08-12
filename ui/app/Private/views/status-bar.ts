import { esc } from "../dom.ts";
import type { AppState, FetchState } from "../types.ts";

/// The fetch bar outranks the generic busy spinner: the fetch runs inside the
/// busy window, and the user needs its progress and stop control instead.
export function statusBar(state: AppState): string {
  if (state.fetch.active) return fetchStatus(state);
  if (state.busy) return `<span class="spinner" aria-hidden="true"></span>Working…`;
  if (state.review) return "Review pending — nothing has been written yet.";
  return "Ready";
}

/// Progress ticks must not replace the stop button mid-press: a full shell
/// re-render between pointerdown and click drops the click. Patch in place.
export function patchFetchProgress(state: AppState): boolean {
  if (!canPatchFetch(state.fetch)) return false;
  const nodes = mountedFetchBar();
  if (!nodes) return false;
  applyFetchPatch(nodes, state.fetch);
  return true;
}

function canPatchFetch(fetch: FetchState): boolean {
  return fetch.active && Boolean(fetch.phase) && fetch.total > 0;
}

function applyFetchPatch(
  nodes: { fill: HTMLElement; bar: Element; label: Element },
  fetch: FetchState,
): void {
  const percent = Math.min(100, Math.round((fetch.done / fetch.total) * 100));
  nodes.fill.style.width = `${percent}%`;
  nodes.bar.setAttribute("aria-valuenow", String(percent));
  nodes.label.textContent = `${fetch.phase} ${percent}%`;
}

function mountedFetchBar(): { fill: HTMLElement; bar: Element; label: Element } | null {
  const root = globalThis.document?.querySelector("footer.status");
  if (!root?.querySelector('[data-event="cancel-fetch"]')) return null;
  return progressNodes(root);
}

function progressNodes(root: Element): { fill: HTMLElement; bar: Element; label: Element } | null {
  const fill = root.querySelector(".fetch-fill") as HTMLElement | null;
  const bar = root.querySelector(".fetch-progress");
  const label = root.querySelector(".fetch-label");
  if (!fill || !bar || !label) return null;
  return { fill, bar, label };
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
