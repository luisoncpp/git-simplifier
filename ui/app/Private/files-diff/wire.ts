/// Wire shapes mirroring src/inspection/model.rs (snake_case), plus the view
/// state that belongs to the user rather than to Rust.

export type FileDiffStatus = "added" | "deleted" | "modified" | "renamed";
export type DiffLineKind = "context" | "add" | "del";
export type DiffLayout = "unified" | "split";
export type DiffCompare = "head" | "local";

export interface DiffLine {
  kind: DiffLineKind;
  /// Absent, never zero, on the side the line does not exist on.
  old_line?: number;
  new_line?: number;
  /// Content without the marker and without the line terminator.
  text: string;
  no_newline?: boolean;
}

export interface DiffHunk {
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  heading: string;
  lines: DiffLine[];
}

export interface FileDiff {
  path: string;
  previous_path?: string;
  status: FileDiffStatus;
  old_mode?: string;
  new_mode?: string;
  binary: boolean;
  /// True only from the expansion command: the hunks then hold every line of the
  /// file, so nothing is left to fetch.
  complete: boolean;
  hunks: DiffHunk[];
}

/// A gap's reveals are two blocks growing inward from its edges: `down` from the
/// previous hunk downward, `up` from the next hunk upward. The gap closes — and
/// its expander disappears — exactly when the two meet.
export interface GapReveal {
  down: number;
  up: number;
  all: boolean;
}

export interface DiffViewState {
  layout: DiffLayout;
  /// HEAD compares merge-base → HEAD; Local compares merge-base → working tree.
  compare: DiffCompare;
  /// Paths the user closed. Every file starts open, so empty means all open —
  /// which is why this is not the shared `state.expanded` set, whose members are
  /// closed-by-default oplog ids.
  collapsed: Set<string>;
  /// path → gap index → reveal. Nested rather than one composite key because a
  /// repository path may contain any character.
  reveals: Map<string, Map<number, GapReveal>>;
  navigatorOpen: boolean;
}

export function createDiffView(): DiffViewState {
  return {
    layout: "unified",
    compare: "head",
    collapsed: new Set(),
    reveals: new Map(),
    navigatorOpen: false,
  };
}
