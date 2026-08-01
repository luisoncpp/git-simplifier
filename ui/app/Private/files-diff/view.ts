import { baseRef } from "../snapshot.ts";
import { compareToggle, diffCompareNote, diffEmptyState, missingBaseGuidance } from "../views/inspection.ts";
import { fileDiffCounts, multiFileDiffBody, multiFileDiffTools } from "./pane.ts";
import { addedCount, removedCount } from "./reads.ts";
import type { AppState } from "../types.ts";
import type { FileDiff } from "./wire.ts";

export function filesDiffView(state: AppState): string {
  if (!baseRef(state)) return missingBaseGuidance("Files diff");
  const files = state.fileDiffs ?? [];
  const pane = { diffView: state.diffView, fileDiffsFull: state.fileDiffsFull };
  return `<div class="pane inspection-pane files-diff-pane">
    ${head(state, files)}${body(pane, files, state.diffView.compare)}
  </div>`;
}

function body(
  pane: { diffView: AppState["diffView"]; fileDiffsFull: AppState["fileDiffsFull"] },
  files: FileDiff[],
  compare: AppState["diffView"]["compare"],
): string {
  const [title, detail] = diffEmptyState(compare);
  return multiFileDiffBody(pane, files, "files-diff", { title, detail });
}

function head(state: AppState, files: FileDiff[]): string {
  return `<header class="inspection-head">
    <div>
      <p class="eyebrow">Files diff</p>
      <h1>Changes since Base</h1>
      <p class="note">${diffCompareNote(state)} · ${fileDiffCounts(files, addedCount, removedCount)}</p>
    </div>
    ${multiFileDiffTools(state.diffView, files.length, compareToggle(state.diffView))}
  </header>`;
}
