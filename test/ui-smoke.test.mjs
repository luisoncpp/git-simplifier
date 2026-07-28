import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { AppController } from "../ui/app/controller.js";
import { shell } from "../ui/app/views.js";

test("workbench contains no prompt-driven Git identifiers or demo repository", async () => {
  const source = await readFile(new URL("../ui/app/controller.js", import.meta.url), "utf8");
  const views = await readFile(new URL("../ui/app/views.js", import.meta.url), "utf8");
  assert.doesNotMatch(source, /window\.prompt|prompt\(/);
  assert.doesNotMatch(views, /meridian-api|origin\/develop|feat\/latency-budget/);
});

test("operation boundary is explicit in the controller", async () => {
  const source = await readFile(new URL("../ui/app/controller.js", import.meta.url), "utf8");
  assert.match(source, /prepare_operation/);
  assert.match(source, /apply_operation/);
  assert.match(source, /cancel_operation/);
});

test("Git-backed Tauri commands run off the window thread", async () => {
  const source = await readFile(
    new URL("../src-tauri/src/commands/actions.rs", import.meta.url),
    "utf8",
  );
  const commands = [...source.matchAll(/#\[tauri::command(?:\(async\))?\]\s+pub fn (\w+)/g)];
  const blocking = commands
    .filter((match) => match[1] !== "app_ready" && !match[0].includes("(async)"))
    .map((match) => match[1]);
  assert.deepEqual(blocking, []);
});

test("saving Base reuses the snapshot returned by set_base", async () => {
  const snapshot = {
    overview: { base: "refs/remotes/origin/main" },
    saved_work: [],
    operations: [],
  };
  const commands = [];
  const bridge = {
    async invoke(command) {
      commands.push(command);
      if (command === "set_base" || command === "load_snapshot") return snapshot;
      return [];
    },
  };
  const controller = new AppController(bridge);
  controller.render = () => {};
  controller.announce = () => {};
  const previousDocument = globalThis.document;
  globalThis.document = {
    querySelector: () => ({ value: "refs/remotes/origin/main" }),
  };
  try {
    await controller.chooseBase();
  } finally {
    globalThis.document = previousDocument;
  }
  assert.equal(commands.filter((command) => command === "load_snapshot").length, 0);
});

test("a failed sync apply reloads state and exposes the resume action", async () => {
  const base = "refs/remotes/origin/main";
  const pausedSnapshot = {
    overview: {
      base,
      sync_status: "base-merge-conflict",
      worktree: { staged: 0, unstaged: 0, untracked: 0, conflicts: 1 },
    },
    saved_work: [],
    operations: [],
  };
  const commands = [];
  const bridge = {
    async invoke(command) {
      commands.push(command);
      if (command === "apply_operation") throw new Error("merge needs resolution");
      if (command === "load_snapshot") return pausedSnapshot;
      return [];
    },
  };
  const controller = new AppController(bridge);
  controller.render = () => {};
  controller.announce = () => {};
  controller.state.snapshot = {
    overview: { base, sync_status: null },
    saved_work: [],
    operations: [],
  };
  controller.state.review = { plan_id: "sync-review" };

  await controller.applyReview();

  assert.equal(controller.state.review, null);
  assert.equal(commands.filter((command) => command === "load_snapshot").length, 1);
  assert.equal(controller.state.error, "merge needs resolution");
  assert.match(shell(controller.state), /data-event="prepare-resume"/);

  controller.state.snapshot.overview.sync_status = "fetch";
  assert.match(shell(controller.state), /Review retry/);
  assert.doesNotMatch(shell(controller.state), /Review resume/);

  controller.state.snapshot.overview.sync_status = "snapshot";
  assert.match(shell(controller.state), /Inspect recovery/);
  assert.doesNotMatch(shell(controller.state), /data-event="prepare-resume"/);
});

test("repository snapshots do not repeat overview aggregation queries", async () => {
  const source = await readFile(
    new URL("../src-tauri/src/commands/repository.rs", import.meta.url),
    "utf8",
  );
  assert.doesNotMatch(source, /repository\.list_saved_work/);
  assert.doesNotMatch(source, /repository\.list_operations/);
  assert.doesNotMatch(source, /repository\.sync_status/);
});

test("repository identity is reused instead of queried on every refresh", async () => {
  const runner = await readFile(
    new URL("../src/git/mod.rs", import.meta.url),
    "utf8",
  );
  const inspection = await readFile(
    new URL("../src/inspection/queries.rs", import.meta.url),
    "utf8",
  );
  assert.match(runner, /git_version: String/);
  assert.match(runner, /git_dir: OnceLock<PathBuf>/);
  assert.match(inspection, /runner\.git_version\(\)/);
});
