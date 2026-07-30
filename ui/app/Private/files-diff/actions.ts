import { revealByDataset } from "../dom.ts";
import { loadViewData } from "../discovery.ts";
import { ensureFullDiff } from "./load.ts";
import type { AppController } from "../controller.ts";
import type { AppState } from "../types.ts";
import type { GapReveal } from "./wire.ts";

const EXPAND_STEP = 20;
const NO_REVEAL: GapReveal = { down: 0, up: 0, all: false };

interface GapTarget {
  path: string;
  index: number;
  direction: string;
}

export function setLayout(controller: AppController, value: string): void {
  controller.state.diffView.layout = value === "split" ? "split" : "unified";
  controller.render();
}

export function setCompare(controller: AppController, value: string): Promise<void> {
  const compare = value === "local" ? "local" : "head";
  if (controller.state.diffView.compare === compare) return Promise.resolve();
  controller.state.diffView.compare = compare;
  return controller.run(() => loadViewData(controller));
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
    widenReveal(controller.state, target);
  });
}

function gapTarget(path: string, node?: HTMLElement): GapTarget | null {
  const index = Number(node?.dataset.gap ?? "");
  const direction = node?.dataset.dir ?? "";
  if (!Number.isInteger(index) || !direction) return null;
  return { path, index, direction };
}

function widenReveal(state: AppState, target: GapTarget): void {
  const reveals = state.diffView.reveals;
  let byIndex = reveals.get(target.path);
  if (!byIndex) {
    byIndex = new Map();
    reveals.set(target.path, byIndex);
  }
  byIndex.set(target.index, widened(byIndex.get(target.index) ?? NO_REVEAL, target.direction));
}

/// Overshoot needs no clamp: once the two blocks cover the gap, `gapWindow`
/// renders it whole and drops the expander.
function widened(reveal: GapReveal, direction: string): GapReveal {
  if (direction === "all") return { ...reveal, all: true };
  if (direction === "up") return { ...reveal, up: reveal.up + EXPAND_STEP };
  return { ...reveal, down: reveal.down + EXPAND_STEP };
}
