import { esc } from "../dom.js";
import { baseRef, currentBranch } from "../snapshot.js";
import { emptyState } from "./parts.js";

export function inspectionView(state) {
  const base = baseRef(state);
  if (!base) return missingBase();
  const diff = state.branchDiff ?? "";
  const branch = currentBranch(state) || "detached HEAD";
  return `<div class="pane inspection-pane">
    <header class="inspection-head">
      <div>
        <p class="eyebrow">Branch diff</p>
        <h1>Changes since Base</h1>
        <p class="note"><code>${esc(base)}...${esc(branch)}</code> · committed changes from the merge base to HEAD</p>
      </div>
      <button class="primary${state.diffCopied ? " is-copied" : ""}" data-event="copy-diff" ${!diff || state.busy ? "disabled" : ""}>${state.diffCopied ? "Copied" : "Copy diff"}</button>
    </header>
    ${diff
      ? `<pre class="diff-output" data-scroll="branch-diff"><code>${esc(diff)}</code></pre>`
      : emptyState("No committed changes", "The current branch has no committed changes outside Base.")}
  </div>`;
}

function missingBase() {
  return `<div class="pane inspection-pane">
    <p class="eyebrow">Branch diff</p>
    <h1>Set Base to generate a diff</h1>
    ${emptyState("Base is required", "Choose the remote branch this work is meant to land on.")}
    <button class="primary inspection-base" data-event="edit-base">Set Base</button>
  </div>`;
}
