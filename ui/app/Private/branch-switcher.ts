import { filteredSwitchTargets } from "./views/branch-picker.ts";
import type { AppController } from "./controller.ts";
import type { FieldNode } from "./types.ts";

export function setPullAfterSwitch(controller: AppController, node: FieldNode): void {
  controller.state.draft.pullAfterSwitch = (node as HTMLInputElement).checked;
  controller.render();
}

export function toggleBranchMenu(controller: AppController): void {
  const draft = controller.state.draft;
  draft.branchMenuOpen = !draft.branchMenuOpen;
  if (draft.branchMenuOpen) draft.branchHighlight = 0;
  controller.render();
}

export function closeBranchMenu(controller: AppController): void {
  if (!controller.state.draft.branchMenuOpen) return;
  controller.state.draft.branchMenuOpen = false;
  controller.render();
}

export function setBranchFilter(controller: AppController, node: FieldNode): void {
  controller.state.draft.branchFilter = node.value;
  controller.state.draft.branchHighlight = 0;
  controller.render();
}

export function pickBranch(controller: AppController, name: string, remote = ""): void {
  const draft = controller.state.draft;
  draft.targetBranch = name;
  draft.createFromRemote = remote;
  draft.branchMenuOpen = false;
  draft.branchFilter = "";
  controller.render();
}

function moveBranchHighlight(controller: AppController, step: number): void {
  const entries = filteredSwitchTargets(controller.state);
  if (!entries.length) return;
  const draft = controller.state.draft;
  draft.branchHighlight = (draft.branchHighlight + step + entries.length) % entries.length;
  controller.render();
}

function activateHighlightedBranch(controller: AppController): void {
  const entry = filteredSwitchTargets(controller.state)[controller.state.draft.branchHighlight];
  if (!entry) return;
  pickBranch(controller, entry.name, entry.remote ?? "");
}

/// Truthy when the key belonged to the open branch menu, so the delegated
/// dispatcher stops looking.
export function handleKeys(controller: AppController, event: KeyboardEvent): boolean {
  if (!controller.state.draft.branchMenuOpen) return false;
  if (event.key === "Escape") {
    event.preventDefault();
    closeBranchMenu(controller);
    return true;
  }
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    moveBranchHighlight(controller, event.key === "ArrowDown" ? 1 : -1);
    return true;
  }
  const target = event.target as HTMLElement | null;
  if (event.key === "Enter" && target?.dataset?.event === "branch-filter") {
    event.preventDefault();
    activateHighlightedBranch(controller);
    return true;
  }
  return false;
}
