import { esc } from "./dom.ts";
import { pathDiffRequest } from "./quick-file-diff/index.ts";
import { baseRef, overviewOf } from "./snapshot.ts";
import type { AppController } from "./controller.ts";
import type { PathContextMenu } from "./types.ts";

export function openPathContextMenu(
  controller: AppController,
  path: string,
  x: number,
  y: number,
): void {
  const viewDiff = pathMenuOffersViewDiff(controller, path);
  if (!path || (!viewDiff && controller.state.view !== "files-diff")) return;
  controller.state.repoContextMenu = null;
  controller.state.pathContextMenu = { path, x, y, viewDiff };
  controller.render();
}

function pathMenuOffersViewDiff(controller: AppController, path: string): boolean {
  if (controller.state.view === "files-diff") return false;
  return Boolean(pathDiffRequest(controller.state.operation, path, baseRef(controller.state)));
}

export function closePathContextMenu(controller: AppController, render = true): void {
  if (!controller.state.pathContextMenu) return;
  controller.state.pathContextMenu = null;
  if (render) controller.render();
}

export async function openPathDiff(controller: AppController, path: string): Promise<void> {
  closePathContextMenu(controller, /*render=*/false);
  const request = pathDiffRequest(controller.state.operation, path, baseRef(controller.state));
  if (!request) return;
  await controller.bridge.invoke("open_file_diff_window", { request });
  controller.render();
}

export async function openPathInIde(controller: AppController, path: string): Promise<void> {
  closePathContextMenu(controller, /*render=*/false);
  const repoPath = overviewOf(controller.state)?.path ?? "";
  if (!repoPath || !path) return;
  try {
    await controller.bridge.invoke("open_file_in_ide", { repoPath, filePath: path });
  } catch (error) {
    controller.fail(error);
  }
  controller.render();
}

export function pathContextMenuMarkup(menu: PathContextMenu | null): string {
  if (!menu) return "";
  const viewDiff = menu.viewDiff
    ? `<button class="repo-context-item" type="button" role="menuitem"
      data-event="view-path-diff" data-value="${esc(menu.path)}">
      View diff
    </button>`
    : "";
  return `<div class="repo-context-menu path-context-menu" role="menu" aria-label="Path actions"
    style="left:${menu.x}px;top:${menu.y}px">
    ${viewDiff}
    <button class="repo-context-item" type="button" role="menuitem"
      data-event="edit-path-in-ide" data-value="${esc(menu.path)}">
      Edit in IDE
    </button>
  </div>`;
}
