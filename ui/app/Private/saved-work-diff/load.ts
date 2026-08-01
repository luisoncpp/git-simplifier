import { collapseOversizedFiles } from "../files-diff/collapse-large.ts";
import { ensureGrammars, languageFor } from "../files-diff/index.ts";
import type { Bridge } from "../types.ts";
import type { FileDiff } from "../files-diff/index.ts";
import type { SavedWorkDiffState } from "./types.ts";

function resetDiffCache(state: SavedWorkDiffState): void {
  state.files = null;
  state.fileDiffsFull.clear();
  state.diffView.collapsed.clear();
  state.diffView.reveals.clear();
}

export async function loadFiles(bridge: Bridge, state: SavedWorkDiffState): Promise<void> {
  resetDiffCache(state);
  const files = await bridge.invoke<FileDiff[]>("generate_saved_work_files_diff");
  state.files = files;
  collapseOversizedFiles(files, state.diffView.collapsed);
  await ensureGrammars(files.map(/*languageOf=*/ (file) => languageFor(file.path)));
}

export async function ensureFullDiff(bridge: Bridge, state: SavedWorkDiffState, path: string): Promise<void> {
  if (state.fileDiffsFull.has(path)) return;
  const full = await bridge.invoke<FileDiff | null>("generate_saved_work_full_file_diff", {
    request: { path },
  });
  if (!full) return;
  state.fileDiffsFull.set(path, full);
}
