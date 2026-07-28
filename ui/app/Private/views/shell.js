import { esc } from "../dom.js";
import { baseRef, currentBranch, overviewOf, upstreamRef, worktreeCounts } from "../snapshot.js";
import { actionsView } from "./actions.js";
import { banners } from "./banners.js";
import { recoveryView, savedView } from "./panels.js";
import { reviewPane } from "./review.js";

const VIEWS = [
  ["actions", "Actions", () => 0],
  ["saved", "Saved work", (state) => state.saved.length],
  ["recovery", "Recovery", (state) => state.operations.length],
];

export function renderShell(state) {
  return `<div class="shell">${rail(state)}${main(state)}</div>`;
}

function rail(state) {
  const overview = overviewOf(state);
  return `<aside class="rail">
    <div class="brand"><span class="mark" aria-hidden="true">gh</span><strong>Git Helper</strong></div>
    <button class="repo-picker" data-event="pick-repository" title="Open another repository">
      <span class="eyebrow">Repository</span>
      <strong>${esc(overview?.name ?? "Open a repository")}</strong>
      <code>${esc(overview?.path ?? "No repository is open")}</code>
    </button>
    <nav aria-label="Sections">${VIEWS.map((entry) => navItem(state, entry)).join("")}</nav>
    <p class="rail-foot">${overview ? `Git ${esc(overview.git_version)}` : "Desktop access required"}</p>
  </aside>`;
}

function navItem(state, [id, label, count]) {
  const total = count(state);
  const badge = total ? `<span class="badge">${total}</span>` : "";
  const current = state.view === id;
  return `<button class="nav-item${current ? " active" : ""}" data-event="set-view" data-value="${id}"
    aria-current="${current ? "page" : "false"}">${esc(label)}${badge}</button>`;
}

/// The banner stack is always present so `.main` keeps exactly four grid rows
/// and the workspace stays the one that scrolls.
function main(state) {
  return `<section class="main">${repoBar(state)}
    <div class="banner-stack">${banners(state)}</div>
    <div class="workspace${state.review ? " split" : ""}">${pane(state)}${reviewPane(state)}</div>
    <footer class="status" role="status">${status(state)}</footer>
  </section>`;
}

function pane(state) {
  if (!state.snapshot) return emptyPane(state);
  if (state.view === "saved") return savedView(state);
  if (state.view === "recovery") return recoveryView(state);
  return actionsView(state);
}

function emptyPane(state) {
  return `<div class="pane empty">
    <h1>No repository is open</h1>
    <p>Git Helper reads every branch, path, and commit from a real repository, so nothing is shown until one
    is open.</p>
    ${state.error ? `<p class="reason"><strong>Last attempt:</strong> ${esc(state.error)}</p>` : ""}
    <button class="primary" data-event="pick-repository">Choose a repository</button>
  </div>`;
}

function repoBar(state) {
  const overview = overviewOf(state);
  if (!overview) return `<header class="repo-bar"><span class="muted">Repository unavailable</span></header>`;
  return `<header class="repo-bar">
    <div class="fact"><span class="eyebrow">Branch</span><strong>${esc(currentBranch(state) || "detached HEAD")}</strong></div>
    <div class="fact"><span class="eyebrow">Base</span>${baseControl(state)}</div>
    ${upstreamRef(state) ? `<div class="fact"><span class="eyebrow">Upstream</span><code>${esc(upstreamRef(state))}</code></div>` : ""}
    <div class="fact worktree"><span class="eyebrow">Working tree</span>${worktreeChips(state)}</div>
    <button class="ghost" data-event="refresh" ${state.busy ? "disabled" : ""}>Refresh</button>
  </header>`;
}

function baseControl(state) {
  if (state.changingBase) return baseChooser(state);
  const base = baseRef(state);
  if (!base) {
    return `<button class="link warn" data-event="edit-base">Set Base &rarr;</button>`;
  }
  return `<span class="base-set"><code>${esc(base)}</code>
    <button class="link" data-event="edit-base">Change</button></span>`;
}

function baseChooser(state) {
  const options = state.baseChoices
    .map((choice) => {
      const value = choice.reference?.value ?? choice.reference;
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

function worktreeChips(state) {
  const counts = worktreeCounts(state);
  if (!counts.length) return `<span class="chip clean">clean</span>`;
  return counts
    .map(([label, count]) => `<span class="chip${label === "conflicts" ? " bad" : ""}">${count} ${label}</span>`)
    .join("");
}

function status(state) {
  if (state.busy) return `<span class="spinner" aria-hidden="true"></span>Working…`;
  if (state.review) return "Review pending — nothing has been written yet.";
  return "Ready";
}
