import type { AppState, CleanupBranch, CleanupDiscovery, Draft } from "../types.ts";

/// The rows the three toggles leave visible. They are display filters over one
/// discovery result, never another query, so flipping one is instant.
export function cleanupChoices(state: AppState): CleanupBranch[] {
  const draft = state.draft;
  const query = draft.cleanupFilter.trim().toLowerCase();
  const choices = state.cleanupBranches?.choices ?? [];
  return choices.filter((choice) => offered(draft, choice) && matches(query, choice));
}

const offered = (draft: Draft, choice: CleanupBranch): boolean =>
  mineEnough(draft, choice) && remoteEnough(draft, choice);

const mineEnough = (draft: Draft, choice: CleanupBranch): boolean =>
  !draft.cleanupOnlyMine || choice.mine;

/// Deleting a remote-only branch *is* a remote deletion, so it needs both the
/// toggle that lists it and the toggle that permits the write.
const remoteEnough = (draft: Draft, choice: CleanupBranch): boolean =>
  choice.kind !== "remote_only" || (draft.cleanupAllRemote && draft.cleanupRemotes);

const matches = (query: string, choice: CleanupBranch): boolean =>
  !query || choice.branch.toLowerCase().includes(query);

/// Everything is ticked by default except a shared name; `cleanupOverrides`
/// holds only the ticks and unticks the user actually made.
export const cleanupTicked = (state: AppState, choice: CleanupBranch): boolean =>
  state.draft.cleanupOverrides.get(choice.reference) ?? !choice.protected;

export const cleanupSelection = (state: AppState): string[] =>
  cleanupChoices(state)
    .filter((choice) => cleanupTicked(state, choice))
    .map((choice) => choice.reference);

/// A branch that left the repository must not keep an override behind, or a
/// later branch reusing that ref name would inherit a stale tick.
export function adoptCleanup(draft: Draft, discovery: CleanupDiscovery | null): void {
  const choices = discovery?.choices ?? [];
  const live = new Set(choices.map((choice) => choice.reference));
  const stale = [...draft.cleanupOverrides.keys()].filter((reference) => !live.has(reference));
  stale.forEach((reference) => draft.cleanupOverrides.delete(reference));
}
