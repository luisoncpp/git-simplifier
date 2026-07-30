import { esc } from "../dom.ts";
import { baseRef, currentBranch } from "../snapshot.ts";
import type { AppState } from "../types.ts";
import { emptyState } from "./parts.ts";

export function inspectionView(state: AppState): string {
  const base = baseRef(state);
  if (!base) return missingBaseGuidance("Raw diff");
  const diff = state.branchDiff ?? "";
  const branch = currentBranch(state) || "detached HEAD";
  return `<div class="pane inspection-pane">
    <header class="inspection-head">
      <div>
        <p class="eyebrow">Raw diff</p>
        <h1>Changes since Base</h1>
        <p class="note"><code>${esc(base)}...${esc(branch)}</code> · committed changes from the merge base to HEAD</p>
      </div>
      <button class="primary${state.diffCopied ? " is-copied" : ""}" data-event="copy-diff" ${!diff || state.busy ? "disabled" : ""}>${state.diffCopied ? "Copied" : "Copy diff"}</button>
    </header>
    ${diff
      ? `<pre class="diff-output" data-scroll="raw-diff"><code>${esc(diff)}</code></pre>`
      : emptyState("No committed changes", "The current branch has no committed changes outside Base.")}
  </div>`;
}

/// Shared by both Inspection sections: neither can produce a diff without Base,
/// and each names itself so the user knows which one is asking.
export function missingBaseGuidance(section: string): string {
  return `<div class="pane inspection-pane">
    <p class="eyebrow">${esc(section)}</p>
    <h1>Set Base to generate a diff</h1>
    ${emptyState("Base is required", "Choose the remote branch this work is meant to land on.")}
    <button class="primary inspection-base" data-event="edit-base">Set Base</button>
  </div>`;
}
