import { esc } from "../dom.ts";
import { addedCount, removedCount } from "../files-diff/reads.ts";
import { fileDiffCounts, multiFileDiffBody, multiFileDiffTools } from "../files-diff/pane.ts";
import { emptyState } from "../views/parts.ts";
import type { FileDiff, FileDiffPaneState } from "../files-diff/wire.ts";
import type { SavedWorkDiffState } from "./types.ts";

export function savedWorkDiffView(state: SavedWorkDiffState): string {
  const session = state.session;
  if (!session) {
    return emptyState("No Saved work selected", "Close this window and open Diff from the Saved work list.");
  }
  const pane = { diffView: state.diffView, fileDiffsFull: state.fileDiffsFull };
  const files = state.files ?? [];
  return `<div class="pane inspection-pane files-diff-pane saved-work-diff">
    ${head(state)}${banners(state)}${body(state, pane, files)}
  </div>`;
}

function head(state: SavedWorkDiffState): string {
  const session = state.session!;
  const files = state.files ?? [];
  const note = session.on_current_branch
    ? "apply onto the current working tree"
    : `apply onto <code>${esc(session.branch)}</code> after switching`;
  return `<header class="inspection-head">
    <div>
      <p class="eyebrow">Saved work</p>
      <h1><code>${esc(session.branch)}</code></h1>
      <p class="note">Restore preview · ${note} · ${fileDiffCounts(files, addedCount, removedCount)}</p>
    </div>
    ${multiFileDiffTools(state.diffView, files.length, "")}
  </header>`;
}

function banners(state: SavedWorkDiffState): string {
  const session = state.session;
  if (!session) return "";
  const notes: string[] = [];
  if (session.worktree_conflicts) {
    notes.push("Git would leave conflict markers in the working tree if you restored now.");
  }
  if (session.index_conflicts) {
    notes.push("The staged split from this snapshot could not be restored with apply --index.");
  }
  if (!notes.length) return "";
  return `<div class="banner warn" role="status">${notes.map((note) => `<p>${esc(note)}</p>`).join("")}</div>`;
}

function body(state: SavedWorkDiffState, pane: FileDiffPaneState, files: FileDiff[]): string {
  if (state.error) {
    return `<div class="banner error" role="alert"><p>${esc(state.error)}</p></div>`;
  }
  if (state.busy && !state.files) {
    return emptyState("Loading diff", "Simulating restore onto the working tree…");
  }
  return multiFileDiffBody(pane, files, "saved-work-diff", {
    title: "Nothing would change",
    detail: "Restore would leave the working tree as it is now.",
  });
}
