import type { Draft } from "../types.ts";

export function createDraft(): Draft {
  return {
    pathFilter: "",
    selectedPaths: new Set(),
    splitPaths: new Set(),
    revertPaths: new Set(),
    revertTarget: "head",
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
    cleanupOnlyMine: true,
    cleanupRemotes: true,
    cleanupAllRemote: false,
    cleanupOverrides: new Map(),
    cleanupFilter: "",
  };
}
