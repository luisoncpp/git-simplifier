import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.js";
import {
  filteredRecents,
  openRecentRepository,
  removeRecentRepository,
  setRepoFilter,
  toggleRepoMenu,
} from "../ui/app/Private/repository-switcher.js";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const RECENTS = [
  { path: "C:/work/alpha", name: "alpha" },
  { path: "C:/work/beta", name: "beta" },
  { path: "C:/work/gamma", name: "gamma" },
];

function withRecents(extra = {}) {
  const controller = controllerWith({
    async invoke(command, args) {
      if (command === "list_recent_repositories") return [...RECENTS];
      if (command === "remove_recent_repository") {
        return RECENTS.filter((entry) => entry.path !== args.path);
      }
      if (command === "open_repository") {
        return snapshotWith({ path: args.request.path, name: args.request.path.split("/").pop() });
      }
      if (command === "load_snapshot") return snapshotWith({});
      return [];
    },
  });
  controller.state.recentRepositories = [...RECENTS];
  Object.assign(controller.state, extra);
  return controller;
}

test("the repository menu lists recents and filters them", () => {
  const controller = withRecents({ repoMenuOpen: true });
  assert.match(renderShell(controller.state), /Filter repositories/);
  assert.match(renderShell(controller.state), /Browse for repository/);

  setRepoFilter(controller, { value: "bet" });
  const markup = renderShell(controller.state);
  assert.match(markup, /beta/);
  assert.doesNotMatch(markup, />alpha</);
  assert.equal(filteredRecents(controller.state).length, 1);
});

test("opening a recent repository reloads from the returned snapshot", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  const commands = [];
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = async (command, args) => {
    commands.push(command);
    return original(command, args);
  };

  await openRecentRepository(controller, "C:/work/beta");

  assert.equal(controller.state.snapshot.overview.path, "C:/work/beta");
  assert.equal(controller.state.repoMenuOpen, false);
  assert.ok(commands.includes("open_repository"));
  assert.ok(commands.includes("list_recent_repositories"));
});

test("removing a recent repository keeps the menu open", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  await removeRecentRepository(controller, "C:/work/alpha");

  assert.equal(controller.state.recentRepositories.length, 2);
  assert.equal(controller.state.repoMenuOpen, true);
  assert.doesNotMatch(renderShell(controller.state), /data-value="C:\/work\/alpha"/);
});

test("toggling the picker opens an empty teach state", () => {
  const controller = withRecents();
  controller.state.recentRepositories = [];
  toggleRepoMenu(controller);

  assert.equal(controller.state.repoMenuOpen, true);
  assert.match(renderShell(controller.state), /No recent repositories yet/);
});
