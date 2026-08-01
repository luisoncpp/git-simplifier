import { esc } from "../dom.ts";
import { actionVerb } from "../review-mode.ts";
import { currentBranch } from "../snapshot.ts";
import type { AppState, RecoveryEntry, SavedWork } from "../types.ts";
import { copyButton, emptyState, humanTime } from "./parts.ts";

export function savedView(state: AppState): string {
  const branch = currentBranch(state);
  return `<div class="pane">
    <p class="note">Saved work belongs to a branch by name. Untracked files are never part of it, and restoring
    consumes the snapshot.</p>
    <div class="rows">${state.saved.length
      ? state.saved.map((item) => savedRow(state, item)).join("")
      : emptyState("No Saved work", "Switching away from a branch with tracked changes creates a snapshot here.")}</div>
  </div>`;
}

function savedRow(state: AppState, item: SavedWork): string {
  const here = item.branch === currentBranch(state);
  return `<article class="row">
    <div><strong>${esc(item.branch)}${here ? " · current branch" : ""}</strong>
      <code>${esc(item.reference)}</code>
      <code class="muted">snapshot ${esc(String(item.snapshot).slice(0, 12))}</code></div>
    <div class="row-actions">
      ${here
        ? `<button class="primary small" data-event="restore-saved" ${state.busy ? "disabled" : ""}>${actionVerb(state.skipReview)} restore</button>`
        : `<button class="ghost small" data-event="switch-to" data-value="${esc(item.branch)}" ${state.busy ? "disabled" : ""}>Switch to ${esc(item.branch)}</button>`}
      <button class="danger small" data-event="delete-saved" data-value="${esc(item.branch)}" ${state.busy ? "disabled" : ""}>Delete</button>
    </div>
  </article>`;
}

export function recoveryView(state: AppState): string {
  return `<div class="pane">
    <p class="note">A read-only record of what this app wrote. Moving refs back does not restore the working tree,
    so treat the command as a starting point.</p>
    <div class="rows">${state.operations.length
      ? state.operations.map((entry) => recoveryRow(state, entry)).join("")
      : emptyState("Nothing recorded yet", "Every operation this app applies is written to .git/githelper/oplog.json.")}</div>
  </div>`;
}

function recoveryRow(state: AppState, entry: RecoveryEntry): string {
  const open = state.expanded.has(entry.id);
  const phase = entry.finished ? "completed" : (entry.phase ?? "interrupted");
  return `<article class="row stacked">
    <div class="row-head">
      <div><strong>${esc(entry.operation)}</strong>
        <span class="chip${entry.finished ? "" : " bad"}">${esc(phase)}</span>
        <span class="muted">${esc(humanTime(entry.started))}</span></div>
      <div class="row-actions">
        ${copyButton(entry.recovery_command, "Copy ref recovery")}
        <button class="ghost small" data-event="toggle-entry" data-value="${esc(entry.id)}"
          aria-expanded="${open}">${open ? "Hide" : "Details"}</button>
      </div>
    </div>
    ${open ? recoveryDetails(entry) : ""}
  </article>`;
}

function recoveryDetails(entry: RecoveryEntry): string {
  return `<div class="row-body">
    ${refTable("Refs before", entry.refs_before)}
    ${refTable("Refs after", entry.refs_after)}
    ${refTable("Snapshots", entry.snapshots)}
    ${entry.commands.length ? `<h4>Recorded commands</h4><pre>${esc(entry.commands.join("\n"))}</pre>` : ""}
    ${entry.reversible ? "" : `<p class="hint">This record is not reversible by moving refs.</p>`}
  </div>`;
}

function refTable(title: string, map: Record<string, string> | null | undefined): string {
  const rows = Object.entries(map ?? {});
  if (!rows.length) return "";
  return `<h4>${esc(title)}</h4><dl class="facts">${rows
    .map(([name, value]) => `<dt><code>${esc(name)}</code></dt><dd><code>${esc(value)}</code></dd>`)
    .join("")}</dl>`;
}
