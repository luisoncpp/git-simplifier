import { createDraft } from "./draft/index.ts";
import { createDiffView } from "./files-diff/index.ts";
import type { AppState, ViewId } from "./types.ts";

const INSPECTION: ViewId[] = ["files-diff", "raw-diff"];

/// Both Inspection sections load read-only diff data on entry, so the gates that
/// used to test a single view id test the group instead.
export const isInspectionView = (view: ViewId): boolean => INSPECTION.includes(view);

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
    dirtySubmodules: [],
    cleanupBranches: null,
    saved: [],
    operations: [],
    branchDiff: null,
    diffCopied: false,
    fileDiffs: null,
    fileDiffsFull: new Map(),
    diffView: createDiffView(),
    recentRepositories: [],
    repoMenuOpen: false,
    repoFilter: "",
    repoHighlight: 0,
    repoOpeningPath: "",
    repoContextMenu: null,
    pathContextMenu: null,
    draft: createDraft(),
    expanded: new Set(),
    review: null,
    outcome: null,
    changingBase: false,
    busy: false,
    fetch: { active: false, phase: "", done: 0, total: 0 },
    error: "",
    warning: "",
    dismissedSavedWorkBranch: null,
    skipReview: false,
  };
}
