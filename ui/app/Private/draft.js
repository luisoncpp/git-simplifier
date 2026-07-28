export function createDraft() {
  return {
    pathFilter: "",
    selectedPaths: new Set(),
    splitPaths: new Set(),
    newBranch: "",
    splitMessage: "",
    commit: "",
    messages: new Map(),
    submodule: "",
    installHook: true,
    disableRecurse: false,
    targetBranch: "",
  };
}

/// Uncommit and Split branch read the same path list but mean opposite things
/// by a tick, so a selection never crosses from one operation to the other.
export const pathSetFor = (state) =>
  state.operation === "split_branch" ? state.draft.splitPaths : state.draft.selectedPaths;

export const pathValue = (entry) => entry.path?.value ?? entry.path;
export const commitValue = (commit) => commit.id?.value ?? commit.id;

/// Discovery runs again after every mutation, so selections that no longer
/// exist have to be dropped instead of being sent back to Rust.
export function adoptPaths(draft, paths) {
  const available = new Set(paths.map(pathValue));
  for (const set of [draft.selectedPaths, draft.splitPaths]) {
    for (const selected of set) {
      if (!available.has(selected)) set.delete(selected);
    }
  }
}

/// Rust returns the Editable range oldest first, but the commit people reword
/// is almost always the newest one, so the UI presents and defaults to that end.
export const newestFirst = (commits) => [...commits].reverse();

export function adoptCommit(draft, commits) {
  const available = newestFirst(commits).map(commitValue);
  if (available.includes(draft.commit)) return;
  draft.commit = available[0] ?? "";
}

export function adoptSubmodule(draft, submodules) {
  const available = submodules.map(pathValue);
  if (available.includes(draft.submodule)) return;
  draft.submodule = available.find((path, index) => !submodules[index].excluded) ?? available[0] ?? "";
}

export function adoptBranch(draft, branches) {
  const available = branches.filter((branch) => !branch.current).map((branch) => branch.name);
  if (available.includes(draft.targetBranch)) return;
  draft.targetBranch = available[0] ?? "";
}

export function visiblePaths(state) {
  const query = state.draft.pathFilter.trim().toLowerCase();
  if (!query) return state.paths;
  return state.paths.filter((entry) => pathValue(entry).toLowerCase().includes(query));
}

export function selectedCommit(state) {
  return state.commits.find((commit) => commitValue(commit) === state.draft.commit) ?? null;
}

export function messageFor(state) {
  const commit = selectedCommit(state);
  if (!commit) return "";
  return state.draft.messages.get(commitValue(commit)) ?? commit.message;
}

export function messageChanged(state) {
  const commit = selectedCommit(state);
  if (!commit) return false;
  return messageFor(state).trim() !== commit.message.trim();
}
