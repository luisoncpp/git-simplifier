import { adoptBranch, adoptCleanup, adoptCommit, adoptDirtySubmodules, adoptPaths, adoptSubmodule } from "./draft/index.ts";
import { loadFileDiffs } from "./files-diff/index.ts";
import { discoveryFor } from "./operations/index.ts";
import { baseRef, currentBranch, savedWorkFor } from "./snapshot.ts";
import { isInspectionView } from "./state.ts";
import type { AppController } from "./controller.ts";
import type { AppState, BaseChoice, RepositorySnapshot } from "./types.ts";

const invokeError = (error: unknown): string => {
  const message = (error as { message?: unknown } | null | undefined)?.message;
  return message == null ? String(error) : String(message);
};

/// Refresh fetches every configured remote before reloading local discovery
/// data. A failed fetch is surfaced as a warning, not a blocking error.
export async function fetchRemotes(controller: AppController): Promise<string> {
  try {
    await controller.bridge.invoke("fetch_remotes");
    return "";
  } catch (error) {
    return invokeError(error);
  }
}

/// A failed snapshot load closes the repository, but a failed discovery call
/// must not: the overview is still valid and the operation panel says why.
export async function reloadState(
  controller: AppController,
  snapshot: RepositorySnapshot | null = null,
  preservedError = "",
): Promise<void> {
  const state = controller.state;
  state.snapshot = await readSnapshot(controller, snapshot);
  state.saved = state.snapshot.saved_work ?? [];
  state.operations = state.snapshot.operations ?? [];
  state.error = preservedError;
  if (!savedWorkFor(state, currentBranch(state))) {
    state.dismissedSavedWorkBranch = null;
  }
  await Promise.all([loadOperationData(controller), loadViewData(controller)]);
}

async function readSnapshot(
  controller: AppController,
  snapshot: RepositorySnapshot | null,
): Promise<RepositorySnapshot> {
  try {
    return snapshot ?? (await controller.bridge.invoke<RepositorySnapshot>("load_snapshot"));
  } catch (error) {
    Object.assign(controller.state, { snapshot: null, saved: [], operations: [] });
    throw error;
  }
}

export async function loadOperationData(controller: AppController): Promise<void> {
  const state = controller.state;
  const base = baseRef(state);
  if (!base) await loadBaseChoices(controller);
  if (state.operation === "submodules") {
    await loadSubmodulesData(controller, base);
    return;
  }
  const discovery = discoveryFor(state.operation);
  if (!discovery || (discovery.needsBase && !base)) return;
  const result = await discovery.load(controller.bridge, base);
  Object.assign(state, { [discovery.key]: result });
  adoptSelections(state);
}

async function loadSubmodulesData(controller: AppController, base: string): Promise<void> {
  const state = controller.state;
  const [submodules, dirtySubmodules] = await Promise.all([
    controller.bridge.invoke<import("./types.ts").SubmoduleChoice[]>("list_submodules"),
    controller.bridge.invoke<import("./types.ts").DirtySubmodule[]>("list_dirty_submodules", {
      request: { base: base || undefined },
    }),
  ]);
  state.submodules = submodules;
  state.dirtySubmodules = dirtySubmodules;
  adoptSubmodule(state.draft, submodules);
  adoptDirtySubmodules(state.draft, dirtySubmodules);
}

export async function loadBaseChoices(controller: AppController): Promise<void> {
  controller.state.baseChoices = await controller.bridge.invoke<BaseChoice[]>("list_base_choices");
}

/// Gated per view, so entering one Inspection section never pays for the other's
/// Git work.
export async function loadViewData(controller: AppController): Promise<void> {
  const state = controller.state;
  if (!isInspectionView(state.view)) return;
  const base = baseRef(state);
  if (state.view === "raw-diff") return loadBranchDiff(controller, base);
  return loadFileDiffs(controller, base);
}

async function loadBranchDiff(controller: AppController, base: string): Promise<void> {
  const state = controller.state;
  state.diffCopied = false;
  state.branchDiff = null;
  if (!base) return;
  state.branchDiff = await controller.bridge.invoke<string>("generate_branch_diff", {
    request: { base, compare: state.diffView.compare },
  });
}

function adoptSelections(state: AppState): void {
  adoptPaths(state.draft, state.paths);
  adoptCommit(state.draft, state.commits);
  adoptSubmodule(state.draft, state.submodules);
  adoptBranch(state.draft, state.branches, baseRef(state));
  adoptCleanup(state.draft, state.cleanupBranches);
}
