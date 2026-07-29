import { esc } from "../dom.ts";
import { pathValue } from "../draft.ts";
import { baseRef, currentBranch, savedWorkFor, upstreamRef, worktreeCounts } from "../snapshot.ts";
import type { AppState, LocalBranch, SubmoduleChoice } from "../types.ts";
import { emptyState, fieldNote } from "./parts.ts";

export function excludeForm(state: AppState): string {
  if (!state.submodules.length) {
    return emptyState("No submodules", "This repository has no gitlink entries, so there is nothing to exclude.");
  }
  const draft = state.draft;
  const chosen = state.submodules.find((entry) => pathValue(entry) === draft.submodule);
  return `<fieldset><legend>Submodule to keep out of local changes</legend>
    ${fieldNote("Exclusion is a standing rule: the pointer stops showing up in status and is blocked at commit time.")}
    <label class="field">Submodule
      <select data-event="select-submodule" data-focus="submodule" aria-label="Submodule to exclude">
        ${state.submodules.map((entry) => submoduleOption(state, entry)).join("")}
      </select>
    </label>
    ${chosen?.excluded ? `<p class="hint">This submodule is already excluded. Applying again re-checks the config and the hook.</p>` : ""}
    <label class="check-row inline"><input type="checkbox" data-event="toggle-install-hook"
      ${draft.installHook ? "checked" : ""} /> Install the <code>pre-commit</code> guard</label>
    <label class="check-row inline"><input type="checkbox" data-event="toggle-disable-recurse"
      ${draft.disableRecurse ? "checked" : ""} /> Also set <code>submodule.recurse=false</code></label>
  </fieldset>`;
}

function submoduleOption(state: AppState, entry: SubmoduleChoice): string {
  const path = pathValue(entry);
  const selected = path === state.draft.submodule ? " selected" : "";
  const mark = entry.excluded ? " · already excluded" : "";
  return `<option value="${esc(path)}"${selected}>${esc(path)}${esc(mark)}</option>`;
}

export function quickSwitchForm(state: AppState): string {
  const targets = state.branches.filter((branch) => !branch.current);
  if (!targets.length) {
    return emptyState("No other local branch", "Quick switch moves between local branches; this repository only has one.");
  }
  const dirty = worktreeCounts(state).some(([label]) => label === "staged" || label === "unstaged");
  const target = targets.find((branch) => branch.name === state.draft.targetBranch);
  const carryNote = dirty && state.draft.carryChanges
    ? `<p class="hint">Tracked changes on ${esc(currentBranch(state))} will be stashed, the branch will switch, then popped onto ${esc(target?.name ?? "the target branch")}. Conflicts are reported afterwards.</p>`
    : dirty
      ? `<p class="hint">Tracked changes on ${esc(currentBranch(state))} will be saved before the switch.</p>`
      : "";
  return `<fieldset><legend>Branch to check out</legend>
    ${fieldNote("By default, tracked changes stay with the branch you leave. Untracked files stay where they are.")}
    <label class="field">Local branch
      <select data-event="select-branch" data-focus="branch" aria-label="Branch to switch to">
        ${targets.map((branch) => branchOption(state, branch)).join("")}
      </select>
    </label>
    ${dirty ? `<label class="check-row inline"><input type="checkbox" data-event="toggle-carry-changes"
      ${state.draft.carryChanges ? "checked" : ""} /> Carry tracked changes to the target branch</label>` : ""}
    ${carryNote}
    ${target?.saved_work ? `<p class="hint">${esc(target.name)} has Saved work waiting. Restore it from the Saved work section after you arrive.</p>` : ""}
  </fieldset>`;
}

function branchOption(state: AppState, branch: LocalBranch): string {
  const selected = branch.name === state.draft.targetBranch ? " selected" : "";
  const mark = branch.saved_work ? " · has Saved work" : "";
  return `<option value="${esc(branch.name)}"${selected}>${esc(branch.name)}${esc(mark)}</option>`;
}

export function syncForm(state: AppState): string {
  const base = baseRef(state);
  if (!base) return emptyState("Set a Base ref", "Sync fetches Base and merges it into the current branch.");
  return `<fieldset><legend>Bring ${esc(currentBranch(state) || "HEAD")} up to date</legend>
    ${fieldNote(`Fetch ${base}, set tracked changes aside, merge, then put the changes back.`)}
    <dl class="facts">
      <dt>Base</dt><dd><code>${esc(base)}</code> <button class="link" data-event="edit-base">Change</button></dd>
      <dt>Saved work</dt><dd>${savedWorkFor(state, currentBranch(state))
        ? "This branch already has Saved work; restore or delete it before syncing."
        : "A backup ref is written before the merge and kept afterwards."}</dd>
    </dl>
  </fieldset>`;
}

export function forcePushForm(state: AppState): string {
  const upstream = upstreamRef(state);
  if (!upstream) {
    return emptyState("No upstream", "Force push updates the remote-tracking ref of this branch. Push it once normally first.");
  }
  return `<fieldset><legend>Publish rewritten history</legend>
    ${fieldNote("Only needed after a rewrite. The push carries a lease, so it is refused if the remote moved.")}
    <dl class="facts">
      <dt>Upstream</dt><dd><code>${esc(upstream)}</code></dd>
      <dt>Lease</dt><dd>Checked against the remote SHA observed while preparing the review.</dd>
    </dl>
  </fieldset>`;
}
