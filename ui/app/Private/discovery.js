import { adoptBranch, adoptCommit, adoptPaths, adoptSubmodule } from "./draft.js";
import { discoveryFor } from "./operations.js";
import { baseRef } from "./snapshot.js";

/// A failed snapshot load closes the repository, but a failed discovery call
/// must not: the overview is still valid and the operation panel says why.
export async function reloadState(controller, snapshot = null, preservedError = "") {
  const state = controller.state;
  state.snapshot = await readSnapshot(controller, snapshot);
  state.saved = state.snapshot.saved_work ?? [];
  state.operations = state.snapshot.operations ?? [];
  state.error = preservedError;
  await Promise.all([loadOperationData(controller), loadViewData(controller)]);
}

async function readSnapshot(controller, snapshot) {
  try {
    return snapshot ?? (await controller.bridge.invoke("load_snapshot"));
  } catch (error) {
    Object.assign(controller.state, { snapshot: null, saved: [], operations: [] });
    throw error;
  }
}

export async function loadOperationData(controller) {
  const state = controller.state;
  const base = baseRef(state);
  if (!base) state.baseChoices = await controller.bridge.invoke("list_base_choices");
  const discovery = discoveryFor(state.operation);
  if (!discovery || (discovery.needsBase && !base)) return;
  state[discovery.key] = await discovery.load(controller.bridge, base);
  adoptSelections(state);
}

export async function loadViewData(controller) {
  const state = controller.state;
  if (state.view !== "inspection") return;
  state.diffCopied = false;
  state.branchDiff = null;
  const base = baseRef(state);
  if (!base) return;
  state.branchDiff = await controller.bridge.invoke("generate_branch_diff", { request: { base } });
}

function adoptSelections(state) {
  adoptPaths(state.draft, state.paths);
  adoptCommit(state.draft, state.commits);
  adoptSubmodule(state.draft, state.submodules);
  adoptBranch(state.draft, state.branches);
}
