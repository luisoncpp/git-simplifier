import type { Draft, LocalBranch } from "../types.ts";

/// `refs/remotes/origin/develop` → `develop`; also accepts the short `origin/develop`.
function baseBranchName(ref: string): string {
  const short = ref.replace(/^refs\/(?:remotes|heads)\//, "");
  const slash = short.indexOf("/");
  return slash === -1 ? short : short.slice(slash + 1);
}

function matchesBaseRemote(branch: LocalBranch, base: string): boolean {
  const remote = branch.remote;
  return Boolean(remote) && (remote === base || `refs/remotes/${remote}` === base);
}

function findDefaultSwitchBranch(available: LocalBranch[], base: string): LocalBranch | undefined {
  if (!base) return available[0];
  const name = baseBranchName(base);
  return (
    available.find((branch) => branch.name === name && !branch.remote) ||
    available.find((branch) => branch.name === name || matchesBaseRemote(branch, base)) ||
    available[0]
  );
}

function selectionStillValid(draft: Draft, available: LocalBranch[]): boolean {
  return available.some(
    (branch) =>
      branch.name === draft.targetBranch &&
      (branch.remote ?? "") === (draft.createFromRemote || ""),
  );
}

function applyDefaultBranch(draft: Draft, available: LocalBranch[], base: string): void {
  const chosen = findDefaultSwitchBranch(available, base);
  draft.targetBranch = chosen ? chosen.name : "";
  draft.createFromRemote = chosen?.remote ? chosen.remote : "";
  draft.branchHighlight = 0;
  draft.branchPicked = false;
}

/// Auto-defaults prefer Base until the user picks a row. A prior alphabetical
/// fallback must not stick after Base becomes available or discovery refreshes.
export function adoptBranch(draft: Draft, branches: LocalBranch[], base = ""): void {
  const available = branches.filter((branch) => !branch.current);
  if (draft.branchPicked && selectionStillValid(draft, available)) return;
  applyDefaultBranch(draft, available, base);
}
