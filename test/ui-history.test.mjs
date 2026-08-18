import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { OPERATIONS, submitState } from "../ui/app/Private/operations/index.ts";
import { createDraft } from "../ui/app/Private/draft/index.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const PAST = {
  id: "a".repeat(40),
  short_id: "aaaaaaa",
  subject: "old state",
  message: "old state\n",
  author: { name: "Ada", email: "ada@example.test", date: "2020-01-01T12:00:00Z" },
};

test("catalog includes History next to Quick switch", () => {
  const ids = OPERATIONS.map((operation) => operation.id);
  assert.equal(ids.indexOf("history"), ids.indexOf("quick_switch") + 1);
  assert.equal(OPERATIONS.find((operation) => operation.id === "history")?.label, "History");
});

test("History carry is unchecked by default and Quick switch carry stays on", () => {
  const draft = createDraft();
  assert.equal(draft.historyCarryChanges, false);
  assert.equal(draft.carryChanges, true);
});

test("History tab lists commits and leaves carry unchecked when dirty", () => {
  const controller = controllerWith(
    { invoke: async () => [PAST] },
    { worktree: { staged: 0, unstaged: 2, untracked: 0, conflicts: 0 } },
  );
  controller.state.operation = "history";
  controller.state.commits = [PAST];
  controller.state.draft.commit = PAST.id;
  const markup = renderShell(controller.state);
  assert.match(markup, /data-value="history"/);
  assert.match(markup, /Carry tracked changes to the target branch/);
  assert.doesNotMatch(markup, /toggle-history-carry"\s+checked/);
  assert.match(markup, /will be saved before leaving present/);
});

test("History outcome offers switch to present", () => {
  const controller = controllerWith({ invoke: async () => null });
  controller.state.outcome = {
    kind: "history",
    headline: "Now in History",
    details: [],
    offer_force_push: false,
    offer_publish_branch: null,
    offer_switch_to_present: "feature",
  };
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="switch-to"\s+data-value="feature"/);
  assert.match(markup, /Switch to feature/);
});

test("detached HEAD with present_branch shows the History repo-bar label and persistent banner", () => {
  const controller = controllerWith(
    { invoke: async () => null },
    { branch: null, present_branch: "feature" },
  );
  const markup = renderShell(controller.state);
  assert.match(markup, /History · present: feature/);
  assert.match(markup, /You are in History/);
  assert.match(markup, /data-event="switch-to" data-value="feature"/);
});

test("adoptBranch prefers present_branch over Base", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: false, saved_work: false },
    { name: "main", head: "b".repeat(40), current: false, saved_work: false },
    { name: "alpha", head: "c".repeat(40), current: false, saved_work: false },
  ];
  const controller = controllerWith(
    { invoke: async () => branches },
    { branch: null, present_branch: "feature", base: "refs/remotes/origin/main" },
  );
  controller.state.branches = branches;
  const { adoptBranch } = await import("../ui/app/Private/draft/index.ts");
  adoptBranch(controller.state.draft, branches, {
    base: "refs/remotes/origin/main",
    present: "feature",
  });
  assert.equal(controller.state.draft.targetBranch, "feature");
  assert.equal(controller.state.draft.branchPicked, false);
});

test("History datetime mode requires a value", () => {
  const controller = controllerWith({ invoke: async () => null });
  controller.state.operation = "history";
  controller.state.draft.historyMode = "until";
  assert.equal(submitState(controller.state).reason, "Choose a date and time.");
});

test("History submit is blocked when already detached", () => {
  const controller = controllerWith({ invoke: async () => null }, { branch: null, present_branch: "feature" });
  controller.state.operation = "history";
  assert.equal(submitState(controller.state).reason, "Return to present first.");
});
