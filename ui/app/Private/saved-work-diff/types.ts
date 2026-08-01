import type { DiffViewState, FileDiff } from "../files-diff/index.ts";

export interface SavedWorkDiffSession {
  branch: string;
  on_current_branch: boolean;
  before_tree: string;
  after_tree: string;
  worktree_conflicts: boolean;
  index_conflicts: boolean;
}

export interface SavedWorkDiffState {
  session: SavedWorkDiffSession | null;
  files: FileDiff[] | null;
  fileDiffsFull: Map<string, FileDiff>;
  diffView: DiffViewState;
  busy: boolean;
  error: string;
}
