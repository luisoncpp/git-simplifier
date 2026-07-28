import { messageChanged, messageFor, selectedCommit } from "./draft.js";
import { baseRef, upstreamRef } from "./snapshot.js";

export const OPERATIONS = [
  { id: "uncommit", label: "Uncommit paths", needsBase: true },
  { id: "edit_message", label: "Edit message", needsBase: true },
  { id: "exclude_submodule", label: "Exclude submodule", needsBase: false },
  { id: "quick_switch", label: "Quick switch", needsBase: false },
  { id: "sync", label: "Sync with Base", needsBase: true },
  { id: "force_push", label: "Force push", needsBase: false },
];

export const operationLabel = (id) => OPERATIONS.find((operation) => operation.id === id)?.label ?? id;

const DISCOVERY = {
  uncommit: (bridge, base) => bridge.invoke("list_changed_paths", { request: { base } }),
  edit_message: (bridge, base) => bridge.invoke("list_editable_commits", { request: { base } }),
  quick_switch: (bridge) => bridge.invoke("list_local_branches"),
  exclude_submodule: (bridge) => bridge.invoke("list_submodules"),
};

const RESULT_KEY = {
  uncommit: "paths",
  edit_message: "commits",
  quick_switch: "branches",
  exclude_submodule: "submodules",
};

export function discoveryFor(operation) {
  const load = DISCOVERY[operation];
  if (!load) return null;
  const needsBase = OPERATIONS.find((entry) => entry.id === operation)?.needsBase ?? false;
  return { load, key: RESULT_KEY[operation], needsBase };
}

export function buildRequest(state) {
  const kind = state.operation;
  const base = baseRef(state);
  const draft = state.draft;
  if (kind === "uncommit") return { kind, base, paths: [...draft.selectedPaths] };
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
  if (kind === "quick_switch") return { kind, target_branch: draft.targetBranch };
  if (kind === "sync") return { kind, base };
  return { kind: "force_push" };
}

/// The submit button always states why it is unavailable; a silently disabled
/// primary action reads as a broken workbench.
export function submitState(state) {
  const blocked = blockingReason(state);
  return { disabled: Boolean(blocked) || state.busy, reason: blocked };
}

function blockingReason(state) {
  const operation = OPERATIONS.find((entry) => entry.id === state.operation);
  if (operation?.needsBase && !baseRef(state)) return "Set a Base ref first.";
  return SPECIFIC_REASON[state.operation]?.(state) ?? "";
}

const SPECIFIC_REASON = {
  uncommit: (state) => {
    if (!state.paths.length) return "Nothing on this branch differs from Base.";
    if (!state.draft.selectedPaths.size) return "Select at least one path.";
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
  quick_switch: (state) => {
    if (state.branches.filter((branch) => !branch.current).length === 0) {
      return "There is no other local branch to switch to.";
    }
    if (!state.draft.targetBranch) return "Select a branch.";
    return "";
  },
  force_push: (state) => (upstreamRef(state) ? "" : "The current branch has no upstream to push to."),
};
