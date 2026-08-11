/// Wire shapes crossing the Tauri bridge, mirroring the Rust structs in
/// src-tauri/src/commands/data.rs and src/inspection/model.rs (snake_case).
/// Rust newtypes (RefName, ObjectId, RepoPath) serialize as plain strings;
/// RefValue additionally tolerates legacy `{ value }` wrappers.

import type { DiffViewState, FileDiff } from "./files-diff/index.ts";

export type RefValue = string | { value?: string } | null | undefined;

export interface WorktreeSummary {
  staged: number;
  unstaged: number;
  untracked: number;
  conflicts: number;
}

export interface RepositoryOverview {
  path: string;
  name: string;
  branch: string | null;
  base: string | null;
  upstream: string | null;
  head: string;
  git_version: string;
  worktree: WorktreeSummary;
  saved_work_count: number;
  recovery_count: number;
  sync_status: string | null;
  quick_switch_status?: string | null;
}

export interface RepositorySnapshot {
  overview: RepositoryOverview;
  saved_work: SavedWork[];
  operations: RecoveryEntry[];
}

export interface SavedWork { branch: string; reference: string; snapshot: string }
export interface RecoveryEntry {
  id: string;
  operation: string;
  started: string;
  finished: string | null;
  refs_before: Record<string, string>;
  refs_after: Record<string, string>;
  snapshots: Record<string, string>;
  details: Record<string, string>;
  phase: string | null;
  commands: string[];
  reversible: boolean;
  recovery_command: string | null;
}

export interface BaseChoice { reference: string; display: string; head: string }

export interface ChangedPath { path: string; previous_path: string | null; status: string }

export interface Signature { name: string; email: string; date: string }

export interface EditableCommit {
  id: string;
  short_id: string;
  subject: string;
  message: string;
  author: Signature;
}

export interface LocalBranch {
  name: string;
  head: string;
  current: boolean;
  saved_work: boolean;
  remote?: string | null;
}

export interface SubmoduleChoice { path: string; object: string; excluded: boolean }

export interface DirtySubmodule {
  path: string;
  local_dirty: boolean;
  in_editable_range: boolean;
}

export interface CleanupRemote {
  remote: string;
  tracking_ref: string;
  remote_ref: string;
  head: string;
  merged: boolean;
}

export interface CleanupBranch {
  branch: string;
  reference: string;
  head: string;
  kind: "local" | "remote_only";
  author_email: string;
  mine: boolean;
  /// A well-known shared name. Offered, but never ticked by default.
  protected: boolean;
  remote?: CleanupRemote | null;
}

export type CleanupExclusionReason =
  | "current_branch"
  | "checked_out_in_worktree"
  | "base_branch"
  | "saved_work";

export interface CleanupExclusion { branch: string; reason: CleanupExclusionReason }

/// The maximal offerable set. The three Cleanup toggles filter this one result,
/// so flipping a toggle never costs another repository scan.
export interface CleanupDiscovery {
  base: string;
  base_head: string;
  identity: string | null;
  choices: CleanupBranch[];
  excluded: CleanupExclusion[];
}

export interface RecentRepository { name: string; path: string }

export interface RepoContextMenu { path: string; x: number; y: number }

export interface PathContextMenu { path: string; x: number; y: number }

export interface OperationReview {
  plan_id: string;
  kind: string;
  title: string;
  impact: string[];
  preserves: string[];
  warnings: string[];
  commands: string[];
  apply_label: string;
}

export interface OperationOutcome {
  kind: string;
  headline: string;
  details: string[];
  offer_force_push: boolean;
  offer_publish_branch: string | null;
  offer_resolve_pull?: boolean;
  offer_restore_saved_work?: boolean;
  has_warning?: boolean;
}

