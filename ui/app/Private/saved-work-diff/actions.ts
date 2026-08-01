import { revealByDataset } from "../dom.ts";
import { gapTarget, widenReveal } from "../files-diff/gap.ts";
import { ensureFullDiff } from "./load.ts";
import type { SavedWorkDiffApp } from "./controller.ts";

export function setLayout(app: SavedWorkDiffApp, value: string): void {
  app.state.diffView.layout = value === "split" ? "split" : "unified";
  app.render();
}

export function toggleFile(app: SavedWorkDiffApp, path: string): void {
  const collapsed = app.state.diffView.collapsed;
  if (!collapsed.delete(path)) collapsed.add(path);
  app.render();
}

export function setAllFiles(app: SavedWorkDiffApp, mode: string): void {
  app.state.diffView.collapsed.clear();
  if (mode === "collapsed") {
    for (const file of app.state.files ?? []) app.state.diffView.collapsed.add(file.path);
  }
  app.render();
}

export function toggleNavigator(app: SavedWorkDiffApp): void {
  app.state.diffView.navigatorOpen = !app.state.diffView.navigatorOpen;
  app.render();
}

export function jumpToFile(app: SavedWorkDiffApp, path: string): void {
  app.state.diffView.collapsed.delete(path);
  app.render();
  revealByDataset("file", path);
}

export function expandGap(app: SavedWorkDiffApp, path: string, node?: HTMLElement): Promise<void> {
  const target = gapTarget(path, node);
  if (!target) return Promise.resolve();
  return app.run(/*revealGapLines=*/ async () => {
    await ensureFullDiff(app.bridge, app.state, target.path);
    widenReveal(app.state.diffView.reveals, target);
  });
}
