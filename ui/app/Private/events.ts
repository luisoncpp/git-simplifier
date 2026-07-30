import * as edit from "./selection.ts";
import * as repos from "./repository-switcher.ts";
import * as branches from "./branch-switcher.ts";
import * as diff from "./files-diff/index.ts";
import type { AppController } from "./controller.ts";
import type { FieldNode, OperationId, ViewId } from "./types.ts";

type ClickHandler = (controller: AppController, value: string, node?: HTMLElement) => unknown;
type FieldHandler = (controller: AppController, node: FieldNode) => void;
type ActionElement = HTMLElement & { disabled?: boolean };

const CLICK: Record<string, ClickHandler> = {
  "toggle-repo-menu": (controller) => repos.toggleRepoMenu(controller),
  "pick-repository": (controller) => repos.openPickedRepository(controller),
  "open-recent": (controller, value) => repos.openRecentRepository(controller, value),
  "remove-recent": (controller, value) => repos.removeRecentRepository(controller, value),
  "toggle-branch-menu": (controller) => branches.toggleBranchMenu(controller),
  "pick-branch": (controller, value, node) =>
    branches.pickBranch(controller, value, node?.dataset.remote ?? ""),
  refresh: (controller) => controller.refresh(),
  "set-view": (controller, value) => controller.setView(value as ViewId),
  "set-operation": (controller, value) => controller.selectOperation(value as OperationId),
  "submit-operation": (controller) => controller.submitOperation(),
  "cancel-review": (controller) => controller.cancelReview(),
  "apply-review": (controller) => controller.applyReview(),
  "edit-base": (controller) => edit.setChangingBase(controller, "true"),
  "cancel-base": (controller) => edit.setChangingBase(controller, "false"),
  "save-base": (controller) => controller.chooseBase(baseChoice()),
  "force-push": (controller) => controller.prepare({ kind: "force_push" }),
  "publish-branch": (controller, value) => controller.prepare({ kind: "publish_branch", branch: value }),
  "resolve-pull-replace": (controller) =>
    controller.prepare({ kind: "resolve_quick_switch_pull", resolution: "replace_with_remote" }),
  "resolve-pull-merge": (controller) =>
    controller.prepare({ kind: "resolve_quick_switch_pull", resolution: "merge_pull" }),
  "resolve-pull-cancel": (controller) =>
    controller.prepare({ kind: "resolve_quick_switch_pull", resolution: "cancel" }),
  "resume-sync": (controller) => controller.prepare({ kind: "resume_sync" }),
  "restore-saved": (controller) => controller.prepare({ kind: "restore_saved_work" }),
  "delete-saved": (controller, value) => controller.prepare({ kind: "delete_saved_work", branch: value }),
  "switch-to": (controller, value) => controller.switchTo(value),
  "select-paths": (controller, value) => edit.selectPaths(controller, value),
  "reset-message": (controller) => edit.resetMessage(controller),
  "dismiss-error": (controller) => edit.dismissError(controller),
  "dismiss-outcome": (controller) => edit.dismissOutcome(controller),
  "toggle-entry": (controller, value) => edit.toggleEntry(controller, value),
  copy: (controller, value) => controller.copy(value),
  "copy-diff": (controller) => controller.copyDiff(),
  "set-diff-layout": (controller, value) => diff.setLayout(controller, value),
  "toggle-file": (controller, value) => diff.toggleFile(controller, value),
  "set-all-files": (controller, value) => diff.setAllFiles(controller, value),
  "expand-gap": (controller, value, node) => diff.expandGap(controller, value, node),
  "toggle-file-navigator": (controller) => diff.toggleNavigator(controller),
  "jump-to-file": (controller, value) => diff.jumpToFile(controller, value),
};

const CHANGE: Record<string, FieldHandler> = {
  "select-commit": edit.setCommit,
  "select-submodule": edit.setSubmodule,
  "select-branch": edit.setTargetBranch,
  "toggle-path": edit.togglePath,
  "toggle-install-hook": edit.setInstallHook,
  "toggle-disable-recurse": edit.setDisableRecurse,
  "toggle-carry-changes": edit.setCarryChanges,
  "toggle-pull-after-switch": branches.setPullAfterSwitch,
};

