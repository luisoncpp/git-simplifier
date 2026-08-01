import { baseRef } from "../snapshot.ts";
import { compareToggle, diffCompareNote, diffEmptyState, missingBaseGuidance } from "../views/inspection.ts";
import { emptyState } from "../views/parts.ts";
import { fileCard } from "./file-card.ts";
import { fileNavigator } from "./navigator.ts";
import { addedCount, removedCount } from "./reads.ts";
import { layoutToggle } from "./single.ts";
import type { AppState } from "../types.ts";
import type { DiffViewState, FileDiff } from "./wire.ts";

export function filesDiffView(state: AppState): string {
  if (!baseRef(state)) return missingBaseGuidance("Files diff");
  const files = state.fileDiffs ?? [];
  return `<div class="pane inspection-pane files-diff-pane">
    ${head(state, files)}${body(state, files)}
  </div>`;
}

function body(state: AppState, files: FileDiff[]): string {
  if (!files.length) {
    const [title, detail] = diffEmptyState(state.diffView.compare);
    return emptyState(title, detail);
  }
  const open = state.diffView.navigatorOpen;
  return `<div class="files-diff-body${open ? " with-navigator" : ""}">
    <div class="file-list" data-scroll="files-diff">
      ${files.map((file, index) => fileCard(state, file, index)).join("")}
    </div>
    ${open ? fileNavigator(files) : ""}
  </div>`;
}

function head(state: AppState, files: FileDiff[]): string {
  const added = files.reduce((total, file) => total + addedCount(file), 0);
  const removed = files.reduce((total, file) => total + removedCount(file), 0);
  return `<header class="inspection-head">
    <div>
      <p class="eyebrow">Files diff</p>
      <h1>Changes since Base</h1>
      <p class="note">${diffCompareNote(state)} · ${files.length} file(s)
        <span class="count-add">+${added}</span> <span class="count-del">&minus;${removed}</span></p>
    </div>
    ${tools(state.diffView, files.length)}
  </header>`;
}

function tools(view: DiffViewState, total: number): string {
  const allClosed = total > 0 && view.collapsed.size >= total;
  const idle = total ? "" : "disabled";
  return `<div class="diff-tools">
    ${compareToggle(view)}
    ${layoutToggle(view)}
    <button class="ghost small" data-event="set-all-files" data-value="${allClosed ? "expanded" : "collapsed"}"
      ${idle}>${allClosed ? "Expand all" : "Collapse all"}</button>
    <button class="ghost small${view.navigatorOpen ? " active" : ""}" data-event="toggle-file-navigator"
      data-focus="diff-navigator" aria-expanded="${view.navigatorOpen}" aria-controls="file-navigator"
      ${idle}>Files (${total})</button>
  </div>`;
}
