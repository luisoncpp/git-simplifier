import { revealByDataset } from "../dom.ts";
import { visibleFileDiffs } from "./filters.ts";
import { gapTarget, widenReveal } from "./gap.ts";
import { ensureFullDiff } from "./load.ts";
import type { UntrackedFilters } from "./wire.ts";
import type { AppController } from "../controller.ts";
import type { FieldNode } from "../types.ts";

export function setLayout(controller: AppController, value: string): void {
  controller.state.diffView.layout = value === "split" ? "split" : "unified";
  controller.render();
}

export function setCompare(controller: AppController, value: string): Promise<void> {
  const compare = value === "local" ? "local" : "head";
  if (controller.state.diffView.compare === compare) return Promise.resolve();
  controller.state.diffView.compare = compare;
  return controller.run(() => controller.reloadViewData());
}

export function toggleFile(controller: AppController, path: string): void {
  const collapsed = controller.state.diffView.collapsed;
  if (!collapsed.delete(path)) collapsed.add(path);
  controller.render();
}

export function setAllFiles(controller: AppController, mode: string): void {
  const state = controller.state;
  const files = visibleFileDiffs(state.fileDiffs ?? [], state.diffView);
  state.diffView.collapsed.clear();
  if (mode === "collapsed") {
    for (const file of files) state.diffView.collapsed.add(file.path);
  }
  controller.render();
}

export function toggleUntrackedFilters(controller: AppController): void {
  controller.state.diffView.untrackedFiltersOpen = !controller.state.diffView.untrackedFiltersOpen;
  controller.render();
}

export function closeUntrackedFilters(controller: AppController): void {
  if (!controller.state.diffView.untrackedFiltersOpen) return;
  controller.state.diffView.untrackedFiltersOpen = false;
  controller.render();
}

export function toggleUntrackedFilter(controller: AppController, node: FieldNode): Promise<void> {
  const key = node.dataset.value as keyof UntrackedFilters;
  const filters = controller.state.diffView.untrackedFilters;
  if (Object.hasOwn(filters, key)) filters[key] = (node as HTMLInputElement).checked;
  if (controller.state.diffView.compare !== "local") {
    controller.render();
    return Promise.resolve();
  }
  // Filters constrain discovery in Git; revealing a wider set needs a fresh list.
  return controller.run(/*reloadConstrainedUntracked=*/ async () => {
    await controller.reloadViewData();
    await hydrateVisibleStubsInPlace(controller);
  });
}

/// Gitignored / node_modules list entries arrive as empty stubs. When a filter
/// reveal leaves them visible, fetch bodies the same way gap expansion does.
async function hydrateVisibleStubsInPlace(controller: AppController): Promise<void> {
  const state = controller.state;
  const stubs = visibleFileDiffs(state.fileDiffs ?? [], state.diffView).filter(
    (file) => file.untracked && !file.complete && !file.hunks.length,
  );
  for (const file of stubs) await ensureFullDiff(controller, file.path);
}

export function toggleNavigator(controller: AppController): void {
  const view = controller.state.diffView;
  view.navigatorOpen = !view.navigatorOpen;
  controller.render();
}

/// A jump to a closed file opens it. The scroll has to happen *after* `render()`
/// because `renderInto` restores each container's previous offset synchronously
/// after the markup swap, so an earlier scroll — or a `#fragment` — loses the race.
export function jumpToFile(controller: AppController, path: string): void {
  controller.state.diffView.collapsed.delete(path);
  controller.render();
  revealByDataset("file", path);
}

/// The file's full context is fetched before the reveal is recorded, so no frame
/// ever claims lines it cannot render.
export function expandGap(
  controller: AppController,
  path: string,
  node?: HTMLElement,
): Promise<void> {
  const target = gapTarget(path, node);
  if (!target) return Promise.resolve();
  return controller.run(/*revealGapLines=*/ async () => {
    await ensureFullDiff(controller, target.path);
    widenReveal(controller.state.diffView.reveals, target);
  });
}
