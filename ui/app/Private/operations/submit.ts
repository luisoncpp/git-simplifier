import {
  cleanupChoices,
  cleanupSelection,
  messageChanged,
  messageFor,
  pathSetFor,
  selectedCommit,
} from "../draft/index.ts";
import { baseRef, upstreamRef } from "../snapshot.ts";
import { OPERATIONS } from "./catalog.ts";
import type { AppState, OperationId } from "../types.ts";

export interface SubmitState {
  disabled: boolean;
  reason: string;
}

/// The submit button always states why it is unavailable; a silently disabled
/// primary action reads as a broken workbench.
export function submitState(state: AppState): SubmitState {
  const blocked = blockingReason(state);
  return { disabled: Boolean(blocked) || state.busy, reason: blocked };
}

export function excludeSubmitState(state: AppState): SubmitState {
  const blocked = excludeBlockingReason(state);
  return { disabled: Boolean(blocked) || state.busy, reason: blocked };
}

export function cleanupSubmitState(state: AppState): SubmitState {
  const blocked = cleanupBlockingReason(state);
  return { disabled: Boolean(blocked) || state.busy, reason: blocked };
}

function blockingReason(state: AppState): string {
  if (state.operation === "submodules") return "";
  const operation = OPERATIONS.find((entry) => entry.id === state.operation);
  if (operation?.needsBase && !baseRef(state)) return "Set a Base ref first.";
  return SPECIFIC_REASON[state.operation]?.(state) ?? "";
}

function excludeBlockingReason(state: AppState): string {
  if (!state.submodules.length) return "This repository has no submodules.";
  if (!state.draft.submodule) return "Select a submodule.";
  return "";
}

function cleanupBlockingReason(state: AppState): string {
  if (!state.dirtySubmodules.length) return "No dirty submodules.";
  if (!state.draft.cleanupSubmodulePaths.size) return "Select at least one submodule.";
  if (!state.draft.cleanupUncommit && !state.draft.cleanupRevert) {
    return "Select at least one cleanup action.";
  }
  if (state.draft.cleanupUncommit && !baseRef(state)) {
    return "Set a Base ref first.";
  }
  return "";
}

const SPECIFIC_REASON: Partial<Record<OperationId, (state: AppState) => string>> = {
  uncommit: (state) => {
    if (!state.paths.length) return "Nothing on this branch differs from Base.";
    if (!state.draft.selectedPaths.size) return "Select at least one path.";
    return "";
  },
  revert: (state) => {
    if (!state.paths.length) {
      return "Nothing differs from Base and there is no tracked local dirt.";
    }
    if (!state.draft.revertPaths.size) return "Select at least one path.";
    return "";
  },
  edit_message: (state) => {
    if (!state.commits.length) return "No commit on this branch is outside Base.";
    if (!selectedCommit(state)) return "Select a commit.";
    if (!messageFor(state).trim()) return "The message cannot be empty.";
    if (!messageChanged(state)) return "The message is unchanged.";
    return "";
  },
  split_branch: (state) => {
    if (!state.paths.length) return "Nothing on this branch differs from Base.";
    if (!state.draft.newBranch.trim()) return "Name the new branch.";
    if (!pathSetFor(state).size) return "Select at least one path to copy.";
    return "";
  },
  quick_switch: (state) => {
    if (state.branches.filter((branch) => !branch.current).length === 0) {
      return "There is no other branch to switch to.";
    }
    if (!state.draft.targetBranch) return "Select a branch.";
    return "";
  },
  force_push: (state) => (upstreamRef(state) ? "" : "The current branch has no upstream to push to."),
  commit_merge: (state) => {
    if (!state.snapshot?.overview.merge_in_progress) return "No merge in progress.";
    if (state.snapshot.overview.worktree.conflicts > 0) return "Resolve merge conflicts first.";
    return "";
  },
  cleanup: (state) => {
    if (!state.cleanupBranches?.choices.length) return "No branch is fully merged into Base.";
    if (!cleanupChoices(state).length) return "No branch matches these filters.";
    if (!cleanupSelection(state).length) return "Tick at least one branch to delete.";
    return "";
  },
};
