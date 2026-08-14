import type { AppController } from "./controller.ts";

interface UiPreferences {
  skip_review: boolean;
  codechart_path: string;
  guessed_codechart_path: string;
  terminal_path?: string;
  default_terminal_name?: string;
  bash_path?: string;
  guessed_bash_path?: string;
}

const DEFAULT_PREFERENCES: UiPreferences = {
  skip_review: false,
  codechart_path: "",
  guessed_codechart_path: "",
  terminal_path: "",
  default_terminal_name: "",
  bash_path: "",
  guessed_bash_path: "",
};

function applyPreferences(controller: AppController, prefs?: UiPreferences): void {
  const p = Object.assign({}, DEFAULT_PREFERENCES, prefs);
  controller.state.skipReview = p.skip_review;
  controller.state.codechartPath = p.codechart_path;
  controller.state.guessedCodechartPath = p.guessed_codechart_path;
  controller.state.terminalPath = p.terminal_path ?? "";
  controller.state.defaultTerminalName = p.default_terminal_name ?? "";
  controller.state.bashPath = p.bash_path ?? "";
  controller.state.guessedBashPath = p.guessed_bash_path ?? "";
}

export async function loadUiPreferences(controller: AppController): Promise<void> {
  try {
    const prefs = await controller.bridge.invoke<UiPreferences>("get_ui_preferences");
    applyPreferences(controller, prefs);
  } catch {
    applyPreferences(controller);
  }
}

export async function setSkipReview(controller: AppController, skip: boolean): Promise<void> {
  if (controller.state.skipReview === skip || controller.state.busy) return;
  controller.state.skipReview = skip;
  controller.render();
  try {
    await controller.bridge.invoke("set_skip_review", { skipReview: skip });
  } catch {
    // A failed save still keeps the session choice; restart restores the file.
  }
}

export async function setCodechartPath(controller: AppController, path: string): Promise<void> {
  if (controller.state.codechartPath === path || controller.state.busy) return;
  controller.state.codechartPath = path;
  controller.render();
  try {
    await controller.bridge.invoke("set_codechart_path", { codechartPath: path });
  } catch {
    // A failed save still keeps the session choice; restart restores the file.
  }
}

export async function setTerminalPath(controller: AppController, path: string): Promise<void> {
  if (controller.state.terminalPath === path || controller.state.busy) return;
  controller.state.terminalPath = path;
  controller.render();
  try {
    await controller.bridge.invoke("set_terminal_path", { terminalPath: path });
  } catch {
    // A failed save still keeps the session choice; restart restores the file.
  }
}

export async function setBashPath(controller: AppController, path: string): Promise<void> {
  if (controller.state.bashPath === path || controller.state.busy) return;
  controller.state.bashPath = path;
  controller.render();
  try {
    await controller.bridge.invoke("set_bash_path", { bashPath: path });
  } catch {
    // A failed save still keeps the session choice; restart restores the file.
  }
}
