import * as branches from "./branch-switcher.ts";
import * as diff from "./files-diff/index.ts";
import * as edit from "./selection.ts";
import * as pathDiff from "./path-diff-menu.ts";
import * as repos from "./repository-switcher.ts";
import { CHANGE, CLICK, INPUT, TAB_STEP } from "./event-tables.ts";
import type { AppController } from "./controller.ts";
import type { FieldNode } from "./types.ts";

type ActionElement = HTMLElement & { disabled?: boolean };

export function bindEvents(controller: AppController): void {
  const target = globalThis.document;
  if (!target) return;
  target.addEventListener("click", /*handleClick=*/ (event) => handleClick(controller, event));
  target.addEventListener("contextmenu", /*handleContextMenu=*/ (event) => handleContextMenu(controller, event));
  target.addEventListener("change", /*handleChange=*/ (event) => dispatchNode(controller, event, CHANGE));
  target.addEventListener("input", /*handleInput=*/ (event) => handleInput(controller, event));
  target.addEventListener("keydown", /*handleKeys=*/ (event) => handleKeys(controller, event));
}

function handleClick(controller: AppController, event: MouseEvent): void {
  const target = event.target as HTMLElement | null;
  dismissOpenOverlays(controller, target);
  const node = target?.closest?.("[data-event]") as ActionElement | null;
  if (!node || node.disabled) return;
  const action = CLICK[node.dataset.event ?? ""];
  if (!action) return;
  event.preventDefault();
  settle(controller, action(controller, node.dataset.value ?? "", node));
}

function dismissOpenOverlays(controller: AppController, target: HTMLElement | null): void {
  if (controller.state.repoContextMenu && !target?.closest?.(".repo-context-menu")) {
    repos.closeRepoContextMenu(controller);
  }
  if (controller.state.pathContextMenu && !target?.closest?.(".path-context-menu")) {
    pathDiff.closePathContextMenu(controller);
  }
  if (controller.state.repoMenuOpen && !target?.closest?.(".repo-switcher")) {
    repos.closeRepoMenu(controller);
  }
  if (controller.state.draft.branchMenuOpen && !target?.closest?.(".branch-picker")) {
    branches.closeBranchMenu(controller);
  }
  if (controller.state.diffView.untrackedFiltersOpen && !target?.closest?.(".untracked-filters")) {
    diff.closeUntrackedFilters(controller);
  }
}

function handleContextMenu(controller: AppController, event: MouseEvent): void {
  const target = event.target as HTMLElement | null;
  const pathRow = target?.closest?.("[data-path-context]") as HTMLElement | null;
  const path = pathRow?.dataset?.pathContext ?? "";
  if (path) {
    event.preventDefault();
    pathDiff.openPathContextMenu(controller, path, event.clientX, event.clientY);
    return;
  }
  const row = target?.closest?.(".repo-row") as HTMLElement | null;
  const picker = target?.closest?.(".repo-picker") as HTMLElement | null;
  const repoPath = row?.dataset?.contextPath ?? picker?.dataset?.contextPath ?? "";
  if (!repoPath) return;
  event.preventDefault();
  repos.openRepoContextMenu(controller, repoPath, event.clientX, event.clientY);
}

function handleInput(controller: AppController, event: Event): void {
  if ((event as InputEvent).isComposing) return;
  dispatchNode(controller, event, INPUT);
}

function dispatchNode(
  controller: AppController,
  event: Event,
  table: Record<string, (controller: AppController, node: FieldNode) => unknown>,
): void {
  const node = event.target as FieldNode | null;
  const action = table[node?.dataset?.event ?? ""];
  if (!action || !node) return;
  settle(controller, action(controller, node));
}

/// A menu's own module owns its keyboard map. A handled key returns a truthy
/// value — sometimes the promise it started — so `settle` still reports a
/// rejected activation here rather than in two more places.
function handleKeys(controller: AppController, event: KeyboardEvent): void {
  if (dismissContextMenusOnEscape(controller, event)) return;
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

function dismissContextMenusOnEscape(controller: AppController, event: KeyboardEvent): boolean {
  if (event.key !== "Escape") return false;
  if (controller.state.pathContextMenu) {
    event.preventDefault();
    pathDiff.closePathContextMenu(controller);
    return true;
  }
  if (controller.state.repoContextMenu) {
    event.preventDefault();
    repos.closeRepoContextMenu(controller);
    return true;
  }
  return false;
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
