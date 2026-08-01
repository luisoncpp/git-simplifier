import { esc } from "../dom.ts";
import { filesDiffView } from "../files-diff/index.ts";
import { pathContextMenuMarkup } from "../path-diff-menu.ts";
import { baseRef, currentBranch, overviewOf, refValue, upstreamRef, worktreeCounts } from "../snapshot.ts";
import { isInspectionView } from "../state.ts";
import type { AppState, ViewId } from "../types.ts";
import { actionsView } from "./actions.ts";
import { banners } from "./banners.ts";
import { inspectionView } from "./inspection.ts";
import { recoveryView, savedView } from "./panels.ts";
import { repoSwitcher } from "./repo-menu.ts";
import { reviewPane } from "./review.ts";

type NavEntry = [ViewId, string, (state: AppState) => number];

const VIEWS: NavEntry[] = [
  ["actions", "Actions", () => 0],
  ["saved", "Saved work", (state) => state.saved.length],
  ["recovery", "Recovery", (state) => state.operations.length],
];
/// Array order is rail order. Files diff leads because it is the readable view;
/// Raw diff stays for copying the patch as text.
const INSPECTION_VIEWS: NavEntry[] = [
  ["files-diff", "Files diff", (state) => state.fileDiffs?.length ?? 0],
  ["raw-diff", "Raw diff", () => 0],
];

export function renderShell(state: AppState): string {
  return `<div class="shell">${rail(state)}${main(state)}</div>
    ${pathContextMenuMarkup(state.pathContextMenu)}`;
}

function rail(state: AppState): string {
  return `<aside class="rail">
    <div class="brand"><span class="mark" aria-hidden="true">gs</span><strong>Git Simplifier</strong></div>
    ${repoSwitcher(state)}
    <nav aria-label="Sections">
      ${VIEWS.map((entry) => navItem(state, entry)).join("")}
      <div class="nav-group">
        <p class="nav-heading">Inspection</p>
        ${INSPECTION_VIEWS.map((entry) => navItem(state, entry)).join("")}
      </div>
    </nav>
    <p class="rail-foot">${railFoot(state)}</p>
  </aside>`;
}

function railFoot(state: AppState): string {
  const overview = overviewOf(state);
  return overview ? `Git ${esc(overview.git_version)}` : "Desktop access required";
}

function navItem(state: AppState, [id, label, count]: NavEntry): string {
  const total = count(state);
  const badge = total ? `<span class="badge">${total}</span>` : "";
  const current = state.view === id;
  return `<button class="nav-item${current ? " active" : ""}" data-event="set-view" data-value="${id}"
    aria-current="${current ? "page" : "false"}">${esc(label)}${badge}</button>`;
}

/// The banner stack is always present so `.main` keeps exactly four grid rows
/// and the workspace stays the one that scrolls.
function main(state: AppState): string {
  return `<section class="main">${repoBar(state)}
    <div class="banner-stack">${banners(state)}</div>
    <div class="workspace${workspaceClass(state)}">${pane(state)}${reviewPane(state)}</div>
    <footer class="status" role="status">${status(state)}</footer>
  </section>`;
}

function workspaceClass(state: AppState): string {
  const classes = [];
  if (state.review) classes.push("split");
  if (isInspectionView(state.view)) classes.push("inspection");
  return classes.length ? ` ${classes.join(" ")}` : "";
}

function pane(state: AppState): string {
  if (!state.snapshot) return emptyPane(state);
  if (state.view === "saved") return savedView(state);
  if (state.view === "recovery") return recoveryView(state);
  if (state.view === "files-diff") return filesDiffView(state);
  if (state.view === "raw-diff") return inspectionView(state);
  return actionsView(state);
}

function emptyPane(state: AppState): string {
  return `<div class="pane empty">
    <h1>No repository is open</h1>
    <p>Git Simplifier reads every branch, path, and commit from a real repository, so nothing is shown until one
    is open.</p>
    ${state.error ? `<p class="reason"><strong>Last attempt:</strong> ${esc(state.error)}</p>` : ""}
    <button class="primary" data-event="pick-repository">Choose a repository</button>
  </div>`;
}

function repoBar(state: AppState): string {
  const overview = overviewOf(state);
  if (!overview) return `<header class="repo-bar"><span class="muted">Repository unavailable</span></header>`;
  return `<header class="repo-bar">
    <div class="fact"><span class="eyebrow">Branch</span><strong>${esc(currentBranch(state) || "detached HEAD")}</strong></div>
    <div class="fact"><span class="eyebrow">Base</span>${baseControl(state)}</div>
    ${upstreamRef(state) ? `<div class="fact"><span class="eyebrow">Upstream</span><code>${esc(upstreamRef(state))}</code></div>` : ""}
    <div class="fact worktree"><span class="eyebrow">Working tree</span>${worktreeChips(state)}</div>
    ${skipReviewToggle(state)}
    <button class="ghost" data-event="refresh" ${state.busy ? "disabled" : ""}>Refresh</button>
  </header>`;
}

function baseControl(state: AppState): string {
  if (state.changingBase) return baseChooser(state);
  const base = baseRef(state);
  if (!base) {
    return `<button class="link warn" data-event="edit-base">Set Base &rarr;</button>`;
  }
  return `<span class="base-set"><code>${esc(base)}</code>
    <button class="link" data-event="edit-base">Change</button></span>`;
}

function baseChooser(state: AppState): string {
  const options = state.baseChoices
    .map((choice) => {
      const value = refValue(choice.reference);
      const selected = value === baseRef(state) ? " selected" : "";
      return `<option value="${esc(value)}"${selected}>${esc(choice.display ?? value)}</option>`;
    })
    .join("");
  if (!options) return `<span class="muted">No remote-tracking ref was found. Fetch a remote first.</span>`;
  return `<span class="base-edit">
    <select id="base-choice" data-focus="base-choice" aria-label="Remote-tracking Base ref">${options}</select>
    <button class="primary small" data-event="save-base" ${state.busy ? "disabled" : ""}>Save</button>
    <button class="link" data-event="cancel-base">Cancel</button>
  </span>`;
}

function worktreeChips(state: AppState): string {
  const counts = worktreeCounts(state);
  if (!counts.length) return `<span class="chip clean">clean</span>`;
  return counts
    .map(([label, count]) => `<span class="chip${label === "conflicts" ? " bad" : ""}">${count} ${label}</span>`)
    .join("");
}

const SKIP_MODES: [boolean, string][] = [
  [false, "Review"],
  [true, "Skip"],
];

function skipReviewToggle(state: AppState): string {
  return `<div class="layout-toggle skip-toggle" role="group" aria-label="Review mode">
    ${SKIP_MODES.map(([skip, label]) => skipModeButton(state, skip, label)).join("")}
  </div>`;
}

function skipModeButton(state: AppState, skip: boolean, label: string): string {
  const current = state.skipReview === skip;
  const skipActive = skip && current ? " skip-active" : "";
  return `<button class="ghost small${current ? " active" : ""}${skipActive}" data-event="set-skip-review"
    data-value="${skip}" data-focus="skip-review:${skip}" aria-pressed="${current}"
    ${state.busy ? "disabled" : ""}>${esc(label)}</button>`;
}

function status(state: AppState): string {
  if (state.busy) return `<span class="spinner" aria-hidden="true"></span>Working…`;
  if (state.review) return "Review pending — nothing has been written yet.";
  return "Ready";
}
