import * as edit from "./selection.js";

const CLICK = {
  "pick-repository": (controller) => controller.openRepository(),
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
};

const INPUT = {
  "path-filter": edit.setPathFilter,
  "commit-message": edit.setMessage,
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
  const submitting = action === "path-filter" || (action === "commit-message" && (event.ctrlKey || event.metaKey));
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
