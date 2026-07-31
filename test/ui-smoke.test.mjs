import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { AppController, renderShell } from "../ui/app/index.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

test("the workbench never invents repository data or prompts for Git identifiers", async () => {
  const controller = await readFile(new URL("../ui/app/Private/controller.ts", import.meta.url), "utf8");
  assert.doesNotMatch(controller, /window\.prompt|prompt\(/);
  const state = new AppController({}).state;
  const markup = renderShell(state);
  assert.match(markup, /No repository is open/);
  assert.doesNotMatch(markup, /origin\/develop|meridian|feat\//);
});

test("the live announcer cannot extend the document below the viewport shell", async () => {
  const css = await readFile(new URL("../ui/styles/workbench.css", import.meta.url), "utf8");
  const srOnlyRule = /\.sr-only\s*\{([^}]*)\}/.exec(css)?.[1] ?? "";

  assert.match(srOnlyRule, /top:\s*0/);
  assert.match(srOnlyRule, /left:\s*0/);
});

test("the operation boundary stays prepare / apply / cancel", async () => {
  const source = await readFile(new URL("../ui/app/Private/controller.ts", import.meta.url), "utf8");
  assert.match(source, /prepare_operation/);
  assert.match(source, /apply_operation/);
  assert.match(source, /cancel_operation/);
});

test("operation tabs still switch when their discovery request fails", async () => {
  const controller = controllerWith({
    async invoke(command) {
      if (command === "list_editable_commits") throw new Error("unable to list editable commits");
      return [];
    },
  });

  await assert.doesNotReject(controller.selectOperation("edit_message"));

  assert.equal(controller.state.operation, "edit_message");
  assert.equal(controller.state.error, "unable to list editable commits");
  const markup = renderShell(controller.state);
  assert.match(markup, /aria-selected="true"[^>]*data-value="edit_message"/);
  assert.match(markup, /unable to list editable commits/);
});

test("Git-backed Tauri commands run off the window thread", async () => {
  const files = ["actions", "diffs"].map((name) =>
    readFile(new URL(`../src-tauri/src/commands/${name}.rs`, import.meta.url), "utf8"),
  );
  const source = (await Promise.all(files)).join("\n");
  const commands = [...source.matchAll(/#\[tauri::command(?:\(async\))?\]\s+pub fn (\w+)/g)];
  const blocking = commands
    .filter((match) => match[1] !== "app_ready" && !match[0].includes("(async)"))
    .map((match) => match[1]);
  assert.deepEqual(blocking, []);
});

test("saving Base reuses the snapshot returned by set_base", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "set_base" || command === "load_snapshot") return snapshotWith({});
      return [];
    },
  });

  await controller.chooseBase("refs/remotes/origin/main");

  assert.equal(commands.filter((command) => command === "load_snapshot").length, 0);
  assert.equal(controller.state.changingBase, false);
});

test("changing Base loads remote choices before opening the selector", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "list_base_choices") {
        return [{ reference: "refs/remotes/origin/develop", display: "origin/develop", head: "2".repeat(40) }];
      }
      return [];
    },
  });

  await controller.editBase();

  assert.deepEqual(commands, ["list_base_choices"]);
  assert.match(renderShell(controller.state), /origin\/develop/);
  assert.doesNotMatch(renderShell(controller.state), /No remote-tracking ref was found/);
});

test("a failed sync apply reloads state once and offers the resume action", async () => {
  const paused = snapshotWith({ sync_status: "base-merge-conflict", worktree: { staged: 0, unstaged: 0, untracked: 0, conflicts: 1 } });
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "apply_operation") throw new Error("merge needs resolution");
      if (command === "load_snapshot") return paused;
      return [];
    },
  });
  controller.state.review = { plan_id: "sync-review" };

  await controller.applyReview();

  assert.equal(controller.state.review, null);
  assert.equal(commands.filter((command) => command === "load_snapshot").length, 1);
  assert.equal(controller.state.error, "merge needs resolution");
  assert.match(renderShell(controller.state), /data-event="resume-sync"/);

  controller.state.snapshot.overview.sync_status = "fetch";
  assert.match(renderShell(controller.state), /Review retry/);

  controller.state.snapshot.overview.sync_status = "snapshot";
  assert.doesNotMatch(renderShell(controller.state), /data-event="resume-sync"/);
  assert.match(renderShell(controller.state), /Inspect recovery/);
});

test("repository snapshots do not repeat overview aggregation queries", async () => {
  const source = await readFile(new URL("../src-tauri/src/commands/repository.rs", import.meta.url), "utf8");
  assert.doesNotMatch(source, /repository\.list_saved_work/);
  assert.doesNotMatch(source, /repository\.list_operations/);
  assert.doesNotMatch(source, /repository\.sync_status/);
});

test("repository identity is reused instead of queried on every refresh", async () => {
  const runner = await readFile(new URL("../src/git/mod.rs", import.meta.url), "utf8");
  const inspection = await readFile(new URL("../src/inspection/queries.rs", import.meta.url), "utf8");
  assert.match(runner, /git_version: String/);
  assert.match(runner, /git_dir: OnceLock<PathBuf>/);
  assert.match(inspection, /runner\.git_version\(\)/);
});
