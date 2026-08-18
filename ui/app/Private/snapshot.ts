import type { AppState, RefValue, RepositoryOverview, SavedWork } from "./types.ts";

export const refValue = (value: RefValue): string => {
  if (value == null) return "";
  if (typeof value === "string") return value;
  return value.value ?? String(value);
};

export const overviewOf = (state: AppState): RepositoryOverview | null => state.snapshot?.overview ?? null;
export const baseRef = (state: AppState): string => refValue(overviewOf(state)?.base);
export const upstreamRef = (state: AppState): string => refValue(overviewOf(state)?.upstream);
export const currentBranch = (state: AppState): string => overviewOf(state)?.branch ?? "";
export const presentBranch = (state: AppState): string => overviewOf(state)?.present_branch ?? "";

export function worktreeCounts(state: AppState): [string, number][] {
  const worktree = overviewOf(state)?.worktree;
  if (!worktree) return [];
  const counts: [string, number][] = [
    ["staged", worktree.staged],
    ["unstaged", worktree.unstaged],
    ["untracked", worktree.untracked],
    ["conflicts", worktree.conflicts],
  ];
  return counts.filter(([, count]) => count > 0);
}

export const savedWorkFor = (state: AppState, branch: string): SavedWork | null =>
  state.saved.find((item) => item.branch === branch) ?? null;

const RESUMABLE = ["fetch", "base-merge-conflict", "wip-reapply-conflict"];

const PHASE_LABELS: Record<string, string> = {
  fetch: "The fetch was interrupted",
  snapshot: "Sync stopped while setting tracked work aside",
  "base-merge": "Sync stopped while merging Base",
  "base-merge-conflict": "Merging Base hit conflicts",
  "wip-reapply": "Sync stopped while reapplying Saved work",
  "wip-reapply-conflict": "Reapplying Saved work hit conflicts",
};

export interface SyncPause {
  phase: string;
  label: string;
  resumable: boolean;
  retry: boolean;
}

export function syncPause(state: AppState): SyncPause | null {
  const phase = overviewOf(state)?.sync_status;
  if (!phase) return null;
  return {
    phase,
    label: PHASE_LABELS[phase] ?? `Sync stopped at ${phase}`,
    resumable: RESUMABLE.includes(phase),
    retry: phase === "fetch",
  };
}

export function quickSwitchPause(state: AppState): boolean {
  return overviewOf(state)?.quick_switch_status === "pull-ff-failed";
}
