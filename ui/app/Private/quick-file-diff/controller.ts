import { TauriBridge } from "../bridge.ts";
import { renderInto } from "../dom.ts";
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

const errorMessage = (error: unknown): string => {
  const message = (error as { message?: unknown } | null | undefined)?.message;
  return message == null ? String(error) : String(message);
};

export class QuickFileDiffApp {
  readonly bridge: Bridge;
  readonly state: QuickDiffState;

  constructor(bridge: Bridge = new TauriBridge()) {
    this.bridge = bridge;
    this.state = createQuickDiffState();
  }

  async start(): Promise<void> {
    bindEvents(this);
    listenForReload(this);
    await this.reload();
  }

  render(): void {
    const root = globalThis.document?.querySelector("#app");
    if (root) renderInto(root, quickDiffView(this.state));
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

  async run(work: () => void | Promise<void>): Promise<void> {
    this.state.busy = true;
    this.state.error = "";
    this.render();
    try {
      await work();
    } catch (error) {
      this.state.error = errorMessage(error);
    } finally {
      this.state.busy = false;
      this.render();
    }
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

function bindEvents(app: QuickFileDiffApp): void {
  const target = globalThis.document;
  if (!target) return;
  target.addEventListener("click", /*handleClick=*/ (event) => {
    const node = (event.target as HTMLElement | null)?.closest?.("[data-event]") as
      | (HTMLElement & { disabled?: boolean })
      | null;
    if (!node || node.disabled) return;
    const action = CLICK[node.dataset.event ?? ""];
    if (!action) return;
    event.preventDefault();
    settle(app, action(app, node.dataset.value ?? ""));
  });
}

function listenForReload(app: QuickFileDiffApp): void {
  const listen = globalThis.__TAURI__?.event?.listen;
  if (typeof listen !== "function") return;
  void listen("file-diff-reload", /*reloadSession=*/ () => {
    settle(app, app.reload());
  });
}

function settle(app: QuickFileDiffApp, result: unknown): void {
  if (!result || typeof (result as { catch?: unknown }).catch !== "function") return;
  (result as Promise<unknown>).catch((error: unknown) => {
    app.state.error = errorMessage(error);
    app.render();
  });
}
