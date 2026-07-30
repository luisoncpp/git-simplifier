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

export interface RecentRepository { name: string; path: string }

export interface RepoContextMenu { path: string; x: number; y: number }

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
}

/// Flat payloads matching PrepareOperationRequest's tagged enum on the Rust
/// side; src-tauri has a test asserting each of these deserializes.
export type OperationRequest =
  | { kind: "uncommit"; base: string; paths: string[] }
  | { kind: "edit_message"; base: string; commit: string; message: string }
  | { kind: "exclude_submodule"; path: string; install_hook: boolean; disable_recurse: boolean }
  | { kind: "split_branch"; base: string; new_branch: string; paths: string[]; message: string }
  | { kind: "publish_branch"; branch: string }
  | { kind: "quick_switch"; target_branch: string; carry_changes?: boolean; pull_after_switch?: boolean; create_from_remote?: string | null }
  | { kind: "resolve_quick_switch_pull"; resolution: "replace_with_remote" | "merge_pull" | "cancel" }
  | { kind: "sync"; base: string }
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
  | "edit_message"
  | "exclude_submodule"
  | "split_branch"
  | "quick_switch"
  | "sync"
  | "force_push";

export interface Draft {
  pathFilter: string;
  selectedPaths: Set<string>;
  splitPaths: Set<string>;
  newBranch: string;
  splitMessage: string;
  commit: string;
  messages: Map<string, string>;
  submodule: string;
  installHook: boolean;
  disableRecurse: boolean;
  targetBranch: string;
  createFromRemote: string;
  pullAfterSwitch: boolean;
  carryChanges: boolean;
  branchFilter: string;
  branchMenuOpen: boolean;
  branchHighlight: number;
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
  draft: Draft;
  expanded: Set<string>;
  review: OperationReview | null;
  outcome: OperationOutcome | null;
  changingBase: boolean;
  busy: boolean;
  error: string;
}

export interface Bridge {
  invoke<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T>;
  pickRepository(): Promise<string | null>;
}

export type FieldNode = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;

interface TauriGlobal {
  core?: { invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
  dialog?: { open?: (options?: Record<string, unknown>) => Promise<unknown> };
}

declare global {
  /// Injected by the Tauri runtime; absent in browser mode.
  var __TAURI__: TauriGlobal | undefined;
}
