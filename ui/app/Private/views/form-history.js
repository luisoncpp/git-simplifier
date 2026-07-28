import { esc } from "../dom.js";
import { commitValue, messageChanged, messageFor, newestFirst, pathValue, selectedCommit, visiblePaths } from "../draft.js";
import { baseRef } from "../snapshot.js";
import { emptyState, fieldNote, humanTime } from "./parts.js";

export function uncommitForm(state) {
  if (!baseRef(state)) return emptyState("Set a Base ref", "Uncommit compares the branch against Base, so Base has to be chosen first.");
  if (!state.paths.length) {
    return emptyState("Nothing differs from Base", `No path on this branch differs from ${baseRef(state)}.`);
  }
  const shown = visiblePaths(state);
  const selected = state.draft.selectedPaths.size;
  return `<fieldset><legend>Paths to take out of the commits</legend>
    ${fieldNote(`Selected paths go back to their ${baseRef(state)} content in every rebuilt commit. Your files on disk are not touched.`)}
    <div class="list-tools">
      <input type="search" placeholder="Filter ${state.paths.length} path(s)" data-event="path-filter"
        data-focus="path-filter" value="${esc(state.draft.pathFilter)}" aria-label="Filter changed paths" />
      <button class="link" data-event="select-paths" data-value="all">Select all ${shown.length === state.paths.length ? "" : "shown"}</button>
      <button class="link" data-event="select-paths" data-value="none" ${selected ? "" : "disabled"}>Clear</button>
      <span class="count" aria-live="polite">${selected} of ${state.paths.length} selected</span>
    </div>
    <div class="check-list" data-scroll="paths">${shown.map((entry) => pathRow(state, entry)).join("") || noMatches(state)}</div>
  </fieldset>`;
}

function pathRow(state, entry) {
  const path = pathValue(entry);
  const checked = state.draft.selectedPaths.has(path) ? " checked" : "";
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

export function editMessageForm(state) {
  if (!baseRef(state)) return emptyState("Set a Base ref", "Editable commits are the ones on this branch and not yet on Base.");
  if (!state.commits.length) {
    return emptyState("No editable commit", `Every commit on this branch is already on ${baseRef(state)}.`);
  }
  const commit = selectedCommit(state);
  return `<fieldset><legend>Commit message</legend>
    <label class="field">Commit
      <select data-event="select-commit" data-focus="commit" aria-label="Commit to edit">
        ${newestFirst(state.commits).map((entry) => commitOption(state, entry)).join("")}
      </select>
    </label>
    ${commit ? commitMeta(commit) : ""}
    <label class="field">New message
      <textarea data-event="commit-message" data-focus="commit-message" data-scroll="commit-message"
        rows="9" spellcheck="true" aria-label="New commit message">${esc(messageFor(state))}</textarea>
    </label>
    <div class="list-tools" id="message-tools">${messageTools(state)}</div>
  </fieldset>`;
}

export function messageTools(state) {
  if (!messageChanged(state)) return `<span class="hint">Edit the text above to enable the review.</span>`;
  return `<button class="link" data-event="reset-message">Reset to the original message</button>`;
}

function commitOption(state, commit) {
  const id = commitValue(commit);
  const selected = id === state.draft.commit ? " selected" : "";
  return `<option value="${esc(id)}"${selected}>${esc(commit.short_id)} — ${esc(commit.subject)}</option>`;
}

function commitMeta(commit) {
  return `<p class="meta"><code>${esc(commit.short_id)}</code> by ${esc(commit.author.name)}
    <span class="muted">· ${esc(humanTime(commit.author.date))}</span></p>`;
}
