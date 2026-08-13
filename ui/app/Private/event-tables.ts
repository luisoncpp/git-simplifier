import * as branches from "./branch-switcher.ts";
import * as diff from "./files-diff/index.ts";
import * as edit from "./selection.ts";
import * as pathDiff from "./path-diff-menu.ts";
import * as savedDiff from "./saved-work-diff/index.ts";
import * as prefs from "./preferences.ts";
import * as project from "./project-settings.ts";
import * as repos from "./repository-switcher.ts";
import type { AppController } from "./controller.ts";
import type { FieldNode, OperationId, ViewId } from "./types.ts";

export type ClickHandler = (
  controller: AppController,
  value: string,
  node?: HTMLElement,
) => unknown;
export type FieldHandler = (controller: AppController, node: FieldNode) => unknown;

export const CLICK: Record<string, ClickHandler> = {
  "toggle-repo-menu": (controller) => repos.toggleRepoMenu(controller),
  "pick-repository": (controller) => repos.openPickedRepository(controller),
  "open-recent": (controller, value) => repos.openRecentRepository(controller, value),
  "remove-recent": (controller, value) => repos.removeRecentRepository(controller, value),
  "copy-repository-path": (controller, value) => repos.copyRepositoryPath(controller, value),
  "reveal-repository": (controller, value) => repos.revealRepository(controller, value),
  "open-in-ide": (controller, value) => repos.openRepositoryInIde(controller, value),
  "open-in-codechart": (controller, value) => repos.openRepositoryInCodechart(controller, value),
  "view-path-diff": (controller, value) => pathDiff.openPathDiff(controller, value),
  "edit-path-in-ide": (controller, value) => pathDiff.openPathInIde(controller, value),
  "toggle-branch-menu": (controller) => branches.toggleBranchMenu(controller),
  "pick-branch": (controller, value, node) =>
    branches.pickBranch(controller, value, node?.dataset.remote ?? ""),
  refresh: (controller) => controller.refresh(),
  "cancel-fetch": (controller) => controller.cancelFetch(),
  "set-skip-review": (controller, value) => prefs.setSkipReview(controller, value === "true"),
  "set-view": (controller, value) => controller.setView(value as ViewId),
  "set-operation": (controller, value) => controller.selectOperation(value as OperationId),
  "submit-operation": (controller) => controller.submitOperation(),
  "submit-exclude-submodule": (controller) => controller.submitExcludeSubmodule(),
  "submit-cleanup-submodules": (controller) => controller.submitCleanupSubmodules(),
  "cancel-review": (controller) => controller.cancelReview(),
  "apply-review": (controller) => controller.applyReview(),
  "edit-base": (controller) => controller.editBase(),
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
  "commit-merge": (controller) => controller.prepare({ kind: "commit_merge" }),
  "restore-saved": (controller) => controller.prepare({ kind: "restore_saved_work" }),
  "delete-saved": (controller, value) => controller.prepare({ kind: "delete_saved_work", branch: value }),
  "switch-to": (controller, value) => controller.switchTo(value),
  "saved-work-diff": (controller, value) => savedDiff.openSavedWorkDiff(controller, value),
  "select-paths": (controller, value) => edit.selectPaths(controller, value),
  "reset-message": (controller) => edit.resetMessage(controller),
  "dismiss-error": (controller) => edit.dismissError(controller),
  "dismiss-warning": (controller) => edit.dismissWarning(controller),
  "dismiss-outcome": (controller) => edit.dismissOutcome(controller),
  "dismiss-saved-work-notice": (controller) => edit.dismissSavedWorkNotice(controller),
  "toggle-entry": (controller, value) => edit.toggleEntry(controller, value),
  copy: (controller, value) => controller.copy(value),
  "copy-diff": (controller) => controller.copyDiff(),
  "set-diff-layout": (controller, value) => diff.setLayout(controller, value),
  "set-diff-compare": (controller, value) => diff.setCompare(controller, value),
  "toggle-file": (controller, value) => diff.toggleFile(controller, value),
  "set-all-files": (controller, value) => diff.setAllFiles(controller, value),
  "expand-gap": (controller, value, node) => diff.expandGap(controller, value, node),
  "toggle-file-navigator": (controller) => diff.toggleNavigator(controller),
  "jump-to-file": (controller, value) => diff.jumpToFile(controller, value),
  "toggle-untracked-filters": (controller) => diff.toggleUntrackedFilters(controller),
};

export const CHANGE: Record<string, FieldHandler> = {
  "select-commit": edit.setCommit,
  "select-submodule": edit.setSubmodule,
  "select-branch": edit.setTargetBranch,
  "toggle-path": edit.togglePath,
  "toggle-cleanup-submodule": edit.toggleCleanupSubmodule,
  "select-revert-target": edit.setRevertTarget,
  "toggle-install-hook": edit.setInstallHook,
  "toggle-disable-recurse": edit.setDisableRecurse,
  "toggle-cleanup-uncommit": edit.setCleanupUncommit,
  "toggle-cleanup-revert": edit.setCleanupRevert,
  "toggle-carry-changes": edit.setCarryChanges,
  "toggle-pull-after-switch": branches.setPullAfterSwitch,
  "toggle-cleanup-only-mine": edit.setCleanupOnlyMine,
  "toggle-cleanup-remotes": edit.setCleanupRemotes,
  "toggle-cleanup-all-remote": edit.setCleanupAllRemote,
  "toggle-cleanup-branch": edit.toggleCleanupBranch,
  "toggle-untracked-filter": diff.toggleUntrackedFilter,
  "select-ide": (controller, node) => project.setIdeKind(controller, node.value),
};

export const INPUT: Record<string, FieldHandler> = {
  "path-filter": edit.setPathFilter,
  "repo-filter": repos.setRepoFilter,
  "branch-filter": branches.setBranchFilter,
  "commit-message": edit.setMessage,
  "split-branch-name": edit.setNewBranch,
  "split-message": edit.setSplitMessage,
  "cleanup-filter": edit.setCleanupFilter,
  "custom-ide-command": (controller, node) => project.setCustomIdeCommand(controller, node.value),
  "codechart-path": (controller, node) => prefs.setCodechartPath(controller, node.value),
};

export const TAB_STEP: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };

const baseChoice = (): string =>
  (globalThis.document?.querySelector("#base-choice") as HTMLSelectElement | null)?.value ?? "";
