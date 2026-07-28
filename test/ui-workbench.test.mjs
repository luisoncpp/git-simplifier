import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.js";
import { setCommit, setMessage, setPathFilter, togglePath } from "../ui/app/Private/selection.js";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const COMMITS = [
  { id: "a".repeat(40), short_id: "aaaaaaa", subject: "first", message: "first\n\nbody one", author: { name: "Dev", email: "d@e", date: "2026-07-01" } },
  { id: "b".repeat(40), short_id: "bbbbbbb", subject: "second", message: "second\n\nbody two", author: { name: "Dev", email: "d@e", date: "2026-07-02" } },
];

const PATHS = [
  { path: "src/keys.env", previous_path: null, status: "A" },
  { path: "src/app.js", previous_path: null, status: "M" },
];

function withData(data) {
  return controllerWith({
    async invoke(command) {
      if (command === "list_editable_commits") return COMMITS;
      if (command === "list_changed_paths") return PATHS;
      if (command === "list_local_branches") return data.branches ?? [];
      if (command === "list_submodules") return data.submodules ?? [];
      if (command === "load_snapshot") return snapshotWith({});
      return [];
    },
  });
}

test("the editor opens on the newest commit and follows the selection", async () => {
  const controller = withData({});
  await controller.selectOperation("edit_message");

  assert.equal(controller.state.draft.commit, COMMITS[1].id);
  assert.match(renderShell(controller.state), /body two/);

  setCommit(controller, { value: COMMITS[0].id });

  const markup = renderShell(controller.state);
  assert.match(markup, /body one/);
  assert.doesNotMatch(markup, /body two/);
  assert.match(markup, /Edit the text above to enable the review/);
});

test("an edited message survives switching commits and back", async () => {
  const controller = withData({});
  await controller.selectOperation("edit_message");
  setMessage(controller, { value: "reworded second" });

  setCommit(controller, { value: COMMITS[0].id });
  assert.match(renderShell(controller.state), /body one/);

  setCommit(controller, { value: COMMITS[1].id });
  assert.match(renderShell(controller.state), /reworded second/);
});

test("an unchanged message cannot be submitted and says so", async () => {
  const controller = withData({});
  await controller.selectOperation("edit_message");

  const invoked = [];
  controller.bridge.invoke = async (command) => {
    invoked.push(command);
    return [];
  };
  await controller.submitOperation();

  assert.deepEqual(invoked, []);
  assert.equal(controller.state.error, "The message is unchanged.");
});

test("uncommit refuses to prepare with nothing selected and explains why", async () => {
  const controller = withData({});
  await controller.refresh();

  await controller.submitOperation();
  assert.equal(controller.state.error, "Select at least one path.");

  togglePath(controller, { value: "src/keys.env", checked: true });
  assert.match(renderShell(controller.state), /1 of 2 selected/);

  let sent = null;
  controller.bridge.invoke = async (command, args) => {
    sent = args;
    return { plan_id: "op-1", title: "t", impact: [], preserves: [], warnings: [], commands: [], apply_label: "Apply" };
  };
  await controller.submitOperation();

  assert.deepEqual(sent.request.paths, ["src/keys.env"]);
});

test("the path filter narrows the list and keeps the query visible", async () => {
  const controller = withData({});
  await controller.refresh();

  setPathFilter(controller, { value: "keys" });

  const markup = renderShell(controller.state);
  assert.match(markup, /src\/keys\.env/);
  assert.doesNotMatch(markup, /src\/app\.js/);
  assert.match(markup, /value="keys"/);
});

test("a stale result banner does not follow the user to another operation", async () => {
  const controller = withData({});
  controller.state.outcome = { kind: "rewrite", headline: "History rewritten", details: [], offer_force_push: true };

  assert.match(renderShell(controller.state), /History rewritten/);
  await controller.selectOperation("quick_switch");

  assert.equal(controller.state.outcome, null);
  assert.doesNotMatch(renderShell(controller.state), /History rewritten/);
});

test("quick switch never offers the branch that is already checked out", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: true, saved_work: false },
    { name: "main", head: "b".repeat(40), current: false, saved_work: true },
  ];
  const controller = withData({ branches });
  await controller.selectOperation("quick_switch");

  const markup = renderShell(controller.state);
  assert.doesNotMatch(markup, /<option value="feature"/);
  assert.match(markup, /<option value="main"[^>]*>main · has Saved work<\/option>/);
  assert.equal(controller.state.draft.targetBranch, "main");
});

test("an already excluded submodule is labelled instead of looking untouched", async () => {
  const submodules = [
    { path: "vendor/theme", object: "c".repeat(40), excluded: true },
    { path: "vendor/sdk", object: "d".repeat(40), excluded: false },
  ];
  const controller = withData({ submodules });
  await controller.selectOperation("exclude_submodule");

  assert.equal(controller.state.draft.submodule, "vendor/sdk");
  assert.match(renderShell(controller.state), /vendor\/theme · already excluded/);
});

test("Saved work on another branch offers the switch instead of a dead Restore button", () => {
  const controller = controllerWith({});
  controller.state.view = "saved";
  controller.state.saved = [
    { branch: "feature", reference: "refs/githelper/wip/feature", snapshot: "e".repeat(40) },
    { branch: "other", reference: "refs/githelper/wip/other", snapshot: "f".repeat(40) },
  ];

  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="restore-saved"/);
  assert.match(markup, /data-event="switch-to" data-value="other"/);
  assert.doesNotMatch(markup, /Arrive on branch/);
});

test("force push states the missing upstream instead of failing after a click", () => {
  const controller = controllerWith({});
  controller.state.operation = "force_push";

  const markup = renderShell(controller.state);
  assert.match(markup, /No upstream/);
  assert.match(markup, /data-event="submit-operation" disabled/);
});
