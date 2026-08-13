import type { AppController } from "./controller.ts";

interface UiPreferences {
  skip_review: boolean;
  codechart_path: string;
  guessed_codechart_path: string;
}

export async function loadUiPreferences(controller: AppController): Promise<void> {
  try {
    const prefs = await controller.bridge.invoke<UiPreferences>("get_ui_preferences");
    controller.state.skipReview = prefs.skip_review;
    controller.state.codechartPath = prefs.codechart_path ?? "";
    controller.state.guessedCodechartPath = prefs.guessed_codechart_path ?? "";
  } catch {
    controller.state.skipReview = false;
    controller.state.codechartPath = "";
    controller.state.guessedCodechartPath = "";
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
