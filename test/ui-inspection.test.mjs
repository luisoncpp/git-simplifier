import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.js";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

test("Inspection loads and renders the current branch diff", async () => {
  const commands = [];
  const patch = "diff --git a/src/app.js b/src/app.js\n+new line\n";
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "generate_branch_diff") return patch;
      return [];
    },
  });

  await controller.setView("inspection");

  const markup = renderShell(controller.state);
  assert.equal(controller.state.branchDiff, patch);
  assert.deepEqual(commands, ["generate_branch_diff"]);
  assert.match(markup, />Inspection</);
  assert.match(markup, /Branch diff/);
  assert.match(markup, /data-event="copy-diff"/);
  assert.match(markup, /\+new line/);
});

test("Inspection explains that Base is required without requesting a diff", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      return [];
    },
  }, { base: null });

  await controller.setView("inspection");

  assert.deepEqual(commands, []);
  assert.match(renderShell(controller.state), /Set Base to generate a diff/);
});

test("refresh regenerates an open Inspection diff", async () => {
  let generation = 0;
  const controller = controllerWith({
    async invoke(command) {
      if (command === "generate_branch_diff") return `patch ${++generation}`;
      if (command === "load_snapshot") return snapshotWith({});
      return [];
    },
  });

  await controller.setView("inspection");
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
  controller.state.view = "inspection";

  try {
    await controller.copyDiff();
  } finally {
    delete globalThis.navigator.clipboard;
  }

  assert.deepEqual(writes, ["diff content"]);
  assert.match(renderShell(controller.state), />Copied</);
  assert.match(renderShell(controller.state), /is-copied/);
});
