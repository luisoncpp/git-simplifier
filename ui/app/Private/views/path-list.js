import { esc } from "../dom.js";
import { pathSetFor, pathValue, visiblePaths } from "../draft.js";

/// Uncommit and Split branch pick from the same list — every path that differs
/// from Base — so the control is shared and only the selection set differs.
export function pathChecklist(state) {
  const shown = visiblePaths(state);
  const selected = pathSetFor(state).size;
  return `<div class="list-tools">
      <input type="search" placeholder="Filter ${state.paths.length} path(s)" data-event="path-filter"
        data-focus="path-filter" value="${esc(state.draft.pathFilter)}" aria-label="Filter changed paths" />
      <button class="link" data-event="select-paths" data-value="all">Select all ${shown.length === state.paths.length ? "" : "shown"}</button>
      <button class="link" data-event="select-paths" data-value="none" ${selected ? "" : "disabled"}>Clear</button>
      <span class="count" aria-live="polite">${selected} of ${state.paths.length} selected</span>
    </div>
    <div class="check-list" data-scroll="paths">${shown.map((entry) => pathRow(state, entry)).join("") || noMatches(state)}</div>`;
}

function pathRow(state, entry) {
  const path = pathValue(entry);
  const checked = pathSetFor(state).has(path) ? " checked" : "";
  const previous = entry.previous_path ? `<span class="was">was ${esc(entry.previous_path?.value ?? entry.previous_path)}</span>` : "";
  return `<label class="check-row">
    <input type="checkbox" data-event="toggle-path" data-focus="path:${esc(path)}" value="${esc(path)}"${checked} />
    <span class="status-tag" title="${esc(statusTitle(entry.status))}">${esc(entry.status)}</span>
    <code>${esc(path)}</code>${previous}
  </label>`;
}

const noMatches = (state) => `<p class="hint pad">No path matches “${esc(state.draft.pathFilter)}”.</p>`;

const STATUS_TITLES = { A: "added", M: "modified", D: "deleted", R: "renamed", C: "copied", T: "type changed" };
const statusTitle = (status) => STATUS_TITLES[String(status).charAt(0)] ?? String(status);
