import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import {
  filteredRecents,
  openRecentRepository,
  openRepoContextMenu,
  removeRecentRepository,
  revealRepository,
  setRepoFilter,
  toggleRepoMenu,
} from "../ui/app/Private/repository-switcher.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const RECENTS = [
  { path: "C:/work/alpha", name: "alpha" },
  { path: "C:/work/beta", name: "beta" },
  { path: "C:/work/gamma", name: "gamma" },
];

function withRecents(extra = {}) {
  let current = snapshotWith({});
  const controller = controllerWith({
    async invoke(command, args) {
      if (command === "list_recent_repositories") return [...RECENTS];
      if (command === "remove_recent_repository") {
        return RECENTS.filter((entry) => entry.path !== args.path);
      }
      if (command === "open_repository") {
        current = snapshotWith({ path: args.request.path, name: args.request.path.split("/").pop() });
        return current;
      }
      if (command === "load_snapshot") return current;
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
  assert.ok(commands.includes("fetch_remotes"));
  assert.ok(commands.includes("list_recent_repositories"));
});

test("opening a repository with an unreachable remote warns without blocking", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = async (command, args) => {
    if (command === "fetch_remotes") throw new Error("Could not connect to remote");
    return original(command, args);
  };

  await openRecentRepository(controller, "C:/work/beta");

  assert.equal(controller.state.snapshot.overview.path, "C:/work/beta");
  assert.equal(controller.state.warning, "Could not connect to remote");
  assert.equal(controller.state.error, "");
  assert.match(renderShell(controller.state), /Fetch failed/);
  assert.match(renderShell(controller.state), /Could not connect to remote/);
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

test("the repository context menu offers reveal in file explorer", () => {
  const controller = withRecents({ repoMenuOpen: true });
  openRepoContextMenu(controller, "C:/work/beta", 120, 80);

  const markup = renderShell(controller.state);
  assert.match(markup, /Reveal in File Explorer/);
  assert.match(markup, /data-event="reveal-repository"/);
  assert.match(markup, /style="left:120px;top:80px"/);
});

test("reveal in file explorer asks the desktop shell to show the folder", async () => {
  const controller = withRecents();
  const commands = [];
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = async (command, args) => {
    commands.push([command, args]);
    return original(command, args);
  };

  await revealRepository(controller, "C:/work/beta");

  assert.deepEqual(commands[0], ["reveal_in_explorer", { path: "C:/work/beta" }]);
  assert.equal(controller.state.repoContextMenu, null);
});

async function flushUntil(condition) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (condition()) return;
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  throw new Error("condition was never met");
}

test("the new repository is visible while its fetch is still running", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  const original = controller.bridge.invoke.bind(controller.bridge);
  let finishFetch;
  controller.bridge.invoke = (command, args) => {
    if (command !== "fetch_remotes") return original(command, args);
    return new Promise((resolve) => { finishFetch = () => resolve(null); });
  };

  const opening = openRecentRepository(controller, "C:/work/beta");
  await flushUntil(() => controller.state.fetch.active);

  assert.equal(controller.state.snapshot.overview.path, "C:/work/beta");

  finishFetch();
  await opening;
  assert.equal(controller.state.snapshot.overview.path, "C:/work/beta");
  assert.equal(controller.state.fetch.active, false);
});

test("opening a repository reloads once more after the fetch", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  const commands = [];
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = async (command, args) => {
    commands.push(command);
    return original(command, args);
  };

  await openRecentRepository(controller, "C:/work/beta");

  const fetchAt = commands.indexOf("fetch_remotes");
  const loads = commands
    .map((command, index) => (command === "load_snapshot" ? index : -1))
    .filter((index) => index >= 0);
  assert.ok(fetchAt > commands.indexOf("open_repository"));
  assert.deepEqual(loads.length, 1);
  assert.ok(loads[0] > fetchAt);
});
