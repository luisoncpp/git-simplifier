import * as edit from "./selection.js";
import * as repos from "./repository-switcher.js";

const CLICK = {
  "toggle-repo-menu": (controller) => repos.toggleRepoMenu(controller),
  "pick-repository": (controller) => repos.openPickedRepository(controller),
  "open-recent": (controller, value) => repos.openRecentRepository(controller, value),
  "remove-recent": (controller, value) => repos.removeRecentRepository(controller, value),
  refresh: (controller) => controller.refresh(),
  "set-view": (controller, value) => controller.setView(value),
  "set-operation": (controller, value) => controller.selectOperation(value),
  "submit-operation": (controller) => controller.submitOperation(),
  "cancel-review": (controller) => controller.cancelReview(),
  "apply-review": (controller) => controller.applyReview(),
  "edit-base": (controller) => edit.setChangingBase(controller, "true"),
  "cancel-base": (controller) => edit.setChangingBase(controller, "false"),
  "save-base": (controller) => controller.chooseBase(baseChoice()),
  "force-push": (controller) => controller.prepare({ kind: "force_push" }),
  "publish-branch": (controller, value) => controller.prepare({ kind: "publish_branch", branch: value }),
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
};

const CHANGE = {
  "select-commit": edit.setCommit,
  "select-submodule": edit.setSubmodule,
  "select-branch": edit.setTargetBranch,
  "toggle-path": edit.togglePath,
  "toggle-install-hook": edit.setInstallHook,
  "toggle-disable-recurse": edit.setDisableRecurse,
  "toggle-carry-changes": edit.setCarryChanges,
};

const INPUT = {
  "path-filter": edit.setPathFilter,
  "repo-filter": repos.setRepoFilter,
  "commit-message": edit.setMessage,
  "split-branch-name": edit.setNewBranch,
  "split-message": edit.setSplitMessage,
};

const TAB_STEP = { ArrowRight: 1, ArrowLeft: -1 };

export function bindEvents(controller) {
  const target = globalThis.document;
  if (!target) return;
  target.addEventListener("click", /*handleClick=*/ (event) => handleClick(controller, event));
  target.addEventListener("change", /*handleChange=*/ (event) => dispatchNode(controller, event, CHANGE));
  target.addEventListener("input", /*handleInput=*/ (event) => handleInput(controller, event));
  target.addEventListener("keydown", /*handleKeys=*/ (event) => handleKeys(controller, event));
}

function handleClick(controller, event) {
  if (controller.state.repoMenuOpen && !event.target.closest?.(".repo-switcher")) {
    repos.closeRepoMenu(controller);
  }
  const node = event.target.closest?.("[data-event]");
  const action = node && !node.disabled ? CLICK[node.dataset.event] : null;
  if (!action) return;
  event.preventDefault();
  settle(controller, action(controller, node.dataset.value ?? ""));
}

function handleInput(controller, event) {
  if (event.isComposing) return;
  dispatchNode(controller, event, INPUT);
}

function dispatchNode(controller, event, table) {
  const action = table[event.target?.dataset?.event];
  if (!action) return;
  settle(controller, action(controller, event.target));
}

function handleKeys(controller, event) {
  if (handleRepoKeys(controller, event)) return;
  if (event.key === "Escape" && controller.state.review) {
    settle(controller, controller.cancelReview());
    return;
  }
  if (event.key === "Enter") {
    handleEnter(controller, event);
    return;
  }
  const step = TAB_STEP[event.key];
  if (!step || !event.target.closest?.('[role="tab"]')) return;
  event.preventDefault();
  settle(controller, edit.stepOperation(controller, step));
}

function handleRepoKeys(controller, event) {
  if (!controller.state.repoMenuOpen) return false;
  if (event.key === "Escape") {
    event.preventDefault();
    repos.closeRepoMenu(controller);
    return true;
  }
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    repos.moveRepoHighlight(controller, event.key === "ArrowDown" ? 1 : -1);
    return true;
  }
  if (event.key === "Enter" && event.target?.dataset?.event === "repo-filter") {
    event.preventDefault();
    settle(controller, repos.activateHighlightedRepository(controller));
    return true;
  }
  return false;
}

/// Without a `<form>` element there is no implicit submit, so the two places a
/// user types an answer get it back explicitly.
function handleEnter(controller, event) {
  const node = event.target;
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
function settle(controller, result) {
  if (!result?.catch) return;
  result.catch((error) => {
    controller.fail(error);
    controller.render();
  });
}

const baseChoice = () => globalThis.document?.querySelector("#base-choice")?.value ?? "";
