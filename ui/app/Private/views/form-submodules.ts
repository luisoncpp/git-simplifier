import { esc } from "../dom.ts";
import { actionVerb, submitHint } from "../review-mode.ts";
import { pathValue } from "../draft/index.ts";
import { baseRef } from "../snapshot.ts";
import type { AppState, SubmoduleChoice } from "../types.ts";
import { emptyState, fieldNote } from "./parts.ts";
import {
  cleanupSubmitState,
  excludeSubmitState,
} from "../operations/submit.ts";

export function submodulesForm(state: AppState): string {
  if (!state.submodules.length) {
    return emptyState("No submodules", "This repository has no gitlink entries.");
  }
  return `<div class="submodules-columns">
    <div class="submodules-column">${excludeColumn(state)}</div>
    <div class="submodules-column">${cleanupColumn(state)}</div>
  </div>`;
}

function excludeColumn(state: AppState): string {
  const draft = state.draft;
  const chosen = state.submodules.find((entry) => pathValue(entry) === draft.submodule);
  const { disabled, reason } = excludeSubmitState(state);
  const verb = actionVerb(state.skipReview);
  return `<fieldset><legend>Exclude submodule</legend>
    ${fieldNote("Keep a submodule out of local changes: hide it from status and block commits.")}
    <label class="field">Submodule
      <select data-event="select-submodule" data-focus="submodule" aria-label="Submodule to exclude">
        ${state.submodules.map((entry) => submoduleOption(state, entry)).join("")}
      </select>
    </label>
    ${chosen?.excluded ? `<p class="hint">Already excluded. Applying again re-checks the config and hook.</p>` : ""}
    <label class="check-row inline"><input type="checkbox" data-event="toggle-install-hook"
      ${draft.installHook ? "checked" : ""} /> Install the <code>pre-commit</code> guard</label>
    <label class="check-row inline"><input type="checkbox" data-event="toggle-disable-recurse"
      ${draft.disableRecurse ? "checked" : ""} /> Also set <code>submodule.recurse=false</code></label>
    <div class="column-submit">
      <button class="primary" data-event="submit-exclude-submodule" ${disabled || state.busy ? "disabled" : ""}>${verb} exclusion</button>
      <p class="hint">${esc(submitHint(state.skipReview, reason))}</p>
    </div>
  </fieldset>`;
}

function cleanupColumn(state: AppState): string {
  const base = baseRef(state);
  if (!state.dirtySubmodules.length) {
    return emptyState(
      "No dirty submodules",
      "Nothing differs from Base and there is no local submodule dirt.",
    );
  }
  const draft = state.draft;
  const { disabled, reason } = cleanupSubmitState(state);
  const verb = actionVerb(state.skipReview);
  const baseNote = base
    ? ""
    : `<p class="hint">Set a Base ref to enable Uncommit from Base…HEAD.</p>`;
  return `<fieldset><legend>Cleanup dirty submodules</legend>
    ${fieldNote("Uncommit removes committed pointer updates; Revert aligns the gitlink and nested checkout to HEAD.")}
    ${baseNote}
    <div class="list-tools" role="group" aria-label="Cleanup actions">
      <label class="check-row inline"><input type="checkbox" data-event="toggle-cleanup-uncommit"
        ${draft.cleanupUncommit ? "checked" : ""}${base ? "" : " disabled"} /> Uncommit from Base…HEAD</label>
      <label class="check-row inline"><input type="checkbox" data-event="toggle-cleanup-revert"
        ${draft.cleanupRevert ? "checked" : ""} /> Revert</label>
    </div>
    ${dirtyChecklist(state)}
    <div class="column-submit">
      <button class="primary" data-event="submit-cleanup-submodules" ${disabled || state.busy ? "disabled" : ""}>${verb} cleanup</button>
      <p class="hint">${esc(submitHint(state.skipReview, reason))}</p>
    </div>
  </fieldset>`;
}

function dirtyChecklist(state: AppState): string {
  const draft = state.draft;
  const shown = state.dirtySubmodules;
  const selected = draft.cleanupSubmodulePaths.size;
  return `<div class="list-tools">
      <span class="count" aria-live="polite">${selected} of ${shown.length} selected</span>
    </div>
    <div class="check-list" data-scroll="cleanup-submodules">${shown.map((entry) => dirtyRow(state, entry)).join("")}</div>`;
}

function dirtyRow(state: AppState, entry: { path: string; local_dirty: boolean; in_editable_range: boolean }): string {
  const path = pathValue(entry);
  const checked = state.draft.cleanupSubmodulePaths.has(path) ? " checked" : "";
  const flags = [
    entry.local_dirty ? "local dirt" : "",
    entry.in_editable_range ? "Base…HEAD" : "",
  ].filter(Boolean).join(" · ");
  return `<label class="check-row" data-path-context="${esc(path)}">
    <input type="checkbox" data-event="toggle-cleanup-submodule" data-focus="cleanup-sub:${esc(path)}"
      value="${esc(path)}"${checked} />
    <code>${esc(path)}</code>
    <span class="was">${esc(flags)}</span>
  </label>`;
}

function submoduleOption(state: AppState, entry: SubmoduleChoice): string {
  const path = pathValue(entry);
  const selected = path === state.draft.submodule ? " selected" : "";
  const mark = entry.excluded ? " · already excluded" : "";
  return `<option value="${esc(path)}"${selected}>${esc(path)}${esc(mark)}</option>`;
}
