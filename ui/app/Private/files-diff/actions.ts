import { revealByDataset } from "../dom.ts";
import { gapTarget, widenReveal } from "./gap.ts";
import { ensureFullDiff } from "./load.ts";
import type { AppController } from "../controller.ts";

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
  state.diffView.collapsed.clear();
  if (mode === "collapsed") {
    for (const file of state.fileDiffs ?? []) state.diffView.collapsed.add(file.path);
  }
  controller.render();
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
