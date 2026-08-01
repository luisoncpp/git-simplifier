import { createDiffView } from "../files-diff/index.ts";
import type { QuickDiffState } from "./types.ts";

export function createQuickDiffState(): QuickDiffState {
  return {
    session: null,
    file: null,
    view: createDiffView(),
    busy: false,
    error: "",
  };
}
