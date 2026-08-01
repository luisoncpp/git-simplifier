import { renderedRows } from "./reads.ts";
import type { FileDiff } from "./wire.ts";

const MAX_ROWS_PER_FILE = 2000;

export function collapseOversizedFiles(files: FileDiff[], collapsed: Set<string>): void {
  for (const file of files) {
    if (renderedRows(file) > MAX_ROWS_PER_FILE) collapsed.add(file.path);
  }
}
