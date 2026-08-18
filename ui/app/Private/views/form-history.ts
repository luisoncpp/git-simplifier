import { esc } from "../dom.ts";
import { messageChanged, messageFor, newestFirst, selectedCommit } from "../draft/index.ts";
import { baseRef } from "../snapshot.ts";
import type { AppState } from "../types.ts";
import { commitMeta, commitOption, emptyState, fieldNote } from "./parts.ts";
import { pathChecklist } from "./path-list.ts";

export function uncommitForm(state: AppState): string {
  if (!baseRef(state)) return emptyState("Set a Base ref", "Uncommit compares the branch against Base, so Base has to be chosen first.");
  if (!state.paths.length) {
    return emptyState("Nothing differs from Base", `No path on this branch differs from ${baseRef(state)}.`);
  }
  return `<fieldset><legend>Paths to take out of the commits</legend>
    ${fieldNote(`Selected paths go back to their ${baseRef(state)} content in every rebuilt commit. Your files on disk are not touched.`)}
    ${pathChecklist(state)}
  </fieldset>`;
}

export function editMessageForm(state: AppState): string {
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

export function messageTools(state: AppState): string {
  if (!messageChanged(state)) return `<span class="hint">Edit the text above to enable the review.</span>`;
  return `<button class="link" data-event="reset-message">Reset to the original message</button>`;
}
