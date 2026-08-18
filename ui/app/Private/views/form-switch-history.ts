import { esc } from "../dom.ts";
import { newestFirst, selectedCommit } from "../draft/index.ts";
import { currentBranch, worktreeCounts } from "../snapshot.ts";
import type { AppState, EditableCommit } from "../types.ts";
import { commitMeta, commitOption, emptyState, fieldNote } from "./parts.ts";

export function historyForm(state: AppState): string {
  const branch = currentBranch(state);
  if (!branch) {
    return emptyState(
      "Return to present first",
      "History starts from the current branch. Switch back to that branch, then pick another commit.",
    );
  }
  const dirty = worktreeCounts(state).some(([label]) => label === "staged" || label === "unstaged");
  return `<fieldset><legend>Commit to check out</legend>
    ${fieldNote("The branch pointer stays at present. HEAD detaches at the chosen commit.")}
    ${modeToggle(state)}
    ${state.draft.historyMode === "until" ? untilField(state) : commitField(state)}
    ${dirty ? carryRow(state) : ""}
    ${carryNote(state, dirty, branch)}
  </fieldset>`;
}

function modeToggle(state: AppState): string {
  return `<div class="list-tools" role="radiogroup" aria-label="History target">
    ${modeOption(state, "commit", "By commit")}
    ${modeOption(state, "until", "By date and time")}
  </div>`;
}

function modeOption(state: AppState, value: "commit" | "until", label: string): string {
  const checked = state.draft.historyMode === value ? " checked" : "";
  return `<label class="check-row inline"><input type="radio" name="history-mode"
    data-event="select-history-mode" value="${value}"${checked} /> ${label}</label>`;
}

function commitField(state: AppState): string {
  if (!state.commits.length) {
    return emptyState("No earlier commit", "This branch has no first-parent commit before HEAD.");
  }
  const query = state.draft.historyFilter.trim().toLowerCase();
  const commits = newestFirst(state.commits).filter((entry) => matchesFilter(entry, query));
  const selected = selectedCommit(state);
  return `<label class="field">Filter
      <input type="search" data-event="history-filter" data-focus="history-filter"
        value="${esc(state.draft.historyFilter)}" placeholder="Subject or SHA" autocomplete="off" />
    </label>
    <label class="field">Commit
      <select data-event="select-commit" data-focus="commit" aria-label="Commit to check out">
        ${commits.map((entry) => commitOption(state, entry)).join("")}
      </select>
    </label>
    ${selected ? commitMeta(selected) : ""}`;
}

function untilField(state: AppState): string {
  return `<label class="field">Date and time
      <input type="datetime-local" data-event="history-until" data-focus="history-until"
        value="${esc(state.draft.historyUntil)}" aria-label="Date and time to check out" />
    </label>
    <p class="hint">Uses the newest first-parent commit at or before this local time.</p>`;
}

function carryRow(state: AppState): string {
  const checked = state.draft.historyCarryChanges ? "checked" : "";
  return `<label class="check-row inline"><input type="checkbox" data-event="toggle-history-carry"
      ${checked} /> Carry tracked changes to the target branch</label>`;
}

function carryNote(state: AppState, dirty: boolean, branch: string): string {
  if (!dirty) return "";
  if (state.draft.historyCarryChanges) {
    return `<p class="hint">Tracked changes on ${esc(branch)} will be stashed, then popped onto the checked-out commit.</p>`;
  }
  return `<p class="hint">Tracked changes on ${esc(branch)} will be saved before leaving present.</p>`;
}

function matchesFilter(commit: EditableCommit, query: string): boolean {
  if (!query) return true;
  const haystack = `${commit.short_id} ${commit.id} ${commit.subject}`.toLowerCase();
  return haystack.includes(query);
}
