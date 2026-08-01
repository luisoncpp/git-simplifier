import type { AppState, ChangedPath, Draft, RefValue, SubmoduleChoice } from "../types.ts";

/// Each path-picking operation keeps its own ticks so a selection made for one
/// never arrives pre-ticked in another.
export const pathSetFor = (state: AppState): Set<string> => {
  if (state.operation === "split_branch") return state.draft.splitPaths;
  if (state.operation === "revert") return state.draft.revertPaths;
  return state.draft.selectedPaths;
};

export const pathValue = (entry: { path: RefValue }): string => {
  const path = entry.path;
  if (path == null) return "";
  if (typeof path === "string") return path;
  return path.value ?? String(path);
};

/// Discovery runs again after every mutation, so selections that no longer
/// exist have to be dropped instead of being sent back to Rust.
export function adoptPaths(draft: Draft, paths: ChangedPath[]): void {
  const available = new Set(paths.map(pathValue));
  for (const set of [draft.selectedPaths, draft.splitPaths, draft.revertPaths]) {
    for (const selected of set) {
      if (!available.has(selected)) set.delete(selected);
    }
  }
}

export function adoptSubmodule(draft: Draft, submodules: SubmoduleChoice[]): void {
  const available = submodules.map(pathValue);
  if (available.includes(draft.submodule)) return;
  draft.submodule = available.find((path, index) => !submodules[index].excluded) ?? available[0] ?? "";
}

export function visiblePaths(state: AppState): ChangedPath[] {
  const query = state.draft.pathFilter.trim().toLowerCase();
  if (!query) return state.paths;
  return state.paths.filter((entry) => pathValue(entry).toLowerCase().includes(query));
}
