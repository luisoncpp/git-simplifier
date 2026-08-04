import { esc } from "../dom.ts";
import { languageFor } from "./highlight.ts";
import { addedCount, fullFor, removedCount } from "./reads.ts";
import { splitTable } from "./split-table.ts";
import { unifiedTable } from "./unified-table.ts";
import type { FileDiffPaneState } from "./wire.ts";
import type { FileRender } from "./reads.ts";
import type { FileDiff, FileDiffStatus } from "./wire.ts";

const STATUS_TAGS: Record<FileDiffStatus, [string, string]> = {
  added: ["A", "added"],
  deleted: ["D", "deleted"],
  modified: ["M", "modified"],
  renamed: ["R", "renamed"],
};

/// The anchor id is the array index, never the path: paths carry slashes, spaces,
/// quotes, and non-ASCII. The path rides along as `data-file` so the navigator can
/// still find the card by dataset value.
export function fileCard(state: FileDiffPaneState, file: FileDiff, index: number): string {
  const open = !state.diffView.collapsed.has(file.path);
  const bodyId = `file-body-${index}`;
  const body = open ? `<div class="file-body" id="${bodyId}">${cardBody(state, file)}</div>` : "";
  return `<section class="file-card" id="file-${index}" data-file="${esc(file.path)}">
    ${cardHead(file, { open, bodyId })}${body}
  </section>`;
}

export function fileCounts(file: FileDiff): string {
  return `<span class="file-counts"><span class="count-add">+${addedCount(file)}</span>
    <span class="count-del">&minus;${removedCount(file)}</span></span>`;
}

export function statusTag(file: FileDiff): string {
  const [letter, title] = STATUS_TAGS[file.status] ?? ["?", file.status];
  return `<span class="status-tag" title="${esc(title)}">${esc(letter)}</span>`;
}

function cardHead(file: FileDiff, open: { open: boolean; bodyId: string }): string {
  const path = esc(file.path);
  const was = file.previous_path ? `<span class="was">was ${esc(file.previous_path)}</span>` : "";
  return `<header class="file-head">
    <button class="file-toggle" data-event="toggle-file" data-value="${path}" data-focus="diff-file:${path}"
      aria-expanded="${open.open}" aria-controls="${open.bodyId}">
      <span class="caret" aria-hidden="true">${open.open ? "&#9662;" : "&#9656;"}</span>
      ${statusTag(file)}<code class="file-path">${path}</code>${was}
    </button>
    ${fileCounts(file)}
  </header>`;
}

function cardBody(state: FileDiffPaneState, file: FileDiff): string {
  return fileContent({
    file,
    full: fullFor(state, file.path),
    view: state.diffView,
    language: languageFor(file.path),
  });
}

/// Shared by the Inspection file list and the quick single-file window.
export function fileContent(render: FileRender): string {
  const file = bodyFile(render);
  if (file.binary) return hint("Binary file not shown. Raw diff carries its patch header.");
  if (!file.hunks.length) return hint(modeNote(file));
  const drawn = { ...render, file };
  return render.view.layout === "split" ? splitTable(drawn) : unifiedTable(drawn);
}

/// Stubbed untracked entries store bodies in `full` after hydration. Ordinary
/// tracked diffs keep using the context-3 `file` so gap windows stay intact.
function bodyFile(render: FileRender): FileDiff {
  if (!render.file.untracked || render.file.hunks.length) return render.file;
  return render.full?.hunks.length ? render.full : render.file;
}

function modeNote(file: FileDiff): string {
  if (file.old_mode === file.new_mode) return "No content changes.";
  return `Mode changed from ${file.old_mode ?? "none"} to ${file.new_mode ?? "none"}. No content changes.`;
}

function hint(text: string): string {
  return `<p class="hint pad">${esc(text)}</p>`;
}
