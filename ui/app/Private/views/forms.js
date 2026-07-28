import { editMessageForm, uncommitForm } from "./form-history.js";
import { excludeForm, forcePushForm, quickSwitchForm, syncForm } from "./form-worktree.js";

const FORMS = {
  uncommit: uncommitForm,
  edit_message: editMessageForm,
  exclude_submodule: excludeForm,
  quick_switch: quickSwitchForm,
  sync: syncForm,
  force_push: forcePushForm,
};

export function operationForm(state) {
  const form = FORMS[state.operation];
  if (!form) return "";
  return form(state);
}
