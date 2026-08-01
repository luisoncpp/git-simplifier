import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import {
  setCleanupAllRemote,
  setCleanupFilter,
  setCleanupOnlyMine,
  setCleanupRemotes,
  toggleCleanupBranch,
} from "../ui/app/Private/selection.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const remote = (branch, head) => ({
  remote: "origin",
  tracking_ref: `refs/remotes/origin/${branch}`,
  remote_ref: `refs/heads/${branch}`,
  head,
  merged: true,
});

const CLEANUP = {
  base: "refs/remotes/origin/main",
  base_head: "0".repeat(40),
  identity: "me@example.test",
  choices: [
    {
      branch: "develop",
      reference: "refs/heads/develop",
      head: "3".repeat(40),
      kind: "local",
      author_email: "me@example.test",
      mine: true,
      protected: true,
      remote: null,
    },
    {
      branch: "orphan",
      reference: "refs/remotes/origin/orphan",
      head: "4".repeat(40),
      kind: "remote_only",
      author_email: "me@example.test",
      mine: true,
      protected: false,
      remote: remote("orphan", "4".repeat(40)),
    },
    {
      branch: "spike",
      reference: "refs/heads/spike",
      head: "1".repeat(40),
      kind: "local",
      author_email: "me@example.test",
      mine: true,
      protected: false,
      remote: remote("spike", "1".repeat(40)),
    },
    {
      branch: "theirs",
      reference: "refs/heads/theirs",
      head: "2".repeat(40),
      kind: "local",
      author_email: "other@example.test",
      mine: false,
      protected: false,
      remote: null,
    },
  ],
  excluded: [
    { branch: "feature", reason: "current_branch" },
    { branch: "main", reason: "base_branch" },
  ],
};

function withCleanup(discovery = CLEANUP) {
  return controllerWith({
    async invoke(command) {
      if (command === "list_cleanup_branches") return discovery;
      if (command === "load_snapshot") return snapshotWith({});
      return [];
    },
  });
}

async function capture(controller) {
  let sent = null;
  controller.bridge.invoke = async (command, args) => {
    sent = args;
    return { plan_id: "op-1", title: "t", impact: [], preserves: [], warnings: [], commands: [], apply_label: "Apply" };
  };
  await controller.submitOperation();
  return sent;
}

test("cleanup pre-ticks every merged branch except a shared name", async () => {
  const controller = await open();

  const markup = renderShell(controller.state);
  assert.match(markup, /1 of 2 ticked/);
  assert.match(markup, />shared</);

  const sent = await capture(controller);
  assert.deepEqual(sent.request.references, ["refs/heads/spike"]);
});

test("unticking a branch drops it from the request", async () => {
  const controller = await open();
  toggleCleanupBranch(controller, { value: "refs/heads/spike", checked: false });

  assert.match(renderShell(controller.state), /0 of 2 ticked/);
  const sent = await capture(controller);
  assert.equal(sent, null);
  assert.match(submitHint(controller), /Tick at least one branch/);
});

test("a shared branch is deleted only after a deliberate tick", async () => {
  const controller = await open();
  toggleCleanupBranch(controller, { value: "refs/heads/develop", checked: true });

  const sent = await capture(controller);
  assert.deepEqual(sent.request.references, ["refs/heads/develop", "refs/heads/spike"]);
});

test("untick survives a filter change instead of being reseeded", async () => {
  const controller = await open();
  toggleCleanupBranch(controller, { value: "refs/heads/spike", checked: false });
  setCleanupOnlyMine(controller, { checked: false });
  setCleanupOnlyMine(controller, { checked: true });

  const sent = await capture(controller);
  assert.equal(sent, null);
});

test("only-mine reveals branches written by someone else", async () => {
  const controller = await open();
  assert.doesNotMatch(renderShell(controller.state), /theirs/);

  setCleanupOnlyMine(controller, { checked: false });

  const markup = renderShell(controller.state);
  assert.match(markup, /theirs/);
  assert.match(markup, /other@example\.test/);
  const sent = await capture(controller);
  assert.deepEqual(sent.request.references, ["refs/heads/spike", "refs/heads/theirs"]);
});

test("a remote-only branch needs both the listing and the deletion toggle", async () => {
  const controller = await open();
  assert.doesNotMatch(renderShell(controller.state), /orphan/);

  setCleanupAllRemote(controller, { checked: true });
  assert.match(renderShell(controller.state), />remote only</);

  setCleanupRemotes(controller, { checked: false });
  const markup = renderShell(controller.state);
  assert.doesNotMatch(markup, /orphan/);
  assert.match(markup, /Remote-only branches stay hidden/);
});

test("turning off remote deletion is carried on the request", async () => {
  const controller = await open();
  setCleanupRemotes(controller, { checked: false });

  const sent = await capture(controller);
  assert.equal(sent.request.delete_remotes, false);
  assert.deepEqual(sent.request.references, ["refs/heads/spike"]);
});

test("the filter narrows the list and keeps the query visible", async () => {
  const controller = await open();
  setCleanupFilter(controller, { value: "dev" });

  const markup = renderShell(controller.state);
  assert.match(markup, /value="dev"/);
  assert.match(markup, /develop/);
  assert.doesNotMatch(markup, /refs\/heads\/spike/);
});

test("cleanup explains which branches a safety rule removed", async () => {
  const controller = await open();

  const markup = renderShell(controller.state);
  assert.match(markup, /feature \(checked out here\)/);
  assert.match(markup, /main \(the branch Base tracks\)/);
});

test("cleanup says nothing is merged instead of showing an empty list", async () => {
  const controller = withCleanup({ ...CLEANUP, choices: [], excluded: [] });
  await controller.selectOperation("cleanup");

  assert.match(renderShell(controller.state), /Nothing to clean up/);
  assert.match(submitHint(controller), /No branch is fully merged into Base/);
});

async function open() {
  const controller = withCleanup();
  await controller.selectOperation("cleanup");
  return controller;
}

const submitHint = (controller) =>
  /<p class="hint">([^<]*)<\/p>\s*<\/div>/.exec(renderShell(controller.state))?.[1] ?? "";
