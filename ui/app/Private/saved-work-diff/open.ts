import type { AppController } from "../controller.ts";

export async function openSavedWorkDiff(controller: AppController, branch: string): Promise<void> {
  await controller.bridge.invoke("open_saved_work_diff_window", { request: { branch } });
}
