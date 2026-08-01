import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { setCommit, setMessage, setNewBranch, setPathFilter, setRevertTarget, setSplitMessage, togglePath } from "../ui/app/Private/selection.ts";
import { OPERATIONS } from "../ui/app/Private/operations/index.ts";
import { submitRow } from "../ui/app/Private/views/actions.ts";
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
      if (command === "list_changed_paths" || command === "list_revert_paths") return PATHS;
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

test("revert keeps its own selection and sends the chosen source", async () => {
  const controller = withData({});
  await controller.refresh();
  togglePath(controller, { value: "src/keys.env", checked: true });

  await controller.selectOperation("revert");
  assert.equal(controller.state.draft.revertPaths.size, 0);
  assert.match(renderShell(controller.state), /0 of 2 selected/);
  assert.match(renderShell(controller.state), />Uncommit</);
  assert.match(renderShell(controller.state), />Revert</);

  togglePath(controller, { value: "src/app.js", checked: true });
  setRevertTarget(controller, { value: "base" });

  let sent = null;
  controller.bridge.invoke = async (command, args) => {
    sent = args;
    return { plan_id: "op-revert", title: "t", impact: [], preserves: [], warnings: [], commands: [], apply_label: "Apply" };
  };
  await controller.submitOperation();

  assert.deepEqual([...controller.state.draft.selectedPaths], ["src/keys.env"]);
  assert.deepEqual(sent.request, {
    kind: "revert",
    base: "refs/remotes/origin/main",
    paths: ["src/app.js"],
    target: "base",
  });
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

test("quick switch offers carry changes when the worktree is dirty", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: true, saved_work: false },
    { name: "main", head: "b".repeat(40), current: false, saved_work: false },
  ];
  const controller = controllerWith({}, { worktree: { staged: 0, unstaged: 2, untracked: 0, conflicts: 0 } });
  controller.state.branches = branches;
  controller.state.operation = "quick_switch";
  controller.state.draft.targetBranch = "main";

  const markup = renderShell(controller.state);
  assert.match(markup, /Carry tracked changes to the target branch/);
  assert.match(markup, /will be saved before the switch/);

  controller.state.draft.carryChanges = true;
  assert.match(renderShell(controller.state), /will be stashed/);
  assert.match(renderShell(controller.state), /Conflicts are reported afterwards/);
});

test("quick switch never offers the branch that is already checked out", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: true, saved_work: false },
    { name: "main", head: "b".repeat(40), current: false, saved_work: true },
  ];
  const controller = withData({ branches });
  await controller.selectOperation("quick_switch");

  const markup = renderShell(controller.state);
  assert.match(markup, /branch-trigger[\s\S]*<strong>main<\/strong>/);
  assert.match(markup, /main has Saved work waiting/);
  assert.match(markup, /Pull from the same-named remote after switching/);
  assert.equal(controller.state.draft.targetBranch, "main");
  assert.equal(controller.state.draft.pullAfterSwitch, true);
});

test("quick switch lists remote-only branches for local creation", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: true, saved_work: false },
    { name: "only-remote", head: "c".repeat(40), current: false, saved_work: false, remote: "origin/only-remote" },
  ];
  const controller = withData({ branches }, { base: "refs/remotes/origin/only-remote" });
  await controller.selectOperation("quick_switch");

  const markup = renderShell(controller.state);
  assert.match(markup, /origin\/only-remote → only-remote/);
  assert.equal(controller.state.draft.targetBranch, "only-remote");
  assert.equal(controller.state.draft.createFromRemote, "origin/only-remote");
});

test("quick switch defaults target branch to local Base when on a non-Base branch", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: true, saved_work: false },
    { name: "alpha", head: "b".repeat(40), current: false, saved_work: false },
    { name: "main", head: "c".repeat(40), current: false, saved_work: false },
  ];
  const controller = withData({ branches }, { base: "refs/remotes/origin/main" });
  await controller.selectOperation("quick_switch");

  assert.equal(controller.state.draft.targetBranch, "main");
  assert.equal(controller.state.draft.createFromRemote, "");
});

