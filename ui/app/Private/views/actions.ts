import { esc } from "../dom.ts";
import { actionVerb, submitHint } from "../review-mode.ts";
import { OPERATIONS, submitState } from "../operations/index.ts";
import type { OperationDef } from "../operations/index.ts";
import type { AppState } from "../types.ts";
import { operationForm } from "./forms.ts";

export function actionsView(state: AppState): string {
  const submit = state.operation === "submodules" ? "" : submitRow(state);
  return `<div class="pane">
    <div class="tabs" role="tablist" aria-label="Operations">${OPERATIONS.map((operation) => tab(state, operation)).join("")}</div>
    <div class="tab-panel" role="tabpanel" id="operation-panel" aria-labelledby="tab-${esc(state.operation)}">
      ${operationForm(state)}${submit}
    </div>
  </div>`;
}

function tab(state: AppState, operation: OperationDef): string {
  const selected = state.operation === operation.id;
  return `<button role="tab" id="tab-${esc(operation.id)}" aria-controls="operation-panel"
    aria-selected="${selected}" tabindex="${selected ? "0" : "-1"}" ${selected ? 'data-focus="tab"' : ""}
    data-event="set-operation" data-value="${esc(operation.id)}">${esc(operation.label)}</button>`;
}

export function submitRow(state: AppState): string {
  const { disabled, reason } = submitState(state);
  const verb = actionVerb(state.skipReview);
  return `<div class="submit-row" id="submit-row">
    <button class="primary" data-event="submit-operation" ${disabled ? "disabled" : ""}>${verb} ${esc(actionWord(state.operation))}</button>
    <p class="hint">${esc(submitHint(state.skipReview, reason))}</p>
  </div>`;
}

const ACTION_WORDS: Record<string, string> = {
  uncommit: "uncommit",
  revert: "revert",
  edit_message: "message edit",
  submodules: "submodule",
  split_branch: "split",
  quick_switch: "switch",
  sync: "sync",
  force_push: "force push",
  cleanup: "cleanup",
};

const actionWord = (operation: string): string => ACTION_WORDS[operation] ?? operation;
