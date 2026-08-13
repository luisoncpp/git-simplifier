import { esc } from "../dom.ts";
import { fileCounts, statusTag } from "./file-card.ts";
import type { FileDiff } from "./wire.ts";

/// Closed, the navigator renders nothing at all — the header toggle is the only
/// affordance, so the default state costs no layout.
export function fileNavigator(files: FileDiff[]): string {
  return `<nav class="file-navigator" id="file-navigator" aria-label="Changed files"
    data-scroll="file-navigator">
    <p class="eyebrow">${files.length} changed file(s)</p>
    <ul>${files.map(navigatorRow).join("")}</ul>
  </nav>`;
}

function navigatorRow(file: FileDiff): string {
  const path = esc(file.path);
  return `<li data-path-context="${path}"><button class="nav-file" data-event="jump-to-file"
    data-value="${path}" title="${path}">
    ${statusTag(file)}<code>${path}</code>${fileCounts(file)}
  </button></li>`;
}
