import { esc } from "../dom.ts";
import { pathSetFor, pathValue } from "../draft.ts";
import { baseRef, currentBranch } from "../snapshot.ts";
import type { AppState } from "../types.ts";
import { emptyState, fieldNote } from "./parts.ts";
import { pathChecklist } from "./path-list.ts";

export function splitBranchForm(state: AppState): string {
  if (!baseRef(state)) {
    return emptyState("Set a Base ref", "Split branch copies what this branch changed over Base, so Base has to be chosen first.");
  }
  if (!state.paths.length) {
    return emptyState("Nothing differs from Base", `No path on this branch differs from ${baseRef(state)}, so there is nothing to split out.`);
  }
  const draft = state.draft;
  return `<fieldset><legend>Changes to copy onto a new branch</legend>
    ${fieldNote(`The new branch starts where this branch left ${baseRef(state)} and carries only the selected changes. ${esc(currentBranch(state) || "This branch")} keeps them too.`)}
    <label class="field">New branch name
      <input type="text" data-event="split-branch-name" data-focus="split-branch-name" spellcheck="false"
        placeholder="e.g. hero-art" value="${esc(draft.newBranch)}" aria-label="Name of the branch to create" />
    </label>
    ${pathChecklist(state)}
    <label class="field">Commit message <span class="muted">(optional)</span>
      <textarea data-event="split-message" data-focus="split-message" rows="3" spellcheck="true"
        aria-label="Message for the commit on the new branch">${esc(draft.splitMessage)}</textarea>
    </label>
    ${messageNote(state)}
    ${metaNote(state)}
  </fieldset>`;
}

const messageNote = (state: AppState): string =>
  state.draft.splitMessage.trim()
    ? ""
    : `<p class="hint">Left empty, the commit is named after the branch and the number of files.</p>`;

/// The planner adds these whether or not they were ticked, so the picker says so
/// rather than letting an unexpected file show up in the review.
function metaNote(state: AppState): string {
  const selected = pathSetFor(state);
  const partners = [...selected].filter((path) => hasPartner(state, path));
  if (!partners.length) return "";
  return `<p class="hint">Unity <code>.meta</code> files travel with their asset, so any changed partner of the selected paths is added automatically.</p>`;
}

function hasPartner(state: AppState, path: string): boolean {
  const partner = path.endsWith(".meta") ? path.slice(0, -".meta".length) : `${path}.meta`;
  return state.paths.some((entry) => pathValue(entry) === partner && !state.draft.splitPaths.has(partner));
}
