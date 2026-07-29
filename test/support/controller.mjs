import { AppController } from "../../ui/app/index.ts";

const BASE = "refs/remotes/origin/main";

export function snapshotWith(overview = {}) {
  return {
    overview: {
      path: "C:/repo",
      name: "repo",
      branch: "feature",
      base: BASE,
      upstream: null,
      head: "1".repeat(40),
      git_version: "2.45.0",
      worktree: { staged: 0, unstaged: 0, untracked: 0, conflicts: 0 },
      sync_status: null,
      ...overview,
    },
    saved_work: [],
    operations: [],
  };
}

/// The controller renders into a document it does not own, so tests replace
/// render/announce instead of standing up a DOM.
export function controllerWith(bridge, overview = {}) {
  const controller = new AppController(bridge);
  controller.render = () => {};
  controller.announce = () => {};
  controller.state.snapshot = snapshotWith(overview);
  return controller;
}
