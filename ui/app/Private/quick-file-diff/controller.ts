import { TauriBridge } from "../bridge.ts";
import { bindClickEvents, listenForReload, renderApp, runBusy } from "../secondary-diff/shell.ts";
import { ensureGrammars, languageFor } from "../files-diff/index.ts";
import type { Bridge } from "../types.ts";
import type { DiffCompare, FileDiff } from "../files-diff/index.ts";
import { createQuickDiffState } from "./state.ts";
import { quickDiffView } from "./view.ts";
import type { FileDiffSession, QuickDiffState } from "./types.ts";

type ClickHandler = (app: QuickFileDiffApp, value: string) => unknown;

const CLICK: Record<string, ClickHandler> = {
  "set-diff-layout": (app, value) => app.setLayout(value),
  "set-diff-compare": (app, value) => app.setCompare(value),
};

export class QuickFileDiffApp {
  readonly bridge: Bridge;
  readonly state: QuickDiffState;

  constructor(bridge: Bridge = new TauriBridge()) {
    this.bridge = bridge;
    this.state = createQuickDiffState();
  }

  async start(): Promise<void> {
    bindClickEvents(this, CLICK);
    listenForReload(this, "file-diff-reload", /*reload=*/ () => this.reload());
    await this.reload();
  }

  render(): void {
    renderApp("#app", quickDiffView(this.state));
  }

  setLayout(value: string): void {
    this.state.view.layout = value === "split" ? "split" : "unified";
    this.render();
  }

  setCompare(value: string): Promise<void> {
    const compare: DiffCompare = value === "local" ? "local" : "head";
    if (!this.state.session || this.state.view.compare === compare) return Promise.resolve();
    this.state.session.compare = compare;
    this.state.view.compare = compare;
    return this.run(() => this.loadFile());
  }

  async reload(): Promise<void> {
    await this.run(async () => {
      this.state.session = await this.bridge.invoke<FileDiffSession>("file_diff_session");
      this.state.view.compare = this.state.session.compare;
      await this.loadFile();
    });
  }

  run(work: () => void | Promise<void>): Promise<void> {
    return runBusy(this, work);
  }

  private async loadFile(): Promise<void> {
    const session = this.state.session;
    if (!session) return;
    const file = await this.bridge.invoke<FileDiff | null>("generate_full_file_diff", {
      request: { base: session.base, path: session.path, compare: session.compare },
    });
    this.state.file = file;
    if (file) await ensureGrammars([languageFor(file.path)]);
  }
}
