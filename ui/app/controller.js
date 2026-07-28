import { TauriBridge } from "./bridge.js";
import { shell } from "./views.js";

export class AppController {
  constructor(bridge = new TauriBridge()) { this.bridge = bridge; this.state = { view: "actions", operation: "uncommit", snapshot: null, paths: [], commits: [], branches: [], submodules: [], baseChoices: [], saved: [], operations: [], review: null, outcome: null, busy: false, error: "" }; }
  async start() { this.bind(); await this.refresh(); }
  bind() { document.addEventListener("click", (event) => this.click(event)); document.addEventListener("submit", (event) => this.submit(event)); }
  render() { document.querySelector("#app").innerHTML = shell(this.state); document.querySelector("#workspace-panel")?.focus({ preventScroll: true }); }
  announce(message) { document.querySelector("#announcer").textContent = message; }
  async refresh(snapshot = null, preservedError = "") {
    this.state.busy = true;
    this.render();
    try {
      this.state.snapshot = snapshot ?? await this.bridge.invoke("load_snapshot");
      this.state.saved = this.state.snapshot.saved_work;
      this.state.operations = this.state.snapshot.operations;
      await this.loadOperationData();
      this.state.error = preservedError;
    } catch (error) {
      this.state.snapshot = null;
      this.state.error = error.message ?? String(error);
      this.announce(this.state.error);
    } finally {
      this.state.busy = false;
      this.render();
    }
  }
  async loadOperationData() { const base = this.state.snapshot?.overview?.base?.value ?? this.state.snapshot?.overview?.base; if (!base) this.state.baseChoices = await this.bridge.invoke("list_base_choices"); if (this.state.operation === "uncommit" && base) this.state.paths = await this.bridge.invoke("list_changed_paths", { request: { base } }); if (this.state.operation === "edit_message" && base) this.state.commits = await this.bridge.invoke("list_editable_commits", { request: { base } }); if (this.state.operation === "quick_switch") this.state.branches = await this.bridge.invoke("list_local_branches"); if (this.state.operation === "exclude_submodule") this.state.submodules = await this.bridge.invoke("list_submodules"); }
  async click(event) { const target = event.target.closest("[data-view],[data-operation],[data-event],[data-saved-restore],[data-saved-delete],[data-copy]"); if (!target) return; if (target.dataset.view) { this.state.view = target.dataset.view; this.state.review = null; this.render(); return; } if (target.dataset.operation) return this.selectOperation(target.dataset.operation); if (target.dataset.savedRestore) return this.prepare({ kind: "restore_saved_work" }); if (target.dataset.savedDelete) return this.prepare({ kind: "delete_saved_work", branch: target.dataset.savedDelete, snapshot: target.dataset.savedSnapshot }); if (target.dataset.copy) { await navigator.clipboard?.writeText(target.dataset.copy); this.announce("Recovery command copied"); return; } if (target.dataset.event === "pick-repository") return this.openRepository(); if (target.dataset.event === "refresh") return this.refresh(); if (target.dataset.event === "cancel-review") return this.cancelReview(); if (target.dataset.event === "apply-review") return this.applyReview(); if (target.dataset.event === "choose-base") return this.chooseBase(); if (target.dataset.event === "prepare-force-push") return this.prepare({ kind: "force_push" }); if (target.dataset.event === "prepare-resume") return this.prepare({ kind: "resume_sync" }); }
  async selectOperation(operation) {
    this.state.operation = operation;
    this.state.review = null;
    this.state.error = "";
    this.state.busy = true;
    this.render();
    try {
      await this.loadOperationData();
    } catch (error) {
      this.state.error = error.message ?? String(error);
      this.announce(this.state.error);
    } finally {
      this.state.busy = false;
      this.render();
    }
  }
  async submit(event) { const form = event.target.closest("form[data-form]"); if (!form || this.state.busy) return; event.preventDefault(); const values = Object.fromEntries(new FormData(form)); const kind = form.dataset.form; const base = this.state.snapshot.overview.base?.value ?? this.state.snapshot.overview.base; if (kind === "uncommit") return this.prepare({ kind, base: values.base, paths: [...form.elements.paths.selectedOptions].map((o) => o.value) }); if (kind === "edit_message") return this.prepare({ kind, base, commit: values.commit, message: values.message }); if (kind === "exclude_submodule") return this.prepare({ kind, path: values.path, install_hook: form.elements.install_hook.checked, disable_recurse: form.elements.disable_recurse.checked }); if (kind === "quick_switch") return this.prepare({ kind, target_branch: values.target_branch }); if (kind === "sync") return this.prepare({ kind, base: values.base }); }
  async prepare(request) { this.state.busy = true; this.render(); try { this.state.review = await this.bridge.invoke("prepare_operation", { request }); this.announce("Operation review ready"); } catch (error) { this.state.error = error.message ?? String(error); this.announce(this.state.error); } finally { this.state.busy = false; this.render(); } }
  async applyReview() {
    if (!this.state.review || this.state.busy) return;
    this.state.busy = true;
    this.render();
    try {
      const outcome = await this.bridge.invoke("apply_operation", { planId: this.state.review.plan_id });
      this.state.review = null;
      this.state.outcome = outcome;
      this.announce(outcome.headline);
      await this.refresh();
    } catch (error) {
      const message = error.message ?? String(error);
      this.state.review = null;
      await this.refresh(/*snapshot=*/null, message);
      this.announce(this.state.error);
    }
  }
  async cancelReview() { if (!this.state.review) return; await this.bridge.invoke("cancel_operation", { planId: this.state.review.plan_id }); this.state.review = null; this.render(); }
  async chooseBase() { const value = document.querySelector("#base-choice")?.value; if (!value || this.state.busy) return; this.state.busy = true; this.render(); try { const snapshot = await this.bridge.invoke("set_base", { request: { base: value } }); await this.refresh(snapshot); this.announce("Base saved"); } catch (error) { this.state.error = error.message ?? String(error); this.announce(this.state.error); this.state.busy = false; this.render(); } }
  async openRepository() { const path = await this.bridge.pickRepository(); if (!path) return; try { const snapshot = await this.bridge.invoke("open_repository", { request: { path } }); await this.refresh(snapshot); } catch (error) { this.state.error = error.message ?? String(error); this.render(); } }
}
