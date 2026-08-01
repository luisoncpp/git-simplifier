import { gapSection, lineCode, rowClass, sign } from "./rows.ts";
import type { FileRender } from "./reads.ts";
import type { DiffLine, DiffLineKind } from "./wire.ts";

interface SplitRow {
  left: DiffLine | null;
  right: DiffLine | null;
}

export function splitTable(render: FileRender): string {
  return `<table class="hunk split">
    <colgroup><col class="c-num" /><col /><col class="c-num" /><col /></colgroup>
    <tbody>${body(render)}</tbody>
  </table>`;
}

/// A context line occupies both sides. Within a run of removals followed by
/// additions the two are paired by position, and whichever run is shorter leaves
/// an empty cell opposite the leftovers.
function pairRows(lines: DiffLine[]): SplitRow[] {
  const rows: SplitRow[] = [];
  let at = 0;
  while (at < lines.length) {
    if (lines[at].kind === "context") {
      rows.push({ left: lines[at], right: lines[at] });
      at += 1;
      continue;
    }
    const removed = runOf(lines, at, "del");
    const added = runOf(lines, at + removed.length, "add");
    if (!removed.length && !added.length) return rows;
    rows.push(...paired(removed, added));
    at += removed.length + added.length;
  }
  return rows;
}

function body(render: FileRender): string {
  // A gap holds only unchanged lines, so each one occupies both sides.
  const gapRow = (line: DiffLine): string => splitRow(render, { left: line, right: line });
  const sections = render.file.hunks.map(
    /*renderHunk=*/ (hunk, index) =>
      `${gapSection(render, index, /*renderRow=*/ gapRow)}${hunkRows(render, hunk.lines)}`,
  );
  const trailing = gapSection(render, render.file.hunks.length, /*renderRow=*/ gapRow);
  return `${sections.join("")}${trailing}`;
}

function hunkRows(render: FileRender, lines: DiffLine[]): string {
  return pairRows(lines)
    .map((row) => splitRow(render, row))
    .join("");
}

function splitRow(render: FileRender, row: SplitRow): string {
  return `<tr>${cell(render, row.left, "del")}${cell(render, row.right, "add")}</tr>`;
}

/// The tint lives on the cells rather than the row, because the two sides of a
/// replacement carry opposite meanings.
function cell(render: FileRender, line: DiffLine | null, side: "del" | "add"): string {
  if (!line) return `<td class="num nil"></td><td class="code nil"></td>`;
  const kind = line.kind === "context" ? "ctx" : rowClass(side);
  const number = side === "del" ? line.old_line : line.new_line;
  return `<td class="num ${kind}">${number ?? ""}</td>
    <td class="code ${kind}"><span class="sign">${sign(line.kind)}</span><code>${lineCode(render, line)}</code></td>`;
}

function runOf(lines: DiffLine[], from: number, kind: DiffLineKind): DiffLine[] {
  const run: DiffLine[] = [];
  for (let at = from; at < lines.length && lines[at].kind === kind; at += 1) run.push(lines[at]);
  return run;
}

function paired(removed: DiffLine[], added: DiffLine[]): SplitRow[] {
  const total = Math.max(removed.length, added.length);
  return Array.from({ length: total }, /*pairAt=*/ (_unused, at) => ({
    left: removed[at] ?? null,
    right: added[at] ?? null,
  }));
}
