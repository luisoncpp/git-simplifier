import { createDiffView } from "../files-diff/index.ts";
import type { SavedWorkDiffState } from "./types.ts";

export function createSavedWorkDiffState(): SavedWorkDiffState {
  return {
    session: null,
    files: null,
    fileDiffsFull: new Map(),
    diffView: createDiffView(),
    busy: false,
    error: "",
  };
}
