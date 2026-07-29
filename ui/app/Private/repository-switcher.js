import { createDraft } from "./draft.js";
import { focusNode } from "./dom.js";
import { overviewOf } from "./snapshot.js";

export function filteredRecents(state) {
  const query = state.repoFilter.trim().toLowerCase();
  if (!query) return state.recentRepositories;
  return state.recentRepositories.filter((entry) => {
    return entry.name.toLowerCase().includes(query) || entry.path.toLowerCase().includes(query);
  });
}

export async function loadRecentRepositories(controller) {
  try {
    controller.state.recentRepositories = await controller.bridge.invoke(
      "list_recent_repositories",
    );
  } catch {
    controller.state.recentRepositories = [];
  }
}

export function toggleRepoMenu(controller) {
  if (controller.state.busy) return;
  controller.state.repoMenuOpen = !controller.state.repoMenuOpen;
  if (controller.state.repoMenuOpen) {
    controller.state.repoFilter = "";
    controller.state.repoHighlight = 0;
  }
  controller.render();
  if (controller.state.repoMenuOpen) focusNode("#repo-filter");
}

export function closeRepoMenu(controller) {
  if (!controller.state.repoMenuOpen) return;
  controller.state.repoMenuOpen = false;
  controller.state.repoFilter = "";
  controller.render();
}

export function setRepoFilter(controller, node) {
  controller.state.repoFilter = node.value;
  controller.state.repoHighlight = 0;
  controller.render();
}

export function moveRepoHighlight(controller, step) {
  const total = filteredRecents(controller.state).length;
  if (!total) return;
  const next = (controller.state.repoHighlight + step + total) % total;
  controller.state.repoHighlight = next;
  controller.render();
}

export async function openPickedRepository(controller) {
  const path = await controller.bridge.pickRepository().catch((error) => {
    controller.fail(error);
    controller.render();
    return null;
  });
  if (!path) return;
  await openRepositoryPath(controller, path);
}

export async function openRecentRepository(controller, path) {
  if (!path || controller.state.busy) return;
  const current = overviewOf(controller.state)?.path;
  if (current && samePath(current, path)) {
    closeRepoMenu(controller);
    return;
  }
  await openRepositoryPath(controller, path);
}

export async function removeRecentRepository(controller, path) {
  if (!path || controller.state.busy) return;
  controller.state.recentRepositories = await controller.bridge.invoke(
    "remove_recent_repository",
    { path },
  );
  const visible = filteredRecents(controller.state).length;
  if (controller.state.repoHighlight >= visible) {
    controller.state.repoHighlight = Math.max(0, visible - 1);
  }
  controller.render();
}

export async function activateHighlightedRepository(controller) {
  const entry = filteredRecents(controller.state)[controller.state.repoHighlight];
  if (!entry) return;
  await openRecentRepository(controller, entry.path);
}

async function openRepositoryPath(controller, path) {
  controller.state.repoMenuOpen = false;
  controller.state.repoFilter = "";
  controller.state.repoOpeningPath = path;
  await controller.run(async () => {
    await controller.cancelReview();
    try {
      const snapshot = await controller.bridge.invoke("open_repository", {
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

function samePath(left, right) {
  return left.replaceAll("/", "\\").toLowerCase() === right.replaceAll("/", "\\").toLowerCase();
}
