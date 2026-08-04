import { pathSetFor, pathValue, visiblePaths } from "./draft/index.ts";
import { OPERATIONS } from "./operations/index.ts";
import { currentBranch } from "./snapshot.ts";
import { messageTools } from "./views/form-history.ts";
import { submitRow } from "./views/actions.ts";
import type { AppController } from "./controller.ts";
import type { FieldNode } from "./types.ts";

export function setPathFilter(controller: AppController, node: FieldNode): void {
  controller.state.draft.pathFilter = node.value;
  controller.render();
}

export function togglePath(controller: AppController, node: FieldNode): void {
  const selected = pathSetFor(controller.state);
  if ((node as HTMLInputElement).checked) selected.add(node.value);
  else selected.delete(node.value);
  controller.render();
}

export function selectPaths(controller: AppController, mode: string): void {
  const selected = pathSetFor(controller.state);
  if (mode === "none") selected.clear();
  else for (const entry of visiblePaths(controller.state)) selected.add(pathValue(entry));
  controller.render();
}

export function setCommit(controller: AppController, node: FieldNode): void {
  controller.state.draft.commit = node.value;
  controller.render();
}

/// Typing must not replace the textarea: an innerHTML swap would drop the
/// native undo stack, so only the dependent controls are patched.
export function setMessage(controller: AppController, node: FieldNode): void {
  const draft = controller.state.draft;
  draft.messages.set(draft.commit, node.value);
  const tools = globalThis.document?.querySelector("#message-tools");
  const row = globalThis.document?.querySelector("#submit-row");
  if (!tools || !row) {
    controller.render();
    return;
  }
  tools.innerHTML = messageTools(controller.state);
  row.outerHTML = submitRow(controller.state);
}

export function resetMessage(controller: AppController): void {
  const draft = controller.state.draft;
  draft.messages.delete(draft.commit);
  controller.render();
}

export function setNewBranch(controller: AppController, node: FieldNode): void {
  controller.state.draft.newBranch = node.value;
  controller.render();
}

/// The message is optional and nothing else depends on it, so typing must not
/// re-render the textarea out from under the caret.
export function setSplitMessage(controller: AppController, node: FieldNode): void {
  controller.state.draft.splitMessage = node.value;
}

export function setSubmodule(controller: AppController, node: FieldNode): void {
  controller.state.draft.submodule = node.value;
  controller.render();
}

export function setTargetBranch(controller: AppController, node: FieldNode): void {
  controller.state.draft.targetBranch = node.value;
  controller.state.draft.branchPicked = true;
  controller.render();
}

export function setInstallHook(controller: AppController, node: FieldNode): void {
  controller.state.draft.installHook = (node as HTMLInputElement).checked;
  controller.render();
}

export function setDisableRecurse(controller: AppController, node: FieldNode): void {
  controller.state.draft.disableRecurse = (node as HTMLInputElement).checked;
  controller.render();
}

export function setCarryChanges(controller: AppController, node: FieldNode): void {
  controller.state.draft.carryChanges = (node as HTMLInputElement).checked;
  controller.render();
}

export function setCleanupOnlyMine(controller: AppController, node: FieldNode): void {
  controller.state.draft.cleanupOnlyMine = (node as HTMLInputElement).checked;
  controller.render();
}

export function setCleanupRemotes(controller: AppController, node: FieldNode): void {
  controller.state.draft.cleanupRemotes = (node as HTMLInputElement).checked;
  controller.render();
}

export function setCleanupAllRemote(controller: AppController, node: FieldNode): void {
  controller.state.draft.cleanupAllRemote = (node as HTMLInputElement).checked;
  controller.render();
}

export function setCleanupFilter(controller: AppController, node: FieldNode): void {
  controller.state.draft.cleanupFilter = node.value;
  controller.render();
}

/// Records an explicit choice. The map holds only what the user changed, so the
/// default — ticked unless the name is a shared one — survives every filter change.
export function toggleCleanupBranch(controller: AppController, node: FieldNode): void {
  const checked = (node as HTMLInputElement).checked;
  controller.state.draft.cleanupOverrides.set(node.value, checked);
  controller.render();
}

export function setRevertTarget(controller: AppController, node: FieldNode): void {
  const value = node.value;
  if (value !== "head" && value !== "base") return;
  controller.state.draft.revertTarget = value;
  controller.render();
}

export function setChangingBase(controller: AppController, value: string): void {
  controller.state.changingBase = value === "true";
  controller.render();
}

export function dismissError(controller: AppController): void {
  controller.state.error = "";
  controller.render();
}

export function dismissWarning(controller: AppController): void {
  controller.state.warning = "";
  controller.render();
}

export function dismissOutcome(controller: AppController): void {
  controller.state.outcome = null;
  controller.render();
}

export function dismissSavedWorkNotice(controller: AppController): void {
  controller.state.dismissedSavedWorkBranch = currentBranch(controller.state) || null;
  controller.render();
}

export function toggleEntry(controller: AppController, id: string): void {
  const expanded = controller.state.expanded;
  if (expanded.has(id)) expanded.delete(id);
  else expanded.add(id);
  controller.render();
}

export function stepOperation(controller: AppController, step: number): Promise<void> {
  const index = OPERATIONS.findIndex((operation) => operation.id === controller.state.operation);
  const next = OPERATIONS[(index + step + OPERATIONS.length) % OPERATIONS.length];
  return controller.selectOperation(next.id);
}
