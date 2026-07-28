import { esc } from "../dom.js";
import { OPERATIONS, submitState } from "../operations.js";
import { operationForm } from "./forms.js";

export function actionsView(state) {
  return `<div class="pane">
    <div class="tabs" role="tablist" aria-label="Operations">${OPERATIONS.map((operation) => tab(state, operation)).join("")}</div>
    <div class="tab-panel" role="tabpanel" id="operation-panel" aria-labelledby="tab-${esc(state.operation)}">
      ${operationForm(state)}${submitRow(state)}
    </div>
  </div>`;
}

function tab(state, operation) {
  const selected = state.operation === operation.id;
  return `<button role="tab" id="tab-${esc(operation.id)}" aria-controls="operation-panel"
    aria-selected="${selected}" tabindex="${selected ? "0" : "-1"}" ${selected ? 'data-focus="tab"' : ""}
    data-event="set-operation" data-value="${esc(operation.id)}">${esc(operation.label)}</button>`;
}

export function submitRow(state) {
  const { disabled, reason } = submitState(state);
  return `<div class="submit-row" id="submit-row">
    <button class="primary" data-event="submit-operation" ${disabled ? "disabled" : ""}>Review ${esc(actionWord(state.operation))}</button>
    <p class="hint">${esc(reason || "Nothing is written until you apply the review.")}</p>
  </div>`;
}

const ACTION_WORDS = {
  uncommit: "uncommit",
  edit_message: "message edit",
  exclude_submodule: "exclusion",
  quick_switch: "switch",
  sync: "sync",
  force_push: "force push",
};

const actionWord = (operation) => ACTION_WORDS[operation] ?? operation;