test("quick switch defaults target branch to remote Base when local Base does not exist", async () => {
  const branches = [
    { name: "feature", head: "a".repeat(40), current: true, saved_work: false },
    { name: "alpha", head: "b".repeat(40), current: false, saved_work: false },
    { name: "main", head: "c".repeat(40), current: false, saved_work: false, remote: "origin/main" },
  ];
  const controller = withData({ branches }, { base: "refs/remotes/origin/main" });
  await controller.selectOperation("quick_switch");

  assert.equal(controller.state.draft.targetBranch, "main");
  assert.equal(controller.state.draft.createFromRemote, "origin/main");
});

test("quick switch defaults target branch to first available branch when current branch is Base", async () => {
  const branches = [
    { name: "main", head: "a".repeat(40), current: true, saved_work: false },
    { name: "alpha", head: "b".repeat(40), current: false, saved_work: false },
    { name: "beta", head: "c".repeat(40), current: false, saved_work: false },
  ];
  const controller = withData({ branches }, { base: "refs/remotes/origin/main" });
  await controller.selectOperation("quick_switch");

  assert.equal(controller.state.draft.targetBranch, "alpha");
  assert.equal(controller.state.draft.createFromRemote, "");
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

test("split branch keeps its own selection instead of inheriting the uncommit one", async () => {
  const controller = withData({});
  await controller.refresh();
  togglePath(controller, { value: "src/keys.env", checked: true });

  await controller.selectOperation("split_branch");

  assert.equal(controller.state.draft.splitPaths.size, 0);
  assert.match(renderShell(controller.state), /0 of 2 selected/);

  togglePath(controller, { value: "src/app.js", checked: true });
  assert.deepEqual([...controller.state.draft.selectedPaths], ["src/keys.env"]);
  assert.deepEqual([...controller.state.draft.splitPaths], ["src/app.js"]);
});

test("split branch names the missing branch name before the missing selection", async () => {
  const controller = withData({});
  await controller.selectOperation("split_branch");

  await controller.submitOperation();
  assert.equal(controller.state.error, "Name the new branch.");

  setNewBranch(controller, { value: "carved" });
  await controller.submitOperation();
  assert.equal(controller.state.error, "Select at least one path to copy.");
});

test("split branch sends the trimmed name, the picked paths, and the typed message", async () => {
  const controller = withData({});
  await controller.selectOperation("split_branch");
  setNewBranch(controller, { value: "  carved  " });
  togglePath(controller, { value: "src/app.js", checked: true });
  setSplitMessage(controller, { value: "hero art only\n" });

  let sent = null;
  controller.bridge.invoke = async (command, args) => {
    sent = args;
    return { plan_id: "op-2", title: "t", impact: [], preserves: [], warnings: [], commands: [], apply_label: "Apply" };
  };
  await controller.submitOperation();

  assert.deepEqual(sent.request, {
    kind: "split_branch",
    base: "refs/remotes/origin/main",
    new_branch: "carved",
    paths: ["src/app.js"],
    message: "hero art only\n",
  });
});

test("an empty split message is sent as empty so Rust derives and shows one", async () => {
  const controller = withData({});
  await controller.selectOperation("split_branch");
  setNewBranch(controller, { value: "carved" });
  togglePath(controller, { value: "src/app.js", checked: true });

  let sent = null;
  controller.bridge.invoke = async (command, args) => {
    sent = args;
    return { plan_id: "op-3", title: "t", impact: [], preserves: [], warnings: [], commands: [], apply_label: "Apply" };
  };
  await controller.submitOperation();

  assert.equal(sent.request.message, "");
  assert.match(renderShell(controller.state), /named after the branch/);
});

test("the split form warns that the asset and its meta file travel together", async () => {
  const controller = controllerWith({
    async invoke(command) {
      if (command === "list_changed_paths") {
        return [
          { path: "Assets/hero.png", previous_path: null, status: "M" },
          { path: "Assets/hero.png.meta", previous_path: null, status: "M" },
        ];
      }
      return [];
    },
  });
  await controller.selectOperation("split_branch");
  togglePath(controller, { value: "Assets/hero.png", checked: true });

  assert.match(renderShell(controller.state), /travel with their asset/);
});

/// The submit label falls back to the raw operation id, which reads as a leaked
/// identifier rather than an action. Every operation must have a real word.
test("no operation offers a submit button labelled with its identifier", () => {
  const controller = controllerWith({});
  for (const { id } of OPERATIONS) {
    controller.state.operation = id;
    const label = /<button class="primary"[^>]*>([^<]+)</.exec(submitRow(controller.state))?.[1] ?? "";
    assert.doesNotMatch(label, /_/, `${id} shows its identifier in the submit button`);
  }
});

test("a created branch offers its first push, not a force push", () => {
  const controller = controllerWith({});
  controller.state.outcome = {
    kind: "split_branch",
    headline: "Branch created",
    details: ["refs/heads/carved points at abc1234"],
    offer_force_push: false,
    offer_publish_branch: "carved",
  };

  const banner = bannerOf(renderShell(controller.state));
  assert.match(banner, /data-event="publish-branch"\s+data-value="carved"/);
  assert.match(banner, /Review push of carved/);
  assert.doesNotMatch(banner, /force push/i);
});

test("a rewrite still offers a force push and never the publish button", () => {
  const controller = controllerWith({});
  controller.state.outcome = {
    kind: "rewrite",
    headline: "History rewritten",
    details: [],
    offer_force_push: true,
    offer_publish_branch: null,
  };

  const banner = bannerOf(renderShell(controller.state));
  assert.match(banner, /Review force push/);
  assert.doesNotMatch(banner, /data-event="publish-branch"/);
});

test("switching onto Saved work offers a restore review from the result banner", () => {
  const controller = controllerWith({});
  controller.state.saved = [
    { branch: "feature", reference: "refs/githelper/wip/feature", snapshot: "e".repeat(40) },
  ];
  controller.state.outcome = {
    kind: "quick_switch",
    headline: "Branch switched",
    details: ["Now on feature", "Saved work for feature is ready to restore"],
    offer_force_push: false,
    offer_publish_branch: null,
    offer_restore_saved_work: true,
  };

  const banner = bannerOf(renderShell(controller.state));
  assert.match(banner, /data-event="restore-saved"/);
  assert.match(banner, /Review restore/);
  assert.doesNotMatch(renderShell(controller.state), /Saved work is waiting/);
});

test("Saved work on the current branch shows a persistent restore offer", () => {
  const controller = controllerWith({});
  controller.state.saved = [
    { branch: "feature", reference: "refs/githelper/wip/feature", snapshot: "e".repeat(40) },
  ];

  const markup = renderShell(controller.state);
  assert.match(markup, /Saved work is waiting/);
  assert.match(markup, /data-event="restore-saved"/);
  assert.match(markup, /data-event="dismiss-saved-work-notice"/);
  assert.match(markup, /Review restore/);
});

test("dismissing the Saved work notice hides it until the snapshot is gone", () => {
  const controller = controllerWith({});
  controller.state.saved = [
    { branch: "feature", reference: "refs/githelper/wip/feature", snapshot: "e".repeat(40) },
  ];

  controller.state.dismissedSavedWorkBranch = "feature";
  assert.doesNotMatch(renderShell(controller.state), /Saved work is waiting/);
});

test("a pending pull decision suppresses the Saved work restore banner", () => {
  const controller = controllerWith({ quick_switch_status: "pull-ff-failed" });
  controller.state.saved = [
    { branch: "feature", reference: "refs/githelper/wip/feature", snapshot: "e".repeat(40) },
  ];
  controller.state.outcome = {
    kind: "quick_switch",
    headline: "Pull needs a decision",
    details: ["Now on feature"],
    offer_force_push: false,
    offer_publish_branch: null,
    offer_resolve_pull: true,
  };

  const markup = renderShell(controller.state);
  assert.match(markup, /Replace with remote/);
  assert.doesNotMatch(markup, /Saved work is waiting/);
  assert.doesNotMatch(markup, /data-event="restore-saved"/);
});

/// The operation strip always contains a Force push tab, so a follow-up
/// assertion has to look at the result banner rather than the whole shell.
function bannerOf(markup) {
  return /<div class="banner good">[\s\S]*?<\/div>\s*<\/div>/.exec(markup)?.[0] ?? "";
}