/// Flat payloads matching PrepareOperationRequest's tagged enum on the Rust
/// side; src-tauri has a test asserting each of these deserializes.
export type OperationRequest =
  | { kind: "uncommit"; base: string; paths: string[] }
  | { kind: "revert"; base: string; paths: string[]; target: "head" | "base" }
  | { kind: "edit_message"; base: string; commit: string; message: string }
  | { kind: "exclude_submodule"; path: string; install_hook: boolean; disable_recurse: boolean }
  | { kind: "cleanup_submodules"; base: string; paths: string[]; uncommit: boolean; revert: boolean }
  | { kind: "split_branch"; base: string; new_branch: string; paths: string[]; message: string }
  | { kind: "publish_branch"; branch: string }
  | { kind: "quick_switch"; target_branch: string; carry_changes?: boolean; pull_after_switch?: boolean; create_from_remote?: string | null }
  | { kind: "resolve_quick_switch_pull"; resolution: "replace_with_remote" | "merge_pull" | "cancel" }
  | { kind: "sync"; base: string }
  | { kind: "cleanup"; base: string; references: string[]; delete_remotes: boolean }
  | { kind: "restore_saved_work" }
  | { kind: "delete_saved_work"; branch: string }
  | { kind: "resume_sync" }
  | { kind: "force_push" };

/// The two Inspection sections are separate views: every gate that used to read
/// `"inspection"` now asks `isInspectionView`, so neither can be mistaken for the
/// group.
export type ViewId = "actions" | "saved" | "recovery" | "files-diff" | "raw-diff";

export type OperationId =
  | "uncommit"
  | "revert"
  | "edit_message"
  | "submodules"
  | "split_branch"
  | "quick_switch"
  | "sync"
  | "force_push"
  | "cleanup";

export interface Draft {
  pathFilter: string;
  selectedPaths: Set<string>;
  splitPaths: Set<string>;
  revertPaths: Set<string>;
  revertTarget: "head" | "base";
  newBranch: string;
  splitMessage: string;
  commit: string;
  messages: Map<string, string>;
  submodule: string;
  installHook: boolean;
  disableRecurse: boolean;
  cleanupSubmodulePaths: Set<string>;
  cleanupSubmoduleFilter: string;
  cleanupUncommit: boolean;
  cleanupRevert: boolean;
  targetBranch: string;
  createFromRemote: string;
  /// True only after the user picks a row; auto-defaults stay eligible to refresh.
  branchPicked: boolean;
  pullAfterSwitch: boolean;
  carryChanges: boolean;
  branchFilter: string;
  branchMenuOpen: boolean;
  branchHighlight: number;
  cleanupOnlyMine: boolean;
  cleanupRemotes: boolean;
  cleanupAllRemote: boolean;
  /// Explicit ticks and unticks only. A row's default follows `protected`, so
  /// one map expresses both "everything pre-ticked" and "shared names are not",
  /// and the three filters can change the visible set without reseeding.
  cleanupOverrides: Map<string, boolean>;
  cleanupFilter: string;
}

export interface AppState {
  view: ViewId;
  operation: OperationId;
  snapshot: RepositorySnapshot | null;
  baseChoices: BaseChoice[];
  paths: ChangedPath[];
  commits: EditableCommit[];
  branches: LocalBranch[];
  submodules: SubmoduleChoice[];
  dirtySubmodules: DirtySubmodule[];
  cleanupBranches: CleanupDiscovery | null;
  saved: SavedWork[];
  operations: RecoveryEntry[];
  branchDiff: string | null;
  diffCopied: boolean;
  fileDiffs: FileDiff[] | null;
  fileDiffsFull: Map<string, FileDiff>;
  diffView: DiffViewState;
  recentRepositories: RecentRepository[];
  repoMenuOpen: boolean;
  repoFilter: string;
  repoHighlight: number;
  repoOpeningPath: string;
  repoContextMenu: RepoContextMenu | null;
  pathContextMenu: PathContextMenu | null;
  draft: Draft;
  expanded: Set<string>;
  review: OperationReview | null;
  outcome: OperationOutcome | null;
  changingBase: boolean;
  busy: boolean;
  error: string;
  /// Non-blocking warning from the latest refresh fetch attempt.
  warning: string;
  /// Branch whose persistent Saved work notice the user dismissed for this visit.
  dismissedSavedWorkBranch: string | null;
  skipReview: boolean;
}

export interface Bridge {
  invoke<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T>;
  pickRepository(): Promise<string | null>;
}

export type FieldNode = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;

interface TauriGlobal {
  core?: { invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
  dialog?: { open?: (options?: Record<string, unknown>) => Promise<unknown> };
  event?: {
    listen?: (event: string, handler: (event: unknown) => void) => Promise<unknown>;
  };
}

declare global {
  /// Injected by the Tauri runtime; absent in browser mode.
  var __TAURI__: TauriGlobal | undefined;
}
