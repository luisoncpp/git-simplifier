import type { FileDiffPaneState } from "./wire.ts";
import type { DiffLine, DiffLineKind, DiffViewState, FileDiff, GapReveal } from "./wire.ts";

/// A run of unchanged lines the loaded diff does not show. `index` is the index
/// of the hunk it precedes — the trailing gap uses `hunks.length` — and stays
/// stable because the context-3 diff is never replaced by an expanded one.
export interface Gap {
  index: number;
  start: number;
  /// Inclusive last hidden line, in new-file numbering. Below `start` when the
  /// gap's length is not known yet, which only happens for the trailing gap
  /// before the file's full context has been fetched.
  end: number;
}

export interface LineRange {
  start: number;
  end: number;
}

export interface GapWindow {
  before: LineRange | null;
  expander: Gap | null;
  after: LineRange | null;
}

/// Threaded down one file's render tree. Purposeful grouping, not the artificial
/// kind: every table, row, and gap needs all four.
export interface FileRender {
  file: FileDiff;
  full: FileDiff | null;
  view: DiffViewState;
  language: string;
}

const NO_REVEAL: GapReveal = { down: 0, up: 0, all: false };

export const addedCount = (file: FileDiff): number => countKind(file, "add");
export const removedCount = (file: FileDiff): number => countKind(file, "del");
export const gapSize = (gap: Gap): number => (gap.end < gap.start ? 0 : gap.end - gap.start + 1);
export const fullFor = (state: FileDiffPaneState, path: string): FileDiff | null =>
  state.fileDiffsFull.get(path) ?? null;

export const renderedRows = (file: FileDiff): number =>
  file.hunks.reduce((total, hunk) => total + hunk.lines.length, 0);

export const revealFor = (view: DiffViewState, path: string, index: number): GapReveal =>
  view.reveals.get(path)?.get(index) ?? NO_REVEAL;

/// Gaps are measured in new-file lines. A wholly added or wholly deleted file
/// already has every one of its lines in the patch, so neither has any gap and
/// neither should offer an expander.
export function gapsOf(file: FileDiff, full: FileDiff | null): Gap[] {
  if (file.status === "added" || file.status === "deleted" || !file.hunks.length) return [];
  const gaps: Gap[] = [];
  let cursor = 1;
  file.hunks.forEach((hunk, index) => {
    const [first, last] = coveredLines(hunk.new_start, hunk.new_lines);
    if (first > cursor) gaps.push({ index, start: cursor, end: first - 1 });
    cursor = last + 1;
  });
  const total = totalLines(full);
  if (!full) gaps.push({ index: file.hunks.length, start: cursor, end: cursor - 1 });
  else if (total >= cursor) gaps.push({ index: file.hunks.length, start: cursor, end: total });
  return gaps;
}

/// Three pieces at most: the block revealed downward from the previous hunk,
/// whatever expander is left, then the block revealed upward toward the next one.
/// An overshooting reveal closes the gap instead of spilling past its edge.
export function gapWindow(gap: Gap, reveal: GapReveal): GapWindow {
  const size = gapSize(gap);
  if (reveal.all || (size > 0 && reveal.down + reveal.up >= size)) {
    return { before: { start: gap.start, end: gap.end }, expander: null, after: null };
  }
  return {
    before: reveal.down > 0 ? { start: gap.start, end: gap.start + reveal.down - 1 } : null,
    expander: gap,
    after: reveal.up > 0 ? { start: gap.end - reveal.up + 1, end: gap.end } : null,
  };
}

export function contextLines(full: FileDiff | null, range: LineRange | null): DiffLine[] {
  if (!full || !range || range.end < range.start) return [];
  return full.hunks
    .flatMap((hunk) => hunk.lines)
    .filter((line) => within(line.new_line, range));
}

/// A zero-length new range means the hunk sits immediately after `new_start`, so
/// the covered region is that single boundary line rather than a span.
function coveredLines(start: number, count: number): [number, number] {
  if (count > 0) return [start, start + count - 1];
  return [start + 1, start];
}

function totalLines(full: FileDiff | null): number {
  const last = full?.hunks[full.hunks.length - 1];
  if (!last) return 0;
  return coveredLines(last.new_start, last.new_lines)[1];
}

function within(line: number | undefined, range: LineRange): boolean {
  return line != null && line >= range.start && line <= range.end;
}

function countKind(file: FileDiff, kind: DiffLineKind): number {
  return file.hunks.reduce(
    (total, hunk) => total + hunk.lines.filter((line) => line.kind === kind).length,
    0,
  );
}
