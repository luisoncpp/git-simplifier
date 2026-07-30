import { messageChanged, messageFor, pathSetFor, selectedCommit } from "./draft.ts";
import { baseRef, upstreamRef } from "./snapshot.ts";
import type {
  AppState,
  Bridge,
  ChangedPath,
  EditableCommit,
  LocalBranch,
  OperationId,
  OperationRequest,
  SubmoduleChoice,
} from "./types.ts";

export interface OperationDef {
  id: OperationId;
  label: string;
  needsBase: boolean;
}

export const OPERATIONS: OperationDef[] = [
  { id: "uncommit", label: "Uncommit", needsBase: true },
  { id: "revert", label: "Revert", needsBase: true },
  { id: "edit_message", label: "Edit message", needsBase: true },
  { id: "exclude_submodule", label: "Exclude submodule", needsBase: false },
  { id: "split_branch", label: "Split branch", needsBase: true },
  { id: "quick_switch", label: "Quick switch", needsBase: false },
  { id: "sync", label: "Sync with Base", needsBase: true },
  { id: "force_push", label: "Force push", needsBase: false },
];

export const operationLabel = (id: string): string =>
  OPERATIONS.find((operation) => operation.id === id)?.label ?? id;

type DiscoveryResult = ChangedPath[] | EditableCommit[] | LocalBranch[] | SubmoduleChoice[];
type DiscoveryKey = "paths" | "commits" | "branches" | "submodules";
type DiscoveryLoad = (bridge: Bridge, base: string) => Promise<DiscoveryResult>;

export interface Discovery {
  load: DiscoveryLoad;
  key: DiscoveryKey;
  needsBase: boolean;
}

const DISCOVERY: Partial<Record<OperationId, DiscoveryLoad>> = {
  uncommit: (bridge, base) => bridge.invoke<ChangedPath[]>("list_changed_paths", { request: { base } }),
  revert: (bridge, base) => bridge.invoke<ChangedPath[]>("list_revert_paths", { request: { base } }),
  edit_message: (bridge, base) => bridge.invoke<EditableCommit[]>("list_editable_commits", { request: { base } }),
  split_branch: (bridge, base) => bridge.invoke<ChangedPath[]>("list_changed_paths", { request: { base } }),
  quick_switch: (bridge) => bridge.invoke<LocalBranch[]>("list_local_branches"),
  exclude_submodule: (bridge) => bridge.invoke<SubmoduleChoice[]>("list_submodules"),
};

const RESULT_KEY: Partial<Record<OperationId, DiscoveryKey>> = {
  uncommit: "paths",
  revert: "paths",
  edit_message: "commits",
  split_branch: "paths",
  quick_switch: "branches",
  exclude_submodule: "submodules",
};

export function discoveryFor(operation: OperationId): Discovery | null {
  const load = DISCOVERY[operation];
  const key = RESULT_KEY[operation];
  if (!load || !key) return null;
  const needsBase = OPERATIONS.find((entry) => entry.id === operation)?.needsBase ?? false;
  return { load, key, needsBase };
}

export function buildRequest(state: AppState): OperationRequest {
  const kind = state.operation;
  const base = baseRef(state);
  const draft = state.draft;
  if (kind === "uncommit") return { kind, base, paths: [...draft.selectedPaths] };
  if (kind === "revert") {
    return { kind, base, paths: [...draft.revertPaths], target: draft.revertTarget };
  }
  if (kind === "edit_message") {
    return { kind, base, commit: draft.commit, message: messageFor(state) };
  }
  if (kind === "exclude_submodule") {
    return {
      kind,
      path: draft.submodule,
      install_hook: draft.installHook,
      disable_recurse: draft.disableRecurse,
    };
  }
  if (kind === "split_branch") {
    return {
      kind,
      base,
      new_branch: draft.newBranch.trim(),
      paths: [...draft.splitPaths],
      message: draft.splitMessage,
    };
  }
  if (kind === "quick_switch") {
    return {
      kind,
      target_branch: draft.targetBranch,
      carry_changes: draft.carryChanges,
      pull_after_switch: draft.pullAfterSwitch,
      create_from_remote: draft.createFromRemote || null,
    };
  }
  if (kind === "sync") return { kind, base };
  return { kind: "force_push" };
}

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

function blockingReason(state: AppState): string {
  const operation = OPERATIONS.find((entry) => entry.id === state.operation);
  if (operation?.needsBase && !baseRef(state)) return "Set a Base ref first.";
  return SPECIFIC_REASON[state.operation]?.(state) ?? "";
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
  exclude_submodule: (state) => {
    if (!state.submodules.length) return "This repository has no submodules.";
    if (!state.draft.submodule) return "Select a submodule.";
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
};
