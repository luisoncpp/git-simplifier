import type { DiffCompare, DiffViewState, FileDiff } from "../files-diff/index.ts";

export interface OpenFileDiffRequest {
  path: string;
  base: string;
  compare: DiffCompare;
  compare_toggle: boolean;
}

export interface FileDiffSession {
  path: string;
  base: string;
  compare: DiffCompare;
  compare_toggle: boolean;
}

export interface QuickDiffState {
  session: FileDiffSession | null;
  file: FileDiff | null;
  view: DiffViewState;
  busy: boolean;
  error: string;
}
