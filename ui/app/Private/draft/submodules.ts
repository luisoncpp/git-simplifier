import type { DirtySubmodule, Draft } from "../types.ts";
import { pathValue } from "./paths.ts";

export function adoptDirtySubmodules(draft: Draft, entries: DirtySubmodule[]): void {
  const available = new Set(entries.map(pathValue));
  for (const selected of draft.cleanupSubmodulePaths) {
    if (!available.has(selected)) draft.cleanupSubmodulePaths.delete(selected);
  }
  if (draft.cleanupSubmodulePaths.size === 0) {
    for (const entry of entries) {
      draft.cleanupSubmodulePaths.add(pathValue(entry));
    }
  }
}

export function visibleDirtySubmodules(
  entries: DirtySubmodule[],
  filter: string,
): DirtySubmodule[] {
  const query = filter.trim().toLowerCase();
  if (!query) return entries;
  return entries.filter((entry) => pathValue(entry).toLowerCase().includes(query));
}
