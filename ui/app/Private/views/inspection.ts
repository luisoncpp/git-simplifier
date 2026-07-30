import { esc } from "../dom.ts";
import { baseRef, currentBranch } from "../snapshot.ts";
import type { AppState } from "../types.ts";
import type { DiffCompare, DiffViewState } from "../files-diff/wire.ts";
import { emptyState } from "./parts.ts";

const COMPARES: [DiffCompare, string][] = [
  ["head", "HEAD"],
  ["local", "Local"],
];

export function inspectionView(state: AppState): string {
  const base = baseRef(state);
  if (!base) return missingBaseGuidance("Raw diff");
  const diff = state.branchDiff ?? "";
  const [title, detail] = diffEmptyState(state.diffView.compare);
  return `<div class="pane inspection-pane">
    <header class="inspection-head">
      <div>
        <p class="eyebrow">Raw diff</p>
        <h1>Changes since Base</h1>
        <p class="note">${diffCompareNote(state)}</p>
      </div>
      <div class="diff-tools">
        ${compareToggle(state.diffView)}
        <button class="primary${state.diffCopied ? " is-copied" : ""}" data-event="copy-diff"
          ${!diff || state.busy ? "disabled" : ""}>${state.diffCopied ? "Copied" : "Copy diff"}</button>
      </div>
    </header>
    ${diff ? `<pre class="diff-output" data-scroll="raw-diff"><code>${esc(diff)}</code></pre>`
      : emptyState(title, detail)}
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

export function compareToggle(view: DiffViewState): string {
  return `<div class="layout-toggle compare-toggle" role="group" aria-label="Diff compare">
    ${COMPARES.map((entry) => compareButton(view, entry)).join("")}
  </div>`;
}

export function diffCompareNote(state: AppState): string {
  const base = baseRef(state)!;
  const branch = currentBranch(state) || "detached HEAD";
  if (state.diffView.compare === "local") {
    return `working tree vs merge base of <code>${esc(base)}</code> and <code>${esc(branch)}</code>`;
  }
  return `<code>${esc(base)}...${esc(branch)}</code> · committed changes from the merge base to HEAD`;
}

export function diffEmptyState(compare: DiffCompare): [string, string] {
  if (compare === "local") {
    return ["No local changes", "Nothing in the working tree differs from Base."];
  }
  return ["No committed changes", "The current branch has no committed changes outside Base."];
}

function compareButton(view: DiffViewState, [mode, label]: [DiffCompare, string]): string {
  const current = view.compare === mode;
  return `<button class="ghost small${current ? " active" : ""}" data-event="set-diff-compare"
    data-value="${mode}" data-focus="diff-compare:${mode}"
    aria-pressed="${current}">${esc(label)}</button>`;
}
