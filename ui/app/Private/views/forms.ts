import type { AppState, OperationId } from "../types.ts";
import { splitBranchForm } from "./form-branch.ts";
import { cleanupForm } from "./form-cleanup.ts";
import { historyForm } from "./form-switch-history.ts";
import { editMessageForm, uncommitForm } from "./form-history.ts";
import { submodulesForm } from "./form-submodules.ts";
import { forcePushForm, quickSwitchForm, revertForm, syncForm, commitMergeForm } from "./form-worktree.ts";

const FORMS: Partial<Record<OperationId, (state: AppState) => string>> = {
  uncommit: uncommitForm,
  revert: revertForm,
  edit_message: editMessageForm,
  submodules: submodulesForm,
  split_branch: splitBranchForm,
  quick_switch: quickSwitchForm,
  history: historyForm,
  sync: syncForm,
  commit_merge: commitMergeForm,
  force_push: forcePushForm,
  cleanup: cleanupForm,
};

export function operationForm(state: AppState): string {
  const form = FORMS[state.operation];
  if (!form) return "";
  return form(state);
}