const INPUT: Record<string, FieldHandler> = {
  "path-filter": edit.setPathFilter,
  "repo-filter": repos.setRepoFilter,
  "branch-filter": branches.setBranchFilter,
  "commit-message": edit.setMessage,
  "split-branch-name": edit.setNewBranch,
  "split-message": edit.setSplitMessage,
};

const TAB_STEP: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };

export function bindEvents(controller: AppController): void {
  const target = globalThis.document;
  if (!target) return;
  target.addEventListener("click", /*handleClick=*/ (event) => handleClick(controller, event));
  target.addEventListener("change", /*handleChange=*/ (event) => dispatchNode(controller, event, CHANGE));
  target.addEventListener("input", /*handleInput=*/ (event) => handleInput(controller, event));
  target.addEventListener("keydown", /*handleKeys=*/ (event) => handleKeys(controller, event));
}

function handleClick(controller: AppController, event: MouseEvent): void {
  const target = event.target as HTMLElement | null;
  if (controller.state.repoMenuOpen && !target?.closest?.(".repo-switcher")) {
    repos.closeRepoMenu(controller);
  }
  if (controller.state.draft.branchMenuOpen && !target?.closest?.(".branch-picker")) {
    branches.closeBranchMenu(controller);
  }
  const node = target?.closest?.("[data-event]") as ActionElement | null;
  if (!node || node.disabled) return;
  const action = CLICK[node.dataset.event ?? ""];
  if (!action) return;
  event.preventDefault();
  settle(controller, action(controller, node.dataset.value ?? "", node));
}

function handleInput(controller: AppController, event: Event): void {
  if ((event as InputEvent).isComposing) return;
  dispatchNode(controller, event, INPUT);
}

function dispatchNode(controller: AppController, event: Event, table: Record<string, FieldHandler>): void {
  const node = event.target as FieldNode | null;
  const action = table[node?.dataset?.event ?? ""];
  if (!action || !node) return;
  settle(controller, action(controller, node));
}

/// A menu's own module owns its keyboard map. A handled key returns a truthy
/// value — sometimes the promise it started — so `settle` still reports a
/// rejected activation here rather than in two more places.
function handleKeys(controller: AppController, event: KeyboardEvent): void {
  const handled = repos.handleKeys(controller, event) || branches.handleKeys(controller, event);
  if (handled) {
    settle(controller, handled);
    return;
  }
  if (event.key === "Escape" && controller.state.review) {
    settle(controller, controller.cancelReview());
    return;
  }
  if (event.key === "Enter") {
    handleEnter(controller, event);
    return;
  }
  const step = TAB_STEP[event.key];
  const target = event.target as HTMLElement | null;
  if (!step || !target?.closest?.('[role="tab"]')) return;
  event.preventDefault();
  settle(controller, edit.stepOperation(controller, step));
}

/// Without a `<form>` element there is no implicit submit, so the two places a
/// user types an answer get it back explicitly.
function handleEnter(controller: AppController, event: KeyboardEvent): void {
  const node = event.target as FieldNode | null;
  const action = node?.dataset?.event;
  if (node?.id === "base-choice") {
    event.preventDefault();
    settle(controller, controller.chooseBase(node.value));
    return;
  }
  const withModifier = event.ctrlKey || event.metaKey;
  const submitting =
    action === "path-filter" ||
    action === "split-branch-name" ||
    ((action === "commit-message" || action === "split-message") && withModifier);
  if (!submitting) return;
  event.preventDefault();
  settle(controller, controller.submitOperation());
}

/// A rejected handler would otherwise be an unhandled rejection and the control
/// would silently look inert, so every delegated action reports its own failure.
function settle(controller: AppController, result: unknown): void {
  if (!result || typeof (result as { catch?: unknown }).catch !== "function") return;
  (result as Promise<unknown>).catch((error: unknown) => {
    controller.fail(error);
    controller.render();
  });
}

const baseChoice = (): string =>
  (globalThis.document?.querySelector("#base-choice") as HTMLSelectElement | null)?.value ?? "";
