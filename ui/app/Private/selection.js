import { pathSetFor, pathValue, visiblePaths } from "./draft.js";
import { OPERATIONS } from "./operations.js";
import { messageTools } from "./views/form-history.js";
import { submitRow } from "./views/actions.js";

export function setPathFilter(controller, node) {
  controller.state.draft.pathFilter = node.value;
  controller.render();
}

export function togglePath(controller, node) {
  const selected = pathSetFor(controller.state);
  if (node.checked) selected.add(node.value);
  else selected.delete(node.value);
  controller.render();
}

export function selectPaths(controller, mode) {
  const selected = pathSetFor(controller.state);
  if (mode === "none") selected.clear();
  else for (const entry of visiblePaths(controller.state)) selected.add(pathValue(entry));
  controller.render();
}

export function setCommit(controller, node) {
  controller.state.draft.commit = node.value;
  controller.render();
}

/// Typing must not replace the textarea: an innerHTML swap would drop the
/// native undo stack, so only the dependent controls are patched.
export function setMessage(controller, node) {
  const draft = controller.state.draft;
  draft.messages.set(draft.commit, node.value);
  const document = globalThis.document;
  const tools = document?.querySelector("#message-tools");
  const row = document?.querySelector("#submit-row");
  if (!tools || !row) {
    controller.render();
    return;
  }
  tools.innerHTML = messageTools(controller.state);
  row.outerHTML = submitRow(controller.state);
}

export function resetMessage(controller) {
  const draft = controller.state.draft;
  draft.messages.delete(draft.commit);
  controller.render();
}

export function setNewBranch(controller, node) {
  controller.state.draft.newBranch = node.value;
  controller.render();
}

/// The message is optional and nothing else depends on it, so typing must not
/// re-render the textarea out from under the caret.
export function setSplitMessage(controller, node) {
  controller.state.draft.splitMessage = node.value;
}

export function setSubmodule(controller, node) {
  controller.state.draft.submodule = node.value;
  controller.render();
}

export function setTargetBranch(controller, node) {
  controller.state.draft.targetBranch = node.value;
  controller.render();
}

export function setInstallHook(controller, node) {
  controller.state.draft.installHook = node.checked;
  controller.render();
}

export function setDisableRecurse(controller, node) {
  controller.state.draft.disableRecurse = node.checked;
  controller.render();
}

export function setChangingBase(controller, value) {
  controller.state.changingBase = value === "true";
  controller.render();
}

export function dismissError(controller) {
  controller.state.error = "";
  controller.render();
}

export function dismissOutcome(controller) {
  controller.state.outcome = null;
  controller.render();
}

export function toggleEntry(controller, id) {
  const expanded = controller.state.expanded;
  if (expanded.has(id)) expanded.delete(id);
  else expanded.add(id);
  controller.render();
}

export function stepOperation(controller, step) {
  const index = OPERATIONS.findIndex((operation) => operation.id === controller.state.operation);
  const next = OPERATIONS[(index + step + OPERATIONS.length) % OPERATIONS.length];
  return controller.selectOperation(next.id);
}
