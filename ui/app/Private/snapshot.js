export const refValue = (value) => value?.value ?? value ?? "";

export const overviewOf = (state) => state.snapshot?.overview ?? null;
export const baseRef = (state) => refValue(overviewOf(state)?.base);
export const upstreamRef = (state) => refValue(overviewOf(state)?.upstream);
export const currentBranch = (state) => overviewOf(state)?.branch ?? "";

export function worktreeCounts(state) {
  const worktree = overviewOf(state)?.worktree;
  if (!worktree) return [];
  return [
    ["staged", worktree.staged],
    ["unstaged", worktree.unstaged],
    ["untracked", worktree.untracked],
    ["conflicts", worktree.conflicts],
  ].filter(([, count]) => count > 0);
}

export const savedWorkFor = (state, branch) => state.saved.find((item) => item.branch === branch) ?? null;

const RESUMABLE = ["fetch", "base-merge-conflict", "wip-reapply-conflict"];

const PHASE_LABELS = {
  fetch: "The fetch was interrupted",
  snapshot: "Sync stopped while setting tracked work aside",
  "base-merge": "Sync stopped while merging Base",
  "base-merge-conflict": "Merging Base hit conflicts",
  "wip-reapply": "Sync stopped while reapplying Saved work",
  "wip-reapply-conflict": "Reapplying Saved work hit conflicts",
};

export function syncPause(state) {
  const phase = overviewOf(state)?.sync_status;
  if (!phase) return null;
  return {
    phase,
    label: PHASE_LABELS[phase] ?? `Sync stopped at ${phase}`,
    resumable: RESUMABLE.includes(phase),
    retry: phase === "fetch",
  };
}
