import { emptyState } from "../views/parts.ts";
import { fileCard } from "./file-card.ts";
import { fileNavigator } from "./navigator.ts";
import { layoutToggle } from "./single.ts";
import type { DiffViewState, FileDiff, FileDiffPaneState } from "./wire.ts";

export interface MultiFileEmpty {
  title: string;
  detail: string;
}

export function multiFileDiffBody(
  pane: FileDiffPaneState,
  files: FileDiff[],
  scrollKey: string,
  empty: MultiFileEmpty,
): string {
  if (!files.length) return emptyState(empty.title, empty.detail);
  const open = pane.diffView.navigatorOpen;
  return `<div class="files-diff-body${open ? " with-navigator" : ""}">
    <div class="file-list" data-scroll="${scrollKey}">
      ${files.map((file, index) => fileCard(pane, file, index)).join("")}
    </div>
    ${open ? fileNavigator(files) : ""}
  </div>`;
}

export function multiFileDiffTools(
  view: DiffViewState,
  total: number,
  compareToggleMarkup: string,
): string {
  const allClosed = total > 0 && view.collapsed.size >= total;
  const idle = total ? "" : "disabled";
  return `<div class="diff-tools">
    ${compareToggleMarkup}
    ${layoutToggle(view)}
    <button class="ghost small" data-event="set-all-files" data-value="${allClosed ? "expanded" : "collapsed"}"
      ${idle}>${allClosed ? "Expand all" : "Collapse all"}</button>
    <button class="ghost small${view.navigatorOpen ? " active" : ""}" data-event="toggle-file-navigator"
      data-focus="diff-navigator" aria-expanded="${view.navigatorOpen}" aria-controls="file-navigator"
      ${idle}>Files (${total})</button>
  </div>`;
}

export function fileDiffCounts(files: FileDiff[], added: (file: FileDiff) => number, removed: (file: FileDiff) => number): string {
  const plus = files.reduce((total, file) => total + added(file), 0);
  const minus = files.reduce((total, file) => total + removed(file), 0);
  return `${files.length} file(s) <span class="count-add">+${plus}</span> <span class="count-del">&minus;${minus}</span>`;
}
