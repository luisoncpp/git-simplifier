import type { AppState, Draft, EditableCommit, RefValue } from "../types.ts";

export const commitValue = (commit: { id: RefValue }): string => {
  const id = commit.id;
  if (id == null) return "";
  if (typeof id === "string") return id;
  return id.value ?? String(id);
};

/// Rust returns the Editable range oldest first, but the commit people reword
/// is almost always the newest one, so the UI presents and defaults to that end.
export const newestFirst = <T>(commits: T[]): T[] => [...commits].reverse();

export function adoptCommit(draft: Draft, commits: EditableCommit[]): void {
  const available = newestFirst(commits).map(commitValue);
  if (available.includes(draft.commit)) return;
  draft.commit = available[0] ?? "";
}

export function selectedCommit(state: AppState): EditableCommit | null {
  return state.commits.find((commit) => commitValue(commit) === state.draft.commit) ?? null;
}

export function messageFor(state: AppState): string {
  const commit = selectedCommit(state);
  if (!commit) return "";
  return state.draft.messages.get(commitValue(commit)) ?? commit.message;
}

export function messageChanged(state: AppState): boolean {
  const commit = selectedCommit(state);
  if (!commit) return false;
  return messageFor(state).trim() !== commit.message.trim();
}
