import { createDraft } from "./draft/index.ts";
import { focusNode } from "./dom.ts";
import { overviewOf } from "./snapshot.ts";
import type { AppController } from "./controller.ts";
import type { AppState, RecentRepository, RepositorySnapshot } from "./types.ts";

export function filteredRecents(state: AppState): RecentRepository[] {
  const query = state.repoFilter.trim().toLowerCase();
  if (!query) return state.recentRepositories;
  return state.recentRepositories.filter((entry) => {
    return entry.name.toLowerCase().includes(query) || entry.path.toLowerCase().includes(query);
  });
}

export async function loadRecentRepositories(controller: AppController): Promise<void> {
  try {
    controller.state.recentRepositories = await controller.bridge.invoke<RecentRepository[]>(
      "list_recent_repositories",
    );
  } catch {
    controller.state.recentRepositories = [];
  }
}

export function toggleRepoMenu(controller: AppController): void {
  if (controller.state.busy) return;
  controller.state.repoMenuOpen = !controller.state.repoMenuOpen;
  if (controller.state.repoMenuOpen) {
    controller.state.repoFilter = "";
    controller.state.repoHighlight = 0;
  }
  controller.render();
  if (controller.state.repoMenuOpen) focusNode("#repo-filter");
}

export function closeRepoMenu(controller: AppController): void {
  if (!controller.state.repoMenuOpen) return;
  controller.state.repoMenuOpen = false;
  controller.state.repoFilter = "";
  closeRepoContextMenu(controller, /*render=*/false);
  controller.render();
}

export function setRepoFilter(controller: AppController, node: { value: string }): void {
  controller.state.repoFilter = node.value;
  controller.state.repoHighlight = 0;
  controller.render();
}

function moveRepoHighlight(controller: AppController, step: number): void {
  const total = filteredRecents(controller.state).length;
  if (!total) return;
  const next = (controller.state.repoHighlight + step + total) % total;
  controller.state.repoHighlight = next;
  controller.render();
}

export async function openPickedRepository(controller: AppController): Promise<void> {
  const path = await controller.bridge.pickRepository().catch((error: unknown) => {
    controller.fail(error);
    controller.render();
    return null;
  });
  if (!path) return;
  await openRepositoryPath(controller, path);
}

export async function openRecentRepository(controller: AppController, path: string): Promise<void> {
  if (!path || controller.state.busy) return;
  const current = overviewOf(controller.state)?.path;
  if (current && samePath(current, path)) {
    closeRepoMenu(controller);
    return;
  }
  await openRepositoryPath(controller, path);
}

export async function removeRecentRepository(controller: AppController, path: string): Promise<void> {
  if (!path || controller.state.busy) return;
  controller.state.recentRepositories = await controller.bridge.invoke<RecentRepository[]>(
    "remove_recent_repository",
    { path },
  );
  const visible = filteredRecents(controller.state).length;
  if (controller.state.repoHighlight >= visible) {
    controller.state.repoHighlight = Math.max(0, visible - 1);
  }
  controller.render();
}

export function openRepoContextMenu(
  controller: AppController,
  path: string,
  x: number,
  y: number,
): void {
  if (!path || controller.state.busy) return;
  controller.state.pathContextMenu = null;
  controller.state.repoContextMenu = { path, x, y };
  controller.render();
}

export function closeRepoContextMenu(controller: AppController, render = true): void {
  if (!controller.state.repoContextMenu) return;
  controller.state.repoContextMenu = null;
  if (render) controller.render();
}

export async function revealRepository(controller: AppController, path: string): Promise<void> {
  if (!path) return;
  closeRepoContextMenu(controller);
  try {
    await controller.bridge.invoke("reveal_in_explorer", { path });
  } catch (error) {
    controller.fail(error);
    controller.render();
  }
}

async function activateHighlightedRepository(controller: AppController): Promise<void> {
  const entry = filteredRecents(controller.state)[controller.state.repoHighlight];
  if (!entry) return;
  await openRecentRepository(controller, entry.path);
}

async function openRepositoryPath(controller: AppController, path: string): Promise<void> {
  controller.state.repoMenuOpen = false;
  controller.state.repoFilter = "";
  controller.state.repoOpeningPath = path;
  await controller.run(async () => {
    await controller.cancelReview();
    try {
      const snapshot = await controller.bridge.invoke<RepositorySnapshot>("open_repository", {
        request: { path },
      });
      controller.state.draft = createDraft();
      controller.state.outcome = null;
      controller.state.expanded.clear();
      await controller.reload(snapshot);
      await loadRecentRepositories(controller);
    } catch (error) {
      await loadRecentRepositories(controller);
      throw error;
    } finally {
      controller.state.repoOpeningPath = "";
    }
  });
}

/// Truthy when the key belonged to the open repository menu, so the delegated
/// dispatcher stops looking. Enter returns its promise so the caller reports a
/// failed open instead of losing it.
export function handleKeys(controller: AppController, event: KeyboardEvent): boolean | Promise<void> {
  if (!controller.state.repoMenuOpen) return false;
  if (event.key === "Escape") {
    event.preventDefault();
    closeRepoMenu(controller);
    return true;
  }
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    moveRepoHighlight(controller, event.key === "ArrowDown" ? 1 : -1);
    return true;
  }
  const target = event.target as HTMLElement | null;
  if (event.key === "Enter" && target?.dataset?.event === "repo-filter") {
    event.preventDefault();
    return activateHighlightedRepository(controller);
  }
  return false;
}

function samePath(left: string, right: string): boolean {
  return left.replaceAll("/", "\\").toLowerCase() === right.replaceAll("/", "\\").toLowerCase();
}
