import { esc } from "../dom.ts";
import { OPERATIONS, submitState } from "../operations/index.ts";
import type { OperationDef } from "../operations/index.ts";
import type { AppState } from "../types.ts";
import { operationForm } from "./forms.ts";

export function actionsView(state: AppState): string {
  return `<div class="pane">
    <div class="tabs" role="tablist" aria-label="Operations">${OPERATIONS.map((operation) => tab(state, operation)).join("")}</div>
    <div class="tab-panel" role="tabpanel" id="operation-panel" aria-labelledby="tab-${esc(state.operation)}">
      ${operationForm(state)}${submitRow(state)}
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
  return `<div class="submit-row" id="submit-row">
    <button class="primary" data-event="submit-operation" ${disabled ? "disabled" : ""}>Review ${esc(actionWord(state.operation))}</button>
    <p class="hint">${esc(reason || "Nothing is written until you apply the review.")}</p>
  </div>`;
}

const ACTION_WORDS: Record<string, string> = {
  uncommit: "uncommit",
  revert: "revert",
  edit_message: "message edit",
  exclude_submodule: "exclusion",
  split_branch: "split",
  quick_switch: "switch",
  sync: "sync",
  force_push: "force push",
};

const actionWord = (operation: string): string => ACTION_WORDS[operation] ?? operation;
