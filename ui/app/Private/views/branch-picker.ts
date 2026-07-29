import { esc } from "../dom.ts";
import type { AppState, LocalBranch } from "../types.ts";

export function switchTargets(state: AppState): LocalBranch[] {
  return state.branches.filter((branch) => !branch.current);
}

export function filteredSwitchTargets(state: AppState): LocalBranch[] {
  const query = state.draft.branchFilter.trim().toLowerCase();
  const targets = switchTargets(state);
  if (!query) return targets;
  return targets.filter((branch) => {
    const haystack = `${branch.name} ${branch.remote ?? ""}`.toLowerCase();
    return haystack.includes(query);
  });
}

export function selectedSwitchTarget(state: AppState): LocalBranch | null {
  return (
    switchTargets(state).find((branch) => {
      if (branch.name !== state.draft.targetBranch) return false;
      return (branch.remote ?? "") === (state.draft.createFromRemote || "");
    }) ?? null
  );
}

export function branchPicker(state: AppState): string {
  const open = state.draft.branchMenuOpen;
  const selected = selectedSwitchTarget(state);
  const label = selected
    ? selected.remote
      ? `${selected.remote} → ${selected.name}`
      : selected.name
    : "Select a branch";
  return `<div class="branch-picker">
    <button class="branch-trigger" type="button" data-event="toggle-branch-menu"
      aria-expanded="${open ? "true" : "false"}" aria-controls="branch-menu"
      aria-label="Branch to switch to" data-focus="branch" ${state.busy ? "disabled" : ""}>
      <span class="eyebrow">Branch</span>
      <strong>${esc(label)}</strong>
    </button>
    ${open ? branchMenu(state) : ""}
  </div>`;
}

function branchMenu(state: AppState): string {
  const entries = filteredSwitchTargets(state);
  return `<div id="branch-menu" class="branch-menu" role="listbox" aria-label="Branches">
    <label class="branch-filter">
      <span class="sr-only">Filter branches</span>
      <input id="branch-filter" data-event="branch-filter" data-focus="branch-filter"
        type="search" placeholder="Filter branches" value="${esc(state.draft.branchFilter)}"
        autocomplete="off" ${state.busy ? "disabled" : ""} />
    </label>
    <div class="branch-menu-list" data-scroll="branch-menu-list">${menuBody(state, entries)}</div>
  </div>`;
}

function menuBody(state: AppState, entries: LocalBranch[]): string {
  if (!switchTargets(state).length) {
    return `<p class="branch-empty">No other branches are available.</p>`;
  }
  if (!entries.length) {
    return `<p class="branch-empty">No branches match.</p>`;
  }
  return entries.map((entry, index) => branchRow(state, entry, index)).join("");
}

function branchRow(state: AppState, entry: LocalBranch, index: number): string {
  const selected =
    entry.name === state.draft.targetBranch &&
    (entry.remote ?? "") === (state.draft.createFromRemote || "");
  const active = index === state.draft.branchHighlight;
  const classes = ["branch-row", selected ? "current" : "", active ? "active" : ""]
    .filter(Boolean)
    .join(" ");
  const mark = entry.saved_work ? " · has Saved work" : "";
  const title = entry.remote
    ? `${entry.remote} → local ${entry.name}${mark}`
    : `${entry.name}${mark}`;
  const remote = entry.remote ?? "";
  return `<button class="${classes}" type="button" role="option"
    aria-selected="${selected ? "true" : "false"}"
    data-event="pick-branch" data-value="${esc(entry.name)}" data-remote="${esc(remote)}"
    ${state.busy ? "disabled" : ""}>${esc(title)}</button>`;
}
