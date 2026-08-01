import { TauriBridge } from "../bridge.ts";
import { bindClickEvents, listenForReload, renderApp, runBusy } from "../secondary-diff/shell.ts";
import type { Bridge } from "../types.ts";
import * as actions from "./actions.ts";
import { loadFiles } from "./load.ts";
import { createSavedWorkDiffState } from "./state.ts";
import { savedWorkDiffView } from "./view.ts";
import type { SavedWorkDiffSession, SavedWorkDiffState } from "./types.ts";

type ClickHandler = (app: SavedWorkDiffApp, value: string, node?: HTMLElement) => unknown;

const CLICK: Record<string, ClickHandler> = {
  "set-diff-layout": (app, value) => actions.setLayout(app, value),
  "toggle-file": (app, value) => actions.toggleFile(app, value),
  "set-all-files": (app, value) => actions.setAllFiles(app, value),
  "toggle-file-navigator": (app) => actions.toggleNavigator(app),
  "jump-to-file": (app, value) => actions.jumpToFile(app, value),
  "expand-gap": (app, value, node) => actions.expandGap(app, value, node),
};

export class SavedWorkDiffApp {
  readonly bridge: Bridge;
  readonly state: SavedWorkDiffState;

  constructor(bridge: Bridge = new TauriBridge()) {
    this.bridge = bridge;
    this.state = createSavedWorkDiffState();
  }

  async start(): Promise<void> {
    bindClickEvents(this, CLICK);
    listenForReload(this, "saved-work-diff-reload", /*reload=*/ () => this.reload());
    await this.reload();
  }

  render(): void {
    renderApp("#app", savedWorkDiffView(this.state));
  }

  async reload(): Promise<void> {
    await this.run(async () => {
      this.state.session = await this.bridge.invoke<SavedWorkDiffSession>("saved_work_diff_session");
      await loadFiles(this.bridge, this.state);
    });
  }

  run(work: () => void | Promise<void>): Promise<void> {
    return runBusy(this, work);
  }
}
