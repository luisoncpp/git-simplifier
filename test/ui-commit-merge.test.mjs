import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { OPERATIONS, submitState } from "../ui/app/Private/operations/index.ts";
import { submitRow } from "../ui/app/Private/views/actions.ts";
import { controllerWith } from "./support/controller.mjs";

test("catalog includes Commit merge", () => {
  const entry = OPERATIONS.find((operation) => operation.id === "commit_merge");
  assert.ok(entry);
  assert.equal(entry.label, "Commit merge");
});

test("no merge shows empty state and disabled submit", () => {
  const controller = controllerWith({ invoke: async () => null });
  controller.state.operation = "commit_merge";
  const markup = renderShell(controller.state);
  assert.match(markup, /No merge in progress/);
  assert.equal(submitState(controller.state).disabled, true);
  assert.equal(submitState(controller.state).reason, "No merge in progress.");
});

test("unmerged conflicts block submit", () => {
  const controller = controllerWith(
    { invoke: async () => null },
    { merge_in_progress: true, worktree: { staged: 0, unstaged: 0, untracked: 0, conflicts: 2 } },
  );
  controller.state.operation = "commit_merge";
  const markup = renderShell(controller.state);
  assert.match(markup, /Conflicts still unmerged/);
  assert.equal(submitState(controller.state).reason, "Resolve merge conflicts first.");
});

test("ready merge enables submit", () => {
  const controller = controllerWith(
    { invoke: async () => null },
    { merge_in_progress: true, worktree: { staged: 1, unstaged: 0, untracked: 0, conflicts: 0 } },
  );
  controller.state.operation = "commit_merge";
  assert.equal(submitState(controller.state).disabled, false);
  assert.equal(submitState(controller.state).reason, "");
});

test("submit label has no underscore", () => {
  const controller = controllerWith({ invoke: async () => null }, { merge_in_progress: true });
  controller.state.operation = "commit_merge";
  const row = submitRow(controller.state);
  assert.ok(!row.includes("_"));
});

test("sync banner offers commit merge while MERGE_HEAD exists", () => {
  const controller = controllerWith(
    { invoke: async () => null },
    {
      sync_status: "base-merge-conflict",
      merge_in_progress: true,
      worktree: { staged: 0, unstaged: 0, untracked: 0, conflicts: 0 },
    },
  );
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="commit-merge"/);
  assert.ok(!markup.includes("data-event=\"resume-sync\""));
});

test("sync banner offers resume after merge committed", () => {
  const controller = controllerWith(
    { invoke: async () => null },
    { sync_status: "base-merge-conflict", merge_in_progress: false },
  );
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="resume-sync"/);
});

test("outcome offers resume sync follow-up", () => {
  const controller = controllerWith({ invoke: async () => null });
  controller.state.outcome = {
    kind: "commit_merge",
    headline: "Merge committed",
    details: [],
    offer_force_push: false,
    offer_resume_sync: true,
  };
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="resume-sync"/);
});
