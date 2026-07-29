import { createDraft } from "./draft.ts";
import type { AppState } from "./types.ts";

export function createState(): AppState {
  return {
    view: "actions",
    operation: "uncommit",
    snapshot: null,
    baseChoices: [],
    paths: [],
    commits: [],
    branches: [],
    submodules: [],
    saved: [],
    operations: [],
    branchDiff: null,
    diffCopied: false,
    recentRepositories: [],
    repoMenuOpen: false,
    repoFilter: "",
    repoHighlight: 0,
    repoOpeningPath: "",
    draft: createDraft(),
    expanded: new Set(),
    review: null,
    outcome: null,
    changingBase: false,
    busy: false,
    error: "",
  };
}
