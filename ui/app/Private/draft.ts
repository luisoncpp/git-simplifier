import type {
  AppState,
  ChangedPath,
  Draft,
  EditableCommit,
  LocalBranch,
  RefValue,
  SubmoduleChoice,
} from "./types.ts";

export function createDraft(): Draft {
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
    createFromRemote: "",
    pullAfterSwitch: true,
    carryChanges: false,
    branchFilter: "",
    branchMenuOpen: false,
    branchHighlight: 0,
  };
}

/// Uncommit and Split branch read the same path list but mean opposite things
/// by a tick, so a selection never crosses from one operation to the other.
export const pathSetFor = (state: AppState): Set<string> =>
  state.operation === "split_branch" ? state.draft.splitPaths : state.draft.selectedPaths;

export const pathValue = (entry: { path: RefValue }): string => {
  const path = entry.path;
  if (path == null) return "";
  if (typeof path === "string") return path;
  return path.value ?? String(path);
};

export const commitValue = (commit: { id: RefValue }): string => {
  const id = commit.id;
  if (id == null) return "";
  if (typeof id === "string") return id;
  return id.value ?? String(id);
};

/// Discovery runs again after every mutation, so selections that no longer
/// exist have to be dropped instead of being sent back to Rust.
export function adoptPaths(draft: Draft, paths: ChangedPath[]): void {
  const available = new Set(paths.map(pathValue));
  for (const set of [draft.selectedPaths, draft.splitPaths]) {
    for (const selected of set) {
      if (!available.has(selected)) set.delete(selected);
    }
  }
}

/// Rust returns the Editable range oldest first, but the commit people reword
/// is almost always the newest one, so the UI presents and defaults to that end.
export const newestFirst = <T>(commits: T[]): T[] => [...commits].reverse();

export function adoptCommit(draft: Draft, commits: EditableCommit[]): void {
  const available = newestFirst(commits).map(commitValue);
  if (available.includes(draft.commit)) return;
  draft.commit = available[0] ?? "";
}

export function adoptSubmodule(draft: Draft, submodules: SubmoduleChoice[]): void {
  const available = submodules.map(pathValue);
  if (available.includes(draft.submodule)) return;
  draft.submodule = available.find((path, index) => !submodules[index].excluded) ?? available[0] ?? "";
}

export function adoptBranch(draft: Draft, branches: LocalBranch[]): void {
  const available = branches.filter((branch) => !branch.current);
  const stillValid = available.some(
    (branch) =>
      branch.name === draft.targetBranch &&
      (branch.remote ?? "") === (draft.createFromRemote || ""),
  );
  if (stillValid) return;
  const first = available[0];
  draft.targetBranch = first?.name ?? "";
  draft.createFromRemote = first?.remote ?? "";
  draft.branchHighlight = 0;
}

export function visiblePaths(state: AppState): ChangedPath[] {
  const query = state.draft.pathFilter.trim().toLowerCase();
  if (!query) return state.paths;
  return state.paths.filter((entry) => pathValue(entry).toLowerCase().includes(query));
}

export function selectedCommit(state: AppState): EditableCommit | null {
  return state.commits.find((commit) => commitValue(commit) === state.draft.commit) ?? null;
}

export function messageFor(state: AppState): string {
  const commit = selectedCommit(state);
  if (!commit) return "";
  return state.draft.messages.get(commitValue(commit)) ?? commit.message;
}

export function messageChanged(state: AppState): boolean {
  const commit = selectedCommit(state);
  if (!commit) return false;
  return messageFor(state).trim() !== commit.message.trim();
}
