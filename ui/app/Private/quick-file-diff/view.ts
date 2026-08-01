import { esc } from "../dom.ts";
import { compareToggle, diffEmptyState } from "../views/inspection.ts";
import { emptyState } from "../views/parts.ts";
import { layoutToggle, singleFileDiff } from "../files-diff/index.ts";
import type { DiffCompare } from "../files-diff/index.ts";
import type { QuickDiffState } from "./types.ts";

export function quickDiffView(state: QuickDiffState): string {
  const session = state.session;
  if (!session) return emptyState("No file selected", "Close this window and open a path from the list.");
  return `<div class="pane inspection-pane files-diff-pane quick-file-diff">
    ${head(state)}${body(state)}
  </div>`;
}

function head(state: QuickDiffState): string {
  const session = state.session!;
  return `<header class="inspection-head">
    <div>
      <p class="eyebrow">File diff</p>
      <h1><code>${esc(session.path)}</code></h1>
      <p class="note">${compareNote(session.compare, session.base)}</p>
    </div>
    <div class="diff-tools">
      ${session.compare_toggle ? compareToggle(state.view) : ""}
      ${layoutToggle(state.view)}
    </div>
  </header>`;
}

function body(state: QuickDiffState): string {
  if (state.error) {
    return `<div class="banner error" role="alert"><p>${esc(state.error)}</p></div>`;
  }
  if (state.busy && !state.file) {
    return emptyState("Loading diff", "Fetching the full file context…");
  }
  if (!state.file) {
    const [title, detail] = diffEmptyState(state.view.compare);
    return emptyState(title, detail);
  }
  return `<div class="files-diff-body">
    <div class="file-list" data-scroll="quick-file-diff">${singleFileDiff(state.file, state.view)}</div>
  </div>`;
}

function compareNote(compare: DiffCompare, base: string): string {
  if (compare === "local") {
    return `working tree vs merge base of <code>${esc(base)}</code>`;
  }
  return `<code>${esc(base)}...HEAD</code> · committed changes from the merge base to HEAD`;
}
