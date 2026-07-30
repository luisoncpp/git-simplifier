import { gapSection, lineCode, rowClass, sign } from "./rows.ts";
import type { FileRender } from "./reads.ts";
import type { DiffLine } from "./wire.ts";

export function unifiedTable(render: FileRender): string {
  return `<table class="hunk unified">
    <colgroup><col class="c-num" /><col class="c-num" /><col /></colgroup>
    <tbody>${body(render)}</tbody>
  </table>`;
}

function body(render: FileRender): string {
  const row = (line: DiffLine): string => unifiedRow(render, line);
  const sections = render.file.hunks.map(
    /*renderHunk=*/ (hunk, index) =>
      `${gapSection(render, index, /*renderRow=*/ row)}${hunk.lines.map(row).join("")}`,
  );
  const trailing = gapSection(render, render.file.hunks.length, /*renderRow=*/ row);
  return `${sections.join("")}${trailing}`;
}

function unifiedRow(render: FileRender, line: DiffLine): string {
  return `<tr class="${rowClass(line.kind)}">
    <td class="num">${line.old_line ?? ""}</td>
    <td class="num">${line.new_line ?? ""}</td>
    <td class="code"><span class="sign">${sign(line.kind)}</span><code>${lineCode(render, line)}</code></td>
  </tr>`;
}
