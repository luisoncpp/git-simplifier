import { TauriBridge } from "./bridge.js";
import { loadOperationData, reloadState } from "./discovery.js";
import { focusNode, renderInto } from "./dom.js";
import { createDraft } from "./draft.js";
import { bindEvents } from "./events.js";
import { buildRequest, submitState } from "./operations.js";
import { loadRecentRepositories } from "./repository-switcher.js";
import { renderShell } from "./views/shell.js";

export class AppController {
  constructor(bridge = new TauriBridge()) {
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

  async start() {
    bindEvents(this);
    await loadRecentRepositories(this);
    await this.refresh();
  }

  render() {
    const root = globalThis.document?.querySelector("#app");
    if (root) renderInto(root, renderShell(this.state));
  }

  announce(message) {
    const node = globalThis.document?.querySelector("#announcer");
    if (node) node.textContent = message;
  }

  fail(error) {
    this.state.error = error?.message ?? String(error);
    this.announce(this.state.error);
  }

  async run(work) {
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

  refresh(snapshot = null) {
    return this.run(() => this.reload(snapshot));
  }

  reload(snapshot = null, preservedError = "") {
    return reloadState(this, snapshot, preservedError);
  }

  async selectOperation(operation) {
    if (this.state.operation === operation || this.state.busy) return;
    await this.cancelReview();
    this.state.operation = operation;
    this.state.outcome = null;
    this.state.error = "";
    await this.run(() => loadOperationData(this));
  }

  async setView(view) {
    if (this.state.view === view) return;
    await this.cancelReview();
    this.state.view = view;
    this.render();
  }

  async prepare(request) {
    await this.run(async () => {
      this.state.outcome = null;
      this.state.error = "";
      this.state.review = await this.bridge.invoke("prepare_operation", { request });
      this.announce(`${this.state.review.title}. Review the plan, then apply it.`);
    });
    if (this.state.review) focusNode("#review-title");
  }

  submitOperation() {
    const { disabled, reason } = submitState(this.state);
    if (!disabled) return this.prepare(buildRequest(this.state));
    if (reason) this.fail(new Error(reason));
    this.render();
    return Promise.resolve();
  }

  async applyReview() {
    const review = this.state.review;
    if (!review || this.state.busy) return;
    await this.run(() => this.applyPlan(review.plan_id));
  }

  async applyPlan(planId) {
    try {
      const outcome = await this.bridge.invoke("apply_operation", { planId });
      this.state.review = null;
      this.state.outcome = outcome;
      await this.reload();
      this.announce(outcome.headline);
    } catch (error) {
      const message = error?.message ?? String(error);
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
  async cancelReview() {
    const review = this.state.review;
    if (!review) return;
    this.state.review = null;
    this.render();
    await this.bridge.invoke("cancel_operation", { planId: review.plan_id }).catch(() => {});
  }

  async chooseBase(value) {
    if (!value || this.state.busy) return;
    await this.run(async () => {
      const snapshot = await this.bridge.invoke("set_base", { request: { base: value } });
      this.state.changingBase = false;
      await this.reload(snapshot);
      this.announce(`Base is now ${value}`);
    });
  }

  async switchTo(branch) {
    this.state.operation = "quick_switch";
    this.state.draft.targetBranch = branch;
    this.state.view = "actions";
    await this.prepare({ kind: "quick_switch", target_branch: branch });
  }

  async copy(value) {
    await globalThis.navigator?.clipboard?.writeText(value);
    this.announce("Command copied to the clipboard");
    this.render();
  }
}
