import type { AppState, OperationId } from "../types.ts";
import { splitBranchForm } from "./form-branch.ts";
import { editMessageForm, uncommitForm } from "./form-history.ts";
import { excludeForm, forcePushForm, quickSwitchForm, revertForm, syncForm } from "./form-worktree.ts";

const FORMS: Partial<Record<OperationId, (state: AppState) => string>> = {
  uncommit: uncommitForm,
  revert: revertForm,
  edit_message: editMessageForm,
  exclude_submodule: excludeForm,
  split_branch: splitBranchForm,
  quick_switch: quickSwitchForm,
  sync: syncForm,
  force_push: forcePushForm,
};

export function operationForm(state: AppState): string {
  const form = FORMS[state.operation];
  if (!form) return "";
  return form(state);
}
