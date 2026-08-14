import { cleanupSelection, messageFor, pathSetFor } from "../draft/index.ts";
import { baseRef } from "../snapshot.ts";
import type { AppState, OperationRequest } from "../types.ts";

export function buildRequest(state: AppState): OperationRequest {
  const kind = state.operation;
  if (kind === "uncommit") return uncommitRequest(state);
  if (kind === "revert") return revertRequest(state);
  if (kind === "edit_message") return editMessageRequest(state);
  if (kind === "split_branch") return splitBranchRequest(state);
  if (kind === "quick_switch") return quickSwitchRequest(state);
  if (kind === "sync") return { kind, base: baseRef(state) };
  if (kind === "commit_merge") return { kind };
  if (kind === "cleanup") return cleanupRequest(state);
  return { kind: "force_push" };
}

export function buildExcludeRequest(state: AppState): OperationRequest {
  return {
    kind: "exclude_submodule",
    path: state.draft.submodule,
    install_hook: state.draft.installHook,
    disable_recurse: state.draft.disableRecurse,
  };
}

export function buildCleanupRequest(state: AppState): OperationRequest {
  return {
    kind: "cleanup_submodules",
    base: baseRef(state),
    paths: [...state.draft.cleanupSubmodulePaths],
    uncommit: state.draft.cleanupUncommit,
    revert: state.draft.cleanupRevert,
  };
}

function cleanupRequest(state: AppState): OperationRequest {
  return {
    kind: "cleanup",
    base: baseRef(state),
    references: cleanupSelection(state),
    delete_remotes: state.draft.cleanupRemotes,
  };
}

function uncommitRequest(state: AppState): OperationRequest {
  return { kind: "uncommit", base: baseRef(state), paths: [...state.draft.selectedPaths] };
}

function revertRequest(state: AppState): OperationRequest {
  return {
    kind: "revert",
    base: baseRef(state),
    paths: [...state.draft.revertPaths],
    target: state.draft.revertTarget,
  };
}

function editMessageRequest(state: AppState): OperationRequest {
  return {
    kind: "edit_message",
    base: baseRef(state),
    commit: state.draft.commit,
    message: messageFor(state),
  };
}

function splitBranchRequest(state: AppState): OperationRequest {
  return {
    kind: "split_branch",
    base: baseRef(state),
    new_branch: state.draft.newBranch.trim(),
    paths: [...pathSetFor(state)],
    message: state.draft.splitMessage,
  };
}

function quickSwitchRequest(state: AppState): OperationRequest {
  const draft = state.draft;
  return {
    kind: "quick_switch",
    target_branch: draft.targetBranch,
    carry_changes: draft.carryChanges,
    pull_after_switch: draft.pullAfterSwitch,
    create_from_remote: draft.createFromRemote || null,
    merge_untracked: draft.mergeUntracked,
  };
}
