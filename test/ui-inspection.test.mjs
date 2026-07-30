import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

test("Raw diff loads and renders the current branch patch", async () => {
  const commands = [];
  const patch = "diff --git a/src/app.js b/src/app.js\n+new line\n";
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "generate_branch_diff") return patch;
      return [];
    },
  });

  await controller.setView("raw-diff");

  const markup = renderShell(controller.state);
  assert.equal(controller.state.branchDiff, patch);
  // Per-view gating: opening Raw diff must not also fetch the structured diff.
  assert.deepEqual(commands, ["generate_branch_diff"]);
  assert.match(markup, />Inspection</);
  assert.match(markup, /Raw diff/);
  assert.doesNotMatch(markup, /Branch diff/);
  assert.match(markup, /data-event="copy-diff"/);
  assert.match(markup, /\+new line/);
});

test("the Inspection rail lists Files diff before Raw diff", () => {
  const controller = controllerWith({});

  const markup = renderShell(controller.state);

  assert.match(markup, /data-value="files-diff"/);
  assert.match(markup, /data-value="raw-diff"/);
  assert.ok(
    markup.indexOf('data-value="files-diff"') < markup.indexOf('data-value="raw-diff"'),
    "Files diff is the readable view and leads the group",
  );
});

test("Raw diff explains that Base is required without requesting a diff", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      return [];
    },
  }, { base: null });

  await controller.setView("raw-diff");

  assert.deepEqual(commands, []);
  assert.match(renderShell(controller.state), /Set Base to generate a diff/);
});

test("refresh regenerates an open Raw diff", async () => {
  let generation = 0;
  const controller = controllerWith({
    async invoke(command) {
      if (command === "generate_branch_diff") return `patch ${++generation}`;
      if (command === "load_snapshot") return snapshotWith({});
      return [];
    },
  });

  await controller.setView("raw-diff");
  await controller.refresh();

  assert.equal(controller.state.branchDiff, "patch 2");
});

test("the generated diff is copied to the clipboard in one action", async () => {
  const controller = controllerWith({});
  const writes = [];
  Object.defineProperty(globalThis.navigator, "clipboard", {
    configurable: true,
    value: { writeText: async (value) => writes.push(value) },
  });
  controller.state.branchDiff = "diff content";
  controller.state.view = "raw-diff";

  try {
    await controller.copyDiff();
  } finally {
    delete globalThis.navigator.clipboard;
  }

  assert.deepEqual(writes, ["diff content"]);
  assert.match(renderShell(controller.state), />Copied</);
  assert.match(renderShell(controller.state), /is-copied/);
});
