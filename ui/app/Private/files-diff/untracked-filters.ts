import { esc } from "../dom.ts";
import type { DiffViewState, UntrackedFilters } from "./wire.ts";

const FILTER_ROWS: [keyof UntrackedFilters, string][] = [
  ["excludeOlderThanHead", "Creation after HEAD"],
  ["excludeRootDot", "Exclude root . paths"],
  ["excludeNodeModules", "Exclude node_modules"],
  ["respectGitignore", "Respect .gitignore"],
  ["excludeUnknownTypes", "Exclude unknown types"],
];

export function untrackedFiltersMenu(view: DiffViewState): string {
  if (view.compare !== "local") return "";
  const open = view.untrackedFiltersOpen;
  return `<div class="untracked-filters">
    <button class="ghost small${open ? " active" : ""}" type="button" data-event="toggle-untracked-filters"
      aria-expanded="${open}" aria-controls="untracked-filters-menu" data-focus="untracked-filters"
      aria-label="Untracked file filters">Untracked filters</button>
    ${open ? menu(view) : ""}
  </div>`;
}

function menu(view: DiffViewState): string {
  return `<div id="untracked-filters-menu" class="untracked-filters-menu" role="group"
    aria-label="Untracked file filters">
    ${FILTER_ROWS.map((row) => filterRow(view.untrackedFilters, row)).join("")}
  </div>`;
}

function filterRow(filters: UntrackedFilters, [key, label]: [keyof UntrackedFilters, string]): string {
  const checked = filters[key] ? " checked" : "";
  return `<label class="check-row">
    <input type="checkbox" data-event="toggle-untracked-filter" data-value="${key}"${checked} />
    ${esc(label)}
  </label>`;
}
