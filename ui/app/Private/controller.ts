import { TauriBridge } from "./bridge.ts";
import { loadOperationData, loadViewData, reloadState } from "./discovery.ts";
import { focusNode, renderInto } from "./dom.ts";
import { createDraft } from "./draft.ts";
import { bindEvents } from "./events.ts";
import { buildRequest, submitState } from "./operations.ts";
import { loadRecentRepositories } from "./repository-switcher.ts";
import { renderShell } from "./views/shell.ts";
import type {
  AppState,
  Bridge,
  OperationId,
  OperationOutcome,
  OperationRequest,
  OperationReview,
  RepositorySnapshot,
  ViewId,
} from "./types.ts";

const errorMessage = (error: unknown): string => {
  const message = (error as { message?: unknown } | null | undefined)?.message;
  return message == null ? String(error) : String(message);
};

export class AppController {
  readonly bridge: Bridge;
  readonly state: AppState;
  private diffCopiedTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(bridge: Bridge = new TauriBridge()) {
    this.bridge = bridge;
    this.state = {
      view: "actions",
      operation: "uncommit",
      snapshot: null,
      baseChoices: [],
      paths: [],
      commits: [],
      branches: [],
      submodules: [],
      saved: [],
      operations: [],
      branchDiff: null,
      diffCopied: false,
      recentRepositories: [],
      repoMenuOpen: false,
      repoFilter: "",
      repoHighlight: 0,
      repoOpeningPath: "",
      draft: createDraft(),
      expanded: new Set(),
      review: null,
      outcome: null,
      changingBase: false,
      busy: false,
      error: "",
    };
  }

  async start(): Promise<void> {
    bindEvents(this);
    await loadRecentRepositories(this);
    await this.refresh();
  }

  render(): void {
    const root = globalThis.document?.querySelector("#app");
    if (root) renderInto(root, renderShell(this.state));
  }

  announce(message: string): void {
    const node = globalThis.document?.querySelector("#announcer");
    if (node) node.textContent = message;
  }

  fail(error: unknown): void {
    this.state.error = errorMessage(error);
    this.announce(this.state.error);
  }

  async run(work: () => void | Promise<void>): Promise<void> {
    this.state.busy = true;
    this.render();
    try {
      await work();
    } catch (error) {
      this.fail(error);
    } finally {
      this.state.busy = false;
      this.render();
    }
  }

  refresh(snapshot: RepositorySnapshot | null = null): Promise<void> {
    return this.run(() => this.reload(snapshot));
  }

  reload(snapshot: RepositorySnapshot | null = null, preservedError = ""): Promise<void> {
    return reloadState(this, snapshot, preservedError);
  }

  async selectOperation(operation: OperationId): Promise<void> {
    if (this.state.operation === operation || this.state.busy) return;
    await this.cancelReview();
    this.state.operation = operation;
    this.state.outcome = null;
    this.state.error = "";
    await this.run(() => loadOperationData(this));
  }

  async setView(view: ViewId): Promise<void> {
    if (this.state.view === view) return;
    await this.cancelReview();
    this.state.view = view;
    this.state.error = "";
    if (view === "inspection") {
      this.state.branchDiff = null;
      this.state.diffCopied = false;
      await this.run(() => loadViewData(this));
      return;
    }
    this.render();
  }

  async prepare(request: OperationRequest): Promise<void> {
    await this.run(async () => {
      this.state.outcome = null;
      this.state.error = "";
      this.state.review = await this.bridge.invoke<OperationReview>("prepare_operation", { request });
      this.announce(`${this.state.review.title}. Review the plan, then apply it.`);
    });
    if (this.state.review) focusNode("#review-title");
  }

  submitOperation(): Promise<void> {
    const { disabled, reason } = submitState(this.state);
    if (!disabled) return this.prepare(buildRequest(this.state));
    if (reason) this.fail(new Error(reason));
    this.render();
    return Promise.resolve();
  }

  async applyReview(): Promise<void> {
    const review = this.state.review;
    if (!review || this.state.busy) return;
    await this.run(() => this.applyPlan(review.plan_id));
  }

  async applyPlan(planId: string): Promise<void> {
    try {
      const outcome = await this.bridge.invoke<OperationOutcome>("apply_operation", { planId });
      this.state.review = null;
      this.state.outcome = outcome;
      await this.reload();
      this.announce(outcome.headline);
    } catch (error) {
      const message = errorMessage(error);
      this.state.review = null;
      this.state.outcome = null;
      await this.reload(null, message).catch(() => {
        this.state.error = message;
      });
      this.announce(this.state.error);
    }
  }

  /// A cancelled or already-consumed plan is gone either way, so a rejected
  /// cancel must not surface as a failure the user has to dismiss.
  async cancelReview(): Promise<void> {
    const review = this.state.review;
    if (!review) return;
    this.state.review = null;
    this.render();
    await this.bridge.invoke("cancel_operation", { planId: review.plan_id }).catch(() => {});
  }

  async chooseBase(value: string): Promise<void> {
    if (!value || this.state.busy) return;
    await this.run(async () => {
      const snapshot = await this.bridge.invoke<RepositorySnapshot>("set_base", { request: { base: value } });
      this.state.changingBase = false;
      await this.reload(snapshot);
      this.announce(`Base is now ${value}`);
    });
  }

  async switchTo(branch: string): Promise<void> {
    this.state.operation = "quick_switch";
    this.state.draft.targetBranch = branch;
    this.state.view = "actions";
    await this.prepare({ kind: "quick_switch", target_branch: branch });
  }

  async copyDiff(): Promise<void> {
    await this.copy(this.state.branchDiff, "Diff copied to the clipboard");
    this.state.diffCopied = true;
    this.render();
    clearTimeout(this.diffCopiedTimer);
    this.diffCopiedTimer = setTimeout(/*clearDiffCopied=*/ () => {
      if (!this.state.diffCopied) return;
      this.state.diffCopied = false;
      this.render();
    }, /*delayInMs=*/ 2000);
  }

  async copy(value: string | null, message = "Command copied to the clipboard"): Promise<void> {
    const clipboard = globalThis.navigator?.clipboard;
    if (!clipboard?.writeText) throw new Error("Clipboard access is unavailable.");
    await clipboard.writeText(value ?? "");
    this.announce(message);
    this.render();
  }
}
