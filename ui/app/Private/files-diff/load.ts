import { baseRef } from "../snapshot.ts";
import { collapseOversizedFiles } from "./collapse-large.ts";
import { ensureGrammars, languageFor } from "./highlight.ts";
import type { AppController } from "../controller.ts";
import type { AppState } from "../types.ts";
import type { FileDiff } from "./wire.ts";

/// Every mutation re-renders the whole shell, so a file past this many rows opens
/// collapsed and its header says why. Raw diff remains the way to read it whole.

/// The layout choice and the navigator survive: they are session preference, not
/// discovery data, so a refresh or a Base change must not undo them.
export function resetFileDiffs(state: AppState): void {
  state.fileDiffs = null;
  state.fileDiffsFull.clear();
  state.diffView.collapsed.clear();
  state.diffView.reveals.clear();
}

export async function loadFileDiffs(controller: AppController, base: string): Promise<void> {
  const state = controller.state;
  resetFileDiffs(state);
  if (!base) return;
  const files = await controller.bridge.invoke<FileDiff[]>("generate_files_diff", {
    request: {
      base,
      compare: state.diffView.compare,
      untracked_filters: state.diffView.untrackedFilters,
    },
  });
  state.fileDiffs = files;
  collapseOversizedFiles(files, state.diffView.collapsed);
  await ensureGrammars(files.map(/*languageOf=*/ (file) => languageFor(file.path)));
}

/// Presence in `fileDiffsFull` *is* the cache, so a second expander click on the
/// same file never reaches Rust again. A `null` reply means the path no longer
/// differs from Base, which the next refresh settles.
export async function ensureFullDiff(controller: AppController, path: string): Promise<void> {
  const state = controller.state;
  if (state.fileDiffsFull.has(path)) return;
  const base = baseRef(state);
  if (!base) return;
  const full = await controller.bridge.invoke<FileDiff | null>("generate_full_file_diff", {
    request: { base, path, compare: state.diffView.compare },
  });
  if (!full) return;
  state.fileDiffsFull.set(path, full);
}
