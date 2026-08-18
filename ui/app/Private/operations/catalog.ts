import type { OperationId } from "../types.ts";

export interface OperationDef {
  id: OperationId;
  label: string;
  needsBase: boolean;
}

export const OPERATIONS: OperationDef[] = [
  { id: "sync", label: "Sync with Base", needsBase: true },
  { id: "quick_switch", label: "Quick switch", needsBase: false },
  { id: "history", label: "History", needsBase: false },
  { id: "commit_merge", label: "Commit merge", needsBase: false },
  { id: "revert", label: "Revert", needsBase: true },
  { id: "uncommit", label: "Uncommit", needsBase: true },
  { id: "edit_message", label: "Edit message", needsBase: true },
  { id: "submodules", label: "Submodules", needsBase: false },
  { id: "split_branch", label: "Split branch", needsBase: true },
  { id: "force_push", label: "Force push", needsBase: false },
  { id: "cleanup", label: "Cleanup", needsBase: true },
];
