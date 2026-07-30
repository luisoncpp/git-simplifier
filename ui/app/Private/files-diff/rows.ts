import { esc } from "../dom.ts";
import { highlightCode } from "./highlight.ts";
import { contextLines, gapSize, gapWindow, gapsOf, revealFor } from "./reads.ts";
import type { FileRender, Gap, LineRange } from "./reads.ts";
import type { DiffLine, DiffLineKind, FileDiff } from "./wire.ts";

/// GitLab's step. One click is a glance; a whole file is the third button.
const EXPAND_STEP = 20;
const DIRECTIONS = ["up", "all", "down"] as const;
const GLYPHS: Record<string, string> = { up: "&uarr;", all: "&ctdot;", down: "&darr;" };
const SIGNS: Record<DiffLineKind, string> = { context: "&nbsp;", add: "+", del: "&minus;" };

type RowRenderer = (line: DiffLine) => string;

export const rowClass = (kind: DiffLineKind): string => (kind === "context" ? "ctx" : kind);

export const sign = (kind: DiffLineKind): string => SIGNS[kind];

/// Prism's output is pre-escaped HTML; the no-newline marker is appended after it
/// so a file with no final newline says so where the eye already is.
export function lineCode(render: FileRender, line: DiffLine): string {
  const marker = line.no_newline
    ? `<span class="no-newline" title="No newline at end of file">&crarr;</span>`
    : "";
  return `${highlightCode(line.text, render.language)}${marker}`;
}

/// The gap that precedes hunk `index`, rendered as the block revealed downward,
/// whatever expander is left, then the block revealed upward. Shared by both
/// layouts so the two can never disagree about what is hidden.
export function gapSection(render: FileRender, index: number, row: RowRenderer): string {
  const gap = gapsOf(render.file, render.full).find((entry) => entry.index === index);
  if (!gap) return "";
  const window = gapWindow(gap, revealFor(render.view, render.file.path, index));
  const revealed = (range: LineRange | null): string =>
    contextLines(render.full, range).map(row).join("");
  const expander = window.expander ? gapRow(render, gap) : "";
  return `${revealed(window.before)}${expander}${revealed(window.after)}`;
}

function gapRow(render: FileRender, gap: Gap): string {
  const columns = render.view.layout === "split" ? 4 : 3;
  const heading = render.file.hunks[gap.index]?.heading ?? "";
  const actions = DIRECTIONS.map((direction) => expander(render.file, gap, direction)).join("");
  return `<tr class="gap"><td colspan="${columns}">
    <span class="gap-actions">${actions}</span>
    <code class="gap-header">${esc(heading)}</code>
  </td></tr>`;
}

function expander(file: FileDiff, gap: Gap, direction: (typeof DIRECTIONS)[number]): string {
  const path = esc(file.path);
  const title = esc(expanderTitle(gap, direction));
  return `<button class="gap-btn" data-event="expand-gap" data-value="${path}" data-gap="${gap.index}"
    data-dir="${direction}" data-focus="diff-gap:${path}:${gap.index}:${direction}"
    title="${title}" aria-label="${title}">${GLYPHS[direction]}</button>`;
}

function expanderTitle(gap: Gap, direction: string): string {
  if (direction === "up") return `Expand ${EXPAND_STEP} lines up`;
  if (direction === "down") return `Expand ${EXPAND_STEP} lines down`;
  const size = gapSize(gap);
  return size > 0 ? `Expand all ${size} unchanged lines` : "Expand the rest of the file";
}
