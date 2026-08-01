import { esc } from "../dom.ts";
import { languageFor } from "./highlight.ts";
import { fileContent, fileCounts, statusTag } from "./file-card.ts";
import type { DiffLayout, DiffViewState, FileDiff } from "./wire.ts";

const LAYOUTS: [DiffLayout, string][] = [
  ["unified", "Unified"],
  ["split", "Side by side"],
];

/// One file, same tables as Inspection, without the multi-file navigator chrome.
export function singleFileDiff(file: FileDiff, view: DiffViewState): string {
  const render = {
    file,
    full: file.complete ? file : null,
    view,
    language: languageFor(file.path),
  };
  return `<section class="file-card" data-file="${esc(file.path)}">
    <header class="file-head">
      <div class="file-toggle static">
        ${statusTag(file)}<code class="file-path">${esc(file.path)}</code>
        ${file.previous_path ? `<span class="was">was ${esc(file.previous_path)}</span>` : ""}
      </div>
      ${fileCounts(file)}
    </header>
    <div class="file-body">${fileContent(render)}</div>
  </section>`;
}

export function layoutToggle(view: DiffViewState): string {
  return `<div class="layout-toggle" role="group" aria-label="Diff layout">
    ${LAYOUTS.map((layout) => layoutButton(view, layout)).join("")}
  </div>`;
}

function layoutButton(view: DiffViewState, [layout, label]: [DiffLayout, string]): string {
  const current = view.layout === layout;
  return `<button class="ghost small${current ? " active" : ""}" data-event="set-diff-layout"
    data-value="${layout}" data-focus="diff-layout:${layout}"
    aria-pressed="${current}">${esc(label)}</button>`;
}
