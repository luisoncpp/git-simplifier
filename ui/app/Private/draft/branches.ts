import type { Draft, LocalBranch } from "../types.ts";

function extractLocalBranchName(ref: string): string {
  if (!ref) return "";
  let name = ref;
  if (name.startsWith("refs/heads/")) {
    name = name.slice("refs/heads/".length);
  } else if (name.startsWith("refs/remotes/")) {
    name = name.slice("refs/remotes/".length);
  }
  const slashIndex = name.indexOf("/");
  if (slashIndex !== -1) {
    name = name.slice(slashIndex + 1);
  }
  return name;
}

function findDefaultSwitchBranch(available: LocalBranch[], base: string): LocalBranch | undefined {
  if (!base) return available[0];
  const targetName = extractLocalBranchName(base);
  const localMatch = available.find((branch) => branch.name === targetName && !branch.remote);
  if (localMatch) return localMatch;
  const remoteMatch = available.find(
    (branch) =>
      branch.name === targetName ||
      branch.remote === base ||
      branch.remote === `refs/remotes/${base}`,
  );
  if (remoteMatch) return remoteMatch;
  return available[0];
}

export function adoptBranch(draft: Draft, branches: LocalBranch[], base: string = ""): void {
  const available = branches.filter((branch) => !branch.current);
  const stillValid = available.some(
    (branch) =>
      branch.name === draft.targetBranch &&
      (branch.remote ?? "") === (draft.createFromRemote || ""),
  );
  if (stillValid) return;
  const chosen = findDefaultSwitchBranch(available, base);
  draft.targetBranch = chosen?.name ?? "";
  draft.createFromRemote = chosen?.remote ?? "";
  draft.branchHighlight = 0;
}
