import { languageFor } from "./highlight.ts";
import type { DiffViewState, FileDiff, UntrackedAnnotations, UntrackedFilters } from "./wire.ts";

export function visibleFileDiffs(files: FileDiff[], view: DiffViewState): FileDiff[] {
  if (view.compare !== "local") return files;
  return files.filter((file) => !isHiddenUntracked(file, view.untrackedFilters));
}

type HideRule = (filters: UntrackedFilters, untracked: UntrackedAnnotations, path: string) => boolean;

const HIDE_RULES: HideRule[] = [
  (filters, untracked) => filters.excludeOlderThanHead && untracked.older_than_or_at_head,
  (filters, untracked) => filters.excludeRootDot && untracked.root_dot,
  (filters, untracked) => filters.excludeNodeModules && untracked.in_node_modules,
  (filters, untracked) => filters.respectGitignore && untracked.gitignored,
  (filters, _untracked, path) => filters.excludeUnknownTypes && languageFor(path) === "",
];

function isHiddenUntracked(file: FileDiff, filters: UntrackedFilters): boolean {
  const untracked = file.untracked;
  if (!untracked) return false;
  return HIDE_RULES.some((rule) => rule(filters, untracked, file.path));
}
