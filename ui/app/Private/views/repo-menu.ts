import { esc } from "../dom.ts";
import { filteredRecents } from "../repository-switcher.ts";
import { overviewOf } from "../snapshot.ts";
import type { AppState, RecentRepository } from "../types.ts";

interface DisplayedRepository {
  name: string;
  path: string;
}

export function repoSwitcher(state: AppState): string {
  const repository = displayedRepository(state);
  const open = state.repoMenuOpen;
  return `<div class="repo-switcher">
    <button class="repo-picker" type="button" data-event="toggle-repo-menu"
      aria-expanded="${open ? "true" : "false"}" aria-controls="repo-menu"
      title="Switch repository" ${state.busy ? "disabled" : ""}>
      <span class="eyebrow">Repository</span>
      <strong>${esc(repository?.name ?? "Open a repository")}</strong>
      <code>${esc(repository?.path ?? "No repository is open")}</code>
    </button>
    ${open ? repoMenu(state) : ""}
  </div>`;
}

function displayedRepository(state: AppState): DisplayedRepository | null {
  if (!state.repoOpeningPath) return overviewOf(state);
  const recent = state.recentRepositories.find((entry) => samePath(entry.path, state.repoOpeningPath));
  return recent ?? { name: "Opening repository", path: state.repoOpeningPath };
}

function repoMenu(state: AppState): string {
  const entries = filteredRecents(state);
  return `<div id="repo-menu" class="repo-menu" role="listbox" aria-label="Repositories">
    <label class="repo-filter">
      <span class="sr-only">Filter repositories</span>
      <input id="repo-filter" data-event="repo-filter" data-focus="repo-filter"
        type="search" placeholder="Filter repositories" value="${esc(state.repoFilter)}"
        autocomplete="off" ${state.busy ? "disabled" : ""} />
    </label>
    <div class="repo-menu-list" data-scroll="repo-menu-list">${menuBody(state, entries)}</div>
    <button class="repo-browse" type="button" data-event="pick-repository"
      ${state.busy ? "disabled" : ""}>Browse for repository&hellip;</button>
  </div>`;
}

function menuBody(state: AppState, entries: RecentRepository[]): string {
  if (!state.recentRepositories.length) {
    return `<p class="repo-empty">No recent repositories yet.</p>`;
  }
  if (!entries.length) {
    return `<p class="repo-empty">No repositories match.</p>`;
  }
  return entries.map((entry, index) => repoRow(state, entry, index)).join("");
}

function repoRow(state: AppState, entry: RecentRepository, index: number): string {
  const current = state.repoOpeningPath || overviewOf(state)?.path || "";
  const selected = samePath(current, entry.path);
  const active = index === state.repoHighlight;
  const classes = ["repo-row", selected ? "current" : "", active ? "active" : ""]
    .filter(Boolean)
    .join(" ");
  return `<div class="${classes}" role="option" aria-selected="${selected ? "true" : "false"}">
    <button class="repo-open" type="button" data-event="open-recent" data-value="${esc(entry.path)}"
      ${state.busy ? "disabled" : ""}>
      <strong>${esc(entry.name)}</strong>
      <code>${esc(entry.path)}</code>
    </button>
    <button class="repo-remove" type="button" data-event="remove-recent" data-value="${esc(entry.path)}"
      aria-label="Remove ${esc(entry.name)} from recent" ${state.busy ? "disabled" : ""}>&times;</button>
  </div>`;
}

function samePath(left: string, right: string): boolean {
  return left.replaceAll("/", "\\").toLowerCase() === right.replaceAll("/", "\\").toLowerCase();
}
