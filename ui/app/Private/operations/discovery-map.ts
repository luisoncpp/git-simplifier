import { OPERATIONS } from "./catalog.ts";
import type {
  Bridge,
  ChangedPath,
  CleanupDiscovery,
  EditableCommit,
  LocalBranch,
  OperationId,
  SubmoduleChoice,
} from "../types.ts";

type DiscoveryResult =
  | ChangedPath[]
  | EditableCommit[]
  | LocalBranch[]
  | SubmoduleChoice[]
  | CleanupDiscovery;
type DiscoveryKey = "paths" | "commits" | "branches" | "submodules" | "cleanupBranches";
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
  cleanup: (bridge, base) => bridge.invoke<CleanupDiscovery>("list_cleanup_branches", { request: { base } }),
};

const RESULT_KEY: Partial<Record<OperationId, DiscoveryKey>> = {
  uncommit: "paths",
  revert: "paths",
  edit_message: "commits",
  split_branch: "paths",
  quick_switch: "branches",
  exclude_submodule: "submodules",
  cleanup: "cleanupBranches",
};

export function discoveryFor(operation: OperationId): Discovery | null {
  const load = DISCOVERY[operation];
  const key = RESULT_KEY[operation];
  if (!load || !key) return null;
  const needsBase = OPERATIONS.find((entry) => entry.id === operation)?.needsBase ?? false;
  return { load, key, needsBase };
}
