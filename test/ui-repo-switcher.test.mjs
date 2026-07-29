import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
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

test("pressing a repository previews its selected state before the menu closes", async () => {
  const css = await readFile(new URL("../ui/styles/repo-menu.css", import.meta.url), "utf8");

  assert.match(css, /\.repo-row:has\(\.repo-open:active\)\s*\{/);
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
  assert.equal(controller.state.repoOpeningPath, "");
  assert.ok(commands.includes("open_repository"));
  assert.ok(commands.includes("list_recent_repositories"));
});

test("opening a recent repository closes the menu and shows the target immediately", async () => {
  let finishOpen;
  const controller = withRecents({ repoMenuOpen: true });
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = (command, args) => {
    if (command !== "open_repository") return original(command, args);
    return new Promise((resolve) => {
      finishOpen = () => resolve(snapshotWith({ path: args.request.path, name: "beta" }));
    });
  };

  const opening = openRecentRepository(controller, "C:/work/beta");
  const markup = renderShell(controller.state);

  assert.equal(controller.state.busy, true);
  assert.equal(controller.state.repoMenuOpen, false);
  assert.doesNotMatch(markup, /id="repo-menu"/);
  assert.match(markup, /<strong>beta<\/strong>\s*<code>C:\/work\/beta<\/code>/);

  await Promise.resolve();
  finishOpen();
  await opening;
});

test("a failed repository open restores the previous visible selection", async () => {
  const controller = withRecents({
    repoMenuOpen: true,
    snapshot: snapshotWith({ path: "C:/work/alpha", name: "alpha" }),
  });
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = async (command, args) => {
    if (command === "open_repository") throw new Error("Not a Git repository");
    return original(command, args);
  };

  await openRecentRepository(controller, "C:/work/beta");

  assert.equal(controller.state.repoOpeningPath, "");
  assert.equal(controller.state.repoMenuOpen, false);
  assert.match(renderShell(controller.state), /<strong>alpha<\/strong>\s*<code>C:\/work\/alpha<\/code>/);
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
