import type { OperationId } from "../types.ts";

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
