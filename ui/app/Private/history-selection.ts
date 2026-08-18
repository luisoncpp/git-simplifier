import type { AppController } from "./controller.ts";
import type { FieldNode } from "./types.ts";

export function setHistoryMode(controller: AppController, node: FieldNode): void {
  const value = node.value;
  if (value !== "commit" && value !== "until") return;
  controller.state.draft.historyMode = value;
  controller.render();
}

export function setHistoryUntil(controller: AppController, node: FieldNode): void {
  controller.state.draft.historyUntil = node.value;
  controller.render();
}

export function setHistoryFilter(controller: AppController, node: FieldNode): void {
  controller.state.draft.historyFilter = node.value;
  controller.render();
}

export function setHistoryCarry(controller: AppController, node: FieldNode): void {
  controller.state.draft.historyCarryChanges = (node as HTMLInputElement).checked;
  controller.render();
}
