import type { AppController } from "./controller.ts";

interface UiPreferences {
  skip_review: boolean;
}

export async function loadUiPreferences(controller: AppController): Promise<void> {
  try {
    const prefs = await controller.bridge.invoke<UiPreferences>("get_ui_preferences");
    controller.state.skipReview = prefs.skip_review;
  } catch {
    controller.state.skipReview = false;
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
